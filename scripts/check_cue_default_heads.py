#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare the original unoverridden default cue font/HEADS path separately."""
import gzip,hashlib,json,sys
from pathlib import Path
from check_cue_headers_rest import check as prior_check, read, sha, near
from check_head_glyph_retrieval import record
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-cue-default-heads'
def check_default(path):
 payload=gzip.decompress(path.read_bytes()) if path.suffix=='.gz' else path.read_bytes()
 full=json.loads(payload);control=full['head_template_selection'];name='Bravura-default-cue'
 assert 'head_catalog_control' not in full and control['source']=='original_unmeasured_default_cue' and control['original_music_font_scale_present'] is False
 assert full['beam_prolog']['head_scale']['music_font'] is None
 g=read(F/'default-heads.json');r=full['head_registration'];assert g['native_outputs_used'] is False
 expected={'family':control['family'],'stored_point_size':control['stored_point_size']}
 for k in ('head_seed_search','head_seed_veto','head_range_search','head_range_catalog','head_candidate_filter','head_glyph_retrieval','head_registration'):
  assert full[k]['template_catalog']==expected and full[k]['template_catalog_required'] is True and full[k]['heads_complete'] is False,k
 assert r['head_registration_complete'] is True and r['staff_dependency_complete'] is True and r['epilog_seed_scale_complete'] is True
 assert full['musicxml_complete'] is False and full['last_completed_stage']=='LEDGERS'
 assert len(r['systems'])==len(g['systems']);sig=r['shared_sig'];by_id={v['id']:v for v in sig['inters']};count=exclusions=small=0;all_original={};all_native={}
 for s,gs in zip(r['systems'],g['systems']):
  assert s['system_id']==gs['system_id'] and len(s['heads'])==len(gs['heads']);ids={}
  for a,e in zip(s['heads'],gs['heads']):
   record(a,e);assert a['abnormal']==e['abnormal'] and a['removed']==e['removed'] is False
   ids[e['id']]=a['id'];all_original[e['id']]=a['id'];all_native[a['id']]=e['id'];record(by_id[a['id']],e);assert by_id[a['id']]['abnormal']==e['abnormal'];count+=1
  expected_edges=[(ids[e['source']],ids[e['target']]) for e in gs['relations'] if e['source'] in ids and e['target'] in ids]
  assert [(e['source'],e['target']) for e in s['exclusions']]==expected_edges;exclusions+=len(expected_edges)
  assert len(s['staffs'])==len(gs['staffs'])
  for a,e in zip(s['staffs'],gs['staffs']):assert a['staff_id']==e['staff_id'] and a['notes']==[ids[v] for v in e['notes']]
  for b in s['small_beams']:
   original=next((v for v in gs['other_inters'] if v['kind']==b['kind'] and v['bounds']==b['bounds']),None)
   if b.get('removed',False):assert original is None
   else:assert original is not None and abs(b['contextual_grade']-original['contextual_grade'])<1e-12
   small+=1
 assert len(r['head_seed_scale'])==len(g['head_seed_scale'])==4
 for a,e in zip(r['head_seed_scale'],g['head_seed_scale']):
  assert a['shape']==e['shape'] and a['side']==e['side']
  if e['dx'] is None:assert a['dx'] is None
  else:assert abs(a['dx']-e['dx'])<1e-14
 # Preserve previous native identities/relations, permitting only original
 # verified small-beam contextualization/removal during the HEADS epilog.
 removed_beams={b['id'] for s in r['systems'] for b in s['small_beams'] if b.get('removed',False)}
 for old in full['shared_sig']['inters']:
  if old['id'] in removed_beams:assert old['id'] not in by_id;continue
  before=dict(old);now=dict(by_id[old['id']]);before.pop('contextual_grade',None);now.pop('contextual_grade',None)
  if 'removed' not in before:now.pop('removed',None)
  assert before==now
 preserved=[e for e in full['shared_sig']['relations'] if e['source'] not in removed_beams and e['target'] not in removed_beams]
 assert sig['relations'][:len(preserved)]==preserved
 assert count==r['retained_heads'] and exclusions==r['overlap_pairs']
 for e in sig['relations']:assert e['source'] in by_id and e['target'] in by_id
 return {'status':'passed','control':name,'retained_physical_heads':count,'ordered_exclusions':exclusions,'staff_note_order':True,'small_beams_checked':small,'seed_epilog_dx_checked':True,'prior_shared_ids_relations_preserved':True,'intermediate_original_performance_or_removal_counts_claimed':False,'actual_synthetic_default_cue_page_parity':True,'heads_complete':False,'report_sha256':hashlib.sha256(payload).hexdigest()}

def check(path):
 prior=prior_check(path);p=read(F/'default-heads-provenance.json');q=read(F/'default-heads-certified-provenance.json');r=read(F/'source-backed-reproduction.json')
 assert r['status']=='passed' and r['native_outputs_used'] is False and r['original_fields_byte_reproduced'] is True
 assert sha(F/'default-heads.json')==r['original_sha256'] and r['input_sha256']==sha(ROOT/'fixtures/official-head-actual-pages/images/Bravura-cue.png')
 for x in [p,q]:
  assert x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and x['release']=='5.11.0'
  assert all(sha(ROOT/n)==h for n,h in x['observer_source_sha256'].items()) and 'compile-only' not in x['runtime_classpath']
  assert len(x['fixtures'])==1 and x['fixtures'][0]['input_sha256']==r['input_sha256'] and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0
 for k in ['observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256']:assert p[k]==q[k]
 for name in ['bank-provenance.json','bank-certified-provenance.json']:
  x=read(F/name);assert x['native_outputs_used'] is False and x['jar_sha256']==p['jar_sha256'] and x['templates']==8 and x['point_sizes']==[28,44]
  assert all(sha(ROOT/n)==h for n,h in x['observer_source_sha256'].items())
  assert all(sha(F/'original-bank'/n)==h for n,h in x['original_files_sha256'].items())
  assert sha(ROOT/'crates/audiveris-template-bank/assets/cue-default/manifest.json')==x['packaged_files_sha256']['manifest.json'] and sha(ROOT/'crates/audiveris-template-bank/assets/cue-default/templates.json')==x['packaged_files_sha256']['templates.json'] and sha(ROOT/'crates/audiveris-template-bank/src/cue_identity.rs')==x['packaged_files_sha256']['cue_identity.rs']
 n=read(F/'native-reproduction.json');assert n['status']=='passed' and sha(F/'native-default.json.gz')==n['native_report_gzip_sha256'] and all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
 default=check_default(F/'native-default.json.gz');assert (default['retained_physical_heads'],default['ordered_exclusions'])==(156,189)
 assert [s['head_point_size'] for row in read(F/'default-heads.json')['systems'] for s in row['staffs']]==[28,44,44,44]
 native=read(F/'native-default.json.gz');assert [s['point_size'] for row in native['head_seed_search']['systems'] for s in row['staffs']]==[28,44,44,44]
 return {'status':'passed','independent_synthetic_default_cue_HEADS':default,'original_default_staff_points':[28,44,44,44],'default_cue_cli_supported':True,'declared_Bravura51_cue_HEADS_distinct':{'heads':168,'exclusions':198},'published_scan_parity':False,'full24_HEADS_page_propagation':False,'automatic_STEMS_complete':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check_default(Path(sys.argv[1])) if len(sys.argv)>2 else check(Path(sys.argv[1])),indent=2))
