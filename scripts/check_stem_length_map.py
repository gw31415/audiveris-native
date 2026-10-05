#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Declared original items, exact getLengthAt and gap/profile length maps."""
from pathlib import Path
import json,sys
from check_stem_cache_head import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-length-map'
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['native_outputs_used'] is False and r['independent_compilations_and_runs']==2 and r['conditions']==11205 and r['builders']==612 and r['items']==1278 and r['original_getLengthAt_retrieveLengths_called'] is True
 assert r['original_sha256']==sha(F/'controls.json') and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemLengthMapProbe' and x['official_cli_step']=='HEADS'
  assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only' not in x['runtime_classpath']
  assert all(sha(ROOT/k)==h for k,h in x['observer_source_sha256'].items())
  assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'];assert all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');assert o['native_outputs_used'] is False and o['gap_map']=={'0':0,'1':3,'2':7,'3':22,'4':44} and len(o['groups'])==len(v['groups'])==612
 count=items=gapped=0;directions=set();modes=set()
 for a,b in zip(o['groups'],v['groups']):
  assert a['prefix_lengths']==b['prefix_lengths'] and len(b['case_lengths'])==len(a['cases'])
  assert [x['prefix_lengths']for x in a['start_y_controls']]==b['start_y_prefix_lengths']
  for c,d in zip(a['cases'],b['case_lengths']):assert c['lengths']==d;modes.add(c['mode']);count+=1
  items+=len(a['items']);gapped+=any(x['gap']for x in a['items']);directions.add(a['direction'])
 assert count==v['controls']==11205 and items==1278 and gapped==27 and directions=={-1,1} and {'actual','empty','preserve-first-profile','threshold-0--1','threshold-4-1'}<=modes
 assert all(v[k]is False for k in ('automatic_items_parts_page_complete','epilog_complete','heads_complete','stems_complete'))
 return {'status':'passed','actual_original_builders':612,'original_items':1278,'exact_length_map_controls':11205,'original_gapped_builders':27,'native_line_bounds_integer_prefixes_and_maps_exact':True,'strict_gap_threshold_sticky_first_stop_empty_signed_fractional_start_verified':True,'original_item_geometry_supplied':True,'automatic_items_parts_page_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
