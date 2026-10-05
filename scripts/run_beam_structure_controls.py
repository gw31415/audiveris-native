#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Regenerate BeamStructure development controls using cached official binaries only.

This script never downloads source/dependencies and never touches a user computer.
It writes into a new directory. Java is not part of the native runtime.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PIN = '9e1e55cd2746037d059345881c53e6a6754bffbd'
JAR_SHA = '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--oracle-home', type=Path, default=ROOT.parent/'linux-oracle')
    parser.add_argument('--output', type=Path, required=True, help='New, nonexisting evidence directory')
    args=parser.parse_args()
    output=args.output.resolve()
    home=args.oracle_home.resolve()
    dist=home/'official/opt/audiveris/lib'
    jar=dist/'app/audiveris.jar'
    java=dist/'runtime/bin/java'
    javac=home/'jdk-21.0.12.1+1/bin/javac'
    if not all(p.is_file() for p in [jar,java,javac]):
        raise RuntimeError('Cached original distribution/JDK missing; no acquisition is attempted')
    if sha(jar)!=JAR_SHA:
        raise RuntimeError('Official Audiveris JAR hash mismatch')
    output.mkdir(parents=True,exist_ok=False)
    classes=output/'observer-classes';classes.mkdir()
    sources=output/'observer-sources';sources.mkdir()
    source=sources/'BeamStructureControlProbe.java'
    shutil.copyfile(ROOT/'scripts/java/BeamStructureControlProbe.java',source)
    generator=sources/'make_beam_structure_controls.py'
    shutil.copyfile(ROOT/'scripts/make_beam_structure_controls.py',generator)
    commands=[['python3',str(generator),str(output/'inputs')],
        [str(javac),'--release','21','-cp',str(dist/'app/*'),'-d',str(classes),str(source)]]
    for stem in ['controls','methods']:
        commands.append([str(java),'-Djava.awt.headless=true','-cp',str(classes)+':'+str(dist/'app/*'),
            'BeamStructureControlProbe',str(output/'inputs'/f'{stem}-input.json'),str(output/f'{stem}-original.json')])
    executions=[]
    for i,command in enumerate(commands):
        with (output/f'{i:02}-console.log').open('w') as log:
            result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=180,check=False)
        executions.append({'command':command,'exit_code':result.returncode})
        if result.returncode:
            raise RuntimeError(f'Control command {i} failed; see its console log')
    provenance={'upstream_commit':PIN,'official_jar_sha256':sha(jar),'runtime_executable_sha256':sha(java),
        'compiler_executable_sha256':sha(javac),'observer_sha256':sha(source),'generator_sha256':sha(generator),
        'scope':'Direct calls to unmodified original BeamStructure; no builder or production dispatch decision replay',
        'method_controls_scope':'Supplied BeamLine states test geometry methods; computeLines is not called for these cases',
        'java_version':subprocess.run([str(java),'-version'],capture_output=True,text=True,check=True).stderr,
        'executions':executions,
        'fixtures':{stem:{'input_sha256':sha(output/'inputs'/f'{stem}-input.json'),'output_sha256':sha(output/f'{stem}-original.json'),
            'case_count':len(json.loads((output/f'{stem}-original.json').read_text())['cases'])} for stem in ['controls','methods']}}
    (output/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
    print(json.dumps(provenance['fixtures'],indent=2))

if __name__=='__main__':
    main()
