#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import gzip,hashlib,json,sys
from pathlib import Path
from check_stem_image_linkers import check as prior_check,compare as head_compare
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-image-beam-targets'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def key(c):return (tuple(c['head_bounds']),c['shape'],c['side'],c['vertical'])
def beam(v):return None if v is None else {k:v[k] for k in ('median','height')}
def compare(o,n):
 assert o['native_outputs_used'] is False and o['original_phase']=='inspectStems_before_linkStems_and_finalizeStems'
 assert o['corners']==n['corner_count']==1408 and o['selected_sibling_member_occurrences']==n['selected_sibling_member_occurrences']==339
 assert len(o['systems'])==len(n['systems'])==2;count=total=0
 for a,b in zip(o['systems'],n['systems']):
  assert a['system']==b['system'] and a['max_beam_side_dx']==b['max_beam_side_dx']==3
  expected={key(c):c for c in a['cases']};assert len(expected)==len(b['cases'])
  for c in b['cases']:
   e=expected[key(c['head'])];assert all(e[f]==c[f] for f in ('intersection','void_side_veto'));assert beam(e['target_beam'])==beam(c['target_beam']);assert [beam(v) for v in e['selected_sibling_members']]==[beam(v) for v in c['selected_sibling_members']];assert c['selected_sibling_member_ids']==[v['id'] for v in c['selected_sibling_members']];assert len(set(c['selected_sibling_member_ids']))==len(c['selected_sibling_member_ids']);total+=len(c['selected_sibling_members']);count+=1
 assert (count,total)==(1408,339) and len(o['controls'])==13
 assert sum(len(s['beam_catalog']) for s in o['systems'])==18
 assert len({(tuple(b['beam']['median']),b['beam']['height']) for s in o['systems'] for b in s['beam_catalog']})==18
 assert sum(len(b['stumps']) for s in o['systems'] for b in s['beam_catalog'])==40
 assert sum(len(b['all_blinkers']) for s in o['systems'] for b in s['beam_catalog'])==o['observed_all_blinkers']==90
 assert n['head_origin_beam_sibling_member_selection_complete'] is True
 assert all(n[k] is False for k in ('beam_sibling_linker_selection_complete','final_stem_glyph_selection_complete','automatic_raster_pipeline_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
 head_compare(read(ROOT/'fixtures/official-stem-image-linkers/controls.json'),n['head_linker_plan'])
 return {'status':'passed','actual_image_corners':count,'ordered_sibling_beam_member_occurrences_exact':total,'original_sibling_side_ULP_controls':13,'original_observed_beams':18,'original_observed_stumps':40,'original_observed_BLinkers':90,'BLinker_catalog_observed_but_not_yet_automatically_generated':True,'same_strong_physical_registry_retained':True,'void_side_veto_and_target_line_intersection_exact':True,'page_cache_SIG_and_epilog_complete':False,'original_XML_loaded_context':True,'fresh_original_transient_lifetime_newly_verified':False,'nonzero_skew_scans_special_staff_full24_pages_verified':False,'heads_complete':False,'stems_complete':False}
def check(path):
 prior=prior_check(path);o=read(F/'controls.json');proof=read(F/'source-backed-reproduction.json');assert proof['status']=='passed' and proof['native_outputs_used'] is False and proof['independent_original_runs']==2 and proof['original_sha256']==sha(F/'controls.json')
 ps=[read(F/f) for f in ('controls-provenance.json','controls-certified-provenance.json')]
 for p in ps:
  assert p['probe']=='StemImageBeamTargetsProbe' and p['official_cli_step']=='HEADS' and p['release']=='5.11.0'
  assert p['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert 'compile-only' not in p['runtime_classpath'] and all(sha(ROOT/k)==v for k,v in p['observer_source_sha256'].items())
  assert len(p['fixtures'])==1;f=p['fixtures'][0];assert f['ok'] and f['exit_code']==0 and f['result_sha256']==sha(F/'controls.json') and f['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert ps[0][k]==ps[1][k]
 r=read(F/'native-reproduction.json');n=read(F/'native-stems-report.json.gz');assert r['status']=='passed' and r['actual_image_CLI_executed'] is True and all(sha(ROOT/k)==v for k,v in r['native_source_sha256'].items());assert sha(F/'native-stems-report.json.gz')==r['native_stems_report_gzip_sha256'];assert hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest()==r['native_stems_report_raw_sha256'];assert n['source_image_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png');assert n['native_HEADS_report_sha256']==r['entire_old_HEADS_report_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9';result=compare(o,n);result['old_HEADS_and_five_prior_CLI_modes_byte_identical']=True;result['historical_prior_checkpoint_contracts']=prior;return result
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
