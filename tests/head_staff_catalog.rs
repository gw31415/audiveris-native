// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    head_ledger_search,
    head_staff_catalog::{self, Processing, StaffSpec},
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn controls() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/official-head-staff-catalog/controls.json"
    ))
    .unwrap()
}
fn switches(mask: u64) -> Processing {
    Processing {
        small_heads: mask & 1 != 0,
        drum_notation: mask & 2 != 0,
        one_line_staves: mask & 4 != 0,
        cross_heads: mask & 8 != 0,
    }
}
#[test]
fn original_384_staff_shape_order_switch_and_main_specific_scale_cases() {
    let g = controls();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["cases"].as_array().unwrap().len(), 384);
    for c in g["cases"].as_array().unwrap() {
        let spec = StaffSpec {
            line_count: c["line_count"].as_u64().unwrap() as usize,
            tablature: false,
            drum: c["drum"].as_bool().unwrap(),
            specific_interline: c["specific_interline"].as_u64().unwrap() as usize,
            main_interline: c["main_interline"].as_u64().unwrap() as usize,
            stored_point_size: 51,
        };
        let plan =
            head_staff_catalog::plan(spec, switches(c["switch_mask"].as_u64().unwrap())).unwrap();
        assert_eq!(
            plan.point_size,
            Some(c["point_size"].as_u64().unwrap() as u16)
        );
        assert_eq!(plan.ledger_allowed, c["ledger_allowed"]);
        assert_eq!(json!(plan.scanners), c["scanners"], "{c}");
    }
}
#[test]
fn original_75_unmapped_or_boundary_drum_pitches_and_two_tablature_skips() {
    let g = controls();
    assert_eq!(g["outside_pitch_controls"].as_array().unwrap().len(), 75);
    for c in g["outside_pitch_controls"].as_array().unwrap() {
        let catalog = head_staff_catalog::catalog(
            c["line_count"].as_u64().unwrap() as usize,
            c["pitch"].as_i64().unwrap() as i32,
            true,
            switches(15),
        );
        for key in ["all_shapes", "stem_shapes", "hollow_shapes"] {
            assert_eq!(json!(catalog)[key], c[key]);
        }
    }
    for c in g["tablature_controls"].as_array().unwrap() {
        assert_eq!(c["SIG_vertex_delta"], 0);
        let spec = StaffSpec {
            line_count: c["line_count"].as_u64().unwrap() as usize,
            tablature: true,
            drum: false,
            specific_interline: 0,
            main_interline: 0,
            stored_point_size: 0,
        };
        let plan = head_staff_catalog::plan(spec, Processing::default()).unwrap();
        assert!(plan.tablature_skipped);
        assert!(!plan.ledger_allowed);
        assert!(plan.point_size.is_none());
        assert!(plan.scanners.is_empty());
    }
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(
                "../fixtures/official-head-staff-catalog/original-drum-set.xml"
            ))
        ),
        head_staff_catalog::DEFAULT_DRUM_MAP_SHA256
    );
}
#[test]
fn main_scale_reaches_ledger_search_and_unsupported_staff_templates_are_explicit() {
    let g = controls();
    let c = g["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["main_interline"] == 11
                && c["specific_interline"] == 7
                && c["line_count"] == 1
                && c["switch_mask"] == 0
        })
        .unwrap();
    let white = audiveris_native::GrayImage {
        width: 128,
        height: 96,
        pixels: vec![255; 128 * 96],
    };
    let table = audiveris_native::template_probes::distances(&white, None).unwrap();
    let bank = Bank::load_builtin().unwrap();
    let input = head_ledger_search::StaffLedgers {
        staff_id: 1,
        interline: 7,
        merged_first: false,
        merged_last: false,
        ledger_lines: vec![head_ledger_search::LedgerLine {
            index: 1,
            line: [0., 60., 100., 60.],
        }],
    };
    let search = head_ledger_search::Search {
        main_interline: 11,
        table: &table,
        bank: &bank,
        seeds: &[],
        x_offsets: &[0],
    };
    let result = head_ledger_search::scan(&input, &search).unwrap();
    assert_eq!(
        result["ledger_scanners"][1]["y_offsets"],
        c["scanners"][2]["y_offsets"]
    );
    assert_eq!(result["proposal_count"], 0);
    assert!(head_staff_catalog::positions(0, false, 11).is_err());
    assert!(head_staff_catalog::positions(129, false, 11).is_err());
    assert!(head_staff_catalog::positions(5, false, 0).is_err());
    let mut grid: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/laura-gray-final.json"
    ))
    .unwrap();
    grid["staffs"][0]["specific_interline"] = json!(7);
    let result = audiveris_native::head_seed_search::analyze(
        &white,
        &grid,
        &json!({}),
        &json!({}),
        &json!({"music_font":{"name":"Bravura","pointSize":51}}),
        5,
        11,
    );
    assert_eq!(
        result.unwrap_err(),
        "unsupported_head_staff_template_catalog"
    );
}
