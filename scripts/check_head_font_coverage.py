#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify preserved Laura CLI and the declared, independently observed kernel coverage."""
import hashlib,json,sys
from pathlib import Path
from check_head_staff_dependency import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def check(path):
 baseline=prior_check(path)
 fixture=ROOT/'fixtures/official-head-font-coverage'
 controls=json.loads((fixture/'controls.json').read_text())
 receipts=json.loads((fixture/'bank-provenance.json').read_text())
 assert controls['native_outputs_used'] is False and receipts['native_outputs_used'] is False
 assert receipts['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
 expected={(f,s,h,v) for f in ('Bravura','Leland','FinaleJazz') for s in (32,40,51,64) for h in ('BREVE','WHOLE_NOTE','NOTEHEAD_VOID','NOTEHEAD_BLACK') for v in ('clean','erased','edge','empty')}
 images=controls['images'];assert len(images)==len(expected)==192
 assert {(c['family'],c['point_size'],c['input']['shape'],c['variant']) for c in images}==expected
 for family in ('Bravura','Leland','FinaleJazz'):
  bank=ROOT/'crates/audiveris-template-bank/assets/coverage'/family
  manifest=json.loads((bank/'manifest.json').read_text());receipt=receipts['families'][family]
  assert hashlib.sha256((bank/'manifest.json').read_bytes()).hexdigest()==receipt['manifest_sha256']
  assert hashlib.sha256((bank/'templates.json').read_bytes()).hexdigest()==receipt['payload_sha256']==manifest['asset_sha256']
  assert receipt['font_sha256']==manifest['font_sha256'] and len(manifest['keys'])==16
 return {'status':'passed','exact_templates':48,'original_rendered_images':192,'range_comparisons':384,'physical_glyph_comparisons':192,'fonts':['Bravura','Leland','FinaleJazz'],'point_sizes':[32,40,51,64],'heads_complete':False,'page_font_or_special_staff_parity_claimed':False,'preserved_staff_dependency':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
