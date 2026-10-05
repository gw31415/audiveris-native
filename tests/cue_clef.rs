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
    json!({"grid_ir":ir,"headers":headers,"clef_candidates":candidates,"clef_registration":registered})
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
fn actual_cue_raster_clef_parts_candidates_and_registration_match_original() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = observe(&root.join("fixtures/official-head-actual-pages/images/Bravura-cue.png"));
    let oracle: Value = serde_json::from_str(include_str!(
        "../fixtures/official-cue-clef/candidates.json"
    ))
    .unwrap();
    let registered: Value = serde_json::from_str(include_str!(
        "../fixtures/official-cue-clef/registration.json"
    ))
    .unwrap();
    near(&out["clef_candidates"]["staffs"], &oracle["staffs"]);
    near(&out["clef_registration"]["staffs"], &registered["staffs"]);
    assert_eq!(
        out["clef_candidates"]["staffs"][0]["outer"],
        json!([84, 130, 47, 73])
    );
    assert_eq!(out["clef_registration"]["staffs"][0]["clef_stop"], 119);
    assert_eq!(
        out["clef_registration"]["staffs"][0]["inters"][0]["bounds"],
        json!([101, 139, 19, 49])
    );
    assert_eq!(out["clef_registration"]["headers_complete"], false);
    let mut graph = audiveris_native::shared_sig::Graph::default();
    graph.import_grid(&out["grid_ir"]).unwrap();
    let before = graph.snapshot();
    let _ = clef_registration::analyze(
        &out["clef_candidates"],
        &out["grid_ir"],
        &out["headers"],
        11,
    )
    .unwrap();
    assert_eq!(graph.snapshot(), before);
}
#[test]
fn explicit_clef_scale_and_sheet_centroid_metric_are_bounded_without_input_mutation() {
    let out = observe(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/official-head-actual-pages/images/Bravura-cue.png"),
    );
    let before = out.clone();
    let metrics = clef_registration::builtin_metrics().unwrap();
    for invalid in [Value::Null, json!(0), json!(129), json!(-1), json!("7")] {
        let mut grid = out["grid_ir"].clone();
        grid["staffs"][0]["specific_interline"] = invalid;
        assert!(
            clef_candidates::browse_rectangles(&grid, &out["headers"], 700, 11)
                .unwrap_err()
                .contains("specific_interline")
        );
        assert!(
            clef_registration::register(
                &out["clef_candidates"],
                &grid,
                &out["headers"],
                11,
                &metrics
            )
            .unwrap_err()
            .contains("specific_interline")
        );
    }
    for bound in [1, 128] {
        let mut grid = out["grid_ir"].clone();
        grid["staffs"][0]["specific_interline"] = json!(bound);
        assert!(clef_candidates::browse_rectangles(&grid, &out["headers"], 1000, 11).is_ok());
    }
    let index = metrics
        .iter()
        .position(|m| m.shape == "G_CLEF" && m.interline == 11)
        .unwrap();
    let mut duplicate = metrics.clone();
    duplicate.push(metrics[index].clone());
    assert!(
        clef_registration::register(
            &out["clef_candidates"],
            &out["grid_ir"],
            &out["headers"],
            11,
            &duplicate
        )
        .unwrap_err()
        .contains("centroid_metric")
    );
    for offset in [f64::NAN, f64::INFINITY, 0.500001] {
        let mut bad = metrics.clone();
        bad[index].centroid_offset_x = offset;
        assert!(
            clef_registration::register(
                &out["clef_candidates"],
                &out["grid_ir"],
                &out["headers"],
                11,
                &bad
            )
            .unwrap_err()
            .contains("centroid_metric")
        );
    }
    assert_eq!(out, before);
}
