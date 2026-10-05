#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import json,sys
from pathlib import Path
from check_stem_parts_prefix import check as prior_check
from check_stem_cache_head import read,sha
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-head-targets'
def ring(points):
 if not points:return []
 return min(p[i:]+p[:i] for p in (points,points[::-1])for i in range(len(p)))
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['independent_compilations_and_runs']==2 and r['native_outputs_used']is False and r['head_builders']==600 and r['target_controls']==3000 and r['ULP_boundary_controls']==1680 and r['original_lookupBeamGroups_computeTargetPoint_called']is True and r['neighborhood_and_group_catalog_supplied']is True and r['complete_image_STEMS']is False
 assert r['original_sha256']==sha(F/'controls.json') and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemHeadTargetsProbe' and x['official_cli_step']=='HEADS' and x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only'not in x['runtime_classpath']
  assert all(sha(ROOT/k)==h for k,h in x['observer_source_sha256'].items());assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'] and all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');assert o['native_outputs_used']is False and o['head_builders']==600 and len(o['cases'])==len(v['cases'])==3000 and len(o['boundaries'])==len(v['boundaries'])==1680
 for a,b in zip(o['cases'],v['cases']):
  for k in ('selected_beams','selected_groups','target_beam','initial_line','target'):assert a[k]==b[k]
  assert a['area_line']==b['area']['theoretical_line'];points=[p['coordinates']for p in a['area_path']if p['type']in(0,1)];assert all(p['type']in(0,1,4)for p in a['area_path']);assert ring(points)==ring(b['area']['polygon'])
 for a,b in zip(o['boundaries'],v['boundaries']):assert a['selected_beams']==b['selected_beams'] and a['selected_groups']==b['selected_groups']
 assert all(v[k]is False for k in ('image_neighborhood_selection_complete','page_cache_SIG_complete','epilog_complete','heads_complete','stems_complete'))
 return {'status':'passed','actual_first_system_head_corners':600,'original_group_target_area_controls':3000,'original_distance_ULP_controls':1680,'exact_group_order_target_border_and_area_edges':True,'neighborhood_and_group_membership_supplied':True,'image_neighborhood_selection_complete':False,'page_cache_SIG_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
