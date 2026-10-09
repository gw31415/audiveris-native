from pathlib import Path
import os,subprocess,json,time,hashlib
root=Path('/workspace/audiveris-native');out=Path('/tmp/slur-lifecycle-v4');out.mkdir()
env=os.environ.copy();env.update(CARGO_HOME='/workspace/native-build-tools/cargo',RUSTUP_HOME='/workspace/native-build-tools/rustup',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0');env['PATH']='/workspace/native-build-tools/cargo/bin:'+env['PATH']
files=set(subprocess.check_output(['git','ls-files','*.rs'],cwd=root,text=True).splitlines())|set(n for n in subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=root,text=True).splitlines() if n.endswith('.rs'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();sources={n:sha(root/n) for n in sorted(files)}
checks=[]
for name,args in [('fmt',['cargo','fmt','--all','--','--check']),('clippy',['cargo','clippy','--offline','--locked','-j2','--all-targets','--','-D','warnings']),('compile',['cargo','test','--offline','--locked','-j2','--lib','--no-run'])]:
 t=time.monotonic()
 with (out/(name+'.log')).open('x') as log:r=subprocess.run(args,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=600)
 checks.append(dict(check=name,argv=args,exit_code=r.returncode,elapsed_seconds=time.monotonic()-t));(out/'CHECKS.json').write_text(json.dumps(checks,indent=2)+'\n');print(json.dumps(checks[-1]),flush=True)
 if r.returncode:raise SystemExit(r.returncode)
binary=root/'target/debug/deps/audiveris_native-5fcd8c0ec93be988';assert all(sha(root/n)==h for n,h in sources.items())
listing=subprocess.check_output([str(binary),'--list'],cwd=root,text=True)
names=[line.removesuffix(': test') for line in listing.splitlines() if line.endswith(': test')]
names=[n for n in names if n.endswith('registered_stem_batch_preserves_order_identity_and_rejects_unrequested_stale_cache')]
assert len(names)==1,names
args=[str(binary),'--exact',*names,'--test-threads=2','--nocapture'];t=time.monotonic()
with (out/'scalar.log').open('x') as log:r=subprocess.run(args,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=300)
fast=dict(argv=args,binary_sha256=sha(binary),targets=len(names),exit_code=r.returncode,elapsed_seconds=time.monotonic()-t);(out/'FAST-RESULT.json').write_text(json.dumps(fast,indent=2)+'\n');print(json.dumps(fast),flush=True)
if r.returncode:raise SystemExit(r.returncode)
names=['music_tuplets::tests::real_image_linked_six_callbacks_and_later_weak_purge_match_original']

args=[str(binary),'--exact',*names,'--test-threads=2','--nocapture'];t=time.monotonic();record=dict(argv=args,start_time=time.time(),source_sha256=sources,binary_sha256=sha(binary),fast_targets=fast['targets'],previous_unique_fast_passed=13,unique_targets=17)
(out/'START.json').write_text(json.dumps(record,indent=2)+'\n');print('Starting only previously failing DerWinter target; AnLaura/rational v2 unchanged XML evidence retained',flush=True)
with (out/'console.log').open('x') as log:
 try:r=subprocess.run(args,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=1800);code=r.returncode
 except subprocess.TimeoutExpired:code=124
record.update(exit_code=code,elapsed_seconds=time.monotonic()-t,source_stable=all(sha(root/n)==h for n,h in sources.items()));(out/'RESULT.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k] for k in ['exit_code','elapsed_seconds','unique_targets','source_stable']}),flush=True);raise SystemExit(code)
