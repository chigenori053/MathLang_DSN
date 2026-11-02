#!/usr/bin/env python3
"""Recount persisted raw results independently; never promote a missing case to PASS."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser()
p.add_argument('run', type=Path)
p.add_argument('--baseline', type=Path, required=True)
p.add_argument('--negative', type=Path, required=True)
p.add_argument('--semantics', type=Path, required=True)
p.add_argument('--rust-log', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
load = lambda x: json.loads(x.read_text())
digest = lambda x: hashlib.sha256(x).hexdigest()
compact = lambda x: json.dumps(x, ensure_ascii=False, separators=(',', ':')).encode()
manifest = load(a.run / 'artifact_manifest.json')
for name, expected in manifest.items():
    assert digest((a.run / name).read_bytes()) == expected, ('artifact mismatch', name)
for name, expected in load(a.run / 'source_manifest.json').items():
    assert digest((ROOT / name).read_bytes()) == expected, ('source changed', name)
obligations = load(a.run / 'proof_obligations.json')
by_id = {o['id']: o for o in obligations}
assert len(by_id) == len(obligations), 'duplicate obligation'
evidence = load(a.run / 'proof_results.json')
counts = Counter()
expected = {'MV-01': 'PROVED', 'MV-02': 'REFUTED', 'MV-03': 'PROVED',
            'MV-04': 'PROVED', 'MV-05': 'PROVED', 'MV-06': 'PROVED',
            'CV-UPDATE': 'PROVED', 'CV-PHASE': 'PROVED', 'CV-PHASE-COLLISION': 'REFUTED'}
max_rss, max_ms = 0, 0
for e in evidence:
    o = by_id[e['obligation_id']]
    assert e['model_id'] == o['model_id'] and e['model_revision'] == o['model_revision']
    source_tuple = [o['model_id'], o['model_revision'], o['goal'], o['context'], o['assumptions'],
                    o['required_lemmas'], o['dependencies'], o['kind']]
    assert digest(compact(source_tuple)) == e['source_ir_hash'], 'IR target mismatch'
    original_id = e['evidence_id']
    copy = dict(e); copy['evidence_id'] = ''
    assert digest(compact(copy)) == original_id, 'evidence content hash'
    directory = a.run / 'registry' / original_id
    assert load(directory / 'evidence.json') == e
    for name, sha in e['artifact_hashes'].items():
        assert digest((directory / name).read_bytes()) == sha
    kernel = e['toolchain']
    assert kernel['sandboxed'] and kernel['toolchain_verified'] and kernel['exit_code'] == 0
    assert kernel['checked_modules'] == ['Foundation', 'Independent', 'Generated']
    assert not kernel['timed_out'] and kernel['peak_rss_bytes'] <= 2 * 1024**3 and kernel['elapsed_ms'] <= 300_000
    report = re.search(r"'Generated.target' (?:depends on axioms: \[([^\]]*)\]|does not depend on any axioms)", kernel['stdout'], re.S)
    assert report, 'missing exact target axiom audit'
    axioms = [s.strip() for s in (report.group(1) or '').split(',') if s.strip()]
    assert axioms == kernel['axioms'] and set(axioms) <= {'propext', 'Classical.choice', 'Quot.sound'}
    assert e['proof_result'] == kernel['status'] == expected[o['id']]
    assert e['translation']['source_ir_hash'] == e['source_ir_hash']
    assert e['translation']['source_model_revision'] == o['model_revision']
    assert e['lean_source_hash'] == kernel['source_hash'] == e['artifact_hashes']['Generated.lean']
    if o['id'].startswith('MV-'):
        assert 'STRUCTURAL_AND_INDEPENDENT_KERNEL_CONTRACT' == e['translation']['validation_status']
        assert 'example : Independent.mv' in (directory / 'Generated.lean').read_text()
    counts[o['id']] += 1
    max_rss, max_ms = max(max_rss, kernel['peak_rss_bytes']), max(max_ms, kernel['elapsed_ms'])
assert set(counts) == set(by_id) == set(expected)
assert all(n == 3 for n in counts.values())
models=load(a.run/'model_registry.json')['models']
for model in load(a.run/'model_hashes.json'):
    assert digest(compact(models[model['id']][str(model['revision'])])) == model['content_hash']
assert not load(a.run/'rejected_typecheck.json')['evidence']['valid']
assert all(c['assumptions_satisfied'] and c['conclusion_false'] for c in load(a.run/'exact_candidates.json')['candidates'])
repeat = load(a.run / 'repeatability.json')
assert repeat['identical'] and repeat['repetitions'] == 3
assert repeat['runs'][0] == repeat['runs'][1] == repeat['runs'][2]
baseline = load(a.baseline / 'summary.json')
assert baseline['passed'] and baseline['source_unchanged']
for name, sha in load(a.baseline / 'source_manifest.json').items():
    assert digest((ROOT.parent / name).read_bytes()) == sha, ('existing source changed', name)
for name, sha in load(a.baseline / 'artifact_manifest.json').items():
    assert digest((a.baseline / name).read_bytes()) == sha
negative = load(a.negative / 'commands.json')
assert len(negative) == 6 and all(x['status'] == 'PASS' for x in negative)
for name in ['LV-05-false', 'LV-06-incomplete']:
    raw = load(a.negative / name / 'proof_results.json')
    assert raw['proof_result'] == 'UNKNOWN' and raw['toolchain']['exit_code'] != 0
    assert raw['toolchain']['checked_modules'][:2] == ['Foundation', 'Independent']
for name in ['LV-07-axiom', 'LV-06-sorry', 'LV-09-weakened', 'LV-SAFE-fork']:
    raw = load(a.negative / (name + '.result.json'))
    assert raw['sandboxed'] and raw['toolchain_verified']
    assert digest((a.negative / (name + '.lean')).read_bytes()) == raw['source_hash']
    if name == 'LV-07-axiom':
        assert raw['exit_code'] == 0 and raw['status'] == 'ERROR'
        assert set(raw['axioms']) - {'propext', 'Classical.choice', 'Quot.sound'}
    else:
        assert raw['exit_code'] != 0 and raw['status'] in ['UNKNOWN', 'ERROR']
    if name == 'LV-06-sorry':
        assert 'sorryAx' in raw['axioms'] or 'sorry' in raw['stdout']
    if name == 'LV-SAFE-fork':
        assert 'sandbox-escape' not in raw['stdout'].splitlines()

semantics = load(a.semantics / 'kernel.json')
assert semantics['status'] == 'PROVED' and semantics['exit_code'] == 0 and semantics['sandboxed']
assert semantics['peak_rss_bytes'] <= 2 * 1024**3 and semantics['elapsed_ms'] <= 300_000
assert 'sorryAx' not in semantics['stdout']
assert digest((a.semantics / 'Generated.lean').read_bytes()) == semantics['source_hash']
semantic_cases = load(a.semantics / 'cases.json')
for i in range(len(semantic_cases)):
    assert f"'Generated.case_{i}'" in semantics['stdout']
rust = a.rust_log.read_text()
assert '33 passed; 0 failed' in rust
assert 'FAILED' not in rust
phases = {phase: 'PASS' for phase in ['P0', 'P1', 'P2', 'P3', 'P4', 'P5']}
summary = dict(specification='MLDSN-MVF-001 v0.1', phases=phases, overall='PASS',
               scope='Bounded v0.1 mathematical IR and representative models; see IR specification and final report.',
               raw_result_count=len(evidence), obligations=dict(counts), expected_results=expected,
               repeatability=True, false_proved=0, missing_obligations=0,
               peak_rss_bytes=max_rss, maximum_case_ms=max_ms,
               independent_translation_cases=len(semantic_cases), negative_cases=len(negative),
               rust_tests_passed=33, existing_rust_tests_passed=baseline['rust_tests_passed'],
               baseline=str(a.baseline), run=str(a.run), negative=str(a.negative), semantics=str(a.semantics),
               artifact_manifest_sha256=digest((a.run / 'artifact_manifest.json').read_bytes()))
a.output.parent.mkdir(parents=True, exist_ok=True)
with a.output.open('x') as f:
    json.dump(summary, f, ensure_ascii=False, indent=2)
print(json.dumps(summary, ensure_ascii=False, indent=2))
