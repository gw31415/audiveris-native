#!/usr/bin/env python3
"""Regenerate using an existing official 5.11.0 macOS distribution; no downloads."""
import hashlib, json, os, pathlib, subprocess
root = pathlib.Path(__file__).resolve().parent.parent
home = pathlib.Path(os.environ['AUDIVERIS_CONTENTS']).resolve()
jar = home / 'app/audiveris.jar'
expected = '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
assert hashlib.sha256(jar.read_bytes()).hexdigest() == expected, 'Unexpected oracle JAR'
java = home / 'runtime/Contents/Home/bin/java'
classes = root / 'oracle/classes'
classes.mkdir(exist_ok=True)
subprocess.run([os.environ.get('JAVAC', 'javac'), '-d', str(classes), str(root/'oracle/TemplateOracle.java')], check=True)
with (root/'oracle/generation.log').open('w') as log:
    subprocess.run([str(java), '-Xmx512m', '-XX:ActiveProcessorCount=2', '-Djava.awt.headless=true', '-cp', str(classes)+os.pathsep+str(home/'app/*'), 'TemplateOracle', str(root/'fixtures/java-golden.json')], stdout=log, stderr=subprocess.STDOUT, check=True)
data = json.loads((root/'fixtures/java-golden.json').read_text())
print('Java', data['java'], 'distance cases:', len(data['distance_cases']), 'scores:', sum(len(t['scores']) for t in data['templates']))
