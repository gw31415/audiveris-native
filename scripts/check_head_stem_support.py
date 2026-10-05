#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Original physical stem-length consistency and preserved HEADS/STEMS boundaries."""
import hashlib,json,sys
from pathlib import Path
from check_head_stem_connection import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-stem-support';g=read(f/'controls.json');p=read(f/'provenance.json');q=read(f/'certified-reproduction-provenance.json');r=read(f/'source-backed-reproduction.json')
 assert g['native_outputs_used'] is False and r['native_outputs_used'] is False and r['status']=='passed' and r['all_original_control_fields_reproduced'] is True
 assert sha(f/'controls.json')==r['controls_sha256'] and len(g['batches'])==r['batches']==1536
 for v in (p,q):
  assert v['release']=='5.11.0' and v['probe']=='HeadStemSupportProbe' and v['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd'
  assert v['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert all(sha(ROOT/n)==pin for n,pin in v['observer_source_sha256'].items()) and 'compile-only' not in v['runtime_classpath']
  assert all(c['exit_code']==0 and c['ok'] for c in v['fixtures'])
  assert v['fixtures'][0]['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 assert p['observer_source_sha256']==r['first_observer_sources']==r['second_observer_sources']
 accepted=[v for v in g['batches'] if v['relation'] is not None];assert len(accepted)==r['accepted']==1152 and len(g['batches'])-len(accepted)==r['rejected']==384
 assert sum(c['glyph']['bounds'][1]<0 for c in g['batches'])==r['signed_negative_top_controls']==96
 assert sum(c['small_head'] for c in accepted)==192
 for c in g['batches']:
  assert c['main_interline']==11 and c['small_head']==(c['shape'] in ('NOTEHEAD_BLACK_SMALL','NOTEHEAD_VOID_SMALL'))
  assert c['after']['prior_SIG_preserved'] and c['after']['staff_notes_preserved']
  if c['relation'] is None:assert c['before'] is None
  else:
   a=c['before'];b=c['relation'];assert a['consistency']==1.
   assert all(a[k]==b[k] for k in ('grade','target_ratio','head_side','extension_point','dx','dy','invading'))
   assert b['consistency']>0 and b['source_ratio']!=a['source_ratio']
 return {'status':'passed','original_physical_length_support_batches':1536,'accepted_relations':1152,'geometrically_rejected':384,'small_head_support_relations':192,'source_backed_all_original_fields_reproduced':True,'main_interline_and_small_shape_verified':True,'connection_grade_and_target_support_preserved':True,'automatic_STEMS_complete':False,'SIG_contextual_grades_complete':False,'stems_complete':False,'heads_complete':False,'preserved_head_connection_actual_pages_and_prior_baselines':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
