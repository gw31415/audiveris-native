"""Development oracle only. Requires existing official app, original PNG and JDK21.
No downloading and no Java production dependency. Env: AUDIVERIS_CONTENTS, JAVAC;
argument: path to the original CC0 An Laura RGBA PNG used by this coverage release.
"""
import os,sys,json,hashlib,time,subprocess
from pathlib import Path
root=Path(__file__).resolve().parent.parent
app=Path(os.environ['AUDIVERIS_CONTENTS']).resolve();inp=Path(sys.argv[1]).resolve()
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha((app/'app/audiveris.jar').read_bytes())=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
assert sha(inp.read_bytes())=='958288a4fcefefa8ea899cbd58dd6d77598dd592974072fff8baa3f5566243df'
classes=root/'oracle/classes';stubs=root/'oracle/compile-only';out=root/'oracle/run'
for p in [classes,stubs,out,root/'oracle/tmp',root/'oracle/native-cache']:p.mkdir(parents=True,exist_ok=True)
javac=os.environ.get('JAVAC','javac')
subprocess.run([javac,'-d',str(stubs)]+[str(p) for p in (root/'oracle/stubs').rglob('*.java')],check=True)
subprocess.run([javac,'-cp',str(stubs),'-d',str(classes),str(root/'oracle/src/oracle/CoverageProbe.java'),str(root/'oracle/BankAssetOracle.java')],check=True)
java=[str(app/'runtime/Contents/Home/bin/java'),'-Xmx768m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(root/'oracle/tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(root/'oracle/native-cache'),'-Doracle.output='+str(out),'-cp',str(classes)+os.pathsep+str(app/'app/*')]
args=java+['Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-step','HEADS','-run','oracle.CoverageProbe','-output',str(out),'--',str(inp)]
start=time.monotonic()
with (out/'console.log').open('w') as log:result=subprocess.run(args,stdout=log,stderr=subprocess.STDOUT,timeout=180)
(out/'execution.json').write_text(json.dumps({'argv':args,'exit':result.returncode,'seconds':time.monotonic()-start,'input_sha256':sha(inp.read_bytes())},indent=2)+'\n')
result.check_returncode()
with (root/'oracle/generation.log').open('w') as log:subprocess.run(java+['BankAssetOracle',str(root/'oracle/generated')],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=60)
subprocess.run([sys.executable,str(root/'oracle/build_bank.py')],check=True)
print('Rebuilt assets. Review release diff and run cargo test before adopting the new root hash.')
