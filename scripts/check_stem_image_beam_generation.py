#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
from pathlib import Path
import gzip,hashlib,json,sys
from check_stem_image_beam_targets import check as prior_check, compare as prior_compare
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-image-beam-generation'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def close(a,b):
 assert len(a)==len(b)
 assert all(abs(x-y)<=1e-12 for x,y in zip(a,b)),(a,b)
def links(a,b):
 assert len(a)==len(b)
 for x,y in zip(a,b):
  for k in ('side','stump','anchor','directions'):assert x[k]==y[k],(k,x,y)
  close(x['reference'],y['reference'])
def compare(o,n):
 assert len(o['systems'])==len(n['systems'])==2
 count=base=final=0
 for a,b in zip(o['systems'],n['systems']):
  assert a['system']==b['system']
  for k in ('main_stem_thickness','max_stem_thickness','min_beam_stems_dx','max_beam_seed_dx','max_beam_linker_dx','min_beam_stump_dy'):assert a[k]==b[k]
  assert len(a['beam_catalog'])==len(b['beam_catalog'])
  for x,y in zip(a['beam_catalog'],b['beam_catalog']):
   assert x['beam']['median']==y['beam']['median'] and x['beam']['height']==y['beam']['height']
   for k in ('admitted_seeds','purged_seeds','stumps'):assert x[k]==y[k],(k,x['beam'])
   op=sorted({tuple(p['coordinates']) for p in x['seed_area_path'] if p['type'] in (0,1)});np=sorted(map(tuple,y['seed_area_polygon']));assert len(op)==len(np)==4
   for u,v in zip(op,np):close(u,v)
   links(x['base_blinkers'],y['base_blinkers']);links(x['final_blinkers'],y['final_blinkers'])
   count+=len(y['stumps']);base+=len(y['base_blinkers']);final+=len(y['final_blinkers'])
  ids={c['beam']['id']:c for c in b['beam_catalog']}
  for c in b['cases']:
   for selected in c['selected_blinkers']:assert selected['linker']==ids[selected['beam_id']]['final_blinkers'][selected['linker_id']-1]
 assert (count,base,final)==(40,43,90)
 assert (n['beam_stump_occurrences'],n['base_blinker_count'],n['final_blinker_count'],n['selected_blinker_occurrences'])==(40,43,90,339)
 assert len(o['purge_controls'])==9 and len(o['nearest_controls'])==5
 assert n['beam_catalog_generation_complete'] is True
 assert all(n[k] is False for k in ('beam_origin_builder_inputs_complete','automatic_raster_pipeline_complete','final_stem_glyph_selection_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
 prior_compare(read(ROOT/'fixtures/official-stem-image-beam-targets/controls.json'),n['beam_target_plan'])
 return {'status':'passed','image_beams':18,'ordered_full_physical_beam_stumps_exact':40,'ordered_base_BLinkers_exact':43,'ordered_final_BLinkers_exact':90,'head_origin_selected_BLinker_occurrences':339,'native_strong_registry_retained':True,'stump_groups_physical_ID_aliases_preserved':True,'original_purge_controls':9,'original_nearest_ULP_controls':5,'floating_reference_absolute_tolerance':1e-12,'beam_origin_sibling_anchor_generation_connected':True,'full_beam_origin_builder_inputs_complete':False,'productive_beam_section_fallback_observed_on_Laura':False,'original_XML_loaded_inspection':True,'fresh_original_transient_lifecycle_newly_verified':False,'nonzero_skew_real_scans_special_staff_full24_pages_verified':False,'automatic_whole_raster_pipeline_complete':False,'page_cache_SIG_and_epilog_complete':False,'heads_complete':False,'stems_complete':False}
def check(path):
 prior=prior_check(path);o=read(F/'controls.json');proof=read(F/'source-backed-reproduction.json');assert proof['status']=='passed' and proof['native_outputs_used'] is False and proof['independent_original_runs']==2 and proof['original_sha256']==sha(F/'controls.json')
 ps=[read(F/f) for f in ('controls-provenance.json','controls-certified-provenance.json')]
 for p in ps:
  assert p['probe']=='StemImageBeamGenerationProbe' and p['official_cli_step']=='HEADS' and p['release']=='5.11.0'
  assert p['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert 'compile-only' not in p['runtime_classpath'] and all(sha(ROOT/k)==v for k,v in p['observer_source_sha256'].items())
  f=p['fixtures'][0];assert len(p['fixtures'])==1 and f['ok'] and f['exit_code']==0 and f['result_sha256']==sha(F/'controls.json') and f['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert ps[0][k]==ps[1][k]
 r=read(F/'native-reproduction.json');n=read(F/'native-stems-report.json.gz');assert r['actual_image_CLI_executed'] is True and all(sha(ROOT/k)==v for k,v in r['native_source_sha256'].items());assert sha(F/'native-stems-report.json.gz')==r['native_stems_report_gzip_sha256'];assert hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest()==r['native_stems_report_raw_sha256'];assert n['source_image_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png');assert n['native_HEADS_report_sha256']==r['entire_old_HEADS_report_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9';assert sha(path)==r['entire_old_HEADS_report_sha256'];result=compare(o,n);result['old_HEADS_and_six_prior_CLI_modes_byte_identical']=True;result['historical_prior_checkpoint_contracts']=prior;return result
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
