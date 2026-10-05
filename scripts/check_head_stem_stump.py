#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Pin original corner-stump contacts and all preserved HEADS/STEMS controls."""
import hashlib,json,sys
from pathlib import Path
from check_head_stem_support import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-stem-stump';g=read(f/'controls.json');p=read(f/'provenance.json');q=read(f/'certified-reproduction-provenance.json');r=read(f/'source-backed-reproduction.json')
 assert g['native_outputs_used'] is False and r['native_outputs_used'] is False
 assert r['status']=='passed' and r['all_original_control_fields_reproduced'] is True
 assert sha(f/'controls.json')==r['controls_sha256'] and len(g['batches'])==r['batches']==6048
 assert sha(f/'HeadLinker-CLinker-original.javap.txt')==r['caller_bytecode_sha256']
 caller=(f/'HeadLinker-CLinker-original.javap.txt').read_text();assert 'Field stump:Lorg/audiveris/omr/glyph/Glyph;' in caller and 'HeadStemRelation.checkRelation:' in caller
 for v in (p,q):
  assert v['release']=='5.11.0' and v['probe']=='HeadStemStumpProbe' and v['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd'
  assert v['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert all(sha(ROOT/n)==pin for n,pin in v['observer_source_sha256'].items()) and 'compile-only' not in v['runtime_classpath']
  assert all(c['exit_code']==0 and c['ok'] for c in v['fixtures'])
  assert v['fixtures'][0]['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 assert p['observer_source_sha256']==r['first_observer_sources']==r['second_observer_sources']
 assert sum(c['relation'] is not None for c in g['batches'])==r['accepted']==4272
 assert sum(c['relation'] is None for c in g['batches'])==r['rejected']==1776
 assert sum(c['stump'] is None for c in g['batches'])==r['null_stumps']==3024
 assert sum(c['relation'] is None and c['union_contact_relation'] is not None for c in g['batches'])==r['whole_union_false_positives']==1104
 assert {c['profile'] for c in g['batches']}=={0,1,2}
 assert all(len(c['parts'])==2 and c['main_interline']==11 and c['after']['prior_SIG_preserved'] and c['after']['staff_notes_preserved'] for c in g['batches'])
 return {'status':'passed','original_explicit_corner_stump_union_batches':6048,'accepted_relations':4272,'rejected_relations':1776,'null_stump_controls':3024,'whole_union_contact_false_positives_prevented':1104,'source_backed_all_original_fields_reproduced':True,'automatic_corner_stump_selection_complete':False,'automatic_STEMS_complete':False,'heads_complete':False,'stems_complete':False,'preserved_support_connection_actual_pages_and_prior_baselines':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
