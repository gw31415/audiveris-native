// SPDX-License-Identifier: AGPL-3.0-or-later
// Regression checks for the development-only original observer. This does not
// claim native BeamInter construction or load oracle data at product runtime.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
fn load(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn sha(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}
#[test]
fn atomic_beam_prolog_observer_preserves_original_whole_final_and_v15_evidence() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let validation = load(&root.join("evidence/beam-prolog-oracle/validation.json"));
    let cases = validation["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 13);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let faithful_path = root.join(case["fixture"].as_str().unwrap());
        let plain_path = root.join(case["original_plain_fixture"].as_str().unwrap());
        let legacy_path = root.join(case["legacy_boundary_fixture"].as_str().unwrap());
        assert_eq!(sha(&faithful_path), case["fixture_sha256"], "{name}");
        assert_eq!(sha(&plain_path), case["original_plain_sha256"], "{name}");
        assert_eq!(sha(&legacy_path), case["legacy_boundary_sha256"], "{name}");
        let faithful = load(&faithful_path);
        let plain = load(&plain_path);
        let legacy = load(&legacy_path);
        assert_eq!(
            faithful["mode"],
            "original_atomic_prolog_with_post_prolog_diagnostic_replays"
        );
        assert_eq!(
            faithful["final"], plain["final"],
            "{name}: whole original final"
        );
        for key in [
            "masks",
            "sheet_spots",
            "replayed_beam_structure_diagnostics",
        ] {
            assert_eq!(faithful[key], legacy[key], "{name}: v15 {key}");
        }
        assert_eq!(
            faithful["prolog"]["head_scale"], faithful["final"]["head_scale"],
            "{name}"
        );
        let prolog = faithful["prolog"]["systems"].as_array().unwrap();
        let detailed = faithful["systems"].as_array().unwrap();
        assert_eq!(prolog.len(), detailed.len());
        for (a, b) in prolog.iter().zip(detailed) {
            assert_eq!(a["system_id"], b["system_id"], "{name}");
            assert_eq!(
                a["grouped_spots"], b["grouped_spots_before_beams"],
                "{name}"
            );
        }
        if name == "laura-gray" {
            let systems = faithful["final"]["systems"].as_array().unwrap();
            for (system, bounds) in systems
                .iter()
                .zip([[993, 629, 15, 13], [811, 1055, 15, 13]])
            {
                let spots = system["remaining_grouped_spots"].as_array().unwrap();
                assert_eq!(spots.len(), 1);
                assert_eq!(spots[0]["glyph"]["bounds"], serde_json::json!(bounds));
                assert_eq!(
                    spots[0]["groups"],
                    serde_json::json!(["BEAM_SPOT", "BLACK_HEAD_SPOT"])
                );
            }
        }
    }
}
