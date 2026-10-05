// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, curves::Curve, head_seed_search, head_seed_veto::Context, template_probes, vertical,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
fn golden() -> Value {
    serde_json::from_str(include_str!("../fixtures/official-head-veto.json")).unwrap()
}
fn comparable(v: &Value) -> Value {
    let mut v = v.clone();
    for k in [
        "id",
        "members",
        "origin",
        "glyph",
        "glyph_id",
        "glyph_id_namespace",
        "contextual_grade",
        "staff_id",
        "source_peak",
    ] {
        v.as_object_mut().unwrap().remove(k);
    }
    v
}
fn compare_inters(a: &Value, b: &Value) {
    let mut a: Vec<Value> = a.as_array().unwrap().iter().map(comparable).collect();
    let mut b: Vec<Value> = b.as_array().unwrap().iter().map(comparable).collect();
    a.sort_by_key(Value::to_string);
    b.sort_by_key(Value::to_string);
    assert_eq!(a, b);
}
fn compare_matches(a: &[Value], e: &Value) {
    let e = e.as_array().unwrap();
    assert_eq!(a.len(), e.len());
    for (a, b) in a.iter().zip(e) {
        for k in ["shape", "source_shape", "anchor", "x", "y", "bounds"] {
            assert_eq!(a[k], b[k], "{k}: {a} != {b}");
        }
        for k in ["score", "grade"] {
            assert!((a[k].as_f64().unwrap() - b[k].as_f64().unwrap()).abs() < 1e-12);
        }
    }
}
#[test]
fn original_real_score_competitor_selection_slices_and_eval_veto() {
    let g = golden();
    assert_eq!(g["native_outputs_used"], false);
    let raw: Value =
        serde_json::from_str(include_str!("../fixtures/official-head-seed-search.json")).unwrap();
    let grid: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/laura-gray-final.json"
    ))
    .unwrap();
    let page =
        GrayImage::decode(include_bytes!("../fixtures/official-grid/laura-gray.png")).unwrap();
    let binary = vertical::binarize(&page, |_, _, _| {});
    let mask = GrayImage::decode(include_bytes!("../fixtures/official-heads-mask.png")).unwrap();
    let table = template_probes::distances(&binary, Some(&mask)).unwrap();
    let bank = Bank::load_builtin().unwrap();
    let (mut scanners, mut pairs, mut matches, mut staff_matches, mut changed) = (0, 0, 0, 0, 0);
    for sys in g["systems"].as_array().unwrap() {
        let context =
            Context::from_inters(sys["prolog"]["all_inters"].as_array().unwrap(), 11, 5).unwrap();
        let report = context.report();
        compare_inters(&report["bars"], &sys["prolog"]["bars"]);
        compare_inters(&report["competitors"], &sys["prolog"]["competitors"]);
        for f in sys["staffs"].as_array().unwrap() {
            let staff = grid["staffs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == f["staff_id"])
                .unwrap();
            for key in ["scanners", "ledger_scanners"] {
                for c in f[key].as_array().unwrap() {
                    let points = if key == "scanners" {
                        serde_json::from_value(
                            staff["lines"][c["line_index"].as_u64().unwrap() as usize]["points"]
                                .clone(),
                        )
                        .unwrap()
                    } else {
                        let l: Vec<f64> = serde_json::from_value(c["main_line"].clone()).unwrap();
                        vec![[l[0], l[1]], [l[2], l[3]]]
                    };
                    let line = Curve::interpolate(points).unwrap();
                    let slice = context
                        .slice(&line, 11, c["dir"].as_i64().unwrap() as i32)
                        .unwrap();
                    let sr = slice.report();
                    compare_inters(&sr["competitors"], &c["slice"]["competitors"]);
                    let mut ab: Vec<Value> = sr["bars"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v["bounds"].clone())
                        .collect();
                    let mut eb: Vec<Value> = c["slice"]["bars"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v["bounds"].clone())
                        .collect();
                    ab.sort_by_key(Value::to_string);
                    eb.sort_by_key(Value::to_string);
                    assert_eq!(ab, eb);
                    let xo: Vec<i32> = serde_json::from_value(c["x_offsets"].clone()).unwrap();
                    let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone()).unwrap();
                    for z in c["seeds"].as_array().unwrap() {
                        let mut a = Vec::new();
                        for an in ["LEFT_STEM", "RIGHT_STEM"] {
                            a.extend(
                                head_seed_search::search_with_veto(
                                    &table,
                                    &bank,
                                    [
                                        z["x0"].as_i64().unwrap() as i32,
                                        z["y0"].as_i64().unwrap() as i32,
                                    ],
                                    (&xo, &yo),
                                    an,
                                    &mut |b| slice.veto(b),
                                )
                                .unwrap(),
                            );
                        }
                        compare_matches(&a, &z["matches"]);
                        pairs += 1;
                        matches += a.len();
                        if key == "scanners" {
                            staff_matches += a.len();
                        }
                    }
                    if key == "scanners" {
                        let old_sys = raw["systems"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|s| s["system_id"] == sys["system_id"])
                            .unwrap();
                        let old_staff = old_sys["staffs"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|s| s["staff_id"] == f["staff_id"])
                            .unwrap();
                        let old = old_staff["scanners"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|s| s["pitch"] == c["pitch"])
                            .unwrap();
                        changed += old["seeds"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .zip(c["seeds"].as_array().unwrap())
                            .filter(|(a, b)| a["matches"] != b["matches"])
                            .count();
                    }
                    scanners += 1;
                }
            }
        }
    }
    assert_eq!(g["control_slice"], json!({"bars":[],"competitors":[]}));
    let empty = Context::from_inters(&[], 11, 5).unwrap();
    let line = Curve::interpolate(vec![[0., 0.], [2000., 0.]]).unwrap();
    let slice = empty.slice(&line, 11, 0).unwrap();
    for c in g["controls"].as_array().unwrap() {
        let xo: Vec<i32> = serde_json::from_value(c["x_offsets"].clone()).unwrap();
        let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone()).unwrap();
        let a = head_seed_search::search_with_veto(
            &table,
            &bank,
            [
                c["x0"].as_i64().unwrap() as i32,
                c["y0"].as_i64().unwrap() as i32,
            ],
            (&xo, &yo),
            c["anchor"].as_str().unwrap(),
            &mut |b| slice.veto(b),
        )
        .unwrap();
        compare_matches(&a, &c["matches"]);
    }
    assert_eq!(
        (scanners, pairs, matches, staff_matches),
        (118, 1137, 358, 328)
    );
    assert!(changed > 0);
    assert!(g["eval_counts"]["central_veto"].as_u64().unwrap() > 0);
    assert!(g["eval_counts"]["noncentral_veto"].as_u64().unwrap() > 0);
    println!(
        "{scanners} original scanners, {pairs} pairs, {matches} gated proposals; {changed} seed records changed; original eval counts {}",
        g["eval_counts"]
    );
}
#[test]
fn veto_uses_positive_area_original_grade_and_group_long_beam() {
    let line = Curve::interpolate(vec![[0., 10.], [50., 10.]]).unwrap();
    let thin = json!({"id":1,"kind":"BarlineInter","shape":"THIN_BARLINE","bounds":[20,0,2,30],"median":[21.,0.,21.,30.],"width":2.,"grade":0.1,"contextual_grade":1.,"frozen":true});
    let hook = json!({"id":2,"kind":"BeamHookInter","shape":"BEAM_HOOK","bounds":[0,9,5,2],"median":[0.,10.,5.,10.],"height":2.,"grade":0.35});
    let long = json!({"id":3,"kind":"BeamInter","shape":"BEAM","bounds":[5,9,28,2],"median":[5.,10.,33.,10.],"height":2.,"grade":0.1});
    let group = json!({"id":4,"kind":"BeamGroupInter","members":[2,3]});
    let context = Context::from_inters(&[thin, hook, long, group], 11, 5).unwrap();
    let report = context.report();
    assert_eq!(report["bars"].as_array().unwrap().len(), 1);
    assert_eq!(report["competitors"].as_array().unwrap().len(), 1);
    assert_eq!(report["competitors"][0]["id"], 2);
    let slice = context.slice(&line, 11, 0).unwrap();
    assert!(slice.veto([20, 10, 1, 1]).unwrap());
    assert!(!slice.veto([22, 10, 1, 1]).unwrap());
    assert!(slice.veto([0, 9, 1, 1]).unwrap());
    assert!(!slice.veto([5, 9, 1, 1]).unwrap());
}
#[test]
fn central_veto_abandons_each_source_shape_before_other_offsets() {
    let page = GrayImage::new(100, 100, vec![0; 10000]).unwrap();
    let table = template_probes::distances(&page, None).unwrap();
    let bank = Bank::load_builtin().unwrap();
    let mut calls = 0;
    let out = head_seed_search::search_with_veto(
        &table,
        &bank,
        [40, 40],
        (&[0, 1, -1], &[0, -1]),
        "LEFT_STEM",
        &mut |_| {
            calls += 1;
            Ok(true)
        },
    )
    .unwrap();
    assert!(out.is_empty());
    assert_eq!(calls, 2);
}
