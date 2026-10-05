#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Declared observed geometry/canonical context; no target selection."""
import json,sys
from pathlib import Path
def make(parts,chunks):
 sections=lambda k:[{n:s[n]for n in ('id','vertical','first_pos','runs')}for s in parts[k]]
 actual=[c for c in parts['cases']if c['mode']=='actual'];assert len(actual)==len(chunks['groups'])==612
 searches=[]
 for upstream,group in zip(actual,chunks['groups']):
  for c in group['cases']:
   assert c['direction']==upstream['direction'] and c['slope']==upstream['slope']
   searches.append({'polygon':[s['coordinates']for s in upstream['area_path']if s['type']in(0,1)],'lookup_parameters':{k:upstream[k]for k in ('direction','slope','max_stem_thickness','max_line_section_dx','last_head_y','theoretical_line')},'initial_catalog':[e['glyph']for e in c['initial_catalog']],'existing_ids':c['existing_ids'],'stump_id':c['stump_id'],'head_glyph':c['head_glyph'],'vip':c['vip'],'max_alignment_dx':c['max_dx'],'max_alignment_dy':c['max_dy']})
 return {'schema':'audiveris-native.stem-chunks-lookup-request.v1','vertical_sections':sections('vertical_sections'),'horizontal_sections':sections('horizontal_sections'),'searches':searches}
if __name__=='__main__':Path(sys.argv[3]).write_text(json.dumps(make(json.loads(Path(sys.argv[1]).read_text()),json.loads(Path(sys.argv[2]).read_text())))+'\n')
