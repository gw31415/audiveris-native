#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent original populateSystem controls; native output is never consulted."""
from pathlib import Path
import json,sys,subprocess,time,hashlib
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1];home=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=False)
def b(x,y,w=50,slope=0,h=8,kind='BeamInter'):return {'median':[x,y,x+w,y+w*slope],'height':h,'kind':kind}
cases=[]
for il in [15,16,17]:
 for overlap in [9,10,11,11.5,12,13]:
  for dy in [18,19,19.5,20,21]:cases.append({'interline':il,'beams':[b(20,30),b(70-overlap,30+dy)]})
for slope in [.064,.065,.066,-.065]:cases.append({'beams':[b(20,30),b(20,42,slope=slope)]})
cases += [{'beams':[]},{'beams':[b(20,30)]},{'beams':[b(20,70),b(20,30),b(20,50)]},{'beams':[b(20,30),b(80,30),b(45,45)]},{'beams':[b(20,30),b(20,48),b(20,66)]},{'beams':[b(20,30),b(20,44,kind='BeamHookInter',w=16)]},{'beams':[b(20,30),b(20,44,kind='SmallBeamInter',h=5)]},{'beams':[b(20,30),b(20,44)],'exclusions':[[1,2]]},{'beams':[b(80,30),b(20,30),b(45,45)]}]
(out/'inputs.json').write_text(json.dumps(cases,separators=(',',':'))+'\n');dist=home/'official/opt/audiveris/lib';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256);(out/'classes').mkdir();stub=out/'stubs';stub.mkdir();empty=out/'empty';empty.mkdir();stubsrc=out/'stub-src';stubsrc.mkdir();(stubsrc/'Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n');(stubsrc/'RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-d',str(stub),*map(str,stubsrc.glob('*.java'))],[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-cp',str(stub)+':'+str(dist/'app/*'),'-sourcepath',str(empty),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamGroupsControlProbe.java'])],[str(dist/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamGroupsControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,cmd in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as f:run=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n');assert run.returncode==0
fixture=root/'fixtures/official-beam-groups.json';assert not fixture.exists();fixture.write_bytes((out/'original.json').read_bytes());result=json.loads(fixture.read_text());assert len(result['cases'])==len(cases)
(out/'provenance.json').write_text(json.dumps({'native_outputs_used':False,'jar_sha256':oracle.JAR_SHA256,'cases':len(cases),'groups':sum(len(c['groups']) for c in result['cases']),'runtime_excludes_compile_shims':True},indent=2)+'\n')
print((out/'provenance.json').read_text())
