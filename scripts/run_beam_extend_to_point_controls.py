#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent inputs; real original extendToPoint oracle, no native outputs."""
import copy, hashlib, json, math, subprocess, sys, time
from pathlib import Path
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1]
home=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
inputs=[]
for side in ['LEFT','RIGHT']:
 for gap in [-2,0,1,1.0001,2,4,10,25]:
  for fill in ['full','gap_blank']:
   point=[30-gap if side=='LEFT' else 70+gap,40]
   pixels=[[x,y] for x in range(max(0,math.floor(min(30,point[0]))),min(160,math.ceil(max(70,point[0])))) for y in range(36,44)]
   if fill=='gap_blank':pixels=[p for p in pixels if 30<=p[0]<70]
   inputs.append({'interline':16,'width':160,'height':96,'side':side,'point':point,'first':{'median':[30,40,70,40],'height':8,'distance_impact':.75},'peer':{'median':[110,40,140,40],'height':8,'distance_impact':1},'black_pixels':pixels})
for height in [4,6,8]:
 for side in ['LEFT','RIGHT']:
  inputs.append({'interline':16,'width':160,'height':96,'side':side,'point':[20 if side=='LEFT' else 80,40],'small_height':4,'first':{'median':[30,40,70,40],'height':height,'distance_impact':.9},'peer':{'median':[110,40,140,40],'height':8,'distance_impact':1},'black_pixels':[[x,y] for x in range(20,80) for y in range(40-height//2,40+height//2)]})
(out/'inputs.json').write_text(json.dumps(inputs,separators=(',',':'))+'\n')
dist=home/'official/opt/audiveris/lib';javac=home/oracle.JDK_DIR/'bin/javac';java=dist/'runtime/bin/java';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256)
for name in ['stubs','stub-src','classes','empty','tmp','cache','prefs','home','output']:(out/name).mkdir()
(out/'stub-src/Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n')
(out/'stub-src/RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(javac),'--release','21','-d',str(out/'stubs'),*map(str,(out/'stub-src').glob('*.java'))],[str(javac),'--release','21','-cp',str(out/'stubs')+':'+str(dist/'app/*'),'-sourcepath',str(out/'empty'),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamExtendToPointControlProbe.java'])],[str(java),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamExtendToPointControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,command in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as log:run=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n')
 if run.returncode:print((out/f'original-{i}.log').read_text()[-5000:]);sys.exit(run.returncode)
result=json.loads((out/'original.json').read_text());assert len(result['cases'])==len(inputs)
fixture=root/'fixtures/official-beam-extend-to-point.json';assert not fixture.exists();fixture.write_text(json.dumps(result,separators=(',',':'))+'\n')
provenance={'native_outputs_used':False,'jar_sha256':oracle.sha(dist/'app/audiveris.jar'),'distribution_sha256':oracle.DEB_SHA256,'jdk_sha256':oracle.JDK_SHA256,'upstream_commit':oracle.UPSTREAM_COMMIT,'cases':len(inputs),'accepted_extensions':sum(c['accepted'] for c in result['cases']),'fixture_sha256':oracle.sha(fixture),'inputs_sha256':oracle.sha(out/'inputs.json'),'observer_sha256':oracle.sha(root/'scripts/java/BeamExtendToPointControlProbe.java'),'runtime_excludes_compile_shims':True}
(out/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');print(json.dumps(provenance))
