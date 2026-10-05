#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Regenerate item-grading controls from cached official binaries, without networking.
Java/reflection is development-only; the Rust runtime never invokes this observer.
"""
import argparse,hashlib,json,shutil,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PIN='9e1e55cd2746037d059345881c53e6a6754bffbd'
JAR_SHA='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--oracle-home',type=Path,default=ROOT.parent/'linux-oracle');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    home=a.oracle_home.resolve();output=a.output.resolve();dist=home/'official/opt/audiveris/lib'
    jar=dist/'app/audiveris.jar';java=dist/'runtime/bin/java';javac=home/'jdk-21.0.12.1+1/bin/javac'
    if not all(x.is_file() for x in [jar,java,javac]):raise RuntimeError('Cached original distribution/JDK missing; no acquisition attempted')
    if sha(jar)!=JAR_SHA:raise RuntimeError('Official Audiveris JAR hash mismatch')
    output.mkdir(parents=True,exist_ok=False);classes=output/'observer-classes';classes.mkdir();sources=output/'observer-sources';sources.mkdir()
    source=sources/'BeamImpactsControlProbe.java';shutil.copyfile(ROOT/'scripts/java/BeamImpactsControlProbe.java',source)
    generator=sources/'make_beam_impacts_controls.py';shutil.copyfile(ROOT/'scripts/make_beam_impacts_controls.py',generator)
    commands=[['python3',str(generator),str(output/'input.json')], [str(javac),'--release','21','-cp',str(dist/'app/*'),'-d',str(classes),str(source)], [str(java),'-Djava.awt.headless=true','-cp',str(classes)+':'+str(dist/'app/*'),'BeamImpactsControlProbe',str(output/'input.json'),str(output/'original.json')]]
    executions=[]
    for i,command in enumerate(commands):
        with (output/f'{i:02}-console.log').open('w') as log:r=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=180,check=False)
        executions.append({'command':command,'exit_code':r.returncode})
        if r.returncode:raise RuntimeError(f'Control command {i} failed; inspect its log')
    original=json.loads((output/'original.json').read_text())
    provenance={'upstream_commit':PIN,'official_jar_sha256':sha(jar),'runtime_executable_sha256':sha(java),'compiler_executable_sha256':sha(javac),'observer_sha256':sha(source),'generator_sha256':sha(generator),'input_sha256':sha(output/'input.json'),'output_sha256':sha(output/'original.json'),'case_count':len(original['cases']),'scope':original['mode'],'observed_methods':['BeamItem.getCoreArea','BeamItem.getBeltArea','AreaMask.fore','BeamsBuilder.computeBeamImpacts','BeamsBuilder.computeHookImpacts','GradeImpacts.getGrade'],'method_boundary':'Supplied independent rasters and item medians; original builder allocated reflectively with only params and pixelFilter. No outer orchestration or production dispatch is asserted.','executions':executions,'java_version':subprocess.run([str(java),'-version'],capture_output=True,text=True,check=True).stderr}
    (output/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');print(json.dumps(provenance,indent=2))
if __name__=='__main__':main()
