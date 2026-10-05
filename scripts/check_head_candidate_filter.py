#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Usage: python scripts/check_head_candidate_filter.py REPORT.json"""
import json
import hashlib
from pathlib import Path
import sys
from check_head_range_catalog import check as check_catalog


def check(report_path):
    root=Path(__file__).resolve().parents[1]
    baseline=check_catalog(report_path,root/'fixtures/official-head-range-catalog/laura.json')
    r=json.loads(report_path.read_text())['head_candidate_filter']
    g=json.loads((root/'fixtures/official-head-candidate-filter/laura.json').read_text())
    assert r['range_aggregation_complete'] is True and r['seed_conflict_kernel_complete'] is True
    assert r['live_seed_conflicts_complete'] is False and r['heads_complete'] is False
    scans=total=0
    for s,gs in zip(r['systems'],g['systems']):
        assert s['system_id']==gs['system_id']
        for f,gf in zip(s['staffs'],gs['staffs']):
            assert f['staff_id']==gf['staff_id']
            for key in ('scanners','ledger_scanners'):
                assert len(f[key])==len(gf[key])
                for c,gc in zip(f[key],gf[key]):
                    expected=gc['candidate_filter']
                    actual=c['candidate_filter']
                    assert actual['aggregate_indices']==expected['aggregate_indices']
                    assert actual['max_template_dx']==expected['max_template_dx']==4
                    assert actual['aggregated']==[c['matches'][i] for i in expected['aggregate_indices']]
                    scans+=1;total+=len(actual['aggregated'])
    assert (scans,total,r['aggregated_count'])==(118,454,454)
    return {'status':'passed','range_scanners':scans,'aggregates':total,'seed_conflict_controls':21,
            'live_seed_conflicts_complete':False,'heads_complete':False,'preserved_catalog':baseline,
            'report_sha256':hashlib.sha256(report_path.read_bytes()).hexdigest()}


if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit(__doc__)
    print(json.dumps(check(Path(sys.argv[1])),indent=2))
