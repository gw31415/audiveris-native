#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent original buildHooks controls; native output is never consulted."""
from pathlib import Path
import json,sys,subprocess,time,hashlib
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1];home=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=False)
def spot(x,y,w,h,rows=None):return {'bounds':[x,y,w,h],'runs':[[i,0,w] for i in (range(h) if rows is None else rows)]}
def case(spots,**kw):
 c={'beams':[{'median':[30,40,100,40],'height':8,'distance':.8}],'spots':spots,'rectangles':[[30,36,70,8]]}
 for g in spots:
  b=g['bounds']
  for y,x,n in g['runs']:c['rectangles'].append([b[0]+x,b[1]+y,n,1])
 c.update(kw);return c
cases=[]
for y in [24,48]:
 for w in [10,11,12,16,24,33]:
  for h in [5,6,8,11,12]:cases.append(case([spot(40,y,w,h)]))
cases += [case([spot(40,24,20,8),spot(42,24,20,8)]),case([spot(42,24,20,8),spot(40,24,20,8)]),case([spot(40,48,20,8),spot(40,24,20,8)]),case([spot(40,30,20,24,range(8,16))]),case([spot(40,24,20,8)],small_parameters=True,beams=[{'median':[30,40,100,40],'height':5,'distance':.8}]),case([spot(40,24,20,8)],beams=[{'median':[30,40,100,40],'height':8,'distance':0.}])]
(out/'inputs.json').write_text(json.dumps(cases,separators=(',',':'))+'\n');dist=home/'official/opt/audiveris/lib';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256);(out/'classes').mkdir();stub=out/'stubs';stub.mkdir();empty=out/'empty';empty.mkdir();stubsrc=out/'stub-src';stubsrc.mkdir();(stubsrc/'Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n');(stubsrc/'RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-d',str(stub),*map(str,stubsrc.glob('*.java'))],[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-cp',str(stub)+':'+str(dist/'app/*'),'-sourcepath',str(empty),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamLateHooksControlProbe.java'])],[str(dist/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamLateHooksControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,cmd in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as f:run=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n');assert run.returncode==0
fixture=root/'fixtures/official-beam-late-hooks.json';assert not fixture.exists();fixture.write_bytes((out/'original.json').read_bytes());result=json.loads(fixture.read_text());assert len(result['cases'])==len(cases)
(out/'provenance.json').write_text(json.dumps({'native_outputs_used':False,'jar_sha256':oracle.JAR_SHA256,'cases':len(cases),'accepted_hooks':sum(len(c['raw'])-len(c['input']['beams']) for c in result['cases']),'runtime_excludes_compile_shims':True},indent=2)+'\n')
print((out/'provenance.json').read_text())
