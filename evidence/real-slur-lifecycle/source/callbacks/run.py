# SPDX-License-Identifier: AGPL-3.0-or-later
import pathlib,json,subprocess,os,hashlib,zipfile
out=pathlib.Path(__file__).parent;base=pathlib.Path('/workspace/native-build-tools/oracle');lib=base/'official/opt/audiveris/lib';book=pathlib.Path('/tmp/real-null-header-original-v1/der-winter-gray.omr');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();before=sha(book)
for n in ['classes','tmp','cache','home','prefs','config','data']:(out/n).mkdir(exist_ok=True)
sources=[pathlib.Path('/tmp/NativeSlurCandidateMathProbe.java'),out/'SlurLifecycleObserver.java',out/'SlurLifecycleRewriter.java']
compile=[str(base/'jdk-21.0.12.1+1/bin/javac'),'-source','21','-target','21','--add-exports','java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED','-d',str(out/'classes')]+list(map(str,sources))
with(out/'compile.log').open('w')as f:subprocess.run(compile,stdout=f,stderr=subprocess.STDOUT,check=True)
rewrite=[str(base/'jdk-21.0.12.1+1/bin/java'),'--add-exports','java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED','-cp',str(out/'classes'),'SlurLifecycleRewriter',str(lib/'app/audiveris.jar'),str(out/'instrumented'),str(out/'instrumentation.json'),str(out/'methods.txt')]
with(out/'rewrite.log').open('w')as f:subprocess.run(rewrite,stdout=f,stderr=subprocess.STDOUT,check=True)
cmd=[str(lib/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-cp',str(out/'instrumented')+os.pathsep+str(lib/'app/*')+os.pathsep+str(out/'classes'),'SlurLifecycleObserver',str(book),str(out/'observed.json'),str(out/'methods.txt')]
env=dict(os.environ)
for k,n in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[k]=str(out/n)
with(out/'console.log').open('w')as f:r=subprocess.run(cmd,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=90)
p={'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':sha(lib/'app/audiveris.jar'),'sources':[{'path':str(p),'sha256':sha(p)}for p in sources],'book':str(book),'book_sha256_before':before,'book_sha256_after':sha(book),'compile_argv':compile,'rewrite_argv':rewrite,'run_argv':cmd,'exit_code':r.returncode,'fresh_CURVES_invocations':1,'upstream_recognition_rerun':False,'native_recognition_invoked':False,'book_save_invoked':False}
if(out/'observed.json').exists():p['observed_sha256']=sha(out/'observed.json')
(out/'PROVENANCE.json').write_text(json.dumps(p,indent=2)+'\n');print(out,r.returncode);raise SystemExit(r.returncode)
