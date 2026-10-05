#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Usage: python scripts/check_head_staff_dependency.py REPORT.json"""
import json,hashlib,sys
from pathlib import Path
from check_head_registration import check as check_registration
from check_head_glyph_retrieval import record

def check(path):
    baseline=check_registration(path)
    full=json.loads(path.read_text());r=full['head_registration'];live=r['live_glyph_retrieval']
    assert r['staff_dependency_complete'] is True and live['staff_dependency_complete'] is True
    assert full['last_verified_heads_substage']=='HEADS.REGISTRATION_AND_EPILOG'
    assert r['heads_complete'] is False and live['heads_complete'] is False
    assert r['shared_sig']['covered_substages'][-1]=='HEADS.REGISTRATION_AND_EPILOG'
    old=full['head_glyph_retrieval']
    for key in ('seed_heads','range_heads','seed_conflicts'):assert live[key]==old[key]
    scans=0
    for s,old_s in zip(live['systems'],old['systems']):
        assert s['system_id']==old_s['system_id'] and len(s['staffs'])==len(old_s['staffs'])
        for f,old_f in zip(s['staffs'],old_s['staffs']):
            assert f['staff_id']==old_f['staff_id']
            assert f['seed_heads']==old_f['seed_heads']
            for key in ('scanners','ledger_scanners'):
                assert len(f[key])==len(old_f[key])
                for c,old_c in zip(f[key],old_f[key]):
                    assert c['filtered_indices']==old_c['filtered_indices']
                    assert c['range_glyphs']==old_c['range_glyphs']
                    assert len(c['seed_competitors'])==len(old_c['seed_competitors'])
                    for a,e in zip(c['seed_competitors'],old_c['seed_competitors']):record(a,e)
                    assert all(v.get('removed',False) is False for v in c['seed_competitors'])
                    scans+=1
    assert scans==118
    return {'status':'passed','staff_dependency_controls':8,'live_physical_scanners':118,
            'heads_complete':False,'shared_phase_refresh_guards_verified':True,
            'certified_HEADS_substage':'HEADS.REGISTRATION_AND_EPILOG','preserved_registration':baseline,
            'report_sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit(__doc__)
    print(json.dumps(check(Path(sys.argv[1])),indent=2))
