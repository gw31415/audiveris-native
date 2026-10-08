#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Measure original base-interline FontSymbol resources without recognition."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import linux_oracle as oracle

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--oracle-home', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
output = args.output.resolve()
output.mkdir()
for name in ('classes', 'home', 'cache', 'data', 'config', 'prefs'):
    (output / name).mkdir()
lib = args.oracle_home.resolve() / 'official/opt/audiveris/lib'
oracle.checked(lib / 'app/audiveris.jar', oracle.JAR_SHA256)
source = oracle.ROOT / 'scripts/java/TupletFontProbe.java'
helper = oracle.ROOT / 'scripts/java/HeadStemPreLinkProbe.java'
compile_args = [str(args.oracle_home.resolve() / oracle.JDK_DIR / 'bin/javac'),
                '--release', '21', '-cp', str(lib / 'app/*'), '-d',
                str(output / 'classes'), str(source), str(helper)]
with (output / 'compile.log').open('w') as log:
    subprocess.run(compile_args, stdout=log, stderr=subprocess.STDOUT, check=True)
run_args = [str(lib / 'runtime/bin/java'), '-Xmx512m', '-XX:ActiveProcessorCount=2',
            '-Djava.awt.headless=true', '-Duser.home=' + str(output / 'home'),
            '-Djava.util.prefs.userRoot=' + str(output / 'prefs'), '-cp',
            str(lib / 'app/*') + os.pathsep + str(output / 'classes'),
            'TupletFontProbe', str(output / 'observed.json')]
env = dict(os.environ)
for key, name in (('XDG_CONFIG_HOME', 'config'), ('XDG_DATA_HOME', 'data'), ('XDG_CACHE_HOME', 'cache')):
    env[key] = str(output / name)
with (output / 'console.log').open('w') as log:
    result = subprocess.run(run_args, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=60)
record = dict(upstream_commit=oracle.UPSTREAM_COMMIT, jar_sha256=oracle.JAR_SHA256,
              observer_sha256=oracle.sha(source), reflection_helper_sha256=oracle.sha(helper),
              compile_argv=compile_args, run_argv=run_args, exit_code=result.returncode,
              console_sha256=oracle.sha(output / 'console.log'), native_outputs_used=False)
if result.returncode == 0:
    record['observed_sha256'] = oracle.sha(output / 'observed.json')
(output / 'provenance.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(dict(output=str(output), exit_code=result.returncode)))
raise SystemExit(result.returncode)
