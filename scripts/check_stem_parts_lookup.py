#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected original section lookup/raw assembly; lookup areas are supplied."""
from pathlib import Path
import sys,json
from check_stem_parts_filter import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-parts-lookup'
def check(path):
    prior=prior_check(path)
    r=read(F/'source-backed-reproduction.json')
    assert r['status']=='passed' and r['native_outputs_used'] is False
    assert r['independent_compilations_and_runs']==2 and r['cases']==2343
    assert sha(F/'controls.json')==r['original_sha256']
    assert r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
    for x in (p,q):
        assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe']=='StemPartsLookupProbe' and x['official_cli_step']=='HEADS'
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
    assert original['native_outputs_used'] is False
    assert len(original['vertical_sections'])==1390 and len(original['horizontal_sections'])==1645
    assert len(original['cases'])==len(native['cases'])==2343
    modes={'actual':612,'distance-below':565,'distance-equal':565,'distance-above':565,'headstop-below':12,'headstop-equal':12,'headstop-above':12}
    assert {m:sum(c['mode']==m for c in original['cases'])for m in modes}==modes
    for a,b in zip(original['cases'],native['cases']):
        assert a['slope']==0 and a['raw_sticks_min_core0']==a['raw_sticks_min_core1']==b['raw_sticks']
        for k in ('selected_vertical_ids','selected_horizontal_ids'):assert a[k]==b[k]
    assert sum(len(c['raw_sticks'])for c in native['cases'])==4459
    return {'status':'passed','original_section_lookup_controls':2343,'actual_first_system_builders':612,'catalog_sections':3035,'raw_glyph_entries':4459,
            'exact_selected_ids_order_centroids_and_physical_glyphs':True,'original_lookup_area_supplied':True,'native_lookup_area_derivation':False,
            'observed_skew':0,'nonzero_skew_supported':False,'centroid_int_overflow_supported':False,
            'automatic_parts_page_CLI_complete':False,'chunk_filter_registration_complete':False,'STEMS_epilog_complete':False,
            'full24_HEADS_page_propagation':False,'published_scan_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
