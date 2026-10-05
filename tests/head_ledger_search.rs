// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, beam_dispatch,
    head_ledger_search::{self, LedgerLine, Search, StaffLedgers},
    template_probes, vertical,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn compare(a: &Value, b: &Value, p: &str) {
    match b {
        Value::Object(v) => {
            for (k, b) in v {
                if k != "id" {
                    compare(&a[k], b, &format!("{p}.{k}"));
                }
            }
        }
        Value::Array(v) => {
            let aa = a.as_array().unwrap();
            assert_eq!(aa.len(), v.len(), "{p}");
            for (i, (a, b)) in aa.iter().zip(v).enumerate() {
                compare(a, b, &format!("{p}[{i}]"));
            }
        }
        Value::Number(v) => {
            assert!(
                (a.as_f64().unwrap() - v.as_f64().unwrap()).abs() < 1e-10,
                "{p}: {a} != {b}"
            );
        }
        _ => assert_eq!(a, b, "{p}"),
    }
}
#[test]
fn original_real_laura_ledger_scanners_references_and_seed_proposals_match() {
    let b: Value =
        serde_json::from_str(include_str!("../fixtures/official-head-ledger-search.json")).unwrap();
    assert_eq!(b["native_outputs_used"], false);
    let raster =
        GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap();
    let binary = vertical::binarize(&raster, |_, _, _| {});
    let mask = GrayImage::decode(include_bytes!("../fixtures/official-heads-mask.png")).unwrap();
    let table = template_probes::distances(&binary, Some(&mask)).unwrap();
    let bank = Bank::load_builtin().unwrap();
    let xo = audiveris_native::head_seed_search::x_offsets(5).unwrap();
    let (mut pairs, mut proposals, mut scanners, mut ordinates) = (0, 0, 0, 0);
    for sys in b["systems"].as_array().unwrap() {
        for f in sys["staffs"].as_array().unwrap() {
            let input: StaffLedgers = serde_json::from_value(f.clone()).unwrap();
            let seeds = f["all_seeds"]
                .as_array()
                .unwrap()
                .iter()
                .map(beam_dispatch::glyph_from_json)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let actual = head_ledger_search::scan(
                &input,
                &Search {
                    main_interline: 11,
                    table: &table,
                    bank: &bank,
                    seeds: &seeds,
                    x_offsets: &xo,
                },
            )
            .unwrap();
            compare(
                &actual["ledger_scanners"],
                &f["ledger_scanners"],
                "ledger_scanners",
            );
            pairs += actual["seed_pitch_pairs"].as_u64().unwrap();
            proposals += actual["proposal_count"].as_u64().unwrap();
            scanners += f["ledger_scanners"].as_array().unwrap().len();
            for c in f["ledger_scanners"].as_array().unwrap() {
                let main = serde_json::from_value(c["main_line"].clone()).unwrap();
                let further =
                    serde_json::from_value::<Vec<[f64; 4]>>(c["further_lines"].clone()).unwrap();
                for y in c["ordinate_controls"].as_array().unwrap() {
                    assert_eq!(
                        head_ledger_search::ordinate(
                            main,
                            &further,
                            c["pitch"].as_i64().unwrap() as i32,
                            input.interline,
                            y["x"].as_i64().unwrap() as i32
                        )
                        .unwrap(),
                        y["y"].as_i64().unwrap() as i32
                    );
                    ordinates += 1;
                }
            }
        }
    }
    assert!(pairs > 0 && proposals > 0 && scanners > 0 && ordinates > 0);
    println!(
        "ledger seed search: {scanners} scanners, {pairs} pairs, {proposals} raw proposals, {ordinates} original ordinate controls"
    );
}
#[test]
fn signed_contiguous_levels_and_merged_limits_preserve_original_traversal() {
    let page = GrayImage::new(50, 50, vec![255; 2500]).unwrap();
    let table = template_probes::distances(&page, None).unwrap();
    let bank = Bank::load_builtin().unwrap();
    let search = Search {
        main_interline: 11,
        table: &table,
        bank: &bank,
        seeds: &[],
        x_offsets: &[0],
    };
    let mut input = StaffLedgers {
        staff_id: 1,
        interline: 11,
        merged_first: false,
        merged_last: false,
        ledger_lines: vec![
            LedgerLine {
                index: -1,
                line: [5., 20., 30., 20.],
            },
            LedgerLine {
                index: -3,
                line: [5., 0., 30., 0.],
            },
            LedgerLine {
                index: 1,
                line: [5., 40., 30., 40.],
            },
        ],
    };
    let a = head_ledger_search::scan(&input, &search).unwrap();
    assert_eq!(
        a["ledger_scanners"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["pitch"].clone())
            .collect::<Vec<_>>(),
        vec![json!(-6), json!(-7), json!(6), json!(7)]
    );
    input.merged_first = true;
    input.merged_last = true;
    let a = head_ledger_search::scan(&input, &search).unwrap();
    assert_eq!(
        a["ledger_scanners"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["pitch"].clone())
            .collect::<Vec<_>>(),
        vec![json!(-6), json!(6)]
    );
    input.ledger_lines[0].line[2] = f64::NAN;
    assert!(head_ledger_search::scan(&input, &search).is_err());
    assert!(head_ledger_search::ordinate([0., 1., 0., 1.], &[], 7, 11, 10).is_err());
}
