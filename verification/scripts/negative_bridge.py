#!/usr/bin/env python3
"""Actual negative Lean executions; retain every source, result and failure log."""
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('output', type=Path)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
results = []
base = json.loads((ROOT / 'fixtures/MV-03.json').read_text())
for name, goal, recipe in [
    ('LV-05-false', {'Equality': [{'Constant': {'Natural': 1}}, {'Constant': {'Natural': 2}}]}, 'Auto'),
    ('LV-06-incomplete', {'Equality': [{'Constant': {'Natural': 1}}, {'Constant': {'Natural': 1}}]}, 'Incomplete')
]:
    req = json.loads(json.dumps(base))
    req['obligation']['id'] = name
    req['obligation']['goal'] = goal
    req['recipe'] = recipe
    request = a.output / (name + '.json')
    request.write_text(json.dumps(req, indent=2))
    cmd = [str(ROOT / 'target/debug/mathlang-verification'), 'verify-obligation', str(request), str(a.output / name)]
    run = subprocess.run(cmd, capture_output=True, text=True, timeout=320)
    (a.output / (name + '.stdout.log')).write_text(run.stdout)
    (a.output / (name + '.stderr.log')).write_text(run.stderr)
    assert run.returncode == 0, run.stderr
    evidence = json.loads((a.output / name / 'proof_results.json').read_text())
    assert evidence['toolchain']['checked_modules'][:2] == ['Foundation', 'Independent'], evidence
    assert evidence['proof_result'] == 'UNKNOWN', evidence
    results.append(dict(test=name, command=cmd, status='PASS', observed=evidence['proof_result']))

modules = {
    'LV-07-axiom': 'public axiom poisoned : False\npublic theorem target : False := poisoned\n',
    'LV-06-sorry': 'public theorem target : False := by sorry\n',
    'LV-09-weakened': 'public theorem target : True := True.intro\nexample : Independent.mv01 := Generated.target\n',
    'LV-SAFE-fork': '#eval IO.Process.output {cmd := "/bin/echo", args := #["sandbox-escape"]}\npublic theorem target : True := True.intro\n',
}
for name, text in modules.items():
    source = a.output / (name + '.lean')
    source.write_text('module\npublic import Foundation\npublic import Independent\n'
                      'set_option warningAsError true\nnamespace Generated\n' + text +
                      '#print axioms Generated.target\nend Generated\n')
    cmd = ['python3', str(ROOT / 'scripts/lean_runner.py'), '--project', str(ROOT / 'lean'), '--source', str(source)]
    run = subprocess.run(cmd, capture_output=True, text=True, timeout=320)
    (a.output / (name + '.result.json')).write_text(run.stdout)
    (a.output / (name + '.stderr.log')).write_text(run.stderr)
    assert run.returncode == 0, run.stderr
    raw = json.loads(run.stdout)
    assert raw['sandboxed'] and raw['toolchain_verified'], raw
    assert raw['status'] in ['ERROR', 'UNKNOWN'], raw
    if name == 'LV-07-axiom':
        assert raw['exit_code'] == 0 and any(x.split('.')[-1] == 'poisoned' for x in raw['axioms']), raw
    elif name == 'LV-SAFE-fork':
        assert raw['exit_code'] != 0 and 'sandbox-escape' not in raw['stdout'].splitlines(), raw
    else:
        assert raw['exit_code'] != 0, raw
    results.append(dict(test=name, command=cmd, status='PASS', observed=raw['status'], axioms=raw['axioms']))
(a.output / 'commands.json').write_text(json.dumps(results, indent=2))
print(json.dumps(results, indent=2))
