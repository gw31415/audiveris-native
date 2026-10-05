#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Certify selected createStem controls; automatic selection/SIG/epilog pending."""
import json, sys
from pathlib import Path
from check_cue_default_heads import check as prior_check
from check_cue_clef import read, sha, near

ROOT = Path(__file__).resolve().parents[1]
F = ROOT / 'fixtures/official-stem-builder-create'

def selected(path):
    original = read(F / 'controls.json')
    native = read(path)
    assert original['native_outputs_used'] is False
    assert original['builder_min_grade'] == native['builder_min_grade'] == 0.1 * 0.8
    assert len(original['cases']) == len(native['cases']) == 144
    accepted = rejected = horizontal = compounds = 0
    for a, e in zip(native['cases'], original['cases']):
        # Original vertex-count observation belongs to the original observer.
        # Native selected API has no SIG argument: no invented native graph count.
        assert e['SIG_unchanged_by_createStem'] is True
        comparable = {k: v for k, v in e.items() if k != 'SIG_unchanged_by_createStem'}
        assert set(a) == set(comparable)
        near(a, comparable)
        assert a['registered_delta'] == 1
        assert a['second_white_image_if_regraded'] == 0.0
        if a['first'] is None:
            rejected += 1
            assert a['second'] is None and a['cache_delta'] == 0 and not a['cache_reused_same_stem']
        else:
            accepted += 1
            assert a['cache_delta'] == 1 and a['cache_reused_same_stem']
            assert a['first'] == a['second'] and a['first']['grade'] >= native['builder_min_grade']
            assert a['first']['id'] == 0 and a['first']['in_sig'] is False
        if len(a['parts']) == 1:
            assert a['registered']['orientation'] == a['parts'][0]['orientation']
            horizontal += a['registered']['orientation'] == 'HORIZONTAL'
        else:
            compounds += 1
            assert a['registered']['orientation'] == 'VERTICAL'
    assert (accepted, rejected, horizontal, compounds) == (96, 48, 24, 96)
    return {'status': 'passed', 'scope': 'Selected parts and independent synthetic pixel filters; original initialized on actual Laura HEADS/STEMS context',
            'original_controls': 144, 'accepted': accepted, 'rejected_registered_glyphs': rejected,
            'single_horizontal_orientation_controls': horizontal, 'compound_vertical_unions': compounds,
            'minimum_grade': native['builder_min_grade'], 'accepted_cache_same_object': True,
            'original_createStem_SIG_vertex_count_unchanged': True,
            'native_SIG_registration_claimed': False, 'automatic_parts_selection_verified': False,
            'cleaner_epilog_verified': False, 'heads_complete': False, 'stems_complete': False}

def check(path):
    prior = prior_check(path)
    r = read(F / 'source-backed-reproduction.json')
    assert r['status'] == 'passed' and r['native_outputs_used'] is False and r['original_fields_byte_reproduced'] is True
    assert sha(F / 'controls.json') == r['original_sha256']
    assert r['input_sha256'] == sha(ROOT / 'fixtures/official-grid/laura-gray.png')
    p = read(F / 'controls-provenance.json')
    q = read(F / 'controls-certified-provenance.json')
    for x in (p, q):
        assert x['jar_sha256'] == '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert x['release'] == '5.11.0' and x['probe'] == 'StemBuilderCreateProbe'
        assert 'compile-only' not in x['runtime_classpath']
        assert all(sha(ROOT / n) == h for n, h in x['observer_source_sha256'].items())
        assert len(x['fixtures']) == 1 and x['fixtures'][0]['ok'] and x['fixtures'][0]['exit_code'] == 0
        assert x['fixtures'][0]['input_sha256'] == r['input_sha256']
    for k in ('observer_source_sha256', 'jar_sha256', 'runtime_executable_sha256', 'compiler_executable_sha256', 'resource_sha256', 'dependency_jar_sha256'):
        assert p[k] == q[k]
    n = read(F / 'native-reproduction.json')
    assert n['status'] == 'passed' and sha(F / 'native-controls.json.gz') == n['native_controls_gzip_sha256']
    assert all(sha(ROOT / k) == h for k, h in n['native_source_sha256'].items())
    return {'status': 'passed', 'selected_StemBuilder_create_grade_cache': selected(F / 'native-controls.json.gz'),
            'full24_HEADS_page_propagation': False, 'published_scan_parity': False,
            'automatic_STEMS_complete': False, 'heads_complete': False, 'stems_complete': False,
            'historical_prior_checkpoint_contracts': prior}

if __name__ == '__main__':
    print(json.dumps(check(Path(sys.argv[1])), indent=2))
