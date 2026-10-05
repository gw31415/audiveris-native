// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, head_range_search, head_registration, head_seed_search, shared_sig::Graph,
    template_probes,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};

#[test]
fn original_150_ulp_boundaries_and_all_24_registration_abnormal_states() {
    let controls: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-all-shapes/boundaries.json"
    ))
    .unwrap();
    assert_eq!(controls["boundaries"].as_array().unwrap().len(), 150);
    for c in controls["boundaries"].as_array().unwrap() {
        let actual = head_range_search::candidate_decision_for_shape(
            c["shape"].as_str().unwrap(),
            f64::from_bits(c["score_bits"].as_u64().unwrap()),
        )
        .unwrap();
        assert_eq!(actual, c["eligibility"]);
    }
    let sets: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-staff-catalog/controls.json"
    ))
    .unwrap();
    let original: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-page-catalog/Leland-40.json"
    ))
    .unwrap();
    let template = original["systems"][0]["heads"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["shape"] == "WHOLE_NOTE")
        .unwrap();
    assert_eq!(
        json!(head_range_search::SUPPORTED_SHAPES),
        sets["shape_sets"]["Heads"]
    );
    assert_eq!(
        json!(head_seed_search::ALL_STEM_SHAPES),
        sets["shape_sets"]["StemHeads"]
    );
    for shape in head_range_search::SUPPORTED_SHAPES {
        let abnormal = head_seed_search::ALL_STEM_SHAPES.contains(shape);
        let mut head = template.clone();
        head["kind"] = json!("HeadInter");
        head["shape"] = json!(shape);
        head["abnormal"] = json!(!abnormal);
        let mut graph = Graph::default();
        let input = json!({"head_glyph_retrieval_complete":true,"live_seed_conflicts_complete":true,"systems":[{"system_id":1,"staffs":[{"staff_id":head["staff_id"],"seed_heads":[head],"scanners":[],"ledger_scanners":[]}]}]});
        let report = head_registration::analyze(&input, &mut graph, 11).unwrap();
        assert_eq!(report["retained_heads"], 1);
        assert_eq!(
            report["systems"][0]["heads"][0]["abnormal"], abnormal,
            "{shape}"
        );
        assert_eq!(
            report["shared_sig"]["inters"][0]["abnormal"], abnormal,
            "{shape}"
        );
        assert_eq!(report["heads_complete"], false);
    }
}

#[test]
fn selected_catalog_preflight_preserves_veto_transaction_and_legacy_api() {
    let image = GrayImage {
        width: 128,
        height: 96,
        pixels: vec![255; 128 * 96],
    };
    let table = template_probes::distances(&image, None).unwrap();
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let old = Bank::load_builtin().unwrap();
    let mut calls = 0;
    for shapes in [
        vec!["BREVE_SMALL"],
        vec!["NOTEHEAD_VOID_SMALL", "NOTEHEAD_VOID_SMALL"],
        vec!["NOTEHEAD_VOID_SMALL", "UNKNOWN"],
    ] {
        assert!(
            head_seed_search::search_all_heads_with_veto(
                &table,
                (&bank, 51),
                [40, 45],
                (&[0, 1], &[0, -1]),
                "LEFT_STEM",
                &shapes,
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .is_err()
        );
    }
    assert!(
        head_seed_search::search_selected_with_veto(
            &table,
            (&bank, 51),
            [40, 45],
            (&[0, 1], &[0, -1]),
            "LEFT_STEM",
            &["NOTEHEAD_VOID_SMALL"],
            &mut |_| {
                calls += 1;
                Ok(false)
            }
        )
        .is_err()
    );
    for shapes in [
        vec!["NOTEHEAD_VOID", "NOTEHEAD_VOID_SMALL"],
        vec!["NOTEHEAD_VOID", "UNKNOWN"],
        vec!["BREVE_SMALL", "BREVE_SMALL"],
    ] {
        assert!(
            head_range_search::candidates_for_shapes(
                &table,
                (&old, 51),
                [40, 45],
                &[0, -1],
                &shapes,
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .is_err()
        );
    }
    assert!(
        head_seed_search::search_all_heads_with_veto(
            &table,
            (&old, 51),
            [40, 45],
            (&[0, 1], &[0, -1]),
            "RIGHT_STEM",
            &["NOTEHEAD_VOID", "NOTEHEAD_VOID_SMALL"],
            &mut |_| {
                calls += 1;
                Ok(false)
            }
        )
        .is_err()
    );
    for size in [0, 31, 33, 50, 52, 65, u16::MAX] {
        assert!(
            head_range_search::candidates_for_shapes(
                &table,
                (&bank, size),
                [40, 45],
                &[0],
                &["BREVE_SMALL"],
                &mut |_| {
                    calls += 1;
                    Ok(false)
                }
            )
            .is_err()
        );
    }
    assert_eq!(calls, 0);
    assert!(
        head_seed_search::search_all_heads_with_veto(
            &table,
            (&bank, 51),
            [40, 45],
            (&[0], &[0]),
            "RIGHT_STEM",
            &[],
            &mut |_| {
                calls += 1;
                Ok(false)
            }
        )
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        head_range_search::candidates_for_shapes(
            &table,
            (&bank, 51),
            [40, 45],
            &[0],
            &[],
            &mut |_| {
                calls += 1;
                Ok(false)
            }
        )
        .unwrap()["matches"],
        json!([])
    );
    assert_eq!(calls, 0);
    assert!(Bank::load_all_head_coverage("UNKNOWN").is_err());
    for score in [f64::NAN, f64::INFINITY, -0.01] {
        assert!(head_range_search::candidate_decision_for_shape("BREVE_SMALL", score).is_err());
    }
    assert!(head_range_search::candidate_decision("BREVE_SMALL", 0.).is_err());
}
