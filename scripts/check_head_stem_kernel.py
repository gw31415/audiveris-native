#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check original STEMS kernel pins and preserved complete HEADS controls."""
import hashlib,json,sys
from pathlib import Path
from check_head_page_catalog import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-stem-kernel';g=json.loads((f/'controls.json').read_text());p=json.loads((f/'provenance.json').read_text());second=json.loads((f/'certified-reproduction-provenance.json').read_text());repro=json.loads((f/'source-backed-reproduction.json').read_text())
 assert g['native_outputs_used'] is False and p['native_outputs_used'] is False
 assert p['release']=='5.11.0' and p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
 assert hashlib.sha256((ROOT/'scripts/java/HeadStemKernelProbe.java').read_bytes()).hexdigest()==p['observer_sha256']
 assert hashlib.sha256((f/'controls.json').read_bytes()).hexdigest()==p['controls_sha256']==repro['controls_sha256']
 assert repro['status']=='passed' and repro['all_original_kernel_fields_reproduced'] is True
 for key in ('jar_sha256','observer_sha256','controls_sha256','runtime_sha256','compiler_sha256'):assert p[key]==second[key]
 assert p['exit_code']==second['exit_code']==0 and second['native_outputs_used'] is False
 assert {k:len(g[k]) for k in ('portions','consistency','invading','gap_limits')}==p['controlled_counts']=={'portions':264,'consistency':20,'invading':70,'gap_limits':3}
 assert g['default_consistency']==1. and len([c for c in g['consistency'] if c['consistency']=='Infinity'])==1
 return {'status':'passed','original_head_stem_portion_controls':264,'original_consistency_controls':20,'original_invading_controls':70,'original_gap_limit_profiles':3,'source_backed_original_kernels_reproduced':True,'small_zero_length_original_infinity_preserved':True,'complete_STEMS_or_stem_attachment':False,'heads_complete':False,'preserved_complete_controlled_HEADS':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
