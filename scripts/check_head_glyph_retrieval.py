#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Usage: python scripts/check_head_glyph_retrieval.py REPORT.json"""
import json
import hashlib
from pathlib import Path
import sys
from check_head_candidate_filter import check as check_filter


def record(a,e):
    for k in ('shape','bounds','staff_id','pitch','glyph'):
        assert a[k]==e[k], (k,a,e)
    for k in ('grade','left_dx','right_dx'):
        if e[k] is None:
            assert a[k] is None
        else:
            assert abs(a[k]-e[k])<1e-12,(k,a,e)


def check(report_path):
    baseline=check_filter(report_path)
    root=Path(__file__).resolve().parents[1]
    r=json.loads(report_path.read_text())['head_glyph_retrieval']
    g=json.loads((root/'fixtures/official-head-glyph-retrieval/laura.json').read_text())
    assert g['native_outputs_used'] is False
    assert r['head_glyph_retrieval_complete'] is True and r['live_seed_conflicts_complete'] is True
    assert r['heads_complete'] is False
    assert (r['seed_heads'],r['range_heads'],r['seed_conflicts'])==(358,16,438)
    scans=0
    assert len(r['systems'])==len(g['systems'])
    for s,gs in zip(r['systems'],g['systems']):
        assert s['system_id']==gs['system_id'] and len(s['staffs'])==len(gs['staffs'])
        for f,gf in zip(s['staffs'],gs['staffs']):
            expected=gf['scanners'][0]['glyph_retrieval']['new_staff_seeds']
            assert len(f['seed_heads'])==len(expected)
            for a,e in zip(f['seed_heads'],expected):record(a,e)
            for key in ('scanners','ledger_scanners'):
                assert len(f[key])==len(gf[key])
                for c,gc in zip(f[key],gf[key]):
                    e=gc['glyph_retrieval']
                    assert c['filtered_indices']==e['filtered_indices']
                    assert len(c['seed_competitors'])==len(e['seed_competitors'])
                    for a,b in zip(c['seed_competitors'],e['seed_competitors']):record(a,b)
                    assert len(c['range_glyphs'])==len(e['range_glyphs'])
                    for a,b in zip(c['range_glyphs'],e['range_glyphs']):
                        assert a['candidate_index']==b['candidate_index']
                        if b['result'] is None:assert a['result'] is None
                        else:record(a['result'],b['result'])
                    scans+=1
    assert scans==118
    return {'status':'passed','range_scanners':118,'seed_glyphs':358,'seed_conflicts':438,
            'range_glyphs':16,'foreground_controls':16,'heads_complete':False,
            'preserved_filter':baseline,'report_sha256':hashlib.sha256(report_path.read_bytes()).hexdigest()}


if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit(__doc__)
    print(json.dumps(check(Path(sys.argv[1])),indent=2))
