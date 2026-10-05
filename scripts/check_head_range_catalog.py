#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare an independent native Laura CLI report to original 5.11 default range catalog.

Usage: python scripts/check_head_range_catalog.py REPORT.json [ORACLE.json]
"""
import hashlib
import json
from pathlib import Path
import sys
from check_head_range_prefix import check as check_prefix


def candidates(actual, expected):
    assert len(actual) == len(expected)
    for match, reference in zip(actual, expected):
        for field in ('shape', 'source_shape', 'anchor', 'x', 'y', 'bounds'):
            assert match[field] == reference[field], (field, match, reference)
        for field in ('score', 'grade'):
            assert abs(match[field] - reference[field]) < 1e-12
        assert set(match['eligibility']) == set(reference['eligibility'])
        for field, value in reference['eligibility'].items():
            actual_value = match['eligibility'][field]
            if isinstance(value, bool):
                assert actual_value is value, (field, match)
            else:
                assert abs(actual_value - value) < 1e-12, (field, match)


def check(report_path, oracle_path):
    root = Path(__file__).resolve().parents[1]
    prefix = check_prefix(report_path, root / 'fixtures/official-head-range-prefix.json')
    report = json.loads(report_path.read_text())
    oracle = json.loads(oracle_path.read_text())
    assert oracle['native_outputs_used'] is False
    actual = report['head_range_catalog']
    assert actual['range_catalog_candidates_complete'] is True
    assert actual['range_search_complete'] is False and actual['heads_complete'] is False
    assert actual['eligibility_parameters'] == oracle['eligibility_parameters']
    for key in ('mask_sha256', 'distance_sha256_be_i16'):
        assert actual['distance_mask'][key] == oracle[key]
    assert actual['dispatch'] == report['head_range_search']['dispatch']
    count = rejected = scanners = 0
    stemless = {}
    assert len(actual['systems']) == len(oracle['systems'])
    for system, expected in zip(actual['systems'], oracle['systems']):
        assert system['system_id'] == expected['system_id']
        assert len(system['staffs']) == len(expected['staffs'])
        for staff, golden_staff in zip(system['staffs'], expected['staffs']):
            for field in ('staff_id', 'point_size', 'header_stop'):
                assert staff[field] == golden_staff[field]
            for key in ('scanners', 'ledger_scanners'):
                assert len(staff[key]) == len(golden_staff[key])
                for scan, golden in zip(staff[key], golden_staff[key]):
                    for field in ('line_index', 'ledger_index', 'ledger_order', 'dir', 'pitch',
                                  'scan_range', 'template_half', 'y_offsets', 'black_relevant_spans',
                                  'visited', 'skipped', 'all_shapes', 'hollow_shapes'):
                        assert scan.get(field) == golden.get(field), (staff['staff_id'], field)
                    canonical = lambda row: json.dumps(row, sort_keys=True)
                    assert sorted(map(canonical, scan['selected_spots'])) == sorted(map(canonical, golden['selected_spots']))
                    candidates(scan['matches'], golden['matches'])
                    candidates(scan['rejected'], golden['rejected'])
                    scanners += 1
                    count += len(scan['matches'])
                    rejected += len(scan['rejected'])
                    for m in scan['matches']:
                        if m['eligibility']['stemless']:
                            stemless[m['shape']] = stemless.get(m['shape'], 0) + 1
    assert (scanners, count, rejected) == (118, 2224, 407)
    assert actual['proposal_count'] == count
    assert actual['rejected_candidate_count'] == rejected
    assert actual['raw_proposal_count'] == count + rejected
    return {'status': 'passed', 'range_scanners': scanners, 'accepted_candidates': count,
            'weak_stemless_rejected': rejected, 'raw_candidates': count + rejected,
            'accepted_stemless': stemless, 'preserved_prefix': prefix, 'heads_complete': False,
            'oracle_sha256': hashlib.sha256(oracle_path.read_bytes()).hexdigest(),
            'report_sha256': hashlib.sha256(report_path.read_bytes()).hexdigest()}


if __name__ == '__main__':
    if len(sys.argv) not in (2, 3):
        raise SystemExit(__doc__)
    oracle = Path(sys.argv[2]) if len(sys.argv) == 3 else Path(__file__).resolve().parents[1] / 'fixtures/official-head-range-catalog/laura.json'
    print(json.dumps(check(Path(sys.argv[1]), oracle), indent=2))
