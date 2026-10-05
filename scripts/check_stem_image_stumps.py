#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import gzip, hashlib, json, sys
from pathlib import Path
from check_stem_image_seeds import check as prior_check, compare as seed_compare

ROOT = Path(__file__).resolve().parents[1]
F = ROOT / 'fixtures/official-stem-image-stumps'
def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):
    return json.loads(gzip.decompress(p.read_bytes()) if p.suffix == '.gz' else p.read_bytes())
def key(c):
    return (tuple(c['head_bounds']), c['shape'], c['side'], c['vertical'])
def compare(o, n, page):
    assert o['native_outputs_used'] is False
    assert o['corners'] == n['corner_count'] == 1408
    assert len(o['systems']) == len(n['systems']) == len(page['grid_ir']['systems']) == 2
    reg = n['physical_registry']
    old = page['head_registration']['live_glyph_retrieval']['registry']
    assert len(old) == 1220 and reg[:len(old)] == old and len(reg) == 1566
    assert [g['id'] for g in reg] == list(range(1, len(reg)+1))
    assert len({json.dumps(g['glyph'], sort_keys=True) for g in reg}) == len(reg)
    assert all(g['groups'] == ['STUMP'] for g in reg[len(old):])
    sections = {s['id']: s for s in page['grid_ir']['vlag']}
    count = selected = seed = fallback = rejected = no_glyph = 0
    catalogs = []
    for a, b, grid in zip(o['systems'], n['systems'], page['grid_ir']['systems']):
        assert a['system'] == b['system'] == grid['id']
        assert a['main_stem_thickness'] == 4 and a['stump_area_dy'] == 2
        assert a['stump_area_dx_in'] == a['stump_area_dx_out'] == 1.1
        native = [sections[i] for i in grid['vertical_section_ids']]
        assert len(native) == len(a['sections'])
        for original, actual in zip(a['sections'], native):
            assert all(original[k] == actual[k] for k in ('id', 'first_pos', 'runs', 'bounds'))
        catalogs.append(len(native))
        expected = {key(c): c for c in a['cases']}
        assert len(expected) == len(a['cases']) == len(b['cases'])
        for c in b['cases']:
            e = expected[key(c['head'])]
            assert e['stump'] == c['selected_stump']
            id = c['selected_stump_id']
            if id is not None:
                assert reg[id-1]['glyph'] == e['stump']
                selected += 1
            if c['seed_id'] is not None:
                assert id == c['seed_id'] and c['assembly'] is None
                seed += 1
            elif c['registered_fallback_id'] is None:
                assert id is None and c['fallback_stands_out'] is None
                no_glyph += 1
            else:
                glyph = reg[c['registered_fallback_id']-1]['glyph']
                assert glyph['orientation'] == 'VERTICAL'
                if c['fallback_stands_out']:
                    assert id == c['registered_fallback_id']
                    fallback += 1
                else:
                    assert id is None
                    rejected += 1
            count += 1
    assert catalogs == [1390, 2346]
    assert (count, selected, seed, fallback, rejected, no_glyph) == (1408, 724, 577, 147, 554, 130)
    assert n['new_registered_fallback_glyphs'] == 346
    assert n['selected_stump_corners'] == selected and n['selected_section_stump_corners'] == fallback
    assert n['registered_rejected_fallback_corners'] == rejected
    assert n['image_corner_stump_selection_complete'] is True
    assert all(n[k] is False for k in ('final_stem_glyph_selection_complete', 'automatic_raster_pipeline_complete', 'page_cache_sig_complete', 'epilog_complete', 'heads_complete', 'stems_complete'))
    assert len(o['run_intersection_controls']) == 8 and len(o['polygon_contains_controls']) == 6 and len(o['subsection_controls']) == 7
    assert o['registry_reuse_control']['same_original_object'] is True
    assert o['registry_reuse_control']['groups'] == ['VERTICAL_SEED']
    assert o['registered_rejected_control']['stands_out'] is False
    assert o['registered_rejected_control']['same_registered_entity'] is True
    return {'status': 'passed', 'image_corners_exact': count, 'selected_stumps_exact': selected,
            'seed_corners': seed, 'section_stump_corners': fallback, 'registered_rejected_corners': rejected,
            'no_section_glyph_corners': no_glyph, 'new_physical_registrations': 346,
            'system_section_catalogs_order_and_pixels_exact': catalogs,
            'whole_HEADS_STEMS_complete': False, 'automatic_final_stem_pipeline_complete': False}
def check(path):
    prior = prior_check(path)
    o = read(F/'controls.json')
    proof = read(F/'source-backed-reproduction.json')
    assert proof['status'] == 'passed' and proof['native_outputs_used'] is False and proof['independent_original_runs'] == 2
    assert proof['original_sha256'] == sha(F/'controls.json')
    for file in ('controls-provenance.json', 'controls-certified-provenance.json'):
        p = read(F/file)
        assert p['probe'] == 'StemImageStumpsProbe' and p['official_cli_step'] == 'HEADS' and p['release'] == '5.11.0'
        assert p['upstream_commit'] == '9e1e55cd2746037d059345881c53e6a6754bffbd'
        assert p['jar_sha256'] == '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert 'compile-only' not in p['runtime_classpath']
        assert all(sha(ROOT/k) == v for k, v in p['observer_source_sha256'].items())
        assert len(p['fixtures']) == 1
        f = p['fixtures'][0]
        assert f['ok'] and f['exit_code'] == 0 and f['result_sha256'] == sha(F/'controls.json')
        assert f['input_sha256'] == sha(ROOT/'fixtures/official-grid/laura-gray.png')
    n = read(F/'native-stems-report.json.gz')
    r = read(F/'native-reproduction.json')
    assert r['status'] == 'passed' and r['actual_image_CLI_executed'] is True
    assert all(sha(ROOT/k) == v for k, v in r['native_source_sha256'].items())
    assert sha(F/'native-stems-report.json.gz') == r['native_stems_report_gzip_sha256']
    assert hashlib.sha256(gzip.decompress((F/'native-stems-report.json.gz').read_bytes())).hexdigest() == r['native_stems_report_raw_sha256']
    assert n['source_image_sha256'] == sha(ROOT/'fixtures/official-grid/laura-gray.png')
    assert n['native_HEADS_report_sha256'] == r['entire_old_HEADS_report_sha256'] == '87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
    page = read(path)
    assert (page['head_registration']['retained_heads'], page['head_registration']['overlap_pairs']) == (352, 513)
    seed_compare(read(ROOT/'fixtures/official-stem-image-seeds/controls.json'), n['seed_plan'])
    result = compare(o, n, page)
    result.update(original_loaded_book_context=True, original_fresh_transient_context_newly_verified=False,
                  nonzero_skew_real_scans_special_staff_full24_pages_verified=False,
                  prior_default_and_seed_report_bytes_unchanged=True, historical_prior_checkpoint_contracts=prior)
    return result
if __name__ == '__main__':
    print(json.dumps(check(Path(sys.argv[1])), indent=2))
