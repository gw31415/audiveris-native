#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent original Glyph / StemChecker provenance and preserved baselines."""
import hashlib,json,sys
from pathlib import Path
from check_head_stem_stump import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-stem-glyph-grade';g=read(f/'controls.json');p=read(f/'provenance.json');q=read(f/'certified-reproduction-provenance.json');r=read(f/'source-backed-reproduction.json')
 assert g['native_outputs_used'] is False and r['native_outputs_used'] is False and r['status']=='passed'
 assert sha(f/'controls.json')==r['controls_sha256'] and len(g['cases'])==r['cases']==1152
 assert r['all_original_control_fields_reproduced'] is True
 finite=[c for c in g['cases'] if isinstance(c['checked']['impacts']['grade'],(int,float))]
 assert len(finite)==r['finite_cases']==996 and len(g['cases'])-len(finite)==r['nonfinite_cases']==156
 assert sum(c['checked']['impacts']['grade']>=c['checked']['min_threshold'] for c in finite)==r['accepted_at_seed_threshold']
 for v in (p,q):
  assert v['release']=='5.11.0' and v['probe']=='StemGlyphGradeProbe' and v['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd'
  assert v['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
  assert all(sha(ROOT/n)==pin for n,pin in v['observer_source_sha256'].items()) and 'compile-only' not in v['runtime_classpath']
  assert all(c['exit_code']==0 and c['ok'] for c in v['fixtures'])
  assert v['fixtures'][0]['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
 for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
 assert p['observer_source_sha256']==r['first_observer_sources']==r['second_observer_sources']
 assert {c['parameters']['profile'] for c in g['cases']}=={0,1}
 return {'status':'passed','original_physical_glyph_grade_cases':1152,'finite_grades_compared':996,'nonfinite_original_cases_explicitly_unsupported':156,'automatic_STEMS_complete':False,'StemBuilder_cache_complete':False,'heads_complete':False,'stems_complete':False,'preserved_head_stem_stump_and_prior_baselines':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
