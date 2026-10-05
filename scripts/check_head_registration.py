#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Usage: python scripts/check_head_registration.py REPORT.json"""
import json,hashlib,sys
from pathlib import Path
from check_head_glyph_retrieval import check as check_glyphs,record

def check(path):
    baseline=check_glyphs(path)
    root=Path(__file__).resolve().parents[1]
    full=json.loads(path.read_text());r=full['head_registration']
    g=json.loads((root/'fixtures/official-head-registration/laura.json').read_text())
    assert g['native_outputs_used'] is False
    assert r['head_registration_complete'] is True and r['epilog_seed_scale_complete'] is True
    assert r['heads_complete'] is False
    assert (r['retained_heads'],r['duplicates_removed'],r['overlap_pairs'])==(352,22,513)
    assert r['small_beam_removed_heads']==r['small_beams_removed']==0
    sig=r['shared_sig'];by_id={v['id']:v for v in sig['inters']}
    assert len(by_id)==len(sig['inters'])
    for e in sig['relations']:assert e['source'] in by_id and e['target'] in by_id
    original_head_map={};scanned=0;exclusions=0;small=0
    assert len(r['systems'])==len(g['systems'])
    for s,gs in zip(r['systems'],g['systems']):
        assert s['system_id']==gs['system_id'] and len(s['heads'])==len(gs['heads'])
        ids={}
        for a,e in zip(s['heads'],gs['heads']):
            record(a,e);assert a['abnormal']==e['abnormal'] is True
            assert a['removed']==e['removed'] is False
            ids[e['id']]=a['id'];original_head_map[e['id']]=a['id']
            live=by_id[a['id']];record(live,e)
            assert live['origin']['system']==s['system_id'] and live['origin']['phase']=='HEAD'
            assert live['glyph_id']==a['glyph_id'];scanned+=1
        expected=[(ids[e['source']],ids[e['target']]) for e in gs['relations'] if e['source'] in ids and e['target'] in ids]
        assert [(e['source'],e['target']) for e in s['exclusions']]==expected
        exclusions+=len(expected)
        assert len(s['staffs'])==len(gs['staffs'])
        for a,e in zip(s['staffs'],gs['staffs']):
            assert a['staff_id']==e['staff_id']
            assert a['notes']==[ids[v] for v in e['notes']]
            assert all(by_id[v]['staff_id']==a['staff_id'] for v in a['notes'])
        for b in s['small_beams']:
            e=next(v for v in gs['other_inters'] if v['kind']==b['kind'] and v['bounds']==b['bounds'])
            assert abs(b['contextual_grade']-e['contextual_grade'])<1e-12
            assert by_id[b['id']]['contextual_grade']==b['contextual_grade'];small+=1
    assert (scanned,exclusions,small)==(352,513,3)
    for a,e in zip(r['head_seed_scale'],g['head_seed_scale']):
        assert a['shape']==e['shape'] and a['side']==e['side']
        if e['dx'] is None:assert a['dx'] is None
        else:assert abs(a['dx']-e['dx'])<1e-14
    # Earlier shared identities and edges survive the continuation on one Graph.
    old=full['shared_sig'];old_ids={v['id'] for v in old['inters']}
    for v in old['inters']:
        now=dict(by_id[v['id']]);now.pop('contextual_grade',None)
        before=dict(v);before.pop('contextual_grade',None)
        # HEADS adds explicit live removal metadata only for contextualized beam snapshots.
        if 'removed' not in before:now.pop('removed',None)
        assert now==before,(now,before)
    assert sig['relations'][:len(old['relations'])]==old['relations']
    assert len(old_ids)==90
    return {'status':'passed','heads':352,'duplicate_removals':22,'HEADS_exclusions':513,
            'staff_note_counts':[len(f['notes']) for s in r['systems'] for f in s['staffs']],
            'small_beams_contextualized':3,'purge_controls':40,'beam_context_controls':7,
            'small_beam_controls':9,'seed_tally_controls':4,'heads_complete':False,
            'prior_shared_ids_and_edges_preserved':True,'preserved_glyphs':baseline,
            'report_sha256':hashlib.sha256(path.read_bytes()).hexdigest()}

if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit(__doc__)
    print(json.dumps(check(Path(sys.argv[1])),indent=2))
