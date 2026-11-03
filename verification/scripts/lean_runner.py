#!/usr/bin/env python3
"""Fail closed. Generated modules run with no network, child commands or external writes.

`--isolation linux-unsealed` is an explicit, weaker mode for Linux hosts without sandbox-exec:
each lean process runs in new user, network and mount namespaces (`unshare -rnm`) with every
writable mount remounted read-only except the staging directory. Child-process denial is not
available, so the result always reports `sandboxed: false` and can never become sealed Evidence."""
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

LINUX_ISOLATION = r'''
staging="$1"; shift
mount --bind "$staging" "$staging" || { echo "mldsn-isolation: setup failed (bind staging)" >&2; exit 97; }
awk '$4 ~ /(^|,)rw(,|$)/ {print $2}' /proc/self/mounts | while read -r m; do
  m=$(printf '%b' "$m")
  [ "$m" = "$staging" ] && continue
  mount -o remount,bind,ro "$m" 2>/dev/null || mount -t tmpfs -o ro,size=4k mldsn-ro "$m" 2>/dev/null || true
done
mount -o remount,bind,ro / || { echo "mldsn-isolation: setup failed (read-only root)" >&2; exit 97; }
# Only the top-most (last listed) mount at each mount point is effective.
awk '{state[$2]=$4} END {for (m in state) if (state[m] ~ /(^|,)rw(,|$)/) print m}' /proc/self/mounts | while read -r m; do
  m=$(printf '%b' "$m")
  [ "$m" = "$staging" ] || echo "mldsn-isolation: still writable $m" >&2
done
exec "$@"
'''

IMPORT = re.compile(r'^\s*(?:public\s+)?import\s+([A-Za-z_][A-Za-z0-9_.\']*)\s*$', re.M)

def knowledge_closure(root, module, manifest_sha):
    """Pack-local import closure of `module` in dependency order, each verified against the
    pinned manifest. Imports outside the pack must be Mathlib (resolved from built packages)."""
    if root is None or manifest_sha is None:
        raise RuntimeError('Knowledge module requires --knowledge-root and --knowledge-manifest-sha256')
    manifest_path = root / 'package_manifest.json'
    if digest(manifest_path) != manifest_sha:
        raise RuntimeError('Knowledge pack manifest hash mismatch')
    files = json.loads(manifest_path.read_text())['files']
    order, seen, active = [], set(), set()
    def visit(name):
        if name in seen:
            return
        if name in active:
            raise RuntimeError('Knowledge import cycle at ' + name)
        rel = name.replace('.', '/') + '.lean'
        path = root / 'formal' / rel
        if not path.is_file():
            if name == 'Mathlib' or name.startswith('Mathlib.'):
                return
            raise RuntimeError('Knowledge import is neither in the pack nor Mathlib: ' + name)
        expected = files.get('formal/' + rel)
        if expected is None or digest(path) != expected:
            raise RuntimeError('Knowledge source does not match pack manifest: formal/' + rel)
        active.add(name)
        for dep in IMPORT.findall(path.read_text()):
            visit(dep)
        active.discard(name)
        seen.add(name)
        order.append((name, rel, expected))
    visit(module)
    if not order or order[-1][0] != module:
        raise RuntimeError('Knowledge module not found in pack: ' + module)
    return order

def digest(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--isolation', choices=['sandbox-exec', 'linux-unsealed'], default='sandbox-exec')
    # Knowledge recipe: compile `--knowledge-module` and its pack-local imports from the pack's
    # sources inside staging. Every file must match the pinned package_manifest.json.
    parser.add_argument('--knowledge-root', type=Path)
    parser.add_argument('--knowledge-module')
    parser.add_argument('--knowledge-manifest-sha256')
    args = parser.parse_args()
    unsealed = args.isolation == 'linux-unsealed'
    project, source = args.project.resolve(), args.source.resolve()
    started = time.monotonic()
    result = dict(checked_modules=[], commands=[], elapsed_ms=0, status='ERROR', diagnostics=[], exit_code=None, stdout='', stderr='',
                  timed_out=False, peak_rss_bytes=0, sandboxed=False, toolchain_verified=False,
                  axioms=[], lean_version='', mathlib_revision='', source_hash=digest(source),
                  foundation_hash=digest(project / 'Foundation.lean'), independent_hash=digest(project / 'Independent.lean'),
                  isolation=args.isolation, isolation_diagnostics=[], knowledge_modules=[])
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
        sandbox = shutil.which('unshare' if unsealed else 'sandbox-exec')
        if not sandbox:
            raise RuntimeError('supported OS sandbox unavailable; refusing unisolated Lean')
        with tempfile.TemporaryDirectory(prefix='mldsn-lean-') as temporary:
            staging = Path(temporary).resolve()
            shutil.copyfile(project / 'Foundation.lean', staging / 'Foundation.lean')
            shutil.copyfile(source, staging / 'Generated.lean')
            shutil.copyfile(project / 'Independent.lean', staging / 'Independent.lean')
            modules = ['Foundation', 'Independent']
            if args.knowledge_module:
                closure = knowledge_closure(args.knowledge_root, args.knowledge_module, args.knowledge_manifest_sha256)
                for name, rel, sha in closure:
                    target = staging / rel
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(args.knowledge_root / 'formal' / rel, target)
                    if digest(target) != sha:
                        raise RuntimeError('Knowledge source changed while staging: ' + rel)
                    modules.append(name)
                result['knowledge_modules'] = [dict(module=n, source='formal/' + r, sha256=h) for n, r, h in closure]
            modules.append('Generated')
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
            for module in modules:
                path = module.replace('.', '/')
                lean_cmd = [str(lean), '-j', '1', '-M', '1536', '-T', '300',
                            '-o', str(staging / (path + '.olean')), str(staging / (path + '.lean'))]
                if unsealed:
                    # Inside private namespaces: keep staging writable, remount every other writable
                    # mount read-only, report anything that stayed writable, then exec lean.
                    cmd = [sandbox, '-r', '-n', '-m', 'sh', '-c', LINUX_ISOLATION, 'sh', str(staging)] + lean_cmd
                else:
                    cmd = [sandbox, '-f', str(profile)] + lean_cmd
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
                    # ru_maxrss is bytes on macOS and KiB on Linux.
                    scale = 1024 if os.uname().sysname == 'Linux' else 1
                    result['peak_rss_bytes'] = max(result['peak_rss_bytes'], int(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss) * scale)
                    if result['peak_rss_bytes'] > LIMIT:
                        result['timed_out'] = True
                        result['diagnostics'].append('RSS budget exceeded (OS peak measurement)')
                    out.seek(0); err.seek(0)
                    all_stdout.append(out.read().decode(errors='replace'))
                    all_stderr.append(err.read().decode(errors='replace'))
                if unsealed:
                    lines = all_stderr[-1].splitlines()
                    notes = [l for l in lines if l.startswith('mldsn-isolation:')]
                    all_stderr[-1] = ''.join(l + '\n' for l in lines if not l.startswith('mldsn-isolation:'))
                    if any('setup failed' in l for l in notes):
                        raise RuntimeError('Linux namespace isolation could not be created; refusing unisolated Lean')
                    result['isolation_diagnostics'].extend(module + ': ' + l for l in notes if 'writable' in l)
                elif 'sandbox_apply:' in all_stderr[-1]:
                    raise RuntimeError('OS denied sandbox creation; refusing unisolated Lean')
                # Unsealed runs are never reported as sandboxed, so they cannot become Evidence.
                result['sandboxed'] = not unsealed
                if result['exit_code'] != 0:
                    if module not in ('Foundation', 'Independent', 'Generated'):
                        # A pinned Knowledge module that does not compile is an infrastructure
                        # failure, never a verdict about the generated proof.
                        result['stdout'], result['stderr'] = ''.join(all_stdout), ''.join(all_stderr)
                        raise RuntimeError('Knowledge module failed to compile: ' + module)
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
        if unsealed and 'Linux namespace isolation' in str(error):
            result['status'] = 'ERROR'
    result['elapsed_ms'] = round((time.monotonic() - started) * 1000)
    print(json.dumps(result))

if __name__ == '__main__':
    main()
