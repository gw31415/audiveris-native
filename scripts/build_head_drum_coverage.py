#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Package exact Linux official templates for declared HEADS kernel coverage.
Development only. Uses existing Linux dependencies. Historical builtin and
ordinary font coverage catalogs remain unchanged. Output must not already exist.
"""
import argparse, hashlib, json, pathlib, platform, subprocess, zipfile
ROOT = pathlib.Path(__file__).resolve().parents[1]
JAR_SHA = '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
COMMIT = '9e1e55cd2746037d059345881c53e6a6754bffbd'
FAMILIES = ('Bravura', 'Leland', 'FinaleJazz')
def sha(data): return hashlib.sha256(data).hexdigest()
def dump(path, value): path.write_text(json.dumps(value, indent=2, allow_nan=False)+'\n')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--oracle-home',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
 assert platform.system()=='Linux'
 dist=a.oracle_home/'official/opt/audiveris/lib';jar=dist/'app/audiveris.jar';assert sha(jar.read_bytes())==JAR_SHA
 java=dist/'runtime/bin/java';javac=a.oracle_home/'jdk-21.0.12.1+1/bin/javac'
 a.output.mkdir(parents=True,exist_ok=False);classes=a.output/'classes';classes.mkdir()
 observer=ROOT/'scripts/java/HeadDrumBankOracle.java'
 compile_cmd=[str(javac),'--release','21','-d',str(classes),str(observer)]
 subprocess.run(compile_cmd,check=True)
 receipts={'jar_sha256':JAR_SHA,'upstream_commit':COMMIT,'platform':platform.platform(),'native_outputs_used':False,'observer_sha256':sha(observer.read_bytes()),'compiler_sha256':sha(javac.read_bytes()),'runtime_sha256':sha(java.read_bytes()),'java_version':subprocess.run([str(java),'-version'],capture_output=True,text=True,check=True).stderr,'compile_command':compile_cmd,'families':{}}
 methods=[]
 with zipfile.ZipFile(jar) as z:
  modelsha=sha(z.read('res/basic-classifier.zip'))
  for family in FAMILIES:
   folder=a.output/family;folder.mkdir()
   command=[str(java),'-Duser.home='+str(folder),'-Djava.util.prefs.userRoot='+str(folder/'prefs'),'-Djava.awt.headless=true','-cp',str(dist/'app/*')+':'+str(classes),'HeadDrumBankOracle',str(folder),family]
   with (a.output/(family+'.log')).open('w') as log: subprocess.run(command,check=True,stdout=log,stderr=subprocess.STDOUT,timeout=120)
   data=json.loads((folder/'assets.json').read_text());cal=json.loads((folder/'calibration.json').read_text())
   assert data['family']==family and len(data['assets'])==36
   templates=[{'key':{'shape':x['shape'],'point_size':x['point_size']},'width':x['width'],'height':x['height'],'fat_bounds':[int(v) for v in x['fat_bounds']],'slim_bounds':[int(v) for v in x['slim_bounds']],'anchors':x['anchors'],'points':[{'x':v[0],'y':v[1],'d':v[2]} for v in x['points']]} for x in data['assets']]
   payload=(json.dumps({'schema':1,'templates':templates,'calibration':cal},separators=(',',':'),allow_nan=False)+'\n').encode()
   fontsha=sha(z.read('res/'+family+'.otf'));version='1.392' if family=='Bravura' else 'bundled-in-5.11.0'
   coverage=(json.dumps({'family':family,'point_sizes':[32,40,51,64],'shapes':['BREVE','WHOLE_NOTE','NOTEHEAD_VOID','NOTEHEAD_BLACK','NOTEHEAD_BLACK_SMALL','NOTEHEAD_CROSS','NOTEHEAD_DIAMOND_FILLED','NOTEHEAD_TRIANGLE_DOWN_FILLED','NOTEHEAD_CIRCLE_X']},sort_keys=True)+'\n').encode()
   manifest={'schema':1,'bank_id':'audiveris-5.11.0-'+family.lower()+'-heads-drum-coverage-v1','upstream_version':'5.11.0','upstream_commit':COMMIT,'font_family':family,'font_version':version,'font_sha256':fontsha,'model_name':'basic-classifier.zip','model_sha256':modelsha,'coverage_input_sha256':sha(coverage),'generator':'HeadDrumBankOracle; official unmodified 5.11 JAR; '+data['java']+'; '+data['os']+'; default TemplateFactory constants','asset_sha256':sha(payload),'keys':[x['key'] for x in templates],'calibration_sizes':[x['point_size'] for x in cal]}
   mb=(json.dumps(manifest,indent=2)+'\n').encode();dest=ROOT/'crates/audiveris-template-bank/assets/drum'/family;dest.mkdir(parents=True,exist_ok=True);(dest/'manifest.json').write_bytes(mb);(dest/'templates.json').write_bytes(payload)
   identity={'manifest_sha256':sha(mb),'upstream_commit':COMMIT,'font_sha256':fontsha,'model_sha256':modelsha,'coverage_input_sha256':sha(coverage)}
   trust='Trust { '+', '.join(k+': "'+v+'"' for k,v in identity.items())+' }'
   methods.append('            "'+family+'" => Self::from_bytes(include_bytes!("../assets/drum/'+family+'/manifest.json"), include_bytes!("../assets/drum/'+family+'/templates.json"), '+trust+'),')
   receipts['families'][family]={'command':command,'manifest_sha256':sha(mb),'payload_sha256':sha(payload),'templates':36,'keypoints':sum(len(x['points']) for x in templates),'font_sha256':fontsha,'calibration':[50,51,54],'font_version_label':version,'coverage_sha256':sha(coverage)}
 (ROOT/'crates/audiveris-template-bank/src/drum_identity.rs').write_text('// SPDX-License-Identifier: AGPL-3.0-or-later\n// Generated exact official drum-template catalog.\nimpl Bank {\n    /// Exact, bounded catalog; missing family/shape/size never interpolates.\n    pub fn load_drum_coverage(family: &str) -> Result<Self, Error> {\n        match family {\n'+'\n'.join(methods)+'\n            _ => Err(Error::UnsupportedTemplate),\n        }\n    }\n}\n')
 dump(a.output/'provenance.json',receipts)
 print('Packaged 108 original templates; historical banks unchanged')
if __name__=='__main__': main()
