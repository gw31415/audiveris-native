#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Render whole score pages independently from the existing pinned original fonts."""
import argparse,hashlib,json,os,subprocess,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PIN='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--oracle-home',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 lib=a.oracle_home/'official/opt/audiveris/lib';jar=lib/'app/audiveris.jar';java=lib/'runtime/bin/java';javac=a.oracle_home/'jdk-21.0.12.1+1/bin/javac';source=ROOT/'scripts/java/HeadActualPageRenderer.java'
 if a.output.exists() or sha(jar)!=PIN:p.error('require pinned existing oracle and new output; no setup')
 out=a.output.resolve();out.mkdir(parents=True);classes=out/'classes';classes.mkdir();env=dict(os.environ);env['XDG_CONFIG_HOME']=str(out/'config')
 compile_argv=[str(javac),'-d',str(classes),str(source)]
 with (out/'compile.log').open('w') as log:subprocess.run(compile_argv,check=True,stdout=log,stderr=subprocess.STDOUT,timeout=60,env=env)
 argv=[str(java),'-Djava.awt.headless=true','-Duser.home='+str(out/'home'),'-Djava.util.prefs.userRoot='+str(out/'prefs'),'-cp',str(lib/'app/*')+':'+str(classes),'HeadActualPageRenderer',str(out/'images')]
 with (out/'render.log').open('w') as log:subprocess.run(argv,check=True,stdout=log,stderr=subprocess.STDOUT,timeout=60,env=env)
 with zipfile.ZipFile(jar) as z:fonts={fam:hashlib.sha256(z.read('res/'+fam+'.otf')).hexdigest() for fam in ('Bravura','Leland','FinaleJazz')}
 receipt={'native_outputs_used':False,'jar_sha256':PIN,'observer_sha256':sha(source),'runtime_sha256':sha(java),'compiler_sha256':sha(javac),'font_sha256':fonts,'compile_argv':compile_argv,'argv':argv,'images':{p.name:sha(p) for p in sorted((out/'images').glob('*.png'))},'scope':'Whole synthetic score pages in actual official fonts; regular five-line staves, percussion headers/one-line and six-line drawings, mixed cue/main interlines. Six-line drawing uses G clef because original Shape enum has no TAB_CLEF symbol. Recognized special staff structure requires explicit original switches and is recorded separately. Generated pages are not scans of externally published scores.'}
 assert len(receipt['images'])==7
 (out/'provenance.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','images':len(receipt['images'])},indent=2))
if __name__=='__main__':main()
