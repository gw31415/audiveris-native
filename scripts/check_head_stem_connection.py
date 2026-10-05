#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Pin independently reproduced original selected-STEMS controls and prior HEADS."""
import hashlib,json,sys
from pathlib import Path
from check_head_actual_pages import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-stem-connection'
 g=read(f/'controls.json');p=read(f/'provenance.json');q=read(f/'certified-reproduction-provenance.json');r=read(f/'source-backed-reproduction.json')
 counts={'references':2112,'gaps':288,'connections':5760,'glyphs':40,'lifecycle':12,'batches':144}
 assert g['native_outputs_used'] is False and r['native_outputs_used'] is False
 assert r['status']=='passed' and r['all_original_control_fields_reproduced'] is True
 assert sha(f/'controls.json')==r['controls_sha256']
 assert {k:len(g[k]) for k in counts}==r['controlled_counts']==counts
 assert g['minimum_grade']==r['original_minimum_grade']==.1
 for v in (p,q):
  assert v['release']=='5.11.0' and v['probe']=='HeadStemConnectionProbe'
  assert v['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd'
  assert v['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert all(sha(ROOT/n)==pin for n,pin in v['observer_source_sha256'].items())
  assert all(c['exit_code']==0 and c['ok'] for c in v['fixtures'])
  assert 'compile-only' not in v['runtime_classpath']
  assert v['fixtures'][0]['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('upstream_commit','jar_sha256','observer_source_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 assert p['observer_source_sha256']==r['first_observer_sources']==r['second_observer_sources']
 shapes=read(ROOT/'fixtures/official-head-staff-catalog/controls.json')['shape_sets']['StemHeads']
 assert [c['shape'] for c in g['lifecycle']]==shapes
 assert all(c['parallel_relation_accepted'] and len(c['states'])==8 and all(s['prior_SIG_preserved'] and s['staff_notes_preserved'] for s in c['states']) for c in g['lifecycle'])
 assert all(c['after']['prior_SIG_preserved'] and c['after']['staff_notes_preserved'] for c in g['batches'])
 assert all(c['glyph']['orientation']=='VERTICAL' for c in g['glyphs']+g['batches'])
 assert all(c['single_part_identity_preserved'] is False for c in g['glyphs'] if len(c['parts'])==1)
 return {'status':'passed','original_selected_STEMS_controls':counts,'all_original_control_fields_reproduced':True,'vertical_union_even_single_horizontal_part':True,'selected_glyph_head_connection_and_SIG_batches':144,'nonmanual_parallel_relation_lifecycles':12,'automatic_STEMS_complete':False,'stems_complete':False,'heads_complete':False,'preserved_actual_font_pages_all_shapes_and_prior_baselines':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
