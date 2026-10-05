#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Beam lookup areas from declared borders/neighborhoods/gap pixels."""
from pathlib import Path
import json,sys
from check_stem_lookup_area import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-beam-lookup-area'
def canonical(points):
    unique=[]
    for point in points:
        if not unique or unique[-1]!=point:unique.append(point)
    if len(unique)>1 and unique[0]==unique[-1]:unique.pop()
    points=unique
    if not points:return []
    rings=[]
    for ring in (points,list(reversed(points))):
        rings.extend(ring[i:]+ring[:i] for i in range(len(ring)))
    return min(rings)
def check(path):
    prior=prior_check(path)
    r=read(F/'source-backed-reproduction.json')
    assert r['status']=='passed' and r['native_outputs_used'] is False
    assert r['independent_compilations_and_runs']==2 and r['cases']==72
    assert r['original_sha256']==sha(F/'controls.json')
    assert r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
    for x in (p,q):
        assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe']=='StemBeamLookupAreaProbe' and x['official_cli_step']=='HEADS'
        assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd'
        assert 'compile-only' not in x['runtime_classpath']
        assert all(sha(ROOT/n)==h for n,h in x['observer_source_sha256'].items())
        assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0
        assert x['fixtures'][0]['input_sha256']==r['input_sha256'] and x['fixtures'][0]['result_sha256']==r['original_sha256']
    for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
    n=read(F/'native-reproduction.json')
    assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256']
    assert all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
    original=read(F/'controls.json');native=read(F/'native-controls.json.gz')
    assert original['native_outputs_used'] is False and original['beam_builders']==12
    assert len(original['cases'])==len(native['cases'])==72
    assert sorted({c['profile']for c in original['cases']})==r['observed_original_profiles']==[1]
    assert sorted({c['gap_pixels']for c in original['cases']})==r['observed_profile_gap_pixels']==[13]
    for a,b in zip(original['cases'],native['cases']):
        assert b['polygon']==canonical([s['coordinates']for s in a['area_path']if s['type']in(0,1)])
        assert b['theoretical_line']==a['theoretical_line']
    assert all(native[k] is False for k in ('beam_neighborhood_selection_complete','closer_limit_selection_complete','heads_complete','stems_complete'))
    return {'status':'passed','actual_beam_builder_references':12,'original_beam_area_controls':72,'observed_original_profiles':[1],'observed_profile_gap_pixels':[13],'exact_nonzero_polygon_edges_and_theoretical_lines':True,'both_directions_and_part_bottom_gap_ULP_zero_ratio_verified':True,'ring_start_winding_and_exact_zero_edges_normalized_only':True,'declared_border_neighborhood_and_profile_gap_pixels':True,'beam_neighborhood_and_closer_limit_selection_complete':False,'chunk_registration_complete':False,'cache_head_SIG_composition_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
