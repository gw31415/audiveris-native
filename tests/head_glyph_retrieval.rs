// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_dispatch::{self, Registry},
    head_glyph_retrieval::{self, Input},
    vertical,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn compare(a: &Value, e: &Value) {
    for k in ["shape", "bounds", "staff_id", "pitch", "glyph"] {
        assert_eq!(a[k], e[k], "{k}: {a} != {e}");
    }
    for k in ["grade", "left_dx", "right_dx"] {
        if e[k].is_null() {
            assert!(a[k].is_null());
        } else {
            assert!(
                (a[k].as_f64().unwrap() - e[k].as_f64().unwrap()).abs() < 1e-12,
                "{k}: {a} != {e}"
            );
        }
    }
}
#[test]
fn original_all_seed_glyphs_live_conflicts_and_range_glyphs() {
    let g: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-glyph-retrieval/laura.json"
    ))
    .unwrap();
    let mut grid: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/laura-gray-final.json"
    ))
    .unwrap();
    let catalog: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-range-catalog/laura.json"
    ))
    .unwrap();
    let mut catalog = catalog;
    catalog["range_catalog_candidates_complete"] = json!(true);
    let mut seeds: Value =
        serde_json::from_str(include_str!("../fixtures/official-head-veto.json")).unwrap();
    seeds["seed_veto_complete"] = json!(true);
    for s in g["systems"].as_array().unwrap() {
        for f in s["staffs"].as_array().unwrap() {
            let gs = grid["staffs"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|x| x["id"] == f["staff_id"])
                .unwrap();
            gs["specific_interline"] =
                f["scanners"][0]["glyph_retrieval"]["specific_interline"].clone();
        }
    }
    let page =
        GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap();
    let binary = vertical::binarize(&page, |_, _, _| {});
    let mask = GrayImage::decode(include_bytes!("../fixtures/official-heads-mask.png")).unwrap();
    let table = audiveris_native::template_probes::distances(&binary, Some(&mask)).unwrap();
    let mut inters = Vec::new();
    for sys in seeds["systems"].as_array().unwrap() {
        for mut v in sys["prolog"]["all_inters"]
            .as_array()
            .unwrap()
            .iter()
            .cloned()
        {
            v["origin"] = json!({"system":sys["system_id"]});
            inters.push(v);
        }
    }
    let shared_sig = json!({"inters":inters});
    let mut registry = Registry::default();
    let r = head_glyph_retrieval::analyze(
        Input {
            binary: &binary,
            grid: &grid,
            seed_veto: &seeds,
            catalog: &catalog,
            main_interline: 11,
            table: &table,
            shared_sig: &shared_sig,
            max_stem: 5,
        },
        &mut registry,
    )
    .unwrap();
    for (s, gs) in r["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(g["systems"].as_array().unwrap())
    {
        for (f, gf) in s["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(gs["staffs"].as_array().unwrap())
        {
            let expected = gf["scanners"][0]["glyph_retrieval"]["new_staff_seeds"]
                .as_array()
                .unwrap();
            let actual = f["seed_heads"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len());
            for (a, e) in actual.iter().zip(expected) {
                compare(a, e);
            }
            for key in ["scanners", "ledger_scanners"] {
                for (c, gc) in f[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(gf[key].as_array().unwrap())
                {
                    let e = &gc["glyph_retrieval"];
                    assert_eq!(c["filtered_indices"], e["filtered_indices"]);
                    let ac = c["seed_competitors"].as_array().unwrap();
                    let ec = e["seed_competitors"].as_array().unwrap();
                    assert_eq!(ac.len(), ec.len());
                    for (a, e) in ac.iter().zip(ec) {
                        compare(a, e);
                    }
                    let ac = c["range_glyphs"].as_array().unwrap();
                    let ec = e["range_glyphs"].as_array().unwrap();
                    assert_eq!(ac.len(), ec.len());
                    for (a, e) in ac.iter().zip(ec) {
                        assert_eq!(a["candidate_index"], e["candidate_index"]);
                        if e["result"].is_null() {
                            assert!(a["result"].is_null());
                        } else {
                            compare(&a["result"], &e["result"]);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(r["seed_heads"], 358);
    assert_eq!(r["range_heads"], 16);
    assert_eq!(r["seed_conflicts"], 438);
    assert_eq!(r["heads_complete"], false);
}
#[test]
fn original_four_shapes_foreground_keypoints_edges_erasure_and_empty() {
    use sha2::{Digest, Sha256};
    let bank = Bank::load_builtin().unwrap();
    let g: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-glyph-retrieval/controls.json"
    ))
    .unwrap();
    assert_eq!(g["native_outputs_used"], false);
    for c in g["controls"].as_array().unwrap() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/official-head-glyph-retrieval")
            .join(c["image"].as_str().unwrap());
        let image = GrayImage::decode(&std::fs::read(p).unwrap()).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&image.pixels)),
            c["binary_sha256"]
        );
        let result = head_glyph_retrieval::retrieve(&image, &bank, &c["input"]).unwrap();
        assert_eq!(
            result
                .as_ref()
                .map(beam_dispatch::glyph_json)
                .unwrap_or(Value::Null),
            c["glyph"]
        );
    }
}
#[test]
fn canonical_registry_reuses_glyphs_and_malformed_retrieval_is_rejected() {
    let bank = Bank::load_builtin().unwrap();
    let image = GrayImage::decode(include_bytes!(
        "../fixtures/official-head-glyph-retrieval/BREVE-clean.png"
    ))
    .unwrap();
    let controls: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-glyph-retrieval/controls.json"
    ))
    .unwrap();
    let input = &controls["controls"][0]["input"];
    let glyph = head_glyph_retrieval::retrieve(&image, &bank, input)
        .unwrap()
        .unwrap();
    let mut registry = Registry::default();
    let id = registry.register(&glyph).unwrap();
    assert_eq!(id, registry.register(&glyph).unwrap());
    assert_eq!(registry.len(), 1);
    let before = registry.snapshot();
    let mut bad = input.clone();
    bad["bounds"] = json!([0, 0, -1, 0]);
    assert!(head_glyph_retrieval::retrieve(&image, &bank, &bad).is_err());
    bad = input.clone();
    bad["shape"] = json!("NOTEHEAD_CROSS");
    assert!(head_glyph_retrieval::retrieve(&image, &bank, &bad).is_err());
    assert_eq!(registry.snapshot(), before);
}

#[test]
fn original_integer_ordinate_and_determinant_seed_pivots() {
    let g: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-glyph-retrieval/controls.json"
    ))
    .unwrap();
    let controls = g["pivot_controls"].as_array().unwrap();
    assert!(controls.len() >= 32);
    for c in controls {
        assert_eq!(c["int_line_overload"], true);
        let bits: [u64; 4] = serde_json::from_value(c["line_bits"].clone()).unwrap();
        let line = bits.map(f64::from_bits);
        let ordinate = f64::from_bits(c["ordinate_bits"].as_u64().unwrap());
        assert_eq!(ordinate, ordinate.round_ties_even());
        assert_eq!(
            head_glyph_retrieval::seed_abscissa(line, ordinate).unwrap(),
            c["x0"].as_i64().unwrap() as i32
        );
    }
    assert!(head_glyph_retrieval::seed_abscissa([0., 0., 10., 0.], 0.).is_err());
    assert!(head_glyph_retrieval::seed_abscissa([0., 0., 0., 10.], f64::NAN).is_err());
}
