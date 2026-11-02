#!/usr/bin/env python3
"""Fail closed. Generated modules run with no network, child commands or external writes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import shutil
import signal
import subprocess
import tempfile
import time

MATHLIB = '56a005855ea17e0b0958f73ec801ae140b0a1679'
LEAN = '4.35.0'
LIMIT = 2 * 1024**3

def digest(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    args = parser.parse_args()
    project, source = args.project.resolve(), args.source.resolve()
    started = time.monotonic()
    result = dict(checked_modules=[], commands=[], elapsed_ms=0, status='ERROR', diagnostics=[], exit_code=None, stdout='', stderr='',
                  timed_out=False, peak_rss_bytes=0, sandboxed=False, toolchain_verified=False,
                  axioms=[], lean_version='', mathlib_revision='', source_hash=digest(source),
                  foundation_hash=digest(project / 'Foundation.lean'), independent_hash=digest(project / 'Independent.lean'))
    try:
        manifest = json.loads((project / 'lake-manifest.json').read_text())
        if (project / 'lean-toolchain').read_text().strip() != 'leanprover/lean4:v' + LEAN:
            raise RuntimeError('Lean pin mismatch')
        packages = project / '.lake/packages'
        for package in manifest['packages']:
            path = packages / package['name']
            revision = subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD'], text=True).strip()
            if revision != package['rev']:
                raise RuntimeError('package revision mismatch: ' + package['name'])
            subprocess.run(['git', '-C', str(path), 'diff', '--quiet', 'HEAD', '--'], check=True)
            if package['name'] == 'mathlib':
                result['mathlib_revision'] = revision
        if result['mathlib_revision'] != MATHLIB:
            raise RuntimeError('Mathlib pin mismatch')
        # Select the pinned toolchain directly; no lake scripts or generated code run here.
        toolchain = Path.home() / '.elan/toolchains' / ('leanprover--lean4---v' + LEAN)
        lean = toolchain / 'bin/lean'
        version = subprocess.check_output([str(lean), '--version'], text=True).strip()
        result['lean_version'] = version
        if 'version ' + LEAN + ',' not in version:
            raise RuntimeError('Lean version mismatch')
        result['toolchain_verified'] = True
        sandbox = shutil.which('sandbox-exec')
        if not sandbox:
            raise RuntimeError('supported OS sandbox unavailable; refusing unisolated Lean')
        with tempfile.TemporaryDirectory(prefix='mldsn-lean-') as temporary:
            staging = Path(temporary).resolve()
            shutil.copyfile(project / 'Foundation.lean', staging / 'Foundation.lean')
            shutil.copyfile(source, staging / 'Generated.lean')
            shutil.copyfile(project / 'Independent.lean', staging / 'Independent.lean')
            profile = staging / 'sandbox.sb'
            # SBPL string literals are JSON-compatible for these absolute paths.
            profile.write_text('(version 1)\n(allow default)\n(deny network*)\n'
                               '(deny file-write*)\n(allow file-write* (subpath ' + json.dumps(str(staging)) + '))\n'
                               '(deny process-fork)\n(deny process-exec)\n'
                               '(allow process-exec (literal ' + json.dumps(str(lean.resolve())) + '))\n')
            env = dict(os.environ)
            env['LEAN_PATH'] = os.pathsep.join([str(staging)] +
                [str((packages / p['name'] / '.lake/build/lib/lean').resolve()) for p in manifest['packages']])
            env['LEAN_SRC_PATH'] = ''
            all_stdout, all_stderr = [], []
            deadline = time.monotonic() + 300
            for module in ['Foundation', 'Independent', 'Generated']:
                cmd = [sandbox, '-f', str(profile), str(lean), '-j', '1', '-M', '1536', '-T', '300',
                       '-o', str(staging / (module + '.olean')), str(staging / (module + '.lean'))]
                result['commands'].append(cmd)
                with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
                    proc = subprocess.Popen(cmd, cwd=staging, env=env, stdout=out, stderr=err, start_new_session=True)
                    while proc.poll() is None:
                        try:
                            rss = int(subprocess.check_output(['/bin/ps', '-o', 'rss=', '-p', str(proc.pid)],
                                                             stderr=subprocess.DEVNULL).strip()) * 1024
                            result['peak_rss_bytes'] = max(result['peak_rss_bytes'], rss)
                        except (ValueError, subprocess.CalledProcessError):
                            pass
                        except PermissionError:
                            os.killpg(proc.pid, signal.SIGKILL)
                            proc.wait()
                            raise RuntimeError('RSS monitor denied by host sandbox')
                        if time.monotonic() >= deadline or result['peak_rss_bytes'] > LIMIT:
                            result['timed_out'] = True
                            result['diagnostics'].append('time or RSS budget exceeded')
                            os.killpg(proc.pid, signal.SIGKILL)
                            break
                        time.sleep(0.05)
                    result['exit_code'] = proc.wait()
                    result['peak_rss_bytes'] = max(result['peak_rss_bytes'], int(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss))
                    if result['peak_rss_bytes'] > LIMIT:
                        result['timed_out'] = True
                        result['diagnostics'].append('RSS budget exceeded (OS peak measurement)')
                    out.seek(0); err.seek(0)
                    all_stdout.append(out.read().decode(errors='replace'))
                    all_stderr.append(err.read().decode(errors='replace'))
                if 'sandbox_apply:' in all_stderr[-1]:
                    raise RuntimeError('OS denied sandbox creation; refusing unisolated Lean')
                result['sandboxed'] = True
                if result['exit_code'] != 0:
                    break
                result['checked_modules'].append(module)
            result['stdout'], result['stderr'] = ''.join(all_stdout), ''.join(all_stderr)
            text = result['stdout']
            audit = re.search(r"'Generated.target' (?:depends on axioms: \[([^\]]*)\]|does not depend on any axioms)", text, re.S)
            if audit:
                result['axioms'] = [x.strip() for x in (audit.group(1) or '').split(',') if x.strip()]
            elif result['exit_code'] == 0:
                raise RuntimeError('missing target-specific axiom report')
            result['status'] = ('ERROR' if not result['timed_out'] and result['checked_modules'][:2] != ['Foundation', 'Independent'] else 'UNKNOWN' if result['timed_out'] or result['exit_code'] != 0 else
                                'ERROR' if any(x not in ['propext', 'Classical.choice', 'Quot.sound'] for x in result['axioms']) else 'PROVED')
    except Exception as error:
        result['diagnostics'].append(str(error))
        result['sandboxed'] = False
    result['elapsed_ms'] = round((time.monotonic() - started) * 1000)
    print(json.dumps(result))

if __name__ == '__main__':
    main()
