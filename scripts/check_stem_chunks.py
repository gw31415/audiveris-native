#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Full original lookupChunks comparison with supplied upstream context."""
from pathlib import Path
import json,sys
from check_stem_chunk_head_filter import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-chunks'
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['native_outputs_used'] is False and r['independent_compilations_and_runs']==2 and r['cases']==3048
 assert r['original_sha256']==sha(F/'controls.json') and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png') and r['direct_original_lookupChunks_called'] is True
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemChunksProbe' and x['official_cli_step']=='HEADS'
  assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only' not in x['runtime_classpath']
  assert all(sha(ROOT/n)==h for n,h in x['observer_source_sha256'].items())
  assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'];assert all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');assert o['native_outputs_used'] is False and o['JVM_ids_normalized_to_canonical_identity_tokens'] is True and o['builders']==612 and len(o['groups'])==len(v['groups'])==612
 previous=[c for c in read(ROOT/'fixtures/official-stem-parts-lookup/controls.json')['cases']if c['mode']=='actual'];count=0
 for a,b,old in zip(o['groups'],v['groups'],previous):
  assert a['cases'][0]['raw_glyphs']==old['raw_sticks_min_core0'];assert len(a['cases'])==len(b['cases'])==(5 if a['kind']=='CLinker' else 4)
  assert a['cases'][3]['mode']=='null-stump' and a['cases'][3]['stump_id'] is None
  for c,d in zip(a['cases'],b['cases']):
   assert c['slope']==0.0
   for k in ('registered_ids','retained_ids','new_count','registry_count','final_catalog'):assert c[k]==d[k]
   assert len(c['final_catalog'])==c['registry_count'];assert set(c['retained_ids'])<=set(c['registered_ids']);count+=1
 assert count==3048 and all(v[k]is False for k in ('automatic_page_STEMS_complete','registered_to_SIG','heads_complete','stems_complete'))
 return {'status':'passed','actual_first_system_builders':612,'direct_original_lookupChunks_controls':3048,'exact_registered_and_sorted_retained_ids_counts_physical_catalog':True,'same_owner_stump_existing_aliases_null_stump_VIP_and_empty_checked':True,'upstream_raw_and_geometry_supplied':True,'automatic_page_STEMS_complete':False,'cache_head_SIG_integration_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
