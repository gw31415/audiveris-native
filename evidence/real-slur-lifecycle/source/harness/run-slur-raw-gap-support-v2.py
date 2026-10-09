import pathlib,json,subprocess,os,hashlib
root=pathlib.Path('/workspace/native-build-tools/oracle');lib=root/'official/opt/audiveris/lib';out=pathlib.Path('/tmp/native-slur-raw-gap-refinement-v2');source=pathlib.Path('/tmp/NativeSlurRawGapSupportProbe.java');helper=pathlib.Path('/tmp/NativeSlurCandidateMathProbe.java');book=pathlib.Path('/tmp/real-rest-original-v1/der-winter-gray.omr');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();before=sha(book)
for n in ['classes','tmp','cache','home','prefs','config','data']:(out/n).mkdir()
compile=[str(root/'jdk-21.0.12.1+1/bin/javac'),'--release','21','-d',str(out/'classes'),str(source),str(helper)]
with (out/'compile.log').open('w')as f:subprocess.run(compile,stdout=f,stderr=subprocess.STDOUT,check=True)
cmd=[str(lib/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-cp',str(lib/'app/*')+os.pathsep+str(out/'classes'),'NativeSlurRawGapSupportProbe',str(book),str(out/'sequences.tsv'),str(out/'observed.json'),str(out/'original-Java-BINARY.raw'),'/tmp/native-slur-binary-support-v1/native-glyph-union.raw']
env=dict(os.environ)
for k,n in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[k]=str(out/n)
with (out/'console.log').open('w')as f:r=subprocess.run(cmd,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=60)
p={'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':sha(lib/'app/audiveris.jar'),'observer_sha256':sha(source),'math_helper_sha256':sha(helper),'original_book_sha256_before':before,'original_book_sha256_after':sha(book),'sequences_sha256':sha(out/'sequences.tsv'),'compile_argv':compile,'run_argv':cmd,'exit_code':r.returncode,'image_recognition_invoked':False,'no_original_coordinates_used_for_selection':True}
for n in ['observed.json','original-Java-BINARY.raw','native-BINARY.raw','native-glyph-union.raw']:
 if(out/n).exists():p[n+'_sha256']=sha(out/n)
p['java_raw_equals_isolated_native_raw']=p.get('original-Java-BINARY.raw_sha256')==sha(pathlib.Path('/tmp/native-slur-binary-support-v1/native-BINARY.raw'))
(out/'provenance.json').write_text(json.dumps(p,indent=2)+'\n');print(str(out),r.returncode);raise SystemExit(r.returncode)
