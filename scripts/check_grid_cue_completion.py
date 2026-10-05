#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify independent cue GRID observation and historical HEADS/STEMS baselines."""
import gzip, hashlib, json, math, sys
from pathlib import Path
from check_grid_second_pass import check as prior_check
ROOT = Path(__file__).resolve().parents[1]
F = ROOT/'fixtures/official-grid-cue-completion'
def read(p):
 return json.loads(gzip.decompress(p.read_bytes()) if p.suffix == '.gz' else p.read_bytes())
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def near(a,b):
 if isinstance(b,list):
  assert len(a)==len(b)
  for x,y in zip(a,b):near(x,y)
 elif isinstance(b,(int,float)) and not isinstance(b,bool):assert math.isfinite(a) and abs(a-b)<1e-9,(a,b)
 else:assert a==b,(a,b)
def glyph_native(g):return (tuple(g[k] for k in ('left','top','width','height')),g['orientation'],tuple(tuple(r[k] for k in ('position','start','length')) for r in g['runs']))
def glyph_original(g):return (tuple(g['bounds']),g['orientation'],tuple(map(tuple,g['runs'])))
def check_cue(path):
 a=read(path);b=read(F/'cue-final.json');p=read(F/'cue-projection.json');ir=a['grid_ir'];assert ir['grid_complete'] is False
 for x,y in zip(a['projection']['staff_projections'],p):
  for k in ('staff_id','left','right','projection','derivative_threshold','blank_threshold','lines_threshold','chunk_threshold','blanks','ending_blanks','peaks','peak_grades'):near(x[k],y[k])
 assert len(a['projection']['staff_projections'])==len(p)==4
 for k in ('mask_sha256','masked_pixel_count','distance_sha256_be_i16'):assert a['mask'][k]==b['note_preparation'][k]
 for k in ('pages','hlag','vlag'):assert ir[k]==b[k]
 native_glyphs={glyph_native(g):g['id'] for g in ir['glyphs']};original_glyphs={glyph_original(g):g['id'] for g in b['glyphs']}
 assert len(native_glyphs)==len(ir['glyphs'])==len(original_glyphs)==len(b['glyphs'])==32 and native_glyphs.keys()==original_glyphs.keys()
 gid={v:original_glyphs[k] for k,v in native_glyphs.items()};assert len(ir['staffs'])==len(b['staffs'])==len(b['staff_states'])==4
 for x,y,state in zip(ir['staffs'],b['staffs'],b['staff_states']):
  assert x['specific_interline']==state['specific_interline'] and x.get('small',False)==state['small']
  assert x['system_id']==y['system_id'] and len(x['lines'])==len(y['lines'])==5
  for l,m in zip(x['lines'],y['lines']):
   near(l['points'],m['points']);near(l['thickness'],m['thickness']);assert gid[l['glyph_id']]==m['glyph_id']
 assert len(ir['systems'])==len(b['systems'])==len(ir['bar_semantics']['systems'])==4
 for x,y,z in zip(ir['systems'],b['systems'],ir['bar_semantics']['systems']):
  for k in ('id','staff_ids','coordinates','horizontal_section_ids','vertical_section_ids','part_groups','parts'):assert x[k]==y[k]
  assert len(z['inters'])==len(y['inters']);ids={}
  for i in z['inters']:
   matches=[j for j in y['inters'] if all(i[k]==j[k] for k in ('kind','bounds','staff_id'))];assert len(matches)==1;j=matches[0];ids[i['id']]=j['id']
   for k in ('shape','grade','contextual_grade','frozen'):near(i[k],j[k])
   if i.get('glyph_id') is not None:assert gid[i['glyph_id']]==j['glyph_id']
  assert {(r['kind'],ids[r['source']],ids[r['target']]) for r in z['relations']}=={(r['kind'],r['source'],r['target']) for r in y['relations']}
 return {'status':'passed','scope':'One independent synthetic cue page, observed GRID state only','staffs':4,'systems':4,'small_staffs':1,'physical_glyphs':32,'hlag_sections':len(ir['hlag']),'vlag_sections':len(ir['vlag']),'masked_pixels':a['mask']['masked_pixel_count'],'mask_sha256':a['mask']['mask_sha256'],'distance_sha256_be_i16':a['mask']['distance_sha256_be_i16'],'original_cue_HEADS_target_unverified':{'heads':168,'exclusions':198},'heads_complete':False,'stems_complete':False}
def check(path):
 prior=prior_check(path);r=read(F/'source-backed-reproduction.json')
 assert r['status']=='passed' and r['native_outputs_used'] is False and r['input_sha256']==sha(ROOT/'fixtures/official-head-actual-pages/images/Bravura-cue.png')
 for row in r['cases']:
  n=row['name'];assert sha(F/(n+'.json'))==row['sha256'] and row['original_fields_byte_reproduced']
  first=read(F/(n+'-provenance.json'));second=read(F/(n+'-certified-provenance.json'))
  for p in (first,second):
   assert p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and p['release']=='5.11.0'
   assert all(sha(ROOT/n)==h for n,h in p['observer_source_sha256'].items()) and 'compile-only' not in p['runtime_classpath']
   assert p['fixtures'][0]['input_sha256']==r['input_sha256'] and all(c['ok'] and c['exit_code']==0 for c in p['fixtures'])
  for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert first[k]==second[k]
  assert first['observer_source_sha256']==row['first_observer_sources']==row['second_observer_sources']
 p=read(F/'native-reproduction.json');assert p['status']=='passed' and all(sha(ROOT/n)==h for n,h in p['native_source_sha256'].items()) and sha(F/'native-cue-grid.json.gz')==p['native_report_gzip_sha256']
 cue=check_cue(F/'native-cue-grid.json.gz');assert cue==p['comparison']
 return {'status':'passed','independent_synthetic_cue_GRID':cue,'cue_GRID_diagnostic_CLI_supported':True,'current_cue_page_CLI_dependency':'unsupported_key_mixed_specific_interline','actual_published_scan_parity':False,'full24_HEADS_page_propagation':False,'automatic_STEMS_complete':False,'grid_complete':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check_cue(Path(sys.argv[1])) if len(sys.argv)>2 and sys.argv[2]=='cue' else check(Path(sys.argv[1])),indent=2))
