#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Add a test-only observer to the official CLI classpath; no algorithm replacement."""
import subprocess,json,sys,hashlib,zipfile
from pathlib import Path
root=Path(__file__).resolve().parents[1];contents=Path(sys.argv[1]).resolve();work=root/'evidence/generated/clef-candidate-probe';work.mkdir(parents=True,exist_ok=True)
jdk=Path(sys.argv[2]).resolve();cp=str(contents/'app/*')
assert hashlib.sha256((contents/'app/audiveris.jar').read_bytes()).hexdigest()=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
# Java21 can compile a reflection observer without interpreting Java25 classfiles.
# Compile-only ABI shims are in a separate directory and NEVER on runtime classpath.
stubs=work/'compile-only';stubs.mkdir(exist_ok=True)
(stubs/'Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}')
(stubs/'RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass { public RunClass(Book b, SortedSet<Integer> ids){} public abstract void process(); }')
subprocess.run([str(jdk/'bin/javac'),'-d',str(stubs),str(stubs/'Book.java'),str(stubs/'RunClass.java')],check=True)
subprocess.run([str(jdk/'bin/javac'),'-cp',str(stubs),'-d',str(work),str(root/'scripts/java/GridProbe.java'),str(root/'scripts/java/GridFinalProbe.java'),str(root/'scripts/java/ClefCandidateProbe.java')],check=True)
for name in ['laura-gray','synthetic-system-bars','cclef-pair','bracket-pair','line-root','rendered-clef-g_clef','rendered-clef-f_clef','rendered-clef-c_clef','rendered-clef-empty','rendered-clef-noise']:
 folder=work/name;folder.mkdir(exist_ok=True);out=folder/'final.json';source=root/'fixtures/official-grid'/f'{name}.png'
 for n in ['tmp','cache','prefs']:(work/n).mkdir(exist_ok=True)
 cmd=[str(contents/'runtime/Contents/Home/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(work/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(work/'cache'),'-Djava.util.prefs.userRoot='+str(work/'prefs'),'-Dnative.probe.output='+str(out),'-cp',cp+':'+str(work),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','ClefCandidateProbe','-output',str(folder),'--',str(source)]
 with (folder/'console.log').open('w')as log:r=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,timeout=120)
 (folder/'execution.json').write_text(json.dumps({'argv':cmd,'exit':r.returncode,'input_sha256':hashlib.sha256(source.read_bytes()).hexdigest()},indent=2))
 assert r.returncode==0 and out.exists(),folder
 target=root/'fixtures/official-grid'/f'{name}-clef.json';target.write_bytes(out.read_bytes());print(name,'final GRID observed',flush=True)
