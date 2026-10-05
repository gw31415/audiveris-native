#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Convert an observed original catalog to the explicit CLI input contract.

Does not generate native Head/Beam lookup areas; refuses non-linear/multiple rings.
"""
import json,sys
def make(f):
    sections=lambda name:[{k:s[k]for k in ('id','vertical','first_pos','runs')}for s in f[name]]
    searches=[]
    for c in f['cases']:
        points=[];closed=False
        for segment in c['area_path']:
            assert not closed
            if segment['type']==0:assert not points;points.append(segment['coordinates'])
            elif segment['type']==1:assert points;points.append(segment['coordinates'])
            elif segment['type']==4:closed=True
            else:raise ValueError('uncertified curve/multipath lookup area')
        assert closed
        searches.append({'polygon':points,'stump':c['stump'],'parameters':{k:c[k]for k in ('direction','slope','max_stem_thickness','max_line_section_dx','last_head_y','theoretical_line')}})
    return {'schema':'audiveris-native.stem-parts-lookup-request.v1','vertical_sections':sections('vertical_sections'),'horizontal_sections':sections('horizontal_sections'),'searches':searches}
if __name__=='__main__':
    with open(sys.argv[1])as f:request=make(json.load(f))
    with open(sys.argv[2],'x')as f:json.dump(request,f)
