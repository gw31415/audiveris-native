#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check declared page seed catalog receipts and preserved Laura CLI outputs."""
import hashlib,json,sys
from pathlib import Path
from check_head_seed_coverage import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-seed-page-catalog';catalogs=json.loads((f/'cases.json').read_text());provenance=json.loads((f/'provenance.json').read_text());cue=json.loads((f/'certified-cue-provenance.json').read_text());reproduced=json.loads((f/'source-backed-cue-reproduction.json').read_text())
 assert catalogs['native_outputs_used'] is False and provenance['native_outputs_used'] is False
 assert len(catalogs['cases'])==13 and {(c['family'],c['stored_point_size'],c['cue_first_staff']) for c in catalogs['cases']}=={(a,b,False) for a in ('Bravura','Leland','FinaleJazz') for b in (32,40,51,64)}|{('Bravura',51,True)}
 observer=ROOT/'scripts/java/HeadSeedPageCatalogProbe.java';assert hashlib.sha256(observer.read_bytes()).hexdigest()==provenance['observer_sha256']==reproduced['source_backed_observer_sha256']
 assert reproduced['status']=='passed' and reproduced['all_original_controlled_cue_result_fields_reproduced'] is True
 for name,digest in cue['observer_source_sha256'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,name
 assert cue['jar_sha256']==provenance['base_provenance']['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
 pairs=proposals=0
 for c in catalogs['cases']:
  p=f/(c['name']+'.json');assert hashlib.sha256(p.read_bytes()).hexdigest()==c['fixture_sha256'];g=json.loads(p.read_text());assert g['native_outputs_used'] is False
  assert g['template_catalog']=={'family':c['family'],'stored_point_size':c['stored_point_size']} and g['cue_first_staff']==c['cue_first_staff']
  assert [t['point_size'] for s in g['systems'] for t in s['staffs']]==c['point_sizes']
  assert sum(len(z['matches']) for s in g['systems'] for t in s['staffs'] for k in ('scanners','ledger_scanners') for row in t[k] for z in row['seeds'])==c['veto_proposals']
  pairs+=c['seed_pitch_pairs'];proposals+=c['veto_proposals']
 assert pairs==14764
 before=json.loads((f/'native-before-seeds.json').read_text());full=json.loads(path.read_text())
 for old,new in [('grid','grid_ir'),('seeds','stem_seeds'),('ledgers','ledger_post_analysis'),('shared_sig','shared_sig')]:assert before[old]==full[new]
 return {'status':'passed','original_configured_seed_page_cases':13,'seed_pitch_pairs':pairs,'veto_proposals_across_controls':proposals,'cue_staff_point_sizes':[32,51,51,51,51,51],'source_backed_cue_reproduced':True,'legacy_CLI_catalog_unchanged':True,'whole_HEADS_catalog_propagation_complete':False,'actual_other_font_or_special_staff_page_parity':False,'heads_complete':False,'preserved_seed_quarter_staff_font_and_laura':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
