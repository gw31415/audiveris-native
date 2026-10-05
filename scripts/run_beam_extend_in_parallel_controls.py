#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent inputs; real original extendInParallel oracle, no native outputs."""
import copy, hashlib, json, math, subprocess, sys, time
from pathlib import Path
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1]
home=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
inputs=[]
def beam(m):return {'median':m,'height':8,'distance_impact':.8}
def case(side,beams,**kw):
 pixels=set()
 for b in beams:
  m=b['median'];slope=(m[3]-m[1])/(m[2]-m[0])
  for x in range(max(0,math.floor(m[0])),min(160,math.ceil(m[2]))):
   y=m[1]+slope*(x-m[0])
   for iy in range(max(0,math.ceil(y-4)),min(96,math.ceil(y+4))):pixels.add((x,iy))
 # Native target extension raster is independent from neighbor output/acceptance.
 for x in range(10,100):
  for y in range(36,44):pixels.add((x,y))
 return {'interline':16,'width':160,'height':96,'system_size':[160,96],'side':side,'beams':beams,'black_pixels':sorted(pixels),**kw}
for side in ['LEFT','RIGHT']:
 for increment in [0,5.999,6,6.001,20]:
  for dy in [0,4,4.0001,12,19.2,19.2001,24]:
   m=[30-increment,40+dy,70,40+dy] if side=='LEFT' else [30,40+dy,70+increment,40+dy]
   inputs.append(case(side,[beam([30,40,70,40]),beam(m)]))
 for overlap in [11.1999,11.2,11.2001]:
  m=[24,52,30+overlap,52] if side=='LEFT' else [70-overlap,52,76,52];inputs.append(case(side,[beam([30,40,70,40]),beam(m)]))
 for slope in [.064999,.065,.065001,.2]:
  m=[24,50,70,50+slope*46] if side=='LEFT' else [30,50,76,50+slope*46];inputs.append(case(side,[beam([30,40,70,40]),beam(m)]))
 a=beam([30,40,70,40]);near=beam([24,52,70,52] if side=='LEFT' else [30,52,76,52]);far=beam([10,56,70,56] if side=='LEFT' else [30,56,90,56])
 for others in [[near,far],[far,near],[a,near]]:inputs.append(case(side,[a,*others]))
 inputs.append(case(side,[a,near,far],pre_removed=2))
 for system_size in ([[24,96],[25,96],[160,44],[160,45]] if side=='LEFT' else [[76,96],[77,96],[160,44],[160,45]]):inputs.append(case(side,[a,near],system_size=system_size))
 c=case(side,[a,near,far]);c['black_pixels']=[p for p in c['black_pixels'] if not (p[0]<30 if side=='LEFT' else p[0]>=70) or p[1]>=48];inputs.append(c)
 inputs.append(case(side,[a,near],compose=True))
(out/'inputs.json').write_text(json.dumps(inputs,separators=(',',':'))+'\n')
dist=home/'official/opt/audiveris/lib';javac=home/oracle.JDK_DIR/'bin/javac';java=dist/'runtime/bin/java';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256)
for name in ['stubs','stub-src','classes','empty','tmp','cache','prefs','home','output']:(out/name).mkdir()
(out/'stub-src/Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n')
(out/'stub-src/RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(javac),'--release','21','-d',str(out/'stubs'),*map(str,(out/'stub-src').glob('*.java'))],[str(javac),'--release','21','-cp',str(out/'stubs')+':'+str(dist/'app/*'),'-sourcepath',str(out/'empty'),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamExtendInParallelControlProbe.java'])],[str(java),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamExtendInParallelControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,command in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as log:run=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n')
 if run.returncode:print((out/f'original-{i}.log').read_text()[-5000:]);sys.exit(run.returncode)
result=json.loads((out/'original.json').read_text());assert len(result['cases'])==len(inputs)
fixture=root/'fixtures/official-beam-extend-in-parallel.json';assert not fixture.exists();fixture.write_text(json.dumps(result,separators=(',',':'))+'\n')
provenance={'native_outputs_used':False,'jar_sha256':oracle.sha(dist/'app/audiveris.jar'),'distribution_sha256':oracle.DEB_SHA256,'jdk_sha256':oracle.JDK_SHA256,'upstream_commit':oracle.UPSTREAM_COMMIT,'cases':len(inputs),'accepted_extensions':sum(c['accepted'] for c in result['cases']),'fixture_sha256':oracle.sha(fixture),'inputs_sha256':oracle.sha(out/'inputs.json'),'observer_sha256':oracle.sha(root/'scripts/java/BeamExtendInParallelControlProbe.java'),'runtime_excludes_compile_shims':True}
(out/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');print(json.dumps(provenance))
