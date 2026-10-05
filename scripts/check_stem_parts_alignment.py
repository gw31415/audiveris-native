#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected original part-alignment kernel; automatic collection remains pending."""
import json,sys
from pathlib import Path
from check_stem_cleaner import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-parts-alignment'
def check(path):
    prior=prior_check(path);r=read(F/'source-backed-reproduction.json')
    assert r['status']=='passed' and r['native_outputs_used'] is False and r['original_fields_byte_reproduced'] is True
    assert r['independent_compilations_and_runs']==2 and r['cases']==1500
    assert sha(F/'controls.json')==r['original_sha256'] and r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
    for x in (p,q):
        assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe']=='StemPartsAlignmentProbe' and 'compile-only' not in x['runtime_classpath']
        assert x['upstream_commit']=='9e1e55cd2746037d059345881c53e6a6754bffbd' and x['official_cli_step']=='HEADS'
        assert all(sha(ROOT/n)==h for n,h in x['observer_source_sha256'].items())
        assert len(x['fixtures'])==1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code']==0 and x['fixtures'][0]['input_sha256']==r['input_sha256']
        assert x['fixtures'][0]['result_sha256']==r['original_sha256']
    for k in ('observer_source_sha256','jar_sha256','runtime_executable_sha256','compiler_executable_sha256','resource_sha256','dependency_jar_sha256'):assert p[k]==q[k]
    n=read(F/'native-reproduction.json')
    assert n['status']=='passed' and sha(F/'native-controls.json.gz')==n['native_controls_gzip_sha256'] and all(sha(ROOT/k)==h for k,h in n['native_source_sha256'].items())
    original=read(F/'controls.json');native=read(F/'native-controls.json.gz')
    assert original['native_outputs_used'] is False and len(original['cases'])==len(native['cases'])==1500
    assert original['cases']==native['cases']
    removed=0
    for c in native['cases']:
        assert c['stump'] is not None and c['slope']==0 and c['max_alignment_dx']==1.65 and c['max_alignment_dy']==44
        assert c['direction'] in (-1,1) and c['mode'] in ('empty','normal','reverse','duplicates','with-stump')
        dropped=set(c['removed_order']);assert c['retained_order']==[x for x in c['input_order'] if x not in dropped]
        removed+=len(c['removed_order'])
    assert removed==2810
    return {'status':'passed','selected_StemBuilder_filterUnaligned_controls':1500,'removed_part_entries':removed,
            'stable_order_object_aliases_actual_stumps_verified':True,'observed_skew':0,
            'null_stump_original_certified':False,'nonzero_skew_supported':False,
            'automatic_candidate_collection_complete':False,'whole_parts_selection_complete':False,
            'STEMS_epilog_complete':False,'full24_HEADS_page_propagation':False,'published_scan_parity':False,
            'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
