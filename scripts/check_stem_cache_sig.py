#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected original cache-to-SIG lifecycle, with explicit JVM-ID normalization."""
import json, sys
from pathlib import Path
from check_stem_builder_create import check as prior_check
from check_cue_clef import read, sha, near
ROOT = Path(__file__).resolve().parents[1]
F = ROOT / 'fixtures/official-stem-cache-sig'
def canonical(cases):
    cases = json.loads(json.dumps(cases))
    for c in cases:
        assert c['before_SIG']['id'] == 0 and c['before_SIG']['in_sig'] is False
        identity = c['after_SIG']['id']; assert identity > 0
        for stage in ('after_SIG', 'after_cached_white', 'after_removed'):
            assert c[stage]['id'] == identity and c[stage]['in_sig'] is True
            c[stage]['id'] = 1
    return cases
def check(path):
    prior = prior_check(path)
    r = read(F / 'source-backed-reproduction.json')
    assert r['status'] == 'passed' and r['native_outputs_used'] is False
    assert r['original_fields_byte_reproduced'] is False and r['original_fields_equal_after_JVM_ID_renumbering'] is True
    assert sha(F / 'controls.json') == r['original_sha256'] and sha(F / 'controls-certified.json') == r['certified_original_sha256']
    assert r['input_sha256'] == sha(ROOT / 'fixtures/official-grid/laura-gray.png')
    original = read(F / 'controls.json'); certified = read(F / 'controls-certified.json')
    assert original['native_outputs_used'] is certified['native_outputs_used'] is False
    assert canonical(original['cases']) == canonical(certified['cases'])
    p = read(F / 'controls-provenance.json'); q = read(F / 'controls-certified-provenance.json')
    for x in (p, q):
        assert x['release'] == '5.11.0' and x['jar_sha256'] == '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['probe'] == 'StemCacheSigLifecycleProbe' and 'compile-only' not in x['runtime_classpath']
        assert all(sha(ROOT / n) == h for n, h in x['observer_source_sha256'].items())
        assert len(x['fixtures']) == 1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code'] == 0
        assert x['fixtures'][0]['input_sha256'] == r['input_sha256']
    for k in ('observer_source_sha256', 'jar_sha256', 'runtime_executable_sha256', 'compiler_executable_sha256', 'resource_sha256', 'dependency_jar_sha256'):
        assert p[k] == q[k]
    n = read(F / 'native-reproduction.json')
    assert n['status'] == 'passed' and sha(F / 'native-controls.json.gz') == n['native_controls_gzip_sha256']
    assert all(sha(ROOT / k) == h for k, h in n['native_source_sha256'].items())
    native = read(F / 'native-controls.json.gz')
    near(canonical(native['cases']), canonical(original['cases']))
    assert len(native['cases']) == 12
    for c in native['cases']:
        assert c['cached_same_object_after_SIG'] and c['cached_same_object_after_removal'] and c['cached_removed_flag'] and c['removed']
        assert c['vertices_added'] == 1 and c['vertices_after_removal_delta'] == 0
        for stage in ('after_SIG', 'after_cached_white', 'after_removed'):
            for field in ('grade', 'bounds', 'median', 'width', 'glyph', 'impacts'):
                assert c[stage][field] == c['before_SIG'][field]
    return {'status': 'passed', 'selected_cache_to_SIG_controls': 12,
            'same_cached_object_after_registration_and_removal': True,
            'single_horizontal_glyph_orientation_preserved': True,
            'original_JVM_IDs_normalized': True, 'original_raw_results_byte_identical': False,
            'head_relation_generation_complete': False, 'automatic_parts_selection_complete': False,
            'cleaner_epilog_complete': False, 'full24_HEADS_page_propagation': False,
            'published_scan_parity': False, 'heads_complete': False, 'stems_complete': False,
            'historical_prior_checkpoint_contracts': prior}
if __name__ == '__main__': print(json.dumps(check(Path(sys.argv[1])), indent=2))
