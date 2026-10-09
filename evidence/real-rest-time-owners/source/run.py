# SPDX-License-Identifier: AGPL-3.0-or-later
import hashlib,json,os,pathlib,subprocess,time
out=pathlib.Path(__file__).parent
base=pathlib.Path('/workspace/native-build-tools/oracle'); lib=base/'official/opt/audiveris/lib'
book=pathlib.Path('/tmp/real-seven-label-original-v1/laura-gray.omr'); jar=lib/'app/audiveris.jar'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
before={str(p):sha(p) for p in (book,jar)}
for n in ('classes','tmp','cache','home','prefs','config','data'): (out/n).mkdir(exist_ok=True)
exports=['--add-exports','java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED','--add-exports','java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED','--add-exports','java.base/jdk.internal.org.objectweb.asm.util=ALL-UNNAMED']
sources=[out/n for n in ('PedalRawStaffBookProbe.java','RestRemovalClockProbe.java','RestRemovalRewriter.java','VerifyRestOverlay.java')]
compile=[str(base/'jdk-21.0.12.1+1/bin/javac'),'-source','21','-target','21']+exports+['-d',str(out/'classes')]+list(map(str,sources))
commands=[]
def run(cmd,log,env=None,timeout=90):
    start=time.monotonic()
    with (out/log).open('w') as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,env=env,timeout=timeout)
    commands.append({'argv':cmd,'log':log,'exit_code':r.returncode,'elapsed_seconds':time.monotonic()-start})
    if r.returncode: raise RuntimeError(f'{log}: {r.returncode}')
try:
    run(compile,'compile.log')
    java21=str(base/'jdk-21.0.12.1+1/bin/java')
    run([java21]+exports+['-cp',str(out/'classes'),'RestRemovalRewriter',str(jar),str(out/'instrumented'),str(out/'instrumentation.json'),str(out/'methods.txt')],'rewrite.log')
    run([java21]+exports+['-cp',str(out/'classes'),'VerifyRestOverlay',str(jar),str(out/'instrumented'),str(out/'instrumentation-audit.json')],'audit.log')
    for label,numerator in [('underfill',3),('excess',1)]:
        private=out/label
        for n in ('tmp','cache','home','prefs','config','data'):(private/n).mkdir(parents=True,exist_ok=True)
        cmd=[str(lib/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(private/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(private/'cache'),'-Djava.util.prefs.userRoot='+str(private/'prefs'),'-Duser.home='+str(private/'home'),'-cp',str(out/'instrumented')+os.pathsep+str(lib/'app/*')+os.pathsep+str(out/'classes'),'RestRemovalClockProbe',str(book),str(out/(label+'-observed.json')),str(numerator)]
        env=dict(os.environ)
        for k,n in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache')]:env[k]=str(private/n)
        run(cmd,label+'-console.log',env)
finally:
    p={'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','scope':'Declared original method controls on actual reloaded SYMBOLS owners, not fresh recognition or normal LINKS clocks. No Java rollback claim.','inputs_before':before,'inputs_after':{str(p):sha(p)for p in (book,jar)},'sources':[{'path':str(p),'sha256':sha(p)}for p in sources],'runtime':{'path':str(lib/'runtime/bin/java'),'sha256':sha(lib/'runtime/bin/java')},'compiler':{'path':str(base/'jdk-21.0.12.1+1/bin/javac'),'sha256':sha(base/'jdk-21.0.12.1+1/bin/javac')},'commands':commands,'image_recognition_invoked':False,'native_invoked':False,'book_save_invoked':False,'reachStep_invoked':False}
    (out/'PROVENANCE.json').write_text(json.dumps(p,indent=2)+'\n')
    assert p['inputs_before']==p['inputs_after'],'source Book/JAR changed'
print(json.dumps({'ok':True,'commands':len(commands),'output':str(out)}))
