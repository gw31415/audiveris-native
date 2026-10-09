import pathlib,json,subprocess,os,hashlib
base=pathlib.Path('/workspace/native-build-tools/oracle');lib=base/'official/opt/audiveris/lib';out=pathlib.Path('/tmp/slur-native-clump-original-v1');book=pathlib.Path('/tmp/real-rest-original-v1/der-winter-gray.omr');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();before=sha(book)
for n in ['classes','tmp','cache','home','prefs','config','data']:(out/n).mkdir(exist_ok=True)
sources=[pathlib.Path('/tmp')/f for f in ['NativeSlurCandidateMathProbe.java','SlurOwnedLookupProbeV2.java','SlurNativeClumpProbe.java']]
compile=[str(base/'jdk-21.0.12.1+1/bin/javac'),'--release','21','-d',str(out/'classes')]+list(map(str,sources))
with(out/'compile.log').open('w')as f:subprocess.run(compile,stdout=f,stderr=subprocess.STDOUT,check=True)
inputbase=pathlib.Path('/tmp/slur-native-curve-lookup-original-v1')
cmd=[str(lib/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-cp',str(lib/'app/*')+os.pathsep+str(out/'classes'),'SlurNativeClumpProbe',str(book),str(inputbase/'curves.tsv'),str(inputbase/'sequences.tsv'),str(out/'groups.tsv'),str(out/'observed.json')]
env=dict(os.environ)
for k,n in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[k]=str(out/n)
with(out/'console.log').open('w')as f:r=subprocess.run(cmd,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=60)
p={'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':sha(lib/'app/audiveris.jar'),'sources':[{'path':str(p),'sha256':sha(p)} for p in sources],'runner_sha256':sha(pathlib.Path(__file__)),'book_sha256_before':before,'book_sha256_after':sha(book),'compile_argv':compile,'run_argv':cmd,'exit_code':r.returncode,'image_recognition_invoked':False,'book_save_invoked':False}
if(out/'observed.json').exists():p['observed_sha256']=sha(out/'observed.json')
(out/'PROVENANCE.json').write_text(json.dumps(p,indent=2)+'\n');print(out,r.returncode);raise SystemExit(r.returncode)
