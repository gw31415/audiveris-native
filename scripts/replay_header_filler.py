#!/usr/bin/env python3
"""Bounded saved-book replay. Never calls reachStep, saves a Book, or recognizes an image."""
from pathlib import Path
import argparse, hashlib, json, os, subprocess, time

parser = argparse.ArgumentParser()
parser.add_argument('--book', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
package = Path(__file__).resolve().parent
out = args.output.resolve()
if out.exists():
    raise SystemExit('output directory must be new; retain previous observations')
out.mkdir(parents=True)
lib = Path('/workspace/native-build-tools/oracle/official/opt/audiveris/lib')
javac = Path('/workspace/native-build-tools/oracle/jdk-21.0.12.1+1/bin/javac')
for name in ['classes', 'tmp', 'cache', 'prefs', 'home', 'config', 'data']:
    (out / name).mkdir()
(out/'logback.xml').write_text('<configuration><root level="ERROR"/></configuration>\n')
hash_file = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
record = {'book_before': hash_file(args.book), 'jar_before': hash_file(lib/'app/audiveris.jar'), 'no_images_or_reachStep': True, 'no_book_or_Jar_writes': True}
command = [str(javac), '--release', '21', '-d', str(out/'classes'), str(package/'java/MeasureFillerClefKeyBookProbe.java'), str(package/'java/MeasureFillerFirstAddReplayProbe.java')]
with (out/'compile.log').open('w') as log:
    compiled = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=60)
record.update(compile=command, compile_exit=compiled.returncode)
if not compiled.returncode:
    command = [str(lib/'runtime/bin/java'), '-Xmx512m', '-XX:ActiveProcessorCount=2', '-Djava.awt.headless=true', '-Dlogback.configurationFile='+str(out/'logback.xml'), '-Djava.io.tmpdir='+str(out/'tmp'), '-Dorg.bytedeco.javacpp.cachedir='+str(out/'cache'), '-Djava.util.prefs.userRoot='+str(out/'prefs'), '-Duser.home='+str(out/'home'), '-cp', str(lib/'app/*')+os.pathsep+str(out/'classes'), 'MeasureFillerFirstAddReplayProbe', str(args.book), str(out/'observed.json')]
    env = os.environ.copy()
    for name, value in [('XDG_CONFIG_HOME', 'config'), ('XDG_CACHE_HOME', 'cache'), ('XDG_DATA_HOME', 'data')]:
        env[name] = str(out/value)
    start = time.monotonic()
    with (out/'console.log').open('w') as log:
        result = subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=90)
    record.update(run=command, run_exit=result.returncode, seconds=time.monotonic()-start)
record.update(book_after=hash_file(args.book), jar_after=hash_file(lib/'app/audiveris.jar'))
(out/'RUN-COMMAND.json').write_text(json.dumps(record, indent=2)+'\n')
if record['book_before'] != record['book_after'] or record['jar_before'] != record['jar_after']:
    raise SystemExit('source Book or Jar changed')
raise SystemExit(record.get('run_exit', record['compile_exit']))
