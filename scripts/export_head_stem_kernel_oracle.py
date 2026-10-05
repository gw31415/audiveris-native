#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Export direct original HeadStemRelation controls with the existing Linux oracle."""
import argparse,hashlib,json,os,subprocess
from pathlib import Path
PIN='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--oracle-home',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 if a.output.exists():p.error('output must not exist')
 home=a.oracle_home.resolve();lib=home/'official/opt/audiveris/lib';jar=lib/'app/audiveris.jar';runtime=lib/'runtime/bin/java';compiler=home/'jdk-21.0.12.1+1/bin/javac';source=ROOT/'scripts/java/HeadStemKernelProbe.java'
 if not source.is_file() or not runtime.is_file() or not compiler.is_file() or sha(jar)!=PIN:p.error('existing pinned oracle/source required; no bootstrap')
 out=a.output.resolve();out.mkdir(parents=True);classes=out/'classes';classes.mkdir();(out/'home').mkdir();env=dict(os.environ);env['XDG_CONFIG_HOME']=str(out/'config')
 compile_argv=[str(compiler),'-d',str(classes),str(source)]
 with (out/'compile.log').open('w') as log:r=subprocess.run(compile_argv,stdout=log,stderr=subprocess.STDOUT,timeout=60,env=env)
 if r.returncode:raise RuntimeError('observer compilation failed; see compile.log')
 argv=[str(runtime),'-Xmx256m','-Djava.awt.headless=true','-Duser.home='+str(out/'home'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-cp',str(lib/'app/*')+':'+str(classes),'HeadStemKernelProbe',str(out/'controls.json')]
 with (out/'console.log').open('w') as log:r=subprocess.run(argv,stdout=log,stderr=subprocess.STDOUT,timeout=60,env=env)
 if r.returncode:raise RuntimeError('original kernel observer failed; see console.log')
 controls=json.loads((out/'controls.json').read_text());assert controls['native_outputs_used'] is False
 (out/'provenance.json').write_text(json.dumps({'native_outputs_used':False,'release':'5.11.0','upstream_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','jar_sha256':PIN,'observer_sha256':sha(source),'controls_sha256':sha(out/'controls.json'),'compiler_sha256':sha(compiler),'runtime_sha256':sha(runtime),'compile_argv':compile_argv,'argv':argv,'exit_code':r.returncode,'controlled_counts':{k:len(controls[k]) for k in ['portions','consistency','invading','gap_limits']},'scope':'Default constants/direct HeadStemRelation kernels; no native input, stem attachment, shared SIG update, complete HEADS or STEMS claim.'},indent=2)+'\n')
 print(json.dumps({'status':'passed','counts':{k:len(controls[k]) for k in ['portions','consistency','invading','gap_limits']}},indent=2))
if __name__=='__main__':main()
