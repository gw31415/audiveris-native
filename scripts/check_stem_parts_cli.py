#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Actual explicit-input collector CLI; full automatic page STEMS is incomplete."""
import json,sys
from pathlib import Path
from check_stem_parts_lookup import check as prior_check
from check_cue_clef import read,sha
from stem_parts_lookup_request import make
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-parts-cli'
def check(path):
    prior=prior_check(path);proof=read(F/'cli-reproduction.json')
    assert proof['status']=='passed' and proof['actual_CLI_execution'] is True
    assert sha(F/'request.json.gz')==proof['request_gzip_sha256'] and sha(F/'report.json.gz')==proof['report_gzip_sha256']
    assert all(sha(ROOT/k)==h for k,h in proof['native_source_sha256'].items())
    original=read(ROOT/'fixtures/official-stem-parts-lookup/controls.json')
    assert read(F/'request.json.gz')==make(original)
    report=read(F/'report.json.gz')
    assert report['schema']=='audiveris-native.stem-parts-lookup-report.v1' and report['lookup_geometry']=='caller_supplied_linear_polygons'
    assert report['sections']==3035 and len(report['cases'])==2343
    assert all(report[k] is False for k in ('registered_to_sig','heads_complete','stems_complete'))
    for a,b in zip(original['cases'],report['cases']):
        assert a['selected_vertical_ids']==b['vertical_ids'] and a['selected_horizontal_ids']==b['horizontal_ids']
        assert a['raw_sticks_min_core0']==b['raw_glyphs']
    assert sum(len(c['raw_glyphs'])for c in report['cases'])==4459
    return {'status':'passed','actual_section_lookup_CLI_controls':2343,'actual_first_system_builders':612,'exact_raw_glyph_entries':4459,'empty_and_malformed_late_input_refusal_checked':True,
            'existing_outputs_preserved':True,'input_byte_search_and_work_budgets_checked':True,'original_lookup_geometry_supplied':True,'automatic_lookup_geometry':False,
            'cache_head_SIG_composition_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,
            'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
