from pathlib import Path
import hashlib,json,os,subprocess
out=Path(__file__).resolve().parent
base=Path('/workspace/native-build-tools/oracle')
lib=base/'official/opt/audiveris/lib'
source=out/'SlurGapObstacleProbe.java'
helper=Path('/tmp/NativeSlurCandidateMathProbe.java')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for name in ['classes','tmp','cache','home','prefs','config','data']:(out/name).mkdir(exist_ok=True)
compile=[str(base/'jdk-21.0.12.1+1/bin/javac'),'--release','21','-d',str(out/'classes'),str(source),str(helper)]
with (out/'compile.log').open('w') as log:subprocess.run(compile,check=True,stdout=log,stderr=subprocess.STDOUT)
cmd=[str(lib/'runtime/bin/java'),'-Xmx256m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-cp',str(lib/'app/*')+os.pathsep+str(out/'classes'),'SlurGapObstacleProbe',str(out/'inputs.tsv'),str(out/'observed.json')]
env=dict(os.environ)
for key,folder in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[key]=str(out/folder)
with (out/'console.log').open('w') as log:r=subprocess.run(cmd,check=False,stdout=log,stderr=subprocess.STDOUT,env=env,timeout=60)
(out/'PROVENANCE.json').write_text(json.dumps({'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':sha(lib/'app/audiveris.jar'),'observer_sha256':sha(source),'helper_sha256':sha(helper),'input_sha256':sha(out/'inputs.tsv'),'compile_argv':compile,'run_argv':cmd,'exit_code':r.returncode,'Book_loaded':False,'image_recognition_invoked':False,'Book_saved':False,'native_run':False},indent=2)+'\n')
print('scalar-only original exit',r.returncode)
raise SystemExit(r.returncode)
