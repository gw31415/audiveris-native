#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent inputs; real original extendToBeam oracle, no native outputs."""
import copy, hashlib, json, math, subprocess, sys, time
from pathlib import Path
import linux_oracle as oracle
root=Path(__file__).resolve().parents[1]
home=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
inputs=[]
for gap in [0,3,4,5,8.5,9.5,30,65]:
 for order in [False,True]:
  for side in ['LEFT','RIGHT']:
   medians=[[10,40,30,40],[30+gap,40,70+gap,40]]
   if order:medians.reverse()
   target=2 if side=='LEFT' and not order or side=='RIGHT' and order else 1
   pixels=[[x,y] for x in range(10,min(160,math.ceil(70+gap))) for y in range(36,44)]
   if gap>=3 and (gap==4 or gap==65):pixels=[p for p in pixels if not 30<=p[0]<30+gap]
   inputs.append({'mode':'side','side':side,'target':target,'beams':[{'median':m} for m in medians],'black_pixels':pixels})
for medians,removed in [([[34,40,70,40],[10,40,30,40]],None),([[70,40,90,40],[34,40,66,40],[10,40,30,40]],None),([[10,40,30,40],[34,40,70,40]],None),([[10,40,30,40],[70,40,90,40],[34,40,66,40]],1),([[34,40,70,40],[10,40,30,40],[10,40,30,40]],None)]:
 c={'mode':'prefix','beams':[{'median':m} for m in medians],'black_pixels':[[x,y] for x in range(10,100) for y in range(36,44)]}
 if removed is not None:c['pre_removed']=removed
 inputs.append(c)
(out/'inputs.json').write_text(json.dumps(inputs,separators=(',',':'))+'\n')
dist=home/'official/opt/audiveris/lib';javac=home/oracle.JDK_DIR/'bin/javac';java=dist/'runtime/bin/java';oracle.checked(dist/'app/audiveris.jar',oracle.JAR_SHA256)
for name in ['stubs','stub-src','classes','empty','tmp','cache','prefs','home','output']:(out/name).mkdir()
(out/'stub-src/Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}\n')
(out/'stub-src/RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass {public RunClass(Book b,SortedSet<Integer> ids){} public abstract void process();}\n')
commands=[[str(javac),'--release','21','-d',str(out/'stubs'),*map(str,(out/'stub-src').glob('*.java'))],[str(javac),'--release','21','-cp',str(out/'stubs')+':'+str(dist/'app/*'),'-sourcepath',str(out/'empty'),'-d',str(out/'classes'),*map(str,[root/'scripts/java/GridProbe.java',root/'scripts/java/BeamImpactsControlProbe.java',root/'scripts/java/BeamExtensionPrefixControlProbe.java'])],[str(java),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(out/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-Duser.home='+str(out/'home'),'-Dnative.probe.inputs='+str(out/'inputs.json'),'-Dnative.probe.output='+str(out/'original.json'),'-cp',str(dist/'app/*')+':'+str(out/'classes'),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','BeamExtensionPrefixControlProbe','-output',str(out/'output'),'--',str(root/'fixtures/official-grid/beams-single.png')]]
receipts=[]
for i,command in enumerate(commands):
 start=time.monotonic()
 with (out/f'original-{i}.log').open('w') as log:run=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=90)
 receipts.append({'command':command,'exit_code':run.returncode,'seconds':time.monotonic()-start});(out/'commands.json').write_text(json.dumps(receipts,indent=2)+'\n')
 if run.returncode:print((out/f'original-{i}.log').read_text()[-5000:]);sys.exit(run.returncode)
result=json.loads((out/'original.json').read_text());assert len(result['cases'])==len(inputs)
fixture=root/'fixtures/official-beam-extension-prefix.json';assert not fixture.exists();fixture.write_text(json.dumps(result,separators=(',',':'))+'\n')
provenance={'native_outputs_used':False,'jar_sha256':oracle.sha(dist/'app/audiveris.jar'),'distribution_sha256':oracle.DEB_SHA256,'jdk_sha256':oracle.JDK_SHA256,'upstream_commit':oracle.UPSTREAM_COMMIT,'cases':len(inputs),'accepted_merges':sum(d['merged'] for c in result['cases'] for d in c['decisions']),'fixture_sha256':oracle.sha(fixture),'inputs_sha256':oracle.sha(out/'inputs.json'),'observer_sha256':oracle.sha(root/'scripts/java/BeamExtensionPrefixControlProbe.java'),'runtime_excludes_compile_shims':True}
(out/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');print(json.dumps(provenance))
