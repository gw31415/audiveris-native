#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent original serif raster controls; native output is never consulted."""
from pathlib import Path
import json,sys,subprocess,time,hashlib
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1];home=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=False)
cases=[]
for w in [1,2,3,4,6,10]:
 for h in [24,32,33,40,48,60]:
  for left in [True,False]:cases.append({'x':450,'left':left,'added_chunk':6,'rectangles':[[450,-h//2,w,h],[450 if left else 380,-4,70,8]]})
cases += [{'x':450,'rectangles':[]},{'x':450,'staff_lines':False,'rectangles':[]},{'x':450,'rectangles':[[448,-16,2,33],[453,-16,2,33],[450,-4,70,8]]}]
(out/'inputs.json').write_text(json.dumps(cases,separators=(',',':'))+'\n');dist=home/'official/opt/audiveris/lib';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256);(out/'classes').mkdir();stub=out/'stubs';stub.mkdir();empty=out/'empty';empty.mkdir();stubsrc=out/'stub-src';stubsrc.mkdir();(stubsrc/'Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n');(stubsrc/'RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-d',str(stub),*map(str,stubsrc.glob('*.java'))],[str(home/oracle.JDK_DIR/'bin/javac'),'--release','21','-cp',str(stub)+':'+str(dist/'app/*'),'-sourcepath',str(empty),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/SerifRasterControlProbe.java'])],[str(dist/'runtime/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','SerifRasterControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,cmd in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as f:run=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':cmd,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n');assert run.returncode==0
fixture=root/'fixtures/official-serif-raster.json';assert not fixture.exists();fixture.write_bytes((out/'original.json').read_bytes());result=json.loads(fixture.read_text());assert len(result['cases'])==len(cases)
keys=['width','height','rows','line_thickness','interline','fore','abscissas'];geometry={k:result['cases'][0][k] for k in keys}
for c in result['cases']:
 for k in keys:assert c.pop(k)==geometry[k]
 runs=[]
 for n in c.pop('projection'):
  if runs and runs[-1][1]==n:runs[-1][0]+=1
  else:runs.append([1,n])
 c['projection_rle']=runs
result['geometry']=geometry;result['encoding']='Lossless shared geometry and projection RLE, derived only from original JSON';fixture.write_text(json.dumps(result,separators=(',',':'))+'\n')

(out/'provenance.json').write_text(json.dumps({'native_outputs_used':False,'jar_sha256':oracle.JAR_SHA256,'cases':len(cases),'accepted_serif_peaks':sum(len(c['accepted_peaks']) for c in result['cases']),'runtime_excludes_compile_shims':True},indent=2)+'\n')
print((out/'provenance.json').read_text())
