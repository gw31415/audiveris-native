#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Add a test-only observer to the official CLI classpath; no algorithm replacement."""
import subprocess,json,sys,hashlib,zipfile
from pathlib import Path
root=Path(__file__).resolve().parents[1];contents=Path(sys.argv[1]).resolve();work=root/'evidence/generated/semantic-final-probe';work.mkdir(parents=True,exist_ok=True)
jdk=Path(sys.argv[2]).resolve();cp=str(contents/'app/*')
assert hashlib.sha256((contents/'app/audiveris.jar').read_bytes()).hexdigest()=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
# Java21 can compile a reflection observer without interpreting Java25 classfiles.
# Compile-only ABI shims are in a separate directory and NEVER on runtime classpath.
stubs=work/'compile-only';stubs.mkdir(exist_ok=True)
(stubs/'Book.java').write_text('package org.audiveris.omr.sheet; public class Book {}')
(stubs/'RunClass.java').write_text('package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass { public RunClass(Book b, SortedSet<Integer> ids){} public abstract void process(); }')
subprocess.run([str(jdk/'bin/javac'),'-d',str(stubs),str(stubs/'Book.java'),str(stubs/'RunClass.java')],check=True)
subprocess.run([str(jdk/'bin/javac'),'-cp',str(stubs),'-d',str(work),str(root/'scripts/java/GridProbe.java'),str(root/'scripts/java/GridFinalProbe.java')],check=True)
subprocess.run([str(jdk/'bin/javac'),'-d',str(work),str(root/'scripts/java/BracketFontProbe.java')],check=True)
with zipfile.ZipFile(contents/'app/audiveris.jar') as jar: font_sha=hashlib.sha256(jar.read('res/Bravura.otf')).hexdigest()
assert font_sha=='dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2'
metrics=root/'fixtures/official-grid/bracket-font-metrics.json'
font_cmd=[str(contents/'runtime/Contents/Home/bin/java'),'-Djava.awt.headless=true','-cp',cp+':'+str(work),'BracketFontProbe',str(metrics)]
font_run=subprocess.run(font_cmd,capture_output=True,text=True,timeout=30)
assert font_run.returncode==0,font_run.stderr
(root/'evidence/bracket-font-provenance.json').write_text(json.dumps({'upstream_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':hashlib.sha256((contents/'app/audiveris.jar').read_bytes()).hexdigest(),'font_sha256':font_sha,'font_family':'Bravura','font_version':'1.392','interlines':[11,16],'java_argv':font_cmd,'java_exit':font_run.returncode,'metrics_sha256':hashlib.sha256(metrics.read_bytes()).hexdigest(),'license':'SIL OFL 1.1; see crates/audiveris-template-bank/assets/BRAVURA-OFL.txt'},indent=2)+'\n')
if '--font-only' in sys.argv: sys.exit(0)
for name in ['laura-gray','synthetic-system-bars','cclef-pair','cclef-tail','cclef-too-wide','cclef-distant','right-extending','line-root','cclef-thick-second','cclef-connected','bracket-pair','bracket-middle','bracket-small-serif']:
 folder=work/name;folder.mkdir(exist_ok=True);out=folder/'final.json';source=root/'fixtures/official-grid'/f'{name}.png'
 for n in ['tmp','cache','prefs']:(work/n).mkdir(exist_ok=True)
 cmd=[str(contents/'runtime/Contents/Home/bin/java'),'-Xmx512m','-XX:ActiveProcessorCount=2','-Djava.awt.headless=true','-Djava.io.tmpdir='+str(work/'tmp'),'-Dorg.bytedeco.javacpp.cachedir='+str(work/'cache'),'-Djava.util.prefs.userRoot='+str(work/'prefs'),'-Dnative.probe.output='+str(out),'-cp',cp+':'+str(work),'Audiveris','-batch','-constant','org.audiveris.omr.Main.persistBatchCliConstants=false','-constant','org.audiveris.omr.Main.runBatchTasksInParallel=false','-step','SCALE','-run','GridFinalProbe','-output',str(folder),'--',str(source)]
 with (folder/'console.log').open('w')as log:r=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,timeout=120)
 (folder/'execution.json').write_text(json.dumps({'argv':cmd,'exit':r.returncode,'input_sha256':hashlib.sha256(source.read_bytes()).hexdigest()},indent=2))
 assert r.returncode==0 and out.exists(),folder
 target=root/'fixtures/official-grid'/f'{name}-final.json';target.write_bytes(out.read_bytes());print(name,'final GRID observed',flush=True)
