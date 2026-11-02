#!/usr/bin/env python3
"""Record original commands and byte hashes separately from new verification tests."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser()
p.add_argument('output', type=Path)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)

def snapshot():
    files = []
    for directory in ['mirp/src', 'mirp/tests', 'mirp/knowledge', 'mathlang_dsn', 'src']:
        files += [x for x in (ROOT / directory).rglob('*') if x.is_file()]
    files += [ROOT / 'mirp/Cargo.toml', ROOT / 'mirp/Cargo.lock']
    return {str(x.relative_to(ROOT)): hashlib.sha256(x.read_bytes()).hexdigest() for x in sorted(files)}

before = snapshot()
(a.output / 'source_manifest.json').write_text(json.dumps(before, indent=2))
(a.output / 'environment.json').write_text(json.dumps({
    'timestamp': datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat(),
    'platform': platform.platform(),
    'versions': {cmd: subprocess.run([cmd, '--version'], capture_output=True, text=True).stdout.strip()
                 for cmd in ['rustc', 'cargo', 'reason', 'lean', 'lake']}
}, indent=2))
commands = [['cargo', 'test', '--manifest-path', 'mirp/Cargo.toml', '--offline'],
            ['reason', 'check'], ['reason', 'project-validate', '.', '--json']]
commands += [['reason', 'run', '--entry', name, '--json', '--trace=off'] for name in
             ['SelfCheck', 'PolynomialSelfCheck', 'KnowledgeSelfCheck', 'JuniorHighSelfCheck', 'QuadraticRuntimeSelfCheck']]
results = []
for i, cmd in enumerate(commands):
    start = time.monotonic()
    run = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=900)
    (a.output / f'{i:02}.stdout.log').write_text(run.stdout)
    (a.output / f'{i:02}.stderr.log').write_text(run.stderr)
    passed = run.returncode == 0
    if cmd[0:2] == ['reason', 'run']:
        passed = passed and json.loads(run.stdout).get('runtime_result', {}).get('result') is True
    if cmd[0:2] == ['reason', 'project-validate']:
        passed = passed and json.loads(run.stdout).get('status') == 'passed'
    results.append(dict(command=cmd, exit_code=run.returncode, passed=passed, seconds=time.monotonic()-start,
                        stdout=f'{i:02}.stdout.log', stderr=f'{i:02}.stderr.log'))
    print(json.dumps(results[-1]), flush=True)
    (a.output / 'commands.json').write_text(json.dumps(results, indent=2))
after = snapshot()
(a.output / 'summary.json').write_text(json.dumps(dict(
    passed=all(r['passed'] for r in results), source_unchanged=before == after,
    rust_tests_passed=sum(int(line.split('test result: ok. ')[1].split(' passed')[0])
        for line in (a.output / '00.stdout.log').read_text().splitlines() if 'test result: ok.' in line)
), indent=2))
manifest = {x.name: hashlib.sha256(x.read_bytes()).hexdigest() for x in sorted(a.output.iterdir()) if x.is_file()}
(a.output / 'artifact_manifest.json').write_text(json.dumps(manifest, indent=2))
raise SystemExit(0 if all(r['passed'] for r in results) and before == after else 1)
