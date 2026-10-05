#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Owned registered cached stems, explicit heads, consistency and SIG callbacks."""
from pathlib import Path
import json,sys
from check_stem_chunks_cli import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-cache-head'
def near(a,b):
 if isinstance(b,float):assert isinstance(a,(float,int)) and abs(a-b)<1e-12,(a,b)
 elif isinstance(b,dict):
  for k,v in b.items():near(a[k],v)
 elif isinstance(b,list):
  assert len(a)==len(b)
  for x,y in zip(a,b):near(x,y)
 else:assert a==b,(a,b)
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['native_outputs_used'] is False and r['independent_compilations_and_runs']==2 and r['cases']==1152 and r['accepted_relations']==768
 assert r['original_sha256']==sha(F/'controls.json') and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png') and r['original_createStem_checkRelation_setConsistency_SIG_callbacks_called'] is True
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemCacheHeadProbe' and x['official_cli_step']=='HEADS'
  assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only' not in x['runtime_classpath']
  assert all(sha(ROOT/k)==h for k,h in x['observer_source_sha256'].items())
  assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and n['float_absolute_tolerance']==1e-12 and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'];assert all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');old=read(ROOT/'fixtures/official-stem-cache-sig/controls.json');assert o['native_outputs_used'] is False and o['interline']==11 and len(o['groups'])==len(v['groups'])==len(old['cases'])==12
 total=accepted=0
 for a,b,previous in zip(o['groups'],v['groups'],old['cases']):
  assert a['parts']==previous['parts'] and a['created']['id']==1 and a['created']['in_sig'] is True
  for k in ('grade','bounds','median','width','glyph','impacts'):assert a['created'][k]==previous['after_SIG'][k]
  near(b['created'],a['created']);assert len(a['cases'])==len(b['cases'])==96
  for c,d in zip(a['cases'],b['cases']):
   near(d['relation'],c['relation'])
   for k in ('before','after','relation_delta','cached_same_object_with_white_image','cached_positive_SIG_id_stable','after_edge_removal'):assert c[k]==d[k]
   assert c['cached_same_object_with_white_image'] is True and c['cached_positive_SIG_id_stable'] is True and c['after_edge_removal']=={'head_abnormal':True,'stem_abnormal':True}
   if c['relation']is None:assert c['relation_delta']==0 and c['before']==c['after']
   else:assert c['relation_delta']==1 and c['after']=={'head_abnormal':False,'stem_abnormal':False};accepted+=1
   total+=1
 assert total==1152 and accepted==768 and v['registry_and_cache_identity_preserved'] is True and all(v[k]is False for k in ('automatic_parts_or_page_STEMS_complete','epilog_complete','heads_complete','stems_complete'))
 return {'status':'passed','selected_original_cached_stems':12,'cache_head_relation_SIG_callback_controls':1152,'accepted_relations':768,'exact_glyph_identity_SIG_states':True,'floating_absolute_tolerance':1e-12,'cached_Arc_positive_SIG_identity_retained_on_white_image':True,'explicit_relation_removal_and_unrelated_SIG_preservation_verified':True,'explicit_Bravura51_two_head_shapes_profiles012_interline11':True,'automatic_items_parts_cache_page_integration_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
