#!/usr/bin/env python3
"""Extract bounded original CHECKED controls; never supply recognition input."""
import argparse,hashlib,json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
seven_path=Path('/tmp/seven-label-checked-proposed-v1/fixtures/official-seven-label-checked/controls.json')
accent_path=Path('/tmp/accent-staff-gap-proposed-v1/fixtures/official-accent-staff-gap/observed.json')
nine_path=Path('/tmp/real-rest-nine-label-original-v1/observed.json')
summary_path=nine_path.parent/'SUMMARY.json'
seven_raw_path=Path('/tmp/real-seven-label-lifecycle-v3/observed.json');seven_raw=json.loads(seven_raw_path.read_text());seven=json.loads(seven_path.read_text());accent=json.loads(accent_path.read_text());nine=json.loads(nine_path.read_text());summary=json.loads(summary_path.read_text())
laura={'labels':['QUINDICESIMA','G_CLEF','CODA','F_CLEF','F_CLEF_SMALL','FERMATA_BELOW','OTTAVA'],'rejected':seven['rejected'],'factory_arrivals':seven['positive_factory_candidates'],'positive_clef_geometry':seven['positive_clef_geometry'],'accent':{k:accent[k]for k in ['system','bounds','center2D','closest_staff','pitch','original_checked','max_gap','inferred_original_checker','exact_original_gap_observed','inference_qualification','original_accent_failure_string_observed']},'accepted_limit_controls':[],'unmatched_native':[]}
for arrival in laura['factory_arrivals']:
 source_factory=[f for f in seven_raw['factory_candidates']if f['system']==arrival['system']and f['bounds']==arrival['bounds']and f['shape']==arrival['shape']and abs(f['evaluation_grade']-arrival['evaluation_grade'])<1e-10]
 assert len(source_factory)==1
 f=source_factory[0]
 source_trial=[t for t in seven_raw['classifier_trials']if t['system']==f['system']and t['glyph_id']==f['glyph_id']]
 assert len(source_trial)==1
 arrival['actual_checked_return']=source_trial[0]['actual_checked_return']
rejected=[]
for d in summary['native_vs_original_checked_differences']:
 if 'original_failure'not in d:continue
 n=d['native'];matches=[r for r in nine['classifier_trials']if r['system']==n['system']and r['bounds']==n['bounds']and any(e['shape']==n['shape']and abs(e['grade']-n['score'])<1e-10 for e in r['raw_target_evaluations'])]
 assert len(matches)==1
 r=matches[0];e=next(e for e in r['raw_target_evaluations']if e['shape']==n['shape'])
 assert e['failure']==d['original_failure'] and r['actual_checked_return']==[]
 rejected.append({'system':r['system'],'bounds':r['bounds'],'center2D':r['center2D'],'shape':e['shape'],'evaluation_grade':e['grade'],'failure':e['failure'],'actual_checked_return':r['actual_checked_return'],'raw_rank':e['raw_rank'],'intermediate_pitch_and_gap_observed':False})
arrivals=[]
for r in nine['factory_candidates']:
 initial=r.get('result')or{}
 trial=[t for t in nine['classifier_trials']if t['system']==r['system']and t['glyph_id']==r['glyph_id']];assert len(trial)==1
 arrivals.append({'system':r['system'],'bounds':r['bounds'],'shape':r['shape'],'evaluation_grade':r['evaluation_grade'],'returns_null':r['returns_null'],'actual_checked_return':trial[0]['actual_checked_return'],'original_factory_initial':{k:initial.get(k)for k in ['kind','staff','manual','abnormal']}})
limits=[]
for r in nine['classifier_trials']:
 for e in r['raw_target_evaluations']:
  if e['failure']is None and e['raw_rank']>=3:
   limits.append({'system':r['system'],'bounds':r['bounds'],'raw_evaluation':e,'actual_checked_return':r['actual_checked_return'],'declared_limit':r['declared_limit'],'min_grade':r['min_grade']})
residual=[d['native']for d in summary['native_vs_original_checked_differences']if d.get('matching_original_raw_count')==0]
assert len(rejected)==10 and len(arrivals)==77 and len(limits)==4 and len(residual)==2
winter={'labels':nine['observed_target_shapes'],'rejected':rejected,'factory_arrivals':arrivals,'accepted_limit_controls':limits,'unmatched_native':residual,'source_runtime_intermediates_qualified':True}
sources=[{'path':str(s),'sha256':hashlib.sha256(s.read_bytes()).hexdigest()}for s in [seven_path,seven_raw_path,accent_path,nine_path,summary_path]]
v={'scope':'Original CHECKED observations and protected factory arrivals only. Partial rejection unit; no factory lifecycle or recognition-quality parity claim. Expected data is never native recognition input.','original_commit':'9e1e55cd2746037d059345881c53e6a6754bffbd','an_laura':laura,'der_winter':winter,'sources':sources}
def bounded_json(value,level=0):
 pad='  '*level
 if isinstance(value,dict):
  return '{\n'+',\n'.join('  '*(level+1)+json.dumps(k)+': '+bounded_json(x,level+1)for k,x in value.items())+'\n'+pad+'}'
 if isinstance(value,list)and value and all(isinstance(x,dict)for x in value):
  return '[\n'+',\n'.join('  '*(level+1)+json.dumps(x,separators=(',',':'))for x in value)+'\n'+pad+']'
 return json.dumps(value,separators=(',',':'))
text=bounded_json(v)+'\n'
assert json.loads(text)==v
a.output.write_text(text)
