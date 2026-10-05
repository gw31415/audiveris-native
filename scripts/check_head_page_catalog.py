#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare a HEADS-only native catalog control with the complete original oracle."""
import gzip,hashlib,json,sys
from pathlib import Path
from check_head_glyph_retrieval import record
ROOT=Path(__file__).resolve().parents[1]
def check_control(path, oracle_path=None):
 payload=gzip.decompress(path.read_bytes()) if path.suffix=='.gz' else path.read_bytes()
 full=json.loads(payload);control=full['head_catalog_control'];name=f"{control['family']}-{control['stored_point_size']}"+('-cue' if control['cue_first_staff'] else '')
 g=json.loads((oracle_path if oracle_path is not None else ROOT/'fixtures/official-head-page-catalog'/(name+'.json')).read_text());r=full['head_registration'];assert g['native_outputs_used'] is False
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
 return {'status':'passed','control':name,'retained_physical_heads':count,'ordered_exclusions':exclusions,'staff_note_order':True,'small_beams_checked':small,'seed_epilog_dx_checked':True,'prior_shared_ids_relations_preserved':True,'intermediate_original_performance_or_removal_counts_claimed':False,'actual_other_font_or_cue_page_parity':False,'heads_complete':False,'report_sha256':hashlib.sha256(payload).hexdigest()}

def check(path):
 from check_head_seed_page_catalog import check as prior_check
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-page-catalog'
 cases=json.loads((f/'cases.json').read_text());p=json.loads((f/'provenance.json').read_text());cert=json.loads((f/'certified-cue-provenance.json').read_text());repro=json.loads((f/'source-backed-cue-reproduction.json').read_text());receipts=json.loads((f/'native-comparisons.json').read_text())
 assert cases['native_outputs_used'] is False and p['native_outputs_used'] is False
 assert receipts['status']=='passed' and len(cases['cases'])==len(receipts['comparisons'])==13
 assert hashlib.sha256((ROOT/'scripts/java/HeadsCatalogPageProbe.java').read_bytes()).hexdigest()==p['observer_sha256']==repro['source_backed_observer_sha256']
 assert repro['status']=='passed' and repro['all_original_controlled_cue_result_fields_reproduced'] is True
 assert cert['jar_sha256']==repro['jar_sha256']==p['base_provenance']['jar_sha256']
 for name,digest in cert['observer_source_sha256'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,name
 controls=[]
 for c,r in zip(cases['cases'],receipts['comparisons']):
  assert hashlib.sha256((f/(c['name']+'.json')).read_bytes()).hexdigest()==c['fixture_sha256']
  current=check_control(f/'native-controls'/(c['name']+'.json.gz'));assert current==r
  assert current['control']==c['name'] and current['retained_physical_heads']==c['heads'] and current['ordered_exclusions']==c['exclusions'];controls.append(current)
 return {'status':'passed','whole_controlled_HEADS_catalog_propagation_complete':True,'source_backed_original_cue_reproduced':True,'original_controlled_page_cases':13,'controls':controls,'actual_other_font_or_special_staff_page_parity':False,'heads_complete':False,'preserved_seed_quarter_staff_font_and_laura':baseline}
if __name__=='__main__':
 path=Path(sys.argv[1]);data=json.loads(path.read_text());print(json.dumps(check_control(path) if 'head_catalog_control' in data else check(path),indent=2))
