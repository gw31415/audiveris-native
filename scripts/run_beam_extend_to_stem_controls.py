#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent inputs; real original extendToStem oracle, no native outputs."""
import copy, hashlib, json, math, subprocess, sys, time
from pathlib import Path
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1]
home=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
inputs=[]
def seed(x,y=28,width=2,height=25,**kw):return {'bounds':[x,y,width,height],'runs':[[i,0,height] for i in range(width)],**kw}
for side in ['LEFT','RIGHT']:
 for x in ([13,14,15,27,28,29,30] if side=='LEFT' else [69,70,71,72,73,85,86]):
  for cap in [None,0,1,2,8,16,40]:
   seeds=[seed(x)]
   pixels=[[px,y] for px in range(max(0,min(x,30)),min(160,max(x+2,70))) for y in range(36,44)]
   inputs.append({'interline':16,'width':160,'height':96,'side':side,'max_extension':cap,'seeds':seeds,'first':{'median':[30,40,70,40],'height':8,'distance_impact':.8},'peer':{'median':[110,40,140,40],'height':8,'distance_impact':1},'black_pixels':pixels})
for side in ['LEFT','RIGHT']:
 for seeds in [[seed(24),seed(20)],[seed(20),seed(24)],[seed(24,28),seed(24,30)],[seed(24,30),seed(24,28)],[seed(24,57),seed(20)],[seed(24,58),seed(20)],[seed(24,owned=False),seed(20)],[seed(24,grouped=False),seed(20)],[],[seed(24,28,width=3,height=2)]]:
  if side=='RIGHT':
   for s0 in seeds:s0['bounds'][0]=100-s0['bounds'][0]
  inputs.append({'interline':16,'width':160,'height':96,'side':side,'max_extension':None,'seeds':seeds,'first':{'median':[30,40,70,40],'height':8,'distance_impact':.8},'peer':{'median':[110,40,140,40],'height':8,'distance_impact':1},'black_pixels':[[x,y] for x in range(10,100) for y in range(36,44)]})
# Nearest selected seed is not bypassed when its grade/gap gate rejects.
for side in ['LEFT','RIGHT']:
 inputs.append({'interline':16,'width':160,'height':96,'side':side,'seeds':[seed(28 if side=='LEFT' else 70,width=3 if side=='LEFT' else 2),seed(20 if side=='LEFT' else 80)],'first':{'median':[30,40,70,40],'height':8,'distance_impact':.8},'peer':{'median':[110,40,140,40],'height':8,'distance_impact':1},'black_pixels':[[x,y] for x in range(10,100) for y in range(36,44)]})
for side in ['LEFT','RIGHT']:
 for inverted in [False,True]:
  g={'bounds':[22 if side=='LEFT' else 76,28,4,25],'runs':[[i,3-i if inverted else i,22] for i in range(4)]}
  inputs.append({'interline':16,'width':160,'height':96,'side':side,'seeds':[g],'first':{'median':[30,39,70,41],'height':8,'distance_impact':.8},'peer':{'median':[110,40,140,40],'height':8,'distance_impact':1},'black_pixels':[[x,y] for x in range(10,100) for y in range(36,44)]})
for side in ['LEFT','RIGHT']:
 inputs.append({'compose':True,'interline':16,'width':160,'height':96,'side':side,'max_extension':2 if side=='LEFT' else 6,'seeds':[seed(28 if side=='LEFT' else 74)],'first':{'median':[30,40,70,40],'height':8,'distance_impact':.8},'peer':{'median':[10,40,24,40] if side=='LEFT' else [80,40,100,40],'height':8,'distance_impact':1},'black_pixels':[[x,y] for x in (list(range(10,24))+list(range(28,70)) if side=='LEFT' else range(30,100)) for y in range(36,44)]})
(out/'inputs.json').write_text(json.dumps(inputs,separators=(',',':'))+'\n')
dist=home/'official/opt/audiveris/lib';javac=home/oracle.JDK_DIR/'bin/javac';java=dist/'runtime/bin/java';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256)
for name in ['stubs','stub-src','classes','empty','tmp','cache','prefs','home','output']:(out/name).mkdir()
(out/'stub-src/Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n')
(out/'stub-src/RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(javac),'--release','21','-d',str(out/'stubs'),*map(str,(out/'stub-src').glob('*.java'))],[str(javac),'--release','21','-cp',str(out/'stubs')+':'+str(dist/'app/*'),'-sourcepath',str(out/'empty'),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamExtendToStemControlProbe.java'])],[str(java),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamExtendToStemControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,command in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as log:run=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n')
 if run.returncode:print((out/f'original-{i}.log').read_text()[-5000:]);sys.exit(run.returncode)
result=json.loads((out/'original.json').read_text());assert len(result['cases'])==len(inputs)
fixture=root/'fixtures/official-beam-extend-to-stem.json';assert not fixture.exists();fixture.write_text(json.dumps(result,separators=(',',':'))+'\n')
provenance={'native_outputs_used':False,'jar_sha256':oracle.sha(dist/'app/audiveris.jar'),'distribution_sha256':oracle.DEB_SHA256,'jdk_sha256':oracle.JDK_SHA256,'upstream_commit':oracle.UPSTREAM_COMMIT,'cases':len(inputs),'accepted_extensions':sum(c['accepted'] for c in result['cases']),'fixture_sha256':oracle.sha(fixture),'inputs_sha256':oracle.sha(out/'inputs.json'),'observer_sha256':oracle.sha(root/'scripts/java/BeamExtendToStemControlProbe.java'),'runtime_excludes_compile_shims':True}
(out/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');print(json.dumps(provenance))
