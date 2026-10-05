#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check saved Laura output and the original staff/catalog contract receipts."""
import hashlib,json,sys
from pathlib import Path
from check_head_font_coverage import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def check(path):
 baseline=prior_check(path);folder=ROOT/'fixtures/official-head-staff-catalog'
 g=json.loads((folder/'controls.json').read_text());b=json.loads((folder/'catalog-build.json').read_text())
 assert g['native_outputs_used'] is False
 assert len(g['cases'])==384 and len(g['outside_pitch_controls'])==75 and len(g['tablature_controls'])==2
 assert {c['main_interline'] for c in g['cases']}=={8,11,16}
 assert {c['specific_interline'] for c in g['cases']}=={7,11}
 assert {c['switch_mask'] for c in g['cases']}==set(range(16))
 assert all(c['tab_skipped'] and c['SIG_vertex_delta']==0 for c in g['tablature_controls'])
 assert hashlib.sha256((folder/'original-drum-set.xml').read_bytes()).hexdigest()==b['resource_sha256']
 return {'status':'passed','staff_plan_controls':384,'outside_drum_pitch_controls':75,'tablature_skip_controls':2,'main_scale_drives_offsets':True,'custom_drum_overrides_covered':False,'special_staff_full_page_parity':False,'heads_complete':False,'preserved_font_kernel_and_laura':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
