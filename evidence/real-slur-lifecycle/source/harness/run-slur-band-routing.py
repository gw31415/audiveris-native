from pathlib import Path
import json,hashlib,os,subprocess
out=Path('/tmp/slur-band-routing-original-v1');base=Path('/workspace/native-build-tools/oracle');lib=base/'official/opt/audiveris/lib';source=Path('/tmp/SlurBandRoutingProbe.java');helper=Path('/tmp/NativeSlurCandidateMathProbe.java');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for n in ['classes','tmp','cache','home','prefs','config','data']:(out/n).mkdir(exist_ok=True)
compile=[str(base/'jdk-21.0.12.1+1/bin/javac'),'--release','21','-d',str(out/'classes'),str(source),str(helper)]
with(out/'compile.log').open('w')as f:subprocess.run(compile,check=True,stdout=f,stderr=subprocess.STDOUT)
cmd=[str(lib/'runtime/bin/java'),'-Xmx256m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-cp',str(lib/'app/*')+os.pathsep+str(out/'classes'),'SlurBandRoutingProbe',str(out/'bands.tsv'),str(out/'queries.tsv'),str(out/'observed.json')]
env=dict(os.environ)
for key,folder in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[key]=str(out/folder)
with(out/'console.log').open('w')as f:r=subprocess.run(cmd,check=False,stdout=f,stderr=subprocess.STDOUT,env=env,timeout=60)
(out/'PROVENANCE.json').write_text(json.dumps({'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':sha(lib/'app/audiveris.jar'),'observer_sha256':sha(source),'helper_sha256':sha(helper),'runner_sha256':sha(Path(__file__)),'compile_argv':compile,'run_argv':cmd,'exit_code':r.returncode,'Book_loaded':False,'image_recognition_invoked':False,'Book_saved':False},indent=2)+'\n')
print('geometry only exit',r.returncode);raise SystemExit(r.returncode)
