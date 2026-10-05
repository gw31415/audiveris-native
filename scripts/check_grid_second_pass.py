#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Pin selected two-pass GRID initial staffs and preserved HEADS/STEMS controls."""
import hashlib,json,sys
from pathlib import Path
from check_stem_glyph_grade import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-grid-second-pass';r=read(f/'source-backed-reproduction.json')
 assert r['status']=='passed' and r['native_outputs_used'] is False
 summaries=[]
 for row in r['cases']:
  n=row['name'];g=read(f/(n+'.json'));p=read(f/(n+'-provenance.json'));q=read(f/(n+'-certified-provenance.json'))
  assert g['native_outputs_used'] is False and sha(f/(n+'.json'))==row['sha256'] and row['original_fields_byte_reproduced'] is True
  for v in (p,q):
   assert v['release']=='5.11.0' and v['probe']=='GridSecondPassProbe' and v['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd'
   assert v['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
   assert all(sha(ROOT/n)==pin for n,pin in v['observer_source_sha256'].items()) and 'compile-only' not in v['runtime_classpath']
   assert all(c['exit_code']==0 and c['ok'] for c in v['fixtures'])
   assert v['fixtures'][0]['input_sha256']==sha(ROOT/'fixtures/official-head-actual-pages/images/Bravura-cue.png')
  for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
  assert p['observer_source_sha256']==row['first_observer_sources']==row['second_observer_sources']
  assert all(len(g[k])==row[k] for k in ('input_filaments','first_discarded','second_discarded','staffs'))
  assert g['second_pass_ran']==row['second_pass_ran']
  summaries.append({k:row[k] for k in ('name','input_filaments','first_discarded','second_discarded','staffs','second_pass_ran')})
 assert [s['staffs'] for s in summaries]==[4,3,0]
 cue=read(f/'cue.json');assert [s['interline'] for s in cue['staffs']]==[7,11,11,11] and [s['small'] for s in cue['staffs']]==[True,False,False,False]
 assert cue['length_consistency']['max_diff_length_ratio']==0.5 and len(cue['length_consistency']['cases'])==12
 return {'status':'passed','original_initial_staff_second_pass_controls':summaries,'native_actual_cue_factory_initial_staffs_compared':4,'original_length_consistency_boundaries':12,'completed_GRID_cue_parity':False,'cue_HEADS_terminal_parity':False,'legacy_page_CLI_cue_stop_preserved':True,'heads_complete':False,'stems_complete':False,'preserved_physical_grades_and_prior_baselines':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
