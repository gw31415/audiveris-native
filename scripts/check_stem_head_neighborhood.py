#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import json,sys
from pathlib import Path
from check_stem_head_targets import check as prior_check,ring
from check_stem_cache_head import read,sha
from stem_head_plan_request import request
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-head-neighborhood'
def compare(o,v):
 assert len(o['systems'])==len(v['systems'])==2
 for a,b in zip(o['systems'],v['systems']):
  assert a['system']==b['system'] and len(a['cases'])==len(b['cases'])
  for c,d in zip(a['cases'],b['cases']):
   assert c['neighbors']==d['neighbors'] and c['candidates']==d['candidates'];t=d['target']
   for k in ('selected_beams','selected_groups','target_beam','initial_line','target'):assert c[k]==t[k]
   assert c['area_line']==t['area']['theoretical_line'];assert all(p['type']in(0,1,4)for p in c['area_path']);assert ring([p['coordinates']for p in c['area_path']if p['type']in(0,1)])==ring(t['area']['polygon'])
 assert v['input_context']=='caller_supplied_page_head_corner_references_and_complete_beam_group_state'
 assert all(v[k]is False for k in ('automatic_raster_pipeline_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['native_outputs_used']is False and r['independent_compilations_and_runs']==2 and r['systems']==2 and r['head_corners']==1408 and r['rectangle_boundary_controls']==14 and r['original_neighborhood_initial_area_group_target_methods_called']is True and r['original_page_state_with_supplied_corner_references']is True and r['image_pipeline_complete']is False and r['original_preinspect_beam_abscissa_order_recreated_after_process']is True
 assert r['original_sha256']==sha(F/'controls.json')and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemHeadNeighborhoodProbe' and x['official_cli_step']=='HEADS' and x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only'not in x['runtime_classpath']
  assert all(sha(ROOT/k)==h for k,h in x['observer_source_sha256'].items());assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and n['actual_CLI_executed']is True and n['page_state_controls']==1408 and n['rectangle_boundary_controls']==14 and all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 for f,k in [('native-controls.json.gz','native_controls_gzip_sha256'),('cli-request.json.gz','cli_request_gzip_sha256'),('cli-report.json.gz','cli_report_gzip_sha256')]:assert sha(F/f)==n[k]
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');assert o['native_outputs_used']is False and o['corners']==1408 and sum(len(s['cases'])for s in o['systems'])==1408
 compare(o,v['report']);compare(o,read(F/'cli-report.json.gz'));assert request(o)==read(F/'cli-request.json.gz');boundaries=[c for s in o['systems']for c in s['rectangle_boundaries']];assert len(boundaries)==len(v['rectangle_boundaries'])==14
 for a,b in zip(boundaries,v['rectangle_boundaries']):assert a['neighbors']==b['neighbors']
 return {'status':'passed','original_Laura_page_state_systems':2,'actual_CLI_original_page_state_head_corner_controls':1408,'original_rectangle_edge_duplicate_removed_empty_controls':14,'automatic_neighborhood_area_group_target_CLI_connected':True,'head_corner_and_complete_beam_group_state_supplied':True,'automatic_raster_pipeline_complete':False,'page_cache_SIG_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
