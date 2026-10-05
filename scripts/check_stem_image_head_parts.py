#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
from pathlib import Path
import gzip,hashlib,json,sys
from check_stem_image_beam_generation import check as prior_check, compare as prior_compare
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'fixtures/official-stem-image-head-parts'
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
def key(c):
 h=c.get('head',c);return tuple(h['head_bounds'])+(h['shape'],h['side'],h['vertical'])
def same(a,b):
 if isinstance(a,dict):
  for k,v in a.items():same(v,b[k])
 elif isinstance(a,list):
  assert len(a)==len(b)
  for u,v in zip(a,b):same(u,v)
 elif isinstance(a,float):assert abs(a-b)<=1e-12,(a,b)
 else:assert a==b,(a,b)
def compare(o,n):
 assert o['head_corners']==n['corner_count']==1408
 assert len(o['systems'])==len(n['systems'])==2
 for k in ('initial_seed_occurrences','initial_linker_occurrences','retained_linker_occurrences'):assert o[k]==n[k]
 assert (n['initial_seed_occurrences'],n['initial_linker_occurrences'],n['retained_linker_occurrences'],n['pre_chunk_item_count'])==(248,1740,1740,1884)
 count=items=0
 for a,b in zip(o['systems'],n['systems']):
  assert a['system']==b['system'];e={key(c):c for c in a['cases']};assert len(e)==len(b['cases'])
  for c in b['cases']:
   expected=e[key(c)]
   for k in ('theoretical_line','builder_y_range','start_stump','initial_seeds','selected_linkers','retained_seeds','retained_linker_indices','last_head_y','pre_chunk_items'):same(expected[k],c[k])
   count+=1;items+=len(c['pre_chunk_items'])
 assert (count,items)==(1408,1884)
 assert n['head_builder_pre_chunk_inputs_complete'] is True
 assert all(n[k] is False for k in ('automatic_raster_pipeline_complete','final_stem_glyph_selection_complete','page_cache_sig_complete','epilog_complete','heads_complete','stems_complete'))
 prior_compare(read(ROOT/'fixtures/official-stem-image-beam-generation/controls.json'),n['beam_generation_plan'])
 return {'status':'passed','actual_image_head_builders_pre_chunk':1408,'image_initial_seed_occurrences':248,'selected_other_head_and_beam_linkers':1740,'retained_linkers':1740,'ordered_physical_pre_chunk_items':1884,'builder_Line2D_bounds_and_start_stump_clip_exact':True,'floating_absolute_tolerance':1e-12,'same_strong_physical_registry_and_native_SIG_origin_metadata_retained':True,'original_XML_loaded_inspection_order_replay':True,'automatic_chunk_registration_final_glyph_cache_SIG_epilog_complete':False,'fresh_original_transient_lifecycle_newly_verified':False,'nonzero_skew_scans_special_staff_full24_pages_verified':False,'heads_complete':False,'stems_complete':False}
def check(path):
 prior=prior_check(path);o=read(F/'controls.json');proof=read(F/'source-backed-reproduction.json');assert proof['status']=='passed' and proof['native_outputs_used'] is False and proof['independent_original_runs']==2 and proof['original_sha256']==sha(F/'controls.json')
 ps=[read(F/f) for f in ('controls-provenance.json','controls-certified-provenance.json')]
 for p in ps:
  assert p['probe']=='StemImageHeadPartsProbe' and p['official_cli_step']=='HEADS' and p['release']=='5.11.0'
  assert p['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert 'compile-only' not in p['runtime_classpath'] and all(sha(ROOT/k)==v for k,v in p['observer_source_sha256'].items())
  f=p['fixtures'][0];assert len(p['fixtures'])==1 and f['ok'] and f['exit_code']==0 and f['result_sha256']==sha(F/'controls.json') and f['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert ps[0][k]==ps[1][k]
 r=read(F/'native-reproduction.json');n=read(F/'native-stems-report.json.gz');assert r['actual_image_CLI_executed'] is True and all(sha(ROOT/k)==v for k,v in r['native_source_sha256'].items());assert sha(F/'native-stems-report.json.gz')==r['native_stems_report_gzip_sha256'];assert hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest()==r['native_stems_report_raw_sha256'];assert n['source_image_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png');assert n['native_HEADS_report_sha256']==r['entire_old_HEADS_report_sha256']=='87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9';assert sha(path)==r['entire_old_HEADS_report_sha256'];result=compare(o,n);result['old_HEADS_and_seven_prior_CLI_modes_byte_identical']=True;result['historical_prior_checkpoint_contracts']=prior;return result
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
