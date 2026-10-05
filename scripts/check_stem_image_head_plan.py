#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
import gzip, hashlib, json, sys
from pathlib import Path
from check_stem_head_reference_context import check as prior_check
ROOT = Path(__file__).resolve().parents[1]
F = ROOT / 'fixtures/official-stem-image-head-plan'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p): return json.loads(gzip.decompress(p.read_bytes()) if p.suffix == '.gz' else p.read_bytes())
def compare(o, n):
    assert o['native_outputs_used'] is False and o['corners'] == n['corner_count'] == 1408
    assert len(o['systems']) == len(n['systems']) == 2
    count = 0
    for a, b in zip(o['systems'], n['systems']):
        c = b['image_context']
        assert a['system'] == b['system'] and a['parts'] == c['parts'] and a['profile'] == c['profile'] == 1
        bm = {}
        for v in c['beams']:
            found = [x for x in a['beams'] if all(x[k] == v[k] for k in ('median', 'top', 'bottom', 'height'))]
            assert len(found) == 1
            bm[v['id']] = found[0]['id']
        assert len(bm) == len(a['beams'])
        for v in c['beam_bounds']:
            x = next(x for x in a['beam_bounds'] if x['id'] == bm[v['id']])
            assert x['bounds'] == v['bounds'] and x['removed'] == v['removed'] is False
        gm = {}
        for v in c['groups']:
            found = [x for x in a['groups'] if sorted(x['members']) == sorted(bm[t] for t in v['members'])]
            assert len(found) == 1
            gm[v['id']] = found[0]['id']
        assert len(gm) == len(a['groups'])
        key = lambda x: (tuple(x['head_bounds']), x['shape'], x['side'], x['vertical'])
        expected = {key(x): x for x in a['cases']}
        assert len(expected) == len(a['cases']) == len(c['corners']) == len(b['cases'])
        for meta, p in zip(c['corners'], b['cases']):
            e, t = expected[key(meta)], p['target']
            assert e['head']['reference'] == meta['reference'] and e['staff_id'] == meta['staff_id']
            for k, v, mp in [('neighbors', p['neighbors'], bm), ('candidates', p['candidates'], bm), ('selected_beams', t['selected_beams'], bm), ('selected_groups', t['selected_groups'], gm)]:
                assert e[k] == [mp[x] for x in v], (k, meta)
            assert e['target_beam'] == (None if t['target_beam'] is None else bm[t['target_beam']])
            assert e['initial_line'] == t['initial_line'] and e['target'] == t['target']
            assert e['area_line'] == t['area']['theoretical_line']
            assert sorted(x['coordinates'] for x in e['area_path'] if x['type'] in (0, 1)) == sorted(t['area']['polygon'])
            count += 1
    assert n['image_head_target_selection_complete'] is True and n['raw_HEADS_report_preserved'] is True
    assert all(n[k] is False for k in ('automatic_raster_pipeline_complete', 'page_cache_sig_complete', 'epilog_complete', 'heads_complete', 'stems_complete'))
    return count
def check(path):
    prior = prior_check(path)
    o = read(F/'controls.json')
    proof = read(F/'source-backed-reproduction.json')
    assert proof['status'] == 'passed' and proof['native_outputs_used'] is False and proof['independent_original_runs'] == 2
    assert proof['original_sha256'] == sha(F/'controls.json')
    for filename in ('controls-provenance.json', 'controls-certified-provenance.json'):
        p = read(F/filename)
        assert p['probe'] == 'StemImageHeadPlanProbe' and p['official_cli_step'] == 'HEADS' and p['release'] == '5.11.0'
        assert p['upstream_commit'] == '9e1e55cd2746037d059345881c53e6a6754bffbd' and p['jar_sha256'] == '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'
        assert 'compile-only' not in p['runtime_classpath'] and all(sha(ROOT/k) == v for k, v in p['observer_source_sha256'].items())
        assert len(p['fixtures']) == 1 and p['fixtures'][0]['ok'] and p['fixtures'][0]['exit_code'] == 0
        assert p['fixtures'][0]['result_sha256'] == proof['original_sha256'] and p['fixtures'][0]['input_sha256'] == sha(ROOT/'fixtures/official-grid/laura-gray.png')
    n = read(F/'native-stems-report.json.gz')
    reproduction = read(F/'native-reproduction.json')
    assert reproduction['status'] == 'passed' and reproduction['actual_image_CLI_executed'] is True
    assert all(sha(ROOT/k) == v for k, v in reproduction['native_source_sha256'].items())
    assert sha(F/'native-stems-report.json.gz') == reproduction['native_stems_report_gzip_sha256']
    assert n['source_image_sha256'] == sha(ROOT/'fixtures/official-grid/laura-gray.png')
    assert n['native_HEADS_report_sha256'] == reproduction['entire_old_HEADS_report_sha256'] == '87aa76d6b8a91fc5260caa01fdb1484dca315eb45faeff038ef7e523f6fcebf9'
    count = compare(o, n)
    assert proof['actual_image_corners_targets'] == count and reproduction['raw_image_HEADS_report_preserved'] is True
    return {'status':'passed','image_CLI_corners_targets':count,'original_loaded_context':True,'original_transient_context_newly_verified':False,'caller_corner_conditions_required':False,'whole_old_report_hash_preserved':True,'image_item_glyph_selection_complete':False,'page_cache_SIG_complete':False,'STEMS_epilog_complete':False,'nonzero_skew_supported':False,'real_scans_special_staff_full24_pages_verified':False,'heads_complete':False,'stems_complete':False,'historical_prior_checkpoint_contracts':prior}
if __name__ == '__main__': print(json.dumps(check(Path(sys.argv[1])), indent=2))
