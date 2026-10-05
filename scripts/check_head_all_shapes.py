#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify all-head original assets/control pins and the preserved CLI contracts."""
import hashlib,json,sys
from pathlib import Path
from check_head_stem_kernel import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-all-shapes';sets=read(f/'shape-sets.json');g=read(f/'controls.json');seed=read(f/'seed-controls.json');bounds=read(f/'boundaries.json');bank=read(f/'bank-provenance.json');first=read(f/'bank-provenance-initial.json');repro=read(f/'bank-reproduction.json')
 assert len(sets['all'])==24 and len(sets['missing'])==15 and len(sets['stem'])==12 and len(sets['hollow'])==18
 original_sets=read(ROOT/'fixtures/official-head-staff-catalog/controls.json')['shape_sets']
 assert sets['all']==original_sets['Heads'] and sets['stem']==original_sets['StemHeads'] and sets['hollow']==original_sets['HollowHeads']
 assert all(v['native_outputs_used'] is False for v in (g,seed,bounds,bank,first,repro))
 for name,pin in read(f/'control-sha256.json').items():assert sha(f/name)==pin
 assert len(g['images'])==720 and len(seed['images'])==240 and len(bounds['boundaries'])==150
 expected={(family,size,shape,variant) for family in ('Bravura','Leland','FinaleJazz') for size in (32,40,51,64) for shape in sets['missing'] for variant in ('clean','erased','edge','empty')}
 assert {(c['family'],c['point_size'],c['input']['shape'],c['variant']) for c in g['images']}==expected
 assert sum(len(c['modes']) for c in g['images'])==1440 and sum(len(c['windows']) for c in seed['images'])==1440
 assert seed['source_shapes']==[s for s in sets['missing'] if s in sets['stem']]
 assert all(c['SIG_registered_in_staff_order'] and c['SIG_retired'] and c['registered_abnormal']==(c['input']['shape'] in sets['stem']) for c in g['images'])
 assert bank['jar_sha256']==first['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
 assert bank['observer_sha256']==sha(ROOT/'scripts/java/HeadAllShapesBankOracle.java')
 assert repro['status']=='passed' and repro['all_three_family_assets_and_manifest_hashes_reproduced'] is True and repro['independent_seed_images_reproduced']==240
 for family,receipt in bank['families'].items():
  folder=ROOT/'crates/audiveris-template-bank/assets/all-heads'/family;m=read(folder/'manifest.json');payload=read(folder/'templates.json')
  assert sha(folder/'manifest.json')==receipt['manifest_sha256']==first['families'][family]['manifest_sha256']
  assert sha(folder/'templates.json')==receipt['payload_sha256']==m['asset_sha256']==first['families'][family]['payload_sha256']
  assert len(m['keys'])==96 and len(payload['templates'])==96 and m['font_sha256']==receipt['font_sha256']
  assert {(k['shape'],k['point_size']) for k in m['keys']}=={(s,p) for s in sets['all'] for p in (32,40,51,64)}
  historical=read(ROOT/'crates/audiveris-template-bank/assets/drum'/family/'templates.json')
  bykey={(t['key']['shape'],t['key']['point_size']):t for t in payload['templates']}
  assert all(bykey[(t['key']['shape'],t['key']['point_size'])]==t for t in historical['templates'])
  assert historical['calibration']==payload['calibration']
 for filename,observer in [('oracle-provenance.json','HeadAllShapesCoverageProbe'),('seed-oracle-provenance.json','HeadAllShapesSeedProbe'),('boundary-oracle-provenance.json','HeadAllShapesBoundaryProbe')]:
  p=read(f/filename);assert p['jar_sha256']==bank['jar_sha256'] and p['release']=='5.11.0' and p['probe']==observer
  assert all(sha(ROOT/n)==pin for n,pin in p['observer_source_sha256'].items())
  assert all(c['exit_code']==0 and c['ok'] for c in p['fixtures'])
 return {'status':'passed','exact_head_templates':288,'new_head_shapes':15,'independent_original_images':720,'range_controls':1440,'new_hollow_seed_windows':1440,'physical_glyph_controls':720,'eligibility_ULP_controls':150,'original_SIG_lifecycle_controls':720,'all_24_shape_registration_abnormal_states':True,'actual_other_font_or_special_staff_page_parity':False,'complete_STEMS_or_stem_attachment':False,'heads_complete':False,'preserved_complete_controlled_HEADS_and_STEMS_kernel':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
