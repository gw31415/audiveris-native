#!/usr/bin/env python3
from pathlib import Path
import os,subprocess,json,hashlib
ROOT=Path(__file__).resolve().parent.parent;os.chdir(ROOT)
APP=Path('/Applications/Audiveris.app/Contents');JAVA=APP/'runtime/Contents/Home/bin/java'
TOOLS=Path(os.environ.get('ORACLE_TOOLS',str(ROOT.parent/'oracle/tools')))
CLASSES=Path('oracle/classes');CLASSES.mkdir(exist_ok=True)
cp=os.pathsep.join(str(p) for p in (APP/'app').glob('*.jar'))
src=[str(p) for p in Path('oracle/upstream').rglob('*.java')]+[str(p) for p in Path('oracle/stubs').rglob('*.java')]+['oracle/NetworkOracle.java','oracle/GlyphOracle.java']
cmd=[str(JAVA),'-cp',str(TOOLS/'bootstrap')+os.pathsep+str(TOOLS/'ecj-3.38.0.jar'),'org.eclipse.jdt.internal.compiler.batch.Main','-21','-proc:none','-encoding','UTF-8','-cp',cp,'-d',str(CLASSES)]+src
r=subprocess.run(cmd,capture_output=True,text=True);Path('verification/java-build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
names=['slf4j-api-2.0.16.jar','logback-classic-1.4.14.jar','logback-core-1.4.14.jar','jaxb-api-2.3.1.jar','jaxb-core-2.3.0.1.jar','jaxb-impl-2.3.1.jar','javax.activation-api-1.2.0.jar','jackson-databind-2.13.0.jar','jackson-core-2.13.0.jar','jackson-annotations-2.13.0.jar']
cp=os.pathsep.join([str(CLASSES)]+[str(APP/'app'/n) for n in names])
outputs=[]
for suffix in ['first','repeat']:
 p=Path(f'verification/java-glyph-{suffix}.json');cmd=[str(JAVA),'-Djava.awt.headless=true','-Xlog:class+load=info:file=verification/java-class-load.log','-cp',cp,'GlyphOracle',str(p),'../audiveris-classifier/models/basic-classifier.zip','oracle/Bravura.otf','oracle/synthetic-inputs.json']
 r=subprocess.run(cmd,capture_output=True,text=True);Path(f'verification/java-{suffix}.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr;outputs.append(p.read_bytes())
assert outputs[0]==outputs[1]==Path('tests/fixtures/java-glyph-golden.json').read_bytes()
assert 'audiveris.jar' not in Path('verification/java-class-load.log').read_text()
print('Pinned official glyph oracle, repeated byte-identical:',hashlib.sha256(outputs[0]).hexdigest())
