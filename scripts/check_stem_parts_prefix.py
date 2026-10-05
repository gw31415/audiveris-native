#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Physical prefix equality/cache API; direct12 and prior1152 composition distinguished."""
from pathlib import Path
import json,sys
from check_stem_item_events import check as prior_check,near
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-parts-prefix'
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json');assert r['status']=='passed' and r['native_outputs_used'] is False and r['independent_compilations_and_runs']==2 and r['prefix_controls']==4680 and r['last_gap_controls']==4680 and r['actual_builders']==612 and r['direct_selected_prefix_cache_SIG_controls']==12 and r['original_getGlyphsUntil_getLastGapBefore_createStem_SIG_called'] is True
 assert r['original_sha256']==sha(F/'controls.json') and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
 for x in (p,q):
  assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['probe']=='StemPartsPrefixProbe' and x['official_cli_step']=='HEADS'
  assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and 'compile-only' not in x['runtime_classpath']
  assert all(sha(ROOT/k)==h for k,h in x['observer_source_sha256'].items())
  assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['result_sha256']==r['original_sha256'] and x['fixtures'][0]['input_sha256']==r['input_sha256']
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'];assert all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 o=read(F/'controls.json');v=read(F/'native-controls.json.gz');groups=o['groups']+o['orientation_groups'];assert len(o['groups'])==612 and len(o['orientation_groups'])==3 and len(groups)==len(v['groups'])==615 and o['native_outputs_used'] is False
 total=0;modes=set()
 for a,b in zip(groups,v['groups']):
  assert len(a['cases'])==len(b['cases'])
  for c,d in zip(a['cases'],b['cases']):
   assert c['prefix_parts']==d['prefix_parts'] and c['last_gap_before']==d['last_gap_before'];total+=len(c['prefix_parts']);modes.add(c['mode'])
   assert all(i['glyph_class'] in (None,'org.audiveris.omr.glyph.Glyph')for i in c['items'])
 assert total==v['prefix_controls']==4680 and modes=={'actual','empty','distinct-equal-glyph-zero-contrib','same-pixels-distinct-orientation'}
 old=read(ROOT/'fixtures/official-stem-cache-head/controls.json');assert len(o['selected_cache_controls'])==len(v['selected_cache_controls'])==len(old['groups'])==12
 for a,b,c in zip(o['selected_cache_controls'],v['selected_cache_controls'],old['groups']):
  assert a['selected']==b['selected']==c['parts'];near(b['created'],a['created']);near(a['created'],c['created'])
  assert a['cached_same_object_with_white_image']==b['cached_same_object_with_white_image'] is True and a['positive_SIG_id_stable']==b['positive_SIG_id_stable'] is True
 head=v['prior_head_controls_with_native_prefix_cache_composition'];assert len(head['groups'])==len(old['groups'])==12
 for a,b in zip(old['groups'],head['groups']):
  near(b['created'],a['created']);assert len(a['cases'])==len(b['cases'])==96
  for c,d in zip(a['cases'],b['cases']):
   near(d['relation'],c['relation'])
   for k in ('before','after','relation_delta','cached_same_object_with_white_image','cached_positive_SIG_id_stable','after_edge_removal'):assert c[k]==d[k]
 assert head['registry_and_cache_identity_preserved'] is True and all(v[k]is False for k in ('automatic_image_endpoint_selection_complete','page_cache_SIG_complete','epilog_complete','heads_complete','stems_complete'))
 return {'status':'passed','actual_original_builders':612,'original_physical_parts_prefix_and_last_gap_controls_each':4680,'direct_original_selected_prefix_createStem_SIG_white_cache_controls':12,'native_prefix_cache_composition_replays_prior_direct_original_head_SIG_controls':1152,'fresh_direct_original_whole_prefix_to_head_SIG_observation':False,'exact_physical_equality_first_order_orientation_alias_empty_and_gap_index_verified':True,'caller_endpoints_selected_painted_cache_inputs':True,'automatic_image_endpoint_selection_complete':False,'page_cache_SIG_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
