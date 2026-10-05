// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only actual-raster cue CLEF observer; it does not complete HEADERS.
use audiveris_native::{
    GrayImage, bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::{Value, json};
fn observe(path: &std::path::Path) -> Value {
    let image = GrayImage::decode(&std::fs::read(path).unwrap()).unwrap();
    let b = vertical::binarize(&image, |_, _, _| {});
    let sc = scale::analyze(&b).result.unwrap();
    let prep = sections::prepare(&b, &sc).unwrap();
    let f = filament_merge::analyze(b.width, &sc, &prep.horizontal);
    let st = clusters::analyze_two_pass(b.width, &sc, &prep.horizontal, &f);
    let p = staff_projection::analyze(&b, &sc, &prep.horizontal, &st);
    let sticks = bar_sticks::analyze(&b, &sc, &prep, &p);
    let slope = f["line_filtering"]["global_slope"].as_f64().unwrap();
    let sys = system_graph::analyze(&b, &sc, &st, &sticks, slope);
    let comp = line_completion::analyze(&b, &sc, &prep, &st, &p, &sticks, &sys, slope);
    let (residual, _) = page_glyphs::extract(&b, &comp, sc.interline.main as u32, None).unwrap();
    let ir = grid_ir::build(&residual, &sc, &prep, &comp, &sys, &sticks).unwrap();
    let headers = header_preparation::from_grid(&ir).unwrap();
    let candidates = clef_candidates::analyze(&residual, &ir, &headers, sc.interline.main).unwrap();
    let registered =
        clef_registration::analyze(&candidates, &ir, &headers, sc.interline.main).unwrap();
    let key =
        audiveris_native::key_preparation::analyze(&residual, &ir, &registered, sc.interline.main)
            .unwrap();
    let peaks = audiveris_native::key_peaks::analyze(&key, sc.interline.main).unwrap();
    let column =
        audiveris_native::key_column::analyze(&residual, &ir, &registered, sc.interline.main)
            .unwrap();
    let time = audiveris_native::time_column::analyze(
        &residual,
        &ir,
        &registered,
        &column,
        sc.interline.main,
    )
    .unwrap();
    json!({"grid_ir":ir,"headers":headers,"clef_candidates":candidates,"clef_registration":registered,"key_preparation":key,"key_peaks":peaks,"time_column":time})
}
fn near(a: &Value, b: &Value) {
    match b {
        Value::Object(m) => {
            for (k, v) in m {
                if k != "id" {
                    near(&a[k], v);
                }
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                near(x, y);
            }
        }
        Value::Number(_) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-9,
            "{a} != {b}"
        ),
        _ => assert_eq!(a, b),
    }
}
#[test]
fn cue_key_projection_peak_scales_and_observed_time_column_match_original() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = observe(&root.join("fixtures/official-head-actual-pages/images/Bravura-cue.png"));
    let prep: Value = serde_json::from_str(include_str!(
        "../fixtures/official-cue-headers-rest/preparation.json"
    ))
    .unwrap();
    let peaks: Value = serde_json::from_str(include_str!(
        "../fixtures/official-cue-headers-rest/peaks.json"
    ))
    .unwrap();
    let time: Value = serde_json::from_str(include_str!(
        "../fixtures/official-cue-headers-rest/time.json"
    ))
    .unwrap();
    near(&out["key_preparation"]["staffs"], &prep["staffs"]);
    for (a, b) in out["key_peaks"]["staffs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(peaks["staffs"].as_array().unwrap())
    {
        assert_eq!(a["staff_id"], b["staff_id"]);
        near(&a["peaks"], &b["peaks"]);
        for (nk, ok) in [
            ("min_value", "min_value"),
            ("min_derivative", "min_peak_derivative"),
            ("quorum", "min_peak_cumul"),
            ("min_gain_ratio", "min_gain_ratio"),
        ] {
            near(&a["parameters"][nk], &b["params"][ok]);
        }
    }
    for (a, b) in out["time_column"]["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(time["systems"].as_array().unwrap())
    {
        for k in [
            "system_id",
            "clef_offset",
            "key_offset",
            "returned_time_offset",
        ] {
            near(&a[k], &b[k]);
        }
        for stage in ["after_retrieve_time", "after_system_time_stop"] {
            near(&canonical(&a[stage]), &canonical(&b[stage]));
        }
    }
    assert_eq!(out["key_peaks"]["staffs"][0]["parameters"]["quorum"], 9);
    assert_eq!(out["key_preparation"]["staffs"][0]["specific_interline"], 7);
    assert!(
        out["key_preparation"]["staffs"][1]
            .get("specific_interline")
            .is_none()
    );
    assert_eq!(out["time_column"]["headers_complete"], false);
}
#[test]
fn cue_rest_actual_grid_staff_selection_and_pitch_match_original_864_boundary_points() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = observe(&root.join("fixtures/official-head-actual-pages/images/Bravura-cue.png"));
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-cue-headers-rest/rest-mapping.json"
    ))
    .unwrap();
    let mut checked = 0;
    for sys in f["systems"].as_array().unwrap() {
        let mapping = audiveris_native::native_rest_mapping::SystemMapping::from_grid_with_main(
            &out["grid_ir"],
            1280,
            720,
            sys["system_id"].as_u64().unwrap(),
            11,
        )
        .unwrap();
        for (a, b) in mapping.staffs.iter().zip(sys["staffs"].as_array().unwrap()) {
            assert_eq!(a.id, b["id"]);
            assert_eq!(a.interline, b["specific_interline"]);
        }
        for q in sys["queries"].as_array().unwrap() {
            let x = q["x"].as_f64().unwrap();
            let y = q["y"].as_f64().unwrap();
            assert_eq!(
                json!(
                    mapping
                        .closest_staff(x, y)
                        .unwrap()
                        .map(|i| mapping.staffs[i].id)
                ),
                q["closest"]
            );
            assert_eq!(
                json!(
                    mapping
                        .staff_at_or_above(x, y)
                        .unwrap()
                        .map(|i| mapping.staffs[i].id)
                ),
                q["above"]
            );
            for (a, b) in mapping.staffs.iter().zip(q["pitches"].as_array().unwrap()) {
                assert_eq!(a.id, b["staff_id"]);
                assert!((a.pitch_at(x, y).unwrap() - b["pitch"].as_f64().unwrap()).abs() < 1e-9);
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 864);
}
#[test]
fn mixed_staff_adapters_reject_unknown_scales_and_legacy_inference_without_mutation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = observe(&root.join("fixtures/official-head-actual-pages/images/Bravura-cue.png"));
    let before = out.clone();
    let grid = &out["grid_ir"];
    assert_eq!(
        audiveris_native::native_rest_mapping::SystemMapping::from_grid(grid, 1280, 720, 1)
            .err()
            .unwrap(),
        "unsupported_rest_native_mixed_interline"
    );
    for n in [0, 7, 128, 129] {
        assert!(
            audiveris_native::native_rest_mapping::SystemMapping::from_grid_with_main(
                grid, 1280, 720, 1, n
            )
            .is_err()
        );
    }
    for value in [
        Value::Null,
        json!(0),
        json!(129),
        json!(-1),
        json!("7"),
        json!(12),
    ] {
        let mut g = grid.clone();
        g["staffs"][0]["specific_interline"] = value;
        assert!(
            audiveris_native::native_rest_mapping::SystemMapping::from_grid_with_main(
                &g, 1280, 720, 1, 11
            )
            .is_err()
        );
        assert!(
            audiveris_native::key_preparation::staff_inputs(&g, &out["clef_registration"], 11)
                .is_err()
        );
    }
    let mut g = grid.clone();
    g["staffs"][0]["small"] = json!(false);
    assert!(
        audiveris_native::native_rest_mapping::SystemMapping::from_grid_with_main(
            &g, 1280, 720, 1, 11
        )
        .is_err()
    );
    assert!(
        audiveris_native::key_preparation::staff_inputs(&g, &out["clef_registration"], 11).is_err()
    );
    g["staffs"] = json!([]);
    assert!(
        audiveris_native::native_rest_mapping::SystemMapping::from_grid_with_main(
            &g, 1280, 720, 1, 11
        )
        .is_err()
    );
    assert_eq!(out, before);
}
fn identity(n: &Value) -> String {
    serde_json::json!([n["kind"], n["shape"], n["bounds"], n["glyph"]]).to_string()
}
fn canonical(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            if m.contains_key("kind") && m.contains_key("grade") {
                m.remove("id");
            }
            m.remove("links");
            if let Some(a) = m.get_mut("sig_inters").and_then(Value::as_array_mut) {
                a.sort_by_key(identity);
            }
            if m.contains_key("sig_inters") {
                let ids = m["sig_inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| (n["id"].as_u64().unwrap(), identity(n)))
                    .collect::<std::collections::BTreeMap<_, _>>();
                for edge in m.get_mut("relations").unwrap().as_array_mut().unwrap() {
                    for k in ["source", "target"] {
                        edge[k] = json!(ids.get(&edge[k].as_u64().unwrap()).unwrap());
                    }
                }
                m.get_mut("relations")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .sort_by_key(|e| json!([e["kind"], e["source"], e["target"]]).to_string());
            }
            for v in m.values_mut() {
                *v = canonical(v);
            }
            Value::Object(m)
        }
        Value::Array(a) => Value::Array(a.iter().map(canonical).collect()),
        _ => v.clone(),
    }
}
