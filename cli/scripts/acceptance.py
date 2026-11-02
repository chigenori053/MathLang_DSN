#!/usr/bin/env python3
"""CLI integration with original stdout/stderr, immutable fixtures and command records.
Run on supported macOS with existing Lean sandbox permissions. No installation/network.
"""
import argparse, hashlib, json, os, shutil, subprocess, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('output',type=Path);p.add_argument('--binary',type=Path,default=ROOT/'cli/target/debug/mathlang');a=p.parse_args()
OUT=a.output.resolve();OUT.mkdir(parents=True,exist_ok=False);BIN=a.binary.resolve();records=[]
def save(name,v): (OUT/name).write_text(json.dumps(v,indent=2,ensure_ascii=False))
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(label,args,code=0,env=None,parse=True,binary=BIN):
 cmd=[str(binary),*map(str,args)];started=time.monotonic();r=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=310,env=env)
 (OUT/f'{label}.stdout.log').write_text(r.stdout);(OUT/f'{label}.stderr.log').write_text(r.stderr)
 record=dict(label=label,command=cmd,exit_code=r.returncode,expected_exit_code=code,passed=r.returncode==code,elapsed_seconds=time.monotonic()-started,stdout=f'{label}.stdout.log',stderr=f'{label}.stderr.log',path_override=env.get('PATH') if env else None);records.append(record);save('commands.json',records)
 assert record['passed'],(label,r.returncode,r.stdout,r.stderr)
 if parse:
  v=json.loads(r.stdout);assert isinstance(v,dict) and v['schema_version']=='mathlang.cli/0.1';return v
 return r.stdout
run('help',['--help'],parse=False);run('version',['--version'],parse=False)
run('bad-args',['unknown','--format','json'],2)
registry=json.loads((ROOT/'cli/evidence/formal-legacy/model_registry.json').read_text());m=registry['models']['USM-MVF']['1'];save('model.json',m);model=OUT/'model.json';original=digest(model)
for op in ['check','inspect','normalize']:run('model-'+op,['model',op,model,'--format','json'])
for i in range(3):run('repeat-'+str(i),['model','normalize',model,'--format','json'])
repeat=[json.loads((OUT/f'repeat-{i}.stdout.log').read_text())['result'] for i in range(3)];assert repeat[0]==repeat[1]==repeat[2]
run('text',['model','inspect',model,'--format','text'],parse=False)
run('quiet-json',['model','check',model,'--quiet','--verbose','--format','json'])
config=OUT/'config.toml';config.write_text('[cli]\noutput_format="json"\n[runtime]\nmode="local"\n[verification]\nmax_rss_mib=2048\ntimeout_seconds=300\nparallelism=1\n')
run('config',['model','check',model,'--config',config]);run('config-override',['model','check',model,'--config',config,'--format','text'],parse=False)
config.write_text('[verification]\nmax_rss_mib=4096\n');run('bad-config',['doctor','--config',config,'--format','json'],4)
doctor=run('doctor',['doctor','--format','json']);assert doctor['result']['readiness'] in ['READY','PARTIAL','NOT_READY']
bad=json.loads(json.dumps(m));bad['proof_obligations'][0]['goal']={'Variable':'not_bound'};save('bad-model.json',bad);run('bad-model',['model','check',OUT/'bad-model.json','--format','json'],3)
bad['proof_obligations'][0]['goal']={'UnsupportedOperation':[]};save('unsupported.json',bad);run('unsupported',['model','check',OUT/'unsupported.json','--format','json'],4)
(OUT/'bad.json').write_text('{broken');run('bad-json',['model','check',OUT/'bad.json','--format','json'],2)
run('missing-input',['model','check',OUT/'missing.json','--format','json'],8)
run('proof-plan',['proof','plan',ROOT/'verification/fixtures/MV-01.json','--format','json'])
o=json.loads((ROOT/'verification/fixtures/MV-03.json').read_text())['obligation'];o.update(id='CLI-FINITE',context={'variables':{},'assumptions':[]},assumptions=[],goal={'ForAll':['x','Bool',{'Equality':[{'Variable':'x'},{'Constant':{'Bool':True}}]}]});save('finite.json',o)
c=run('candidate',['proof','counterexample',OUT/'finite.json','--format','json']);assert c['formal_status']=='UNKNOWN' and len(c['result']['candidates'])==1
large=json.loads(json.dumps(o));large['goal']['ForAll'][1]={'Finite':1001};large['goal']['ForAll'][2]={'Equality':[{'Variable':'x'},{'Variable':'x'}]};save('budget.json',large);run('budget',['proof','counterexample',OUT/'budget.json','--format','json'],6)
run('knowledge',['knowledge','inspect',ROOT/'mirp/knowledge/legacy.json','--format','json'])
run('impact',['knowledge','impact',ROOT/'cli/evidence/formal-legacy/model_registry.json','--revision','1','--format','json'])
run('knowledge-update',['knowledge','update',model,'--format','json'],4)
run('compare-same',['compare',model,model,'--format','json'])
b=json.loads(json.dumps(m));b['proof_obligations'][0]['goal']={'Equality':[{'Constant':{'Natural':1}},{'Constant':{'Natural':2}}]};save('different.json',b)
v=run('compare-different',['compare',model,OUT/'different.json','--format','json']);assert v['result']['comparability']=='NOT_COMPARABLE' and v['result']['ranking'] is None
for label,goal,recipe in [('false',{'Equality':[{'Constant':{'Natural':1}},{'Constant':{'Natural':2}}]},'Auto'),('incomplete',{'Equality':[{'Constant':{'Natural':1}},{'Constant':{'Natural':1}}]},'Incomplete')]:
 req=dict(obligation=json.loads(json.dumps(o)),recipe=recipe,refutation=False);req['obligation']['goal']=goal;save(label+'.json',req)
 v=run(label,['verify',OUT/(label+'.json'),'--output',OUT/(label+'-evidence'),'--format','json']);assert v['formal_status']=='UNKNOWN'
v=run('refuted',['verify',ROOT/'verification/fixtures/MV-02.json','--output',OUT/'refuted-evidence','--format','json']);assert v['formal_status']=='REFUTED'
run('collision',['verify',ROOT/'verification/fixtures/MV-02.json','--output',OUT/'refuted-evidence','--format','json'],8)
directory=next((OUT/'refuted-evidence/registry').iterdir())
for op in ['show','validate']:run('evidence-'+op,['evidence',op,directory,'--format','json'])
newreg=json.loads(json.dumps(registry));revision=json.loads(json.dumps(m));revision['revision']=2
for o2 in revision['proof_obligations']:o2['model_revision']=2
newreg['models']['USM-MVF']['2']=revision;save('stale-registry.json',newreg)
run('stale',['evidence','validate',directory,'--registry',OUT/'stale-registry.json','--format','json'],7)
(directory/'Generated.lean').write_text('tampered');run('tampered',['evidence','validate',directory,'--format','json'],7)
run('missing-evidence',['evidence','validate',OUT/'nonexistent','--format','json'],7)
(OUT/'symlink').symlink_to(OUT,target_is_directory=True);run('symlink',['model','check',model,'--output',OUT/'symlink/output.json','--format','json'],8)
# The controlled missing-runner environment must retain ERROR evidence, never PROVED.
env=dict(os.environ,PATH='/unavailable');v=run('missing-runner',['verify',ROOT/'verification/fixtures/MV-01.json','--output',OUT/'missing-runner-evidence','--format','json'],5,env=env);assert v['formal_status']=='ERROR'
problem='Solve x^2 - 5x + 6 = 0.'
new=run('solve',['solve',problem,'--format','json'])
old=run('old-solve',['solve-problem',problem],parse=False,binary=ROOT/'mirp/target/debug/mathlang-mirp');old=json.loads(old)
# Runtime-generated evidence IDs vary; retain and compare the actual answer, status, semantic state and full plan.
assert new['result']['result']['answer']==old['result']['answer']==[2,3]
assert new['result']['result']['status']==old['result']['status']
assert new['result']['plan']==old['plan']
assert new['result']['semantic_state']==old['semantic_state']
assert digest(model)==original
save('regression_results.json',dict(solver_answer=True,solver_status=True,solver_plan=True,semantic_state=True,model_normalization_repetitions=3,input_unchanged=True))
save('negative_results.json',[r for r in records if r['expected_exit_code']!=0 or r['label'] in ['false','incomplete','candidate']])
save('test_results.json',dict(passed=all(r['passed'] for r in records),commands=len(records),doctor_readiness=doctor['result']['readiness']))
save('environment.json',dict(binary=str(BIN),binary_sha256=digest(BIN),platform=os.uname().sysname))
save('source_manifest.json',{str(x.relative_to(ROOT)):digest(x) for folder in ['cli/src','cli/tests','cli/scripts'] for x in sorted((ROOT/folder).rglob('*')) if x.is_file()})
save('artifact_manifest.json',{str(x.relative_to(OUT)):digest(x) for x in sorted(OUT.rglob('*')) if x.is_file() and not x.is_symlink()})
print(json.dumps(dict(passed=True,commands=len(records))))
