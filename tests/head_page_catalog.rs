// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    curves::Curve,
    head_candidate_filter,
    head_range_search::{self, Scan},
    head_registration,
    shared_sig::Graph,
    template_probes,
};
use audiveris_template_bank::Bank;
use serde_json::json;

#[test]
fn exact_range_size_is_preflighted_even_for_empty_ranges_before_callbacks() {
    let image = GrayImage {
        width: 16,
        height: 16,
        pixels: vec![255; 256],
    };
    let table = template_probes::distances(&image, None).unwrap();
    let bank = Bank::load_coverage("Leland").unwrap();
    let line = Curve::interpolate(vec![[0., 8.], [16., 8.]]).unwrap();
    let mut ordinates = 0;
    let mut vetoes = 0;
    for size in [31, 32, 40, 51, 64, 65] {
        let result = head_range_search::scan_catalog_at_size(
            Scan {
                table: &table,
                bank: &bank,
                line: &line,
                interline: 7,
                dir: 0,
                range: [1, 0],
                main_interline: 11,
                y_offsets: &[0, -1],
                spots: &[],
            },
            size,
            &mut |_| {
                ordinates += 1;
                Ok(8)
            },
            &mut |_| {
                vetoes += 1;
                Ok(false)
            },
        );
        if [32, 40, 51, 64].contains(&size) {
            let report = result.unwrap();
            assert_eq!(report["matches"], json!([]));
        } else {
            assert!(result.is_err());
        }
    }
    assert_eq!((ordinates, vetoes), (0, 0));
}

#[test]
fn required_catalog_cannot_disappear_at_aggregation_or_sig_transaction() {
    let filter = json!({"range_catalog_candidates_complete":true,"template_catalog_required":true,"systems":[]});
    assert_eq!(
        head_candidate_filter::analyze(&filter, 11).unwrap_err(),
        "missing_head_template_catalog"
    );
    let mut graph = Graph::default();
    graph.import_grid(&json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]}]}})).unwrap();
    let before = graph.snapshot();
    let mut glyphs = json!({"head_glyph_retrieval_complete":true,"live_seed_conflicts_complete":true,"template_catalog_required":true,"systems":[]});
    assert_eq!(
        head_registration::analyze(&glyphs, &mut graph, 11).unwrap_err(),
        "missing_head_template_catalog"
    );
    assert_eq!(graph.snapshot(), before);
    glyphs["template_catalog"] = json!({"family":"unknown","stored_point_size":51});
    assert_eq!(
        head_registration::analyze(&glyphs, &mut graph, 11).unwrap_err(),
        "unsupported_head_template_catalog"
    );
    assert_eq!(graph.snapshot(), before);
}

#[test]
fn stemless_registration_is_normal_and_live_sig_preserves_original_whole_note() {
    let shapes: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-staff-catalog/controls.json"
    ))
    .unwrap();
    assert_eq!(shapes["native_outputs_used"], false);
    let original: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-page-catalog/Leland-40.json"
    ))
    .unwrap();
    let expected = original["systems"][0]["heads"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["shape"] == "WHOLE_NOTE")
        .unwrap();
    assert_eq!(expected["abnormal"], false);
    let mut head = expected.clone();
    head["kind"] = json!("HeadInter");
    let mut graph = Graph::default();
    for (shape, abnormal) in [
        ("WHOLE_NOTE", false),
        ("BREVE", false),
        ("NOTEHEAD_BLACK", true),
        ("NOTEHEAD_VOID", true),
        ("NOTEHEAD_BLACK_SMALL", true),
        ("NOTEHEAD_CROSS", true),
        ("NOTEHEAD_DIAMOND_FILLED", true),
        ("NOTEHEAD_TRIANGLE_DOWN_FILLED", true),
        ("NOTEHEAD_CIRCLE_X", true),
    ] {
        assert_eq!(
            shapes["shape_sets"]["StemHeads"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s == shape),
            abnormal
        );
        head["shape"] = json!(shape);
        // Deliberately supply the opposite incoming state: registration must
        // reproduce HeadInter.added from shape, before any stem edge exists.
        head["abnormal"] = json!(!abnormal);
        let input = json!({"head_glyph_retrieval_complete":true,"live_seed_conflicts_complete":true,
            "systems":[{"system_id":1,"staffs":[{"staff_id":head["staff_id"],
                "seed_heads":[head],"scanners":[],"ledger_scanners":[]}]}]});
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
fn control_cli_rejects_invalid_catalog_and_existing_outputs_before_reading_input() {
    let executable = env!("CARGO_BIN_EXE_audiveris-native");
    for (family, point, tag, expected) in [
        ("unknown", "51", None, "unsupported_head_template_catalog"),
        ("Bravura", "50", None, "unsupported_head_template_catalog"),
        ("Bravura", "51", Some("invalid"), "usage:"),
    ] {
        let mut command = std::process::Command::new(executable);
        command.args([
            "heads-catalog-control",
            "missing-input",
            "missing-output.pgm",
            "missing-report.json",
            family,
            point,
        ]);
        if let Some(tag) = tag {
            command.arg(tag);
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
    let existing = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let original = std::fs::read(&existing).unwrap();
    let output = std::process::Command::new(executable)
        .args(["heads-catalog-control", "missing-input"])
        .arg(&existing)
        .args(["missing-report.json", "Bravura", "51"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("output paths must be distinct and must not exist")
    );
    assert_eq!(std::fs::read(existing).unwrap(), original);
}
