#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check newly rendered actual-font page HEADS terminals and explicit special-staff gaps."""
import gzip,hashlib,json,sys
from pathlib import Path
from check_head_all_shapes import check as prior_check
from check_head_page_catalog import check_control
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-head-actual-pages'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def check_pair(native, name):
 case=next(c for c in read(F/'cases.json')['cases'] if c['name']==name)
 assert case['parity']=='passed' and case['processing']=='default'
 full=json.loads(gzip.decompress(native.read_bytes()) if native.suffix=='.gz' else native.read_bytes())
 assert full['head_catalog_control']['family']==case['family'] and full['head_catalog_control']['stored_point_size']==51 and full['head_catalog_control']['cue_first_staff'] is False
 r=check_control(native,F/(name+'.json'))
 original=read(F/(name+'.json'))
 assert all(f['line_count']==5 and f['specific_interline']==11 and not f['is_drum'] and not f['is_tablature'] and not f['is_small'] for s in original['systems'] for f in s['staffs'])
 assert (r['retained_physical_heads'],r['ordered_exclusions'])==(case['heads'],case['exclusions'])
 r.update(control=name,input_sha256=case['image_sha256'],actual_other_font_page_HEADS_terminal_parity=True,source='Independent original-font whole synthetic score raster; original default upstream steps, selected HEADS family/51pt; not a published score scan or complete OMR.')
 return r
def check(path):
 baseline=prior_check(path);cases=read(F/'cases.json');render=read(F/'render-provenance.json');repro=read(F/'render-reproduction.json');receipts=read(F/'native-comparisons.json')
 assert cases['native_outputs_used'] is False and render['native_outputs_used'] is False and repro['status']=='passed' and repro['all_seven_independent_original_rendered_PNG_hashes_reproduced'] is True
 assert sha(ROOT/'scripts/java/HeadActualPageRenderer.java')==render['observer_sha256']
 assert len(render['images'])==7 and render['images']==repro['image_sha256'] and all(sha(F/'images'/n)==v for n,v in render['images'].items())
 for name,provenance in cases['provenance'].items():
  p=read(F/provenance);assert p['release']=='5.11.0' and p['jar_sha256']==render['jar_sha256']
  assert all(sha(ROOT/n)==v for n,v in p['observer_source_sha256'].items())
  assert all(c['ok'] and c['exit_code']==0 for c in p['fixtures'])
  for c in p['fixtures']:assert c['input_sha256']==render['images'][Path(c['input']).name]
 for c in cases['cases']:
  assert sha(F/(c['name']+'.json'))==c['oracle_sha256'] and sha(F/'images'/(c['name']+'.png'))==c['image_sha256']
  observed=[f for s in read(F/(c['name']+'.json'))['systems'] for f in s['staffs']]
  assert observed==c['recognized_staffs']
  if c['processing']=='one-line-drums':assert any(f['line_count']==1 and f['is_drum'] for f in observed)
  if c['processing']=='drums':assert any(f['line_count']==5 and f['is_drum'] for f in observed)
  if c['processing']=='six-string-tab':assert any(f['line_count']==6 and f['is_tablature'] for f in observed) and c['heads']==0
 comparisons=[check_pair(F/'native-controls'/(c['name']+'.json.gz'),c['name']) for c in cases['cases'] if c['parity']=='passed']
 assert receipts['status']=='passed' and comparisons==receipts['comparisons'] and len(comparisons)==3
 cue=next(c for c in cases['cases'] if c['name']=='Bravura-cue');assert cue['parity']=='unsupported' and cue['native_dependency']=='multiple_interline_requires_second_pass'
 cg=read(F/'Bravura-cue.json');assert sum(len(s['heads']) for s in cg['systems'])==168
 assert any(f['is_small'] and f['specific_interline']==7 for s in cg['systems'] for f in s['staffs'])
 return {'status':'passed','actual_font_whole_synthetic_pages_HEADS_compared':3,'comparisons':comparisons,'new_cue_page_original_heads':168,'cue_native_dependency':'GRID multiple_interline_requires_second_pass','special_staff_pages':[{'name':c['name'],'parity':c['parity'],'native_dependency':c.get('native_dependency')} for c in cases['cases'] if c['parity']!='passed'],'special_staff_full_page_parity':False,'heads_complete':False,'complete_STEMS_or_stem_attachment':False,'preserved_all_shapes_and_prior_baselines':baseline}
if __name__=='__main__':
 print(json.dumps(check_pair(Path(sys.argv[1]),sys.argv[2]) if len(sys.argv)==3 else check(Path(sys.argv[1])),indent=2))
