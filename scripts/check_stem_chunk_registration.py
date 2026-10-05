#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Chunk registry canonical identity/removeAll prefix only."""
from pathlib import Path
import json,sys
from check_stem_beam_lookup_area import check as prior_check
from check_cue_clef import read,sha
ROOT=Path(__file__).resolve().parents[1]
F=ROOT/'fixtures/official-stem-chunk-registration'
def check(path):
    prior=prior_check(path)
    r=read(F/'source-backed-reproduction.json')
    assert r['status']=='passed' and r['native_outputs_used'] is False
    assert r['independent_compilations_and_runs']==2 and r['cases']==2448
    assert r['original_sha256']==sha(F/'controls.json')
    assert r['input_sha256']==sha(ROOT/'fixtures/official-grid/laura-gray.png')
    p=read(F/'controls-provenance.json');q=read(F/'controls-certified-provenance.json')
    for x in (p,q):
        assert x['release']=='5.11.0' and x['jar_sha256']=='9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe']=='StemChunkRegistrationProbe' and x['official_cli_step']=='HEADS'
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
    assert original['native_outputs_used'] is False and original['builders']==612 and original['actual_raw_glyphs']==1364
    assert original['JVM_ids_normalized_to_canonical_identity_tokens'] is True
    old=read(ROOT/'fixtures/official-stem-parts-lookup/controls.json')
    actual=[c for c in old['cases']if c['mode']=='actual']
    assert len(actual)==len(original['groups'])==len(native['groups'])==612
    assert sum(len(g['cases'])for g in native['groups'])==2448
    assert sum(len(c['raw_glyphs'])for g in original['groups']for c in g['cases'])==r['control_raw_glyph_entries']==6025
    assert sum(not g['cases'][0]['raw_glyphs']for g in original['groups'])==43
    for previous,a,b in zip(actual,original['groups'],native['groups']):
        assert a['cases'][0]['raw_glyphs']==previous['raw_sticks_min_core0']
        assert len(a['cases'])==len(b['cases'])==4
        for c,d in zip(a['cases'],b['cases']):
            for k in ('registered_ids','retained_ids','new_count','registry_count'):assert c[k]==d[k]
        assert b['cases'][2]['retained_ids']==[]
        assert all(c['registry_count']==b['cases'][0]['registry_count']for c in b['cases'])
    assert native['JVM_ids_normalized_to_canonical_identity_tokens'] is True
    assert all(native[k] is False for k in ('chunk_filters_complete','registered_to_SIG','heads_complete','stems_complete'))
    return {'status':'passed','actual_builders':612,'actual_raw_glyphs':1364,'original_registration_prefix_controls':2448,'control_raw_glyph_entries':6025,'empty_actual_builders':43,'exact_canonical_identity_token_order_aliases_and_registry_counts':True,'registered_excluded_glyphs_retained':True,'original_JVM_ids_normalized':True,'original_index_scope':'fresh index per actual builder, actual sheet initialization, strongly held canonical objects','full_chunk_filter_sort_composition_complete':False,'page_shared_index_cache_head_SIG_composition_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'full24_HEADS_page_propagation':False,'published_scan_or_special_staff_full_page_parity':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}

if __name__=='__main__':print(json.dumps(check(Path(sys.argv[1])),indent=2))
