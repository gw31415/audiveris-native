#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare an independent native Laura CLI report with the pinned original oracle.

Usage: python scripts/check_head_range_prefix.py REPORT.json [ORACLE.json]
"""
import hashlib
import json
from pathlib import Path
import sys


def check(report_path, oracle_path):
    report = json.loads(report_path.read_text())
    oracle = json.loads(oracle_path.read_text())
    assert oracle["native_outputs_used"] is False
    raw, veto, actual = (report[k] for k in
                         ("head_seed_search", "head_seed_veto", "head_range_search"))
    assert raw["proposal_count"] == 418 and veto["proposal_count"] == 358
    assert veto["seed_veto_complete"] is True
    assert actual["range_black_void_prefix_complete"] is True
    assert actual["range_search_complete"] is False and actual["heads_complete"] is False
    assert report["musicxml_complete"] is False
    for key in ("mask_sha256", "distance_sha256_be_i16"):
        assert actual["distance_mask"][key] == oracle[key]
        assert veto["distance_preparation"][key] == oracle[key]
    dispatch = actual["dispatch"]
    assert dispatch["head_spots_sha256"] == oracle["head_spots_sha256"]
    assert dispatch["system_spot_references"] == oracle["system_spot_references"] == 402
    assert len(actual["systems"]) == len(oracle["systems"])
    scanners = visited = skipped = proposals = 0
    for system, expected in zip(actual["systems"], oracle["systems"]):
        assert system["system_id"] == expected["system_id"]
        spots = next(s for s in dispatch["systems"] if s["system_id"] == system["system_id"])
        canonical = lambda row: json.dumps({k: v for k, v in row.items()
                                           if k != "sheet_spot_index"}, sort_keys=True)
        assert sorted(map(canonical, spots["spots"])) == sorted(map(canonical, expected["spots"]))
        assert len(system["staffs"]) == len(expected["staffs"])
        for staff, control in zip(system["staffs"], expected["staffs"]):
            assert staff["staff_id"] == control["staff_id"]
            assert staff["point_size"] == control["point_size"] == 51
            for key in ("scanners", "ledger_scanners"):
                assert len(staff[key]) == len(control[key])
                for scan, golden in zip(staff[key], control[key]):
                    for field in ("line_index", "ledger_index", "ledger_order", "dir", "pitch",
                                  "scan_range", "template_half", "black_relevant_spans", "visited", "skipped"):
                        assert scan.get(field) == golden.get(field), (staff["staff_id"], field)
                    assert sorted(map(canonical, scan["selected_spots"])) == sorted(map(canonical, golden["selected_spots"]))
                    assert len(scan["matches"]) == len(golden["matches"])
                    for match, reference in zip(scan["matches"], golden["matches"]):
                        for field in ("shape", "source_shape", "anchor", "x", "y", "bounds"):
                            assert match[field] == reference[field], (staff["staff_id"], field)
                        for field in ("score", "grade"):
                            assert abs(match[field] - reference[field]) < 1e-12
                    scanners += 1
                    visited += len(scan["visited"])
                    skipped += len(scan["skipped"])
                    proposals += len(scan["matches"])
    assert (scanners, visited, skipped, proposals) == (118, 47790, 869, 2221)
    return {"status": "passed", "raw_seed_proposals": 418, "veto_seed_proposals": 358,
            "head_spot_system_references": 402, "range_scanners": scanners,
            "visited_abscissae": visited, "skipped_windows": skipped,
            "black_void_prefix_proposals": proposals, "heads_complete": False,
            "oracle_sha256": hashlib.sha256(oracle_path.read_bytes()).hexdigest(),
            "report_sha256": hashlib.sha256(report_path.read_bytes()).hexdigest()}


if __name__ == "__main__":
    if len(sys.argv) not in (2, 3):
        raise SystemExit(__doc__)
    oracle = Path(sys.argv[2]) if len(sys.argv) == 3 else Path(__file__).resolve().parents[1] / "fixtures/official-head-range-prefix.json"
    print(json.dumps(check(Path(sys.argv[1]), oracle), indent=2))
