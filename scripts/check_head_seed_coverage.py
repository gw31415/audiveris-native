#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check original seed coverage receipts and preserved full Laura CLI reports."""
import hashlib,json,sys
from pathlib import Path
from check_head_drum_coverage import check as prior_check
ROOT=Path(__file__).resolve().parents[1]
def check(path):
 baseline=prior_check(path);f=ROOT/'fixtures/official-head-seed-coverage';g=json.loads((f/'controls.json').read_text());p=json.loads((f/'provenance.json').read_text());r=json.loads((f/'render-reproduction.json').read_text())
 shapes=('NOTEHEAD_VOID','NOTEHEAD_BLACK','NOTEHEAD_BLACK_SMALL','NOTEHEAD_CROSS','NOTEHEAD_DIAMOND_FILLED','NOTEHEAD_TRIANGLE_DOWN_FILLED','NOTEHEAD_CIRCLE_X')
 assert g['native_outputs_used'] is False and g['source_shapes']==list(shapes)
 assert p['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5' and p['release']=='5.11.0' and p['probe']=='HeadSeedCoverageProbe'
 for name,digest in p['observer_source_sha256'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest,name
 expected={(family,size,shape,variant) for family in ('Bravura','Leland','FinaleJazz') for size in (32,40,51,64) for shape in shapes for variant in ('clean','erased','edge','empty')}
 assert len(g['images'])==336 and {(c['family'],c['point_size'],c['rendered_shape'],c['variant']) for c in g['images']}==expected
 for c in g['images']:
  assert (ROOT/'fixtures'/c['image']).is_file()
  assert len(c['windows'])==6 and {(w['anchor'],w['veto_mode']) for w in c['windows']}=={(a,v) for a in ('LEFT_STEM','RIGHT_STEM') for v in ('none','central','shifted')}
  for w in c['windows']:
   assert w['x_offsets']==[0,1,-1,2,-2] and w['y_offsets']==[0,-1,1]
   assert w['original_bar_veto_count']==w['original_central_veto_count']+w['original_noncentral_veto_count']
   if w['veto_mode']=='central':assert not w['matches'] and w['original_eval_count']==7
 assert r['images_independently_reproduced']==336 and r['all_original_rendered_png_bytes_match_existing_fixtures'] is True and r['original_windows']==2016 and r['original_noncentral_vetoes']>0
 assert r['controls_sha256']==hashlib.sha256((f/'controls.json').read_bytes()).hexdigest()
 return {'status':'passed','original_seed_windows':2016,'independently_reproduced_existing_images':336,'matched_stem_shapes':7,'original_central_vetoes':r['original_central_vetoes'],'original_noncentral_vetoes':r['original_noncentral_vetoes'],'strict_shape_order_and_legacy_Bravura51_API_preserved':True,'page_catalog_propagation_complete':False,'heads_complete':False,'preserved_quarter_staff_font_and_laura':baseline}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
