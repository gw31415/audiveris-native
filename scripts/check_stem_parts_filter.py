#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected original filter4152; automatic section discovery remains pending."""
import json,sys
from pathlib import Path
from check_stem_parts_alignment import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-parts-filter'
def check(path):
    prior=prior_check(path);r=read(F/'source-backed-reproduction.json')
    assert r['status']=='passed' and r['native_outputs_used'] is False and r['original_fields_byte_reproduced'] is True
    assert r['independent_compilations_and_runs']==2 and r['cases']==4152 and r['builders_observed']==612
    assert sha(F/'controls.json')==r['original_sha256'] and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
    for x in (p,q):
        assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe']=='StemPartsFilterProbe' and 'compile-only' not in x['runtime_classpath']
        assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and x['official_cli_step']=='HEADS'
        assert all(sha(ROOT/n)==h for n,h in x['observer_source_sha256'].items())
        assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['input_sha256']==r['input_sha256']
        assert x['fixtures'][0]['result_sha256']==r['original_sha256']
    for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
    n=read(F/'native-reproduction.json')
    assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'] and all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
    original=read(F/'controls.json');native=read(F/'native-controls.json.gz')
    assert original['native_outputs_used'] is False and len(original['cases'])==len(native['cases'])==4152
    assert original['cases']==native['cases']
    cases=native['cases'];assert original['builders_observed']==612
    assert all(c['slope']==0 and c['max_alignment_dx']==1.65 and c['max_alignment_dy']==44 for c in cases)
    assert sum(c['start_stump_id'] is None for c in cases)==1974
    assert sum(c['y_range'][2]==0 for c in cases)==1680
    assert sum(c['last_head_y'] is not None for c in cases)==312
    assert max(len(c['linker_order']) for c in cases)==7
    assert n['maximum_center_line_coordinate_difference']==0
    return {'status':'passed','selected_original_filter_controls':4152,'actual_first_system_builders':612,
            'nonnull_beam_last_head_y_controls':312,'null_start_stump_controls':1974,'zero_width_range_controls':1680,
            'all_retained_orders_contributions_and_center_lines_exact':True,'observed_maximum_linkers':7,
            'linker_sort_size32_or_more_supported':False,'observed_skew':0,'nonzero_skew_supported':False,
            'automatic_candidate_discovery_complete':False,'item_expansion_gap_length_maps_complete':False,
            'STEMS_epilog_complete':False,'full24_HEADS_page_propagation':False,'published_scan_parity':False,
            'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
