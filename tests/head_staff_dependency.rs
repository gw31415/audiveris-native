// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    curves::Curve, head_candidate_filter, head_glyph_retrieval as glyph, shared_sig::Graph,
};
use serde_json::{Value, json};
fn declared(r: &Value) -> Value {
    let mut r = r.clone();
    r["kind"] = json!("HeadInter");
    r["staff_id"] = json!(r["staff"].as_u64().unwrap() + 1);
    r
}
#[test]
fn original_eight_previous_staff_retirement_and_next_staff_seed_conflicts() {
    let g: Value = serde_json::from_str(include_str!(
        "../fixtures/official-head-staff-dependency/controls.json"
    ))
    .unwrap();
    assert_eq!(g["native_outputs_used"], false);
    assert_eq!(g["controls"].as_array().unwrap().len(), 8);
    for c in g["controls"].as_array().unwrap() {
        let seed = c["seed_inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(declared)
            .collect::<Vec<_>>();
        let input = json!({"seed_heads":seed,"scanners":[],"ledger_scanners":[]});
        let before = input.clone();
        let retired = glyph::retire_staff_seeds(&input).unwrap();
        assert_eq!(input, before);
        for (a, e) in retired
            .iter()
            .zip(c["seed_after_previous_staff"].as_array().unwrap())
        {
            for key in [
                "removed", "shape", "bounds", "grade", "pitch", "staff_id", "left_dx", "right_dx",
            ] {
                assert_eq!(a[key], e[key], "mode {} field {key}", c["mode"]);
            }
        }
        let line = Curve::interpolate(serde_json::from_value(c["line"].clone()).unwrap()).unwrap();
        let slice = glyph::live_seed_slice(&retired, &line, 11, 0).unwrap();
        assert_eq!(
            json!(
                slice
                    .iter()
                    .map(|h| h["id"].as_u64().unwrap() - 1)
                    .collect::<Vec<_>>()
            ),
            c["slice_indices"]
        );
        let accepted =
            head_candidate_filter::filter_seed_indices(&[declared(&c["proposal"])], &slice)
                .unwrap();
        assert_eq!(
            !accepted.is_empty(),
            c["accepted"].as_bool().unwrap(),
            "mode {}",
            c["mode"]
        );
        if c["mode"] == 2 {
            let stale = input["seed_heads"].as_array().unwrap();
            assert!(
                head_candidate_filter::filter_seed_indices(&[declared(&c["proposal"])], stale)
                    .unwrap()
                    .is_empty(),
                "the old unretired competitor incorrectly vetoes this range head"
            );
        }
    }
}
#[test]
fn removed_seed_skipping_and_upstream_phase_refresh_reject_without_mutation() {
    let line = Curve::interpolate(vec![[0., 55.], [100., 55.]]).unwrap();
    let seeds = vec![json!({"removed":true,"bounds":null})];
    assert!(
        glyph::live_seed_slice(&seeds, &line, 11, 0)
            .unwrap()
            .is_empty()
    );
    assert!(
        glyph::live_seed_slice(&[], &line, 11, 0)
            .unwrap()
            .is_empty()
    );
    assert!(glyph::live_seed_slice(&[], &line, 0, 0).is_err());
    assert!(
        glyph::live_seed_slice(
            &[json!({"kind":"HeadInter","grade":0.7,"bounds":[0,0,-1,10]})],
            &line,
            11,
            0
        )
        .is_err()
    );
    let grid = json!({"bar_semantics":{"systems":[{"id":1,"inters":[{"id":1,"kind":"BarlineInter"}],"relations":[]}]}});
    let mut graph = Graph::default();
    graph.import_grid(&grid).unwrap();
    graph
        .sync_heads(1, &[json!({"id":1,"kind":"HeadInter"})], &[], &[])
        .unwrap();
    let before = graph.snapshot();
    assert!(graph.import_grid(&grid).is_err());
    assert_eq!(before, graph.snapshot());
    assert!(graph.sync_ledgers(1, &[]).is_err());
    assert_eq!(before, graph.snapshot());
    let state = audiveris_native::beam_candidates::SystemCandidates::empty(1);
    assert!(graph.sync_candidates(&state).is_err());
    assert_eq!(before, graph.snapshot());
    let groups = audiveris_native::beam_groups::GroupPass {
        ordered_ids: vec![],
        groups: vec![],
        supports: vec![],
    };
    assert!(
        graph
            .sync_system(
                &state,
                &groups,
                &audiveris_native::multiple_rest_registration::RestGraph::default()
            )
            .is_err()
    );
    assert_eq!(before, graph.snapshot());
}

use audiveris_native::{
    GrayImage,
    beam_dispatch::Registry,
    head_glyph_retrieval::{self, Input},
    vertical,
};
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
fn original_118_physical_scanners_with_staff_retirement() {
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
    let r = head_glyph_retrieval::analyze_for_registration(
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
