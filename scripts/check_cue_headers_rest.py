#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify actual synthetic cue CLEF state, with separate historical baselines."""
import gzip, hashlib, json, math, sys
from pathlib import Path
from check_cue_clef import check as prior_check, near, sha, read
from check_head_page_catalog import check_control
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-cue-headers-rest'
def canonical(v):
 if isinstance(v,list): return [canonical(x) for x in v]
 if not isinstance(v,dict): return v
 v=dict(v)
 if 'kind' in v and 'grade' in v:v.pop('id',None)
 v.pop('links',None)
 if 'sig_inters' in v:
  identity=lambda n:json.dumps([n.get('kind'),n.get('shape'),n.get('bounds'),n.get('glyph')],sort_keys=True,separators=(',',':'))
  ids={n['id']:identity(n) for n in v['sig_inters']}
  v['sig_inters']=sorted(v['sig_inters'],key=identity)
  v['relations']=sorted([dict(e,source=ids[e['source']],target=ids[e['target']]) for e in v['relations']],key=lambda e:json.dumps([e['kind'],e['source'],e['target']]))
 return {k:canonical(x) for k,x in v.items()}
def check_cue(path):
 a=read(path);near(a['key_preparation']['staffs'],read(F/'preparation.json')['staffs'])
 for x,y in zip(a['key_peaks']['staffs'],read(F/'peaks.json')['staffs']):
  assert x['staff_id']==y['staff_id'];near(x['peaks'],y['peaks'])
  for nk,ok in [('min_value','min_value'),('min_derivative','min_peak_derivative'),('quorum','min_peak_cumul'),('min_gain_ratio','min_gain_ratio')]:near(x['parameters'][nk],y['params'][ok])
 for x,y in zip(a['time_column']['systems'],read(F/'time.json')['systems']):
  for k in ['system_id','clef_offset','key_offset','returned_time_offset']:near(x[k],y[k])
  for stage in ['after_retrieve_time','after_system_time_stop']:near(canonical(x[stage]),canonical(y[stage]))
 assert len(a['key_preparation']['staffs'])==len(a['key_peaks']['staffs'])==len(a['time_column']['systems'])==4
 original=read(F/'beams.json')
 for x,y in zip(a['later_beams']['systems'],original['systems']):
  assert x['system_id']==y['system_id'] and x['groups']==y['groups']['groups']==[] and x['supports']==y['groups']['supports']==[]
  assert x['rests']['complete'] is True and x['rests']['conversions']==x['rests']['registered_serifs']==0
  assert x['extensions']['initial_ids']==[] and x['hooks']['initial_ids']==[] and y['after_create_beams']['inters']==[] and y['after_extend_beams']['inters']==[] and y['after_late_hooks']['inters']==[]
 assert a['later_beams']['epilog_complete'] is True
 rest=read(F/'native-rest-queries.json.gz');near(rest['systems'],read(F/'rest-mapping.json')['systems'])
 assert sum(len(s['queries']) for s in rest['systems'])==864
 terminal=check_control(path,ROOT/'fixtures/official-head-actual-pages/Bravura-cue.json')
 assert (terminal['retained_physical_heads'],terminal['ordered_exclusions'])==(168,198)
 return {'status':'passed','scope':'One independent synthetic cue raster, declared HEADS Bravura51 catalog; not published scans or all24 shapes','cue_KEY_projection_peaks_and_observed_TIME_staffs':4,'cue_rest_boundary_points':864,'cue_rest_conversions':0,'cue_catalog_HEADS_terminal':terminal,'default_page_HEADS_staff_catalog_dependency_pending':True,'heads_complete':False,'stems_complete':False}
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['native_outputs_used'] is False and r['status']=='passed'
 assert r['input_sha256']==sha(ROOT/'fixtures/official-head-actual-pages/images/Bravura-cue.png')
 for row in r['cases']:
  n=row['name'];assert sha(F/(n+'.json'))==row['sha256'] and row['original_fields_byte_reproduced']
  a=read(F/(n+'-provenance.json'));b=read(F/(n+'-certified-provenance.json'))
  for p in [a,b]:
   assert p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and p['release']=='5.11.0'
   assert all(sha(ROOT/n)==h for n,h in p['observer_source_sha256'].items()) and 'compile-only' not in p['runtime_classpath']
   assert len(p['fixtures'])==1 and p['fixtures'][0]['input_sha256']==r['input_sha256'] and p['fixtures'][0]['ok'] and p['fixtures'][0]['exit_code']==0
  for k in ['observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256']:assert a[k]==b[k]
 p=read(F/'native-reproduction.json');assert sha(F/'native-rest-queries.json.gz')==p['native_queries_gzip_sha256'];assert p['status']=='passed' and all(sha(ROOT/n)==h for n,h in p['native_source_sha256'].items()) and sha(F/'native.json.gz')==p['native_report_gzip_sha256']
 return {'status':'passed','independent_synthetic_cue_HEADERS_rest_and_catalog_HEADS':check_cue(F/'native.json.gz'),'current_default_cue_page_CLI_dependency':'HEADS staff-specific template catalog','actual_published_scan_parity':False,'full24_HEADS_page_propagation':False,'automatic_STEMS_complete':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check_cue(Path(sys.argv[1])) if len(sys.argv)>2 else check(Path(sys.argv[1])),indent=2))
