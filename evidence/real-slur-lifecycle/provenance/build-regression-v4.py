from pathlib import Path
import os,json,hashlib,subprocess,time
root=Path('/workspace/audiveris-native');out=Path('/tmp/slur-lifecycle-v4-build.json');assert not out.exists()
env=os.environ.copy();env.update(CARGO_HOME='/workspace/native-build-tools/cargo',RUSTUP_HOME='/workspace/native-build-tools/rustup',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0');env['PATH']='/workspace/native-build-tools/cargo/bin:'+env['PATH']
argv=['cargo','build','--offline','--locked','-j2','--bin','audiveris-musicxml'];start=time.monotonic()
with Path('/tmp/slur-lifecycle-v4-build.log').open('x') as log:p=subprocess.run(argv,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=600)
v={'argv':argv,'exit_code':p.returncode,'seconds':time.monotonic()-start}
if not p.returncode:v['binary_sha256']=hashlib.sha256((root/'target/debug/audiveris-musicxml').read_bytes()).hexdigest()
out.write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v),flush=True)
if p.returncode:raise SystemExit(p.returncode)
argv=['python3','scripts/check_retained_musicxml_regression.py','--native-bin','target/debug/audiveris-musicxml','--schema-dir','/workspace/native-build-tools/musicxml4','--output','/tmp/slur-lifecycle-v4-cli-regression']
with Path('/tmp/slur-lifecycle-v4-cli.log').open('x') as log:p=subprocess.run(argv,cwd=root,stdout=log,stderr=subprocess.STDOUT,timeout=600)
print('CLI regression exit',p.returncode,flush=True);raise SystemExit(p.returncode)
