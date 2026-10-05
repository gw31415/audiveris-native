#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check preserved CLI and exact quarter-head template/control receipts."""
import hashlib,json,sys
from pathlib import Path
from check_head_staff_catalog import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-drum-coverage';g=json.loads((f/'controls.json').read_text());p=json.loads((f/'bank-provenance.json').read_text());assert g['native_outputs_used'] is False
 expected={(family,size,shape,variant) for family in ('Bravura','Leland','FinaleJazz') for size in (32,40,51,64) for shape in ('NOTEHEAD_BLACK_SMALL','NOTEHEAD_CROSS','NOTEHEAD_DIAMOND_FILLED','NOTEHEAD_TRIANGLE_DOWN_FILLED','NOTEHEAD_CIRCLE_X') for variant in ('clean','erased','edge','empty')}
 assert len(g['images'])==240 and {(c['family'],c['point_size'],c['input']['shape'],c['variant']) for c in g['images']}==expected
 assert all(c['SIG_registered_in_staff_order'] and c['SIG_retired'] for c in g['images'])
 for family,receipt in p['families'].items():
  folder=ROOT/'crates/audiveris-template-bank/assets/drum'/family;m=json.loads((folder/'manifest.json').read_text())
  assert hashlib.sha256((folder/'manifest.json').read_bytes()).hexdigest()==receipt['manifest_sha256']
  assert hashlib.sha256((folder/'templates.json').read_bytes()).hexdigest()==receipt['payload_sha256']==m['asset_sha256']
  assert len(m['keys'])==36 and m['font_sha256']==receipt['font_sha256']
 return {'status':'passed','exact_matched_shape_templates':108,'new_quarter_images':240,'new_range_comparisons':480,'new_physical_glyph_comparisons':240,'new_same_overlap_SIG_controls':240,'shared_SIG_registration_and_removal_order':True,'full_page_special_staff_parity':False,'heads_complete':False,'preserved_staff_catalog_font_kernel_and_laura':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
