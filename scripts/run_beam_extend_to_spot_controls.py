#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent inputs; real original extendToSpot oracle, no native outputs."""
import copy, hashlib, json, math, subprocess, sys, time
from pathlib import Path
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1]
home=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
inputs=[]
def spot(x,y=36,width=4,height=8,**kw):return {'bounds':[x,y,width,height],'runs':[[i,0,width] for i in range(height)],**kw}
def case(side,spots,cap=None,fill='full',**kw):
 pixels=[[x,y] for x in range(10,105) for y in range(36,44)]
 if fill=='gap_blank':pixels=[p for p in pixels if 30<=p[0]<70]
 return {'interline':16,'width':160,'height':96,'side':side,'max_extension':cap,'spots':spots,'first':{'median':[30,40,70,40],'height':8,'distance_impact':.8},'peer':{'median':[110,40,140,40],'height':8,'distance_impact':1},'black_pixels':pixels,**kw}
for side in ['LEFT','RIGHT']:
 for x in ([21,22,23,27,28,29,30] if side=='LEFT' else [69,70,71,72,73,77,78]):
  for cap in [None,0,1,2,4,8,20]:inputs.append(case(side,[spot(x)],cap))
for side in ['LEFT','RIGHT']:
 for spots in [[spot(24),spot(22)],[spot(22),spot(24)],[spot(24,37),spot(24,36)],[spot(24,36),spot(24,37)],[spot(24,44),spot(22)],[spot(24,43),spot(22)],[spot(24,owned=False),spot(22)],[spot(24,grouped=False),spot(22)],[spot(24,assigned=True),spot(22)],[],[spot(30,36,40,8,own=True),spot(24)]]:
  if side=='RIGHT':
   for g in spots:
    if not g.get('own'):g['bounds'][0]=100-g['bounds'][0]
  inputs.append(case(side,spots))
 # A nearest candidate that cannot extend is not replaced by a farther candidate.
 inputs.append(case(side,[spot(29 if side=='LEFT' else 71,width=1),spot(24 if side=='LEFT' else 76)]))
 inputs.append(case(side,[spot(24 if side=='LEFT' else 74)],fill='gap_blank'))
 # Actual original preceding stem branch has an empty owning seed set.
 inputs.append(case(side,[spot(24 if side=='LEFT' else 74)],None if side=='LEFT' else 36,compose=True))
for side in ['LEFT','RIGHT']:
 near=spot(28 if side=='LEFT' else 71,width=1);far=spot(24 if side=='LEFT' else 76)
 pixels=[[x,y] for x in (list(range(24,28))+list(range(30,70)) if side=='LEFT' else range(10,105)) for y in range(36,44)]
 for spots in [[near,far],[far]]:
  inputs.append(case(side,spots,black_pixels=pixels,no_fallback_control=True))
for il in [13,15,17,19]:
 for side in ['LEFT','RIGHT']:inputs.append(case(side,[spot(24 if side=='LEFT' else 74)],interline=il))
for side in ['LEFT','RIGHT']:
 c=case(side,[spot(24 if side=='LEFT' else 74)],None if side=='LEFT' else 36,compose=True,stem_success=True)
 c['seeds']=[{'bounds':[24 if side=='LEFT' else 76,28,2,25],'runs':[[i,0,25] for i in range(2)]}];inputs.append(c)
(out/'inputs.json').write_text(json.dumps(inputs,separators=(',',':'))+'\n')
dist=home/'official/opt/audiveris/lib';javac=home/oracle.JDK_DIR/'bin/javac';java=dist/'runtime/bin/java';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256)
for name in ['stubs','stub-src','classes','empty','tmp','cache','prefs','home','output']:(out/name).mkdir()
(out/'stub-src/Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n')
(out/'stub-src/RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(javac),'--release','21','-d',str(out/'stubs'),*map(str,(out/'stub-src').glob('*.java'))],[str(javac),'--release','21','-cp',str(out/'stubs')+':'+str(dist/'app/*'),'-sourcepath',str(out/'empty'),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamExtendToSpotControlProbe.java'])],[str(java),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamExtendToSpotControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,command in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as log:run=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n')
 if run.returncode:print((out/f'original-{i}.log').read_text()[-5000:]);sys.exit(run.returncode)
result=json.loads((out/'original.json').read_text());assert len(result['cases'])==len(inputs)
fixture=root/'fixtures/official-beam-extend-to-spot.json';assert not fixture.exists();fixture.write_text(json.dumps(result,separators=(',',':'))+'\n')
provenance={'native_outputs_used':False,'jar_sha256':oracle.sha(dist/'app/audiveris.jar'),'distribution_sha256':oracle.DEB_SHA256,'jdk_sha256':oracle.JDK_SHA256,'upstream_commit':oracle.UPSTREAM_COMMIT,'cases':len(inputs),'accepted_extensions':sum(c['accepted'] for c in result['cases']),'accepted_spot_extensions':sum(c['accepted'] and c['spot_method_called'] for c in result['cases']),'accepted_earlier_stems':sum(not c['spot_method_called'] for c in result['cases']),'fixture_sha256':oracle.sha(fixture),'inputs_sha256':oracle.sha(out/'inputs.json'),'observer_sha256':oracle.sha(root/'scripts/java/BeamExtendToSpotControlProbe.java'),'runtime_excludes_compile_shims':True}
(out/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');print(json.dumps(provenance))
