#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent original multiple-rest admission controls; native output is never consulted."""
from pathlib import Path
import json,sys,subprocess,time,hashlib
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1];home=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=False)
cases=[]
for il in [15,16,17]:
 for width in [4*il-1,4*il,4*il+1]:
  for a,b in [(0,0),(-1.5,1.5),(-1.6,1.6),(-1.61,0),(0,1.61),(3,0),(0,-3)]:cases.append({'interline':il,'width':width,'left_offset':a,'right_offset':b})
for h in [1,1.5,2,2.5,7.5,8,8.5,9.5]:cases.append({'width':80,'height':h})
for kind in ['BeamHookInter','SmallBeamInter']:cases.append({'width':80,'kind':kind})
(out/'inputs.json').write_text(json.dumps(cases,separators=(',',':'))+'\n');dist=home/'official/opt/audiveris/lib';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256);(out/'classes').mkdir();stub=out/'stubs';stub.mkdir();empty=out/'empty';empty.mkdir();stubsrc=out/'stub-src';stubsrc.mkdir();(stubsrc/'Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n');(stubsrc/'RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-d',str(stub),*map(str,stubsrc.glob('*.java'))],[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-cp',str(stub)+':'+str(dist/'app/*'),'-sourcepath',str(empty),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/MultipleRestAdmissionControlProbe.java'])],[str(dist/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','MultipleRestAdmissionControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,cmd in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as f:run=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n');assert run.returncode==0
fixture=root/'fixtures/official-multiple-rest-admission.json';assert not fixture.exists();fixture.write_bytes((out/'original.json').read_bytes());result=json.loads(fixture.read_text());assert len(result['cases'])==len(cases)
(out/'provenance.json').write_text(json.dumps({'native_outputs_used':False,'jar_sha256':oracle.JAR_SHA256,'cases':len(cases),'admitted_to_projector':sum(c['admitted_to_projector'] for c in result['cases']),'runtime_excludes_compile_shims':True},indent=2)+'\n')
print((out/'provenance.json').read_text())
