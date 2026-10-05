#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Original VIP-dependent head chunk pixel filter."""
from pathlib import Path
import json,sys
from check_stem_chunk_registration import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-chunk-head-filter'
def check(path):
    prior=prior_check(path)
    r=read(F/'source-backed-reproduction.json')
    assert r['status']=='passed' and r['native_outputs_used'] is False
    assert r['independent_compilations_and_runs']==2 and r['cases']==6600
    assert r['original_sha256']==sha(F/'controls.json')
    assert r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
    for x in (p,q):
        assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe']=='StemChunkHeadFilterProbe' and x['official_cli_step']=='HEADS'
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
    assert original['native_outputs_used'] is False and original['builders']==600
    assert original['JVM_ids_normalized_to_canonical_identity_tokens'] is True
    assert len(original['groups'])==len(native['groups'])==600
    assert sum(len(g['cases'])for g in original['groups'])==6600
    old=read(ROOT/'fixtures/official-stem-parts-lookup/controls.json')
    actual=[c for c in old['cases']if c['mode']=='actual'][:600]
    for previous,a,b in zip(actual,original['groups'],native['groups']):
        catalog={g['id']:g['glyph']for g in a['catalog']}
        assert [catalog[i]for i in a['cases'][0]['part_ids']]==previous['raw_sticks_min_core0']
        assert len(a['cases'])==len(b['cases'])==11
        for c,d in zip(a['cases'],b['cases']):
            for k in ('remaining_weights','retained_ids'):assert c[k]==d[k]
        for c in a['cases'][1:7]:
            remaining=int(c['mode'].split('-')[1]);assert c['remaining_weights']==[remaining]
            assert c['retained_ids']==([]if c['vip']and remaining<15 else c['part_ids'])
        assert a['cases'][7]['retained_ids']==[]
        assert a['cases'][8]['remaining_weights'][0]in(0,1) and a['cases'][8]['retained_ids']==[]
        assert a['cases'][9]['remaining_weights']==[None] and a['cases'][9]['retained_ids']==a['cases'][9]['part_ids']
        assert a['cases'][10]['retained_ids']==[]
    assert native['original_VIP_condition_preserved'] is True
    assert all(native[k] is False for k in ('chunk_filter_sort_composition_complete','registered_to_SIG','heads_complete','stems_complete'))
    return {'status':'passed','actual_head_builders':600,'original_head_chunk_filter_controls':6600,'exact_physical_pixel_remaining_weights_and_retained_token_order':True,'original_VIP_removal_condition_preserved':True,'remaining_14_15_16_duplicates_last_row_one_past_and_empty_checked':True,'registered_excluded_glyphs_preserved':True,'original_JVM_ids_normalized':True,'full_lookupChunks_composition_complete':False,'cache_head_SIG_integration_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}

if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
