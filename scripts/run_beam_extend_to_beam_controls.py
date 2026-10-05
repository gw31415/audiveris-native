#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent inputs; real original extendToBeam oracle, no native outputs."""
import copy, hashlib, json, math, subprocess, sys, time
from pathlib import Path
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1]
home=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
inputs=[]
for gap in [-4,0,math.nextafter(3,-math.inf),3,math.nextafter(3,math.inf),4,5,30]:
 for pattern in ['stripe','gap-white','white']:
  for swapped in [False,True]:
   a={'median':[10,40,30,40],'height':8,'distance_impact':1}
   b={'median':[30+gap,40,70+gap,40],'height':8,'distance_impact':.8}
   pixels=[]
   for x in range(10,math.ceil(70+gap)):
    if pattern=='white' or (pattern=='gap-white' and 30<=x<30+gap):continue
    pixels.extend([[x,y] for y in range(36,44)])
   inputs.append({'label':f'{gap}/{pattern}/swap={swapped}','interline':16,'width':128,'height':96,'first':b if swapped else a,'second':a if swapped else b,'black_pixels':pixels})
for count in [23,24,25]:
 c=copy.deepcopy(inputs[36]);c['label']=f'gap5 black={count}/40';c['first']['median']=[10,40,30,40];c['second']['median']=[35,40,75,40]
 c['black_pixels']=[[x,y] for x in list(range(10,30))+list(range(35,75)) for y in range(36,44)]+[[30+i%5,36+i//5] for i in range(count)];inputs.append(c)
for slope in [-.125,.125]:
 for dy in [0,1,8]:
  c=copy.deepcopy(inputs[36]);c['label']=f'slope={slope}/ordinate={dy}';c['first']['median']=[10,40,30,40+20*slope];c['second']['median']=[35,40+25*slope+dy,75,40+65*slope+dy];c['black_pixels']=[[x,y] for x in range(10,75) for y in range(96) if 40+(x-10)*slope-4<=y<40+(x-10)*slope+4];inputs.append(c)
for median,height,il in [([30,40,70,40],8,1),([35,40,75,40],0,16),([130,40,170,40],8,16),([35,-40,75,-40],8,16)]:
 c=copy.deepcopy(inputs[0]);c.update(label='empty/clipped',gate_only=True,interline=il,black_pixels=[]);c['first']['height']=height;c['second']['height']=height;c['second']['median']=median;inputs.append(c)
(out/'inputs.json').write_text(json.dumps(inputs,separators=(',',':'))+'\n')
dist=home/'official/opt/audiveris/lib';javac=home/oracle.JDK_DIR/'bin/javac';java=dist/'runtime/bin/java';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256)
for name in ['stubs','stub-src','classes','empty','tmp','cache','prefs','home','output']:(out/name).mkdir()
(out/'stub-src/Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n')
(out/'stub-src/RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(javac),'--release','21','-d',str(out/'stubs'),*map(str,(out/'stub-src').glob('*.java'))],[str(javac),'--release','21','-cp',str(out/'stubs')+':'+str(dist/'app/*'),'-sourcepath',str(out/'empty'),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamExtendToBeamControlProbe.java'])],[str(java),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamExtendToBeamControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,command in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as log:run=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n')
 if run.returncode:print((out/f'original-{i}.log').read_text()[-5000:]);sys.exit(run.returncode)
result=json.loads((out/'original.json').read_text());assert len(result['cases'])==len(inputs)
fixture=root/'fixtures/official-beam-extend-to-beam.json';assert not fixture.exists();fixture.write_text(json.dumps(result,separators=(',',':'))+'\n')
provenance={'native_outputs_used':False,'jar_sha256':oracle.sha(dist/'app/audiveris.jar'),'distribution_sha256':oracle.DEB_SHA256,'jdk_sha256':oracle.JDK_SHA256,'upstream_commit':oracle.UPSTREAM_COMMIT,'cases':len(inputs),'accepted':sum(c['accepted'] is True for c in result['cases']),'fixture_sha256':oracle.sha(fixture),'inputs_sha256':oracle.sha(out/'inputs.json'),'observer_sha256':oracle.sha(root/'scripts/java/BeamExtendToBeamControlProbe.java'),'runtime_excludes_compile_shims':True}
(out/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');print(json.dumps(provenance))
