#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import gzip,hashlib,json,sys
from pathlib import Path
from check_stem_image_sources import check as prior_check, compare as source_compare
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-image-linkers'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def key(c):return (tuple(c['head_bounds']),c['shape'],c['side'],c['vertical'])
def norm(c):return {k:c[k] for k in ('head_bounds','shape','side','vertical','reference','stump')}
def compare(o,n):
 assert o['native_outputs_used'] is False and o['original_phase']=='inspectStems_before_linkStems_and_finalizeStems'
 assert o['corners']==n['corner_count']==1408
 assert o['selected_other_head_linkers']==n['selected_other_head_linker_occurrences']==1401
 assert len(o['systems'])==len(n['systems'])==2
 count=total=0
 for a,b in zip(o['systems'],n['systems']):
  assert a['system']==b['system'] and b['min_head_head_dy']==3
  expected={key(c['head']):c for c in a['cases']};assert len(expected)==len(b['cases'])==len(a['cases'])
  catalog={h['id']:h for h in b['head_catalog']}
  for c in b['cases']:
   e=expected[key(c['head'])];assert e['selected']==[norm(v) for v in c['selected_other_head_linkers']]
   source=catalog[c['head']['head_id']]
   for v in c['selected_other_head_linkers']:
    id=v['head_id'];assert id!=source['id'] and id not in source['competing_ids']
    assert v['head_origin']['phase']=='HEAD' and v['head_origin']['system']==b['system']
    assert any(v==corner for corner in catalog[id]['corners'])
   total+=len(e['selected']);count+=1
 assert (count,total)==(1408,1401) and len(o['controls'])==14
 assert n['other_head_linker_selection_complete'] is True
 assert all(n[k] is False for k in ('beam_sibling_linker_selection_complete','final_stem_glyph_selection_complete','automatic_raster_pipeline_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
 source_compare(read(ROOT/'fixtures/official-stem-image-sources/controls.json'),n['source_plan'])
 return {'status':'passed','actual_image_head_corners':count,'ordered_other_head_linker_occurrences_exact':total,'original_other_head_condition_controls':14,'current_HEADS_exclusions_and_target_local_origins_used':True,'same_physical_stump_registry_retained':True,'beam_origin_and_sibling_targets_items_final_glyph_cache_SIG_epilog_complete':False,'original_XML_loaded_context':True,'original_transient_context_newly_verified':False,'nonzero_skew_scans_special_staff_full24_pages_verified':False,'heads_complete':False,'stems_complete':False}
def check(path):
 prior=prior_check(path);o=read(F/'controls.json');proof=read(F/'source-backed-reproduction.json')
 assert proof['status']=='passed' and proof['native_outputs_used'] is False and proof['independent_original_runs']==2 and proof['original_sha256']==sha(F/'controls.json')
 ps=[read(F/f) for f in ('controls-provenance.json','controls-certified-provenance.json')]
 for p in ps:
  assert p['probe']=='StemImageLinkersProbe' and p['official_cli_step']=='HEADS' and p['release']=='5.11.0'
  assert p['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert 'compile-only' not in p['runtime_classpath'] and all(sha(ROOT/k)==v for k,v in p['observer_source_sha256'].items())
  assert len(p['fixtures'])==1
  f=p['fixtures'][0];assert f['ok'] and f['exit_code']==0 and f['result_sha256']==sha(F/'controls.json') and f['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert ps[0][k]==ps[1][k]
 r=read(F/'native-reproduction.json');n=read(F/'native-stems-report.json.gz')
 assert r['status']=='passed' and r['actual_image_CLI_executed'] is True and all(sha(ROOT/k)==v for k,v in r['native_source_sha256'].items())
 assert sha(F/'native-stems-report.json.gz')==r['native_stems_report_gzip_sha256']
 assert hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest()==r['native_stems_report_raw_sha256']
 assert n['source_image_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 assert n['native_HEADS_report_sha256']==r['entire_old_HEADS_report_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
 result=compare(o,n);result['old_HEADS_and_four_prior_CLI_modes_byte_identical']=True;result['historical_prior_checkpoint_contracts']=prior;return result
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
