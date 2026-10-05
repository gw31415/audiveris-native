// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, bar_sticks, clef_candidates, clusters, filament_merge, grid_ir, header_preparation,
    line_completion, page_glyphs, scale, sections, staff_projection, system_graph, vertical,
};
use serde_json::Value;
fn near(a: &Value, b: &Value) {
    match b {
        Value::Object(map) => {
            for (k, v) in map {
                if k != "id" {
                    near(&a[k], v)
                }
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len(), "{a} != {b}");
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                near(x, y)
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
fn native_clef_parts_and_candidate_maps_match_original_rasters() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in [
        "laura-gray",
        "synthetic-system-bars",
        "cclef-pair",
        "bracket-pair",
        "line-root",
        "rendered-clef-g_clef",
        "rendered-clef-f_clef",
        "rendered-clef-c_clef",
        "rendered-clef-empty",
        "rendered-clef-noise",
        "horizontal-rendered-clefs",
    ] {
        let image = GrayImage::decode(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}.png"))).unwrap(),
        )
        .unwrap();
        let binary = vertical::binarize(&image, |_, _, _| {});
        let sc = scale::analyze(&binary).result.unwrap();
        let prep = sections::prepare(&binary, &sc).unwrap();
        let factory = filament_merge::analyze(image.width, &sc, &prep.horizontal);
        let st = clusters::analyze(&sc, &prep.horizontal, &factory);
        let projection = staff_projection::analyze(&binary, &sc, &prep.horizontal, &st);
        let sticks = bar_sticks::analyze(&binary, &sc, &prep, &projection);
        let slope = factory["line_filtering"]["global_slope"].as_f64().unwrap();
        let systems = system_graph::analyze(&binary, &sc, &st, &sticks, slope);
        let complete = line_completion::analyze(
            &binary,
            &sc,
            &prep,
            &st,
            &projection,
            &sticks,
            &systems,
            slope,
        );
        let (residual, _) =
            page_glyphs::extract(&binary, &complete, sc.interline.main as u32, None).unwrap();
        let grid = grid_ir::build(&residual, &sc, &prep, &complete, &systems, &sticks).unwrap();
        let headers = header_preparation::from_grid(&grid).unwrap();
        let mut shared = audiveris_native::shared_sig::Graph::default();
        shared.import_grid(&grid).unwrap();
        let graph_before = shared.snapshot();
        let actual =
            clef_candidates::analyze(&residual, &grid, &headers, sc.interline.main).unwrap();
        let oracle: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-clef.json"))).unwrap(),
        )
        .unwrap();
        println!("checking {name}");
        near(&actual["staffs"], &oracle["staffs"]);
        assert_eq!(actual["headers_complete"], false);
        let registered = audiveris_native::clef_registration::analyze(
            &actual,
            &grid,
            &headers,
            sc.interline.main,
        )
        .unwrap();
        let official: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!(
                "fixtures/official-grid/{name}-clef-registration.json"
            )))
            .unwrap(),
        )
        .unwrap();
        near(&registered["staffs"], &official["staffs"]);
        assert_eq!(registered["clef_selection_complete"], false);
        assert_eq!(shared.snapshot(), graph_before);
        if name == "horizontal-rendered-clefs" {
            assert_eq!(registered["staffs"].as_array().unwrap().len(), 2);
            assert!(
                registered["staffs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|s| !s["inters"].as_array().unwrap().is_empty())
            );
        }
    }
}

#[test]
fn controlled_original_registration_checks_ties_and_complete_exclusions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle: Value = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/official-grid/clef-registration-controlled.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(oracle["controlled_input"], true);
    let input=oracle["input_candidates"].as_array().unwrap().iter().map(|c|serde_json::json!({"shape":c["shape"],"kind":"TREBLE","grade":c["grade"],"glyph":oracle["input_glyph"]})).collect::<Vec<_>>();
    let sid = oracle["staffs"][0]["staff_id"].clone();
    let candidates =
        serde_json::json!({"staffs":[{"staff_id":sid,"first_best":input,"second_best":[]}]});
    let grid = serde_json::json!({"staffs":[{"id":sid,"system_id":1}]});
    let headers = serde_json::json!({"staff_headers":[{"staff_id":sid,"start":100,"stop":100}]});
    let actual =
        audiveris_native::clef_registration::analyze(&candidates, &grid, &headers, 16).unwrap();
    near(&actual["staffs"], &oracle["staffs"]);
    assert_eq!(
        actual["staffs"][0]["exclusions"].as_array().unwrap().len(),
        3
    );
}

#[test]
fn original_clef_neighbor_browse_rectangle_controls_match() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-clef-neighbor-rects.json"
    ))
    .unwrap();
    assert_eq!(f["native_outputs_used"], false);
    for c in f["configurations"].as_array().unwrap() {
        let input = &c["input"];
        // This historical observer sets specificInterline=11 on every Java
        // staff, including supplied JSON whose geometry metadata says 16.
        // Compare its actual state without rewriting the pinned fixture.
        let mut observed_grid = input["grid"].clone();
        for staff in observed_grid["staffs"].as_array_mut().unwrap() {
            staff["specific_interline"] = serde_json::json!(11);
        }
        for q in c["queries"].as_array().unwrap() {
            let hs=input["grid"]["staffs"].as_array().unwrap().iter().map(|s|serde_json::json!({"staff_id":s["id"],"start":if s["id"]==q["staff_id"]{q["start"].as_i64().unwrap()}else{s["left"].as_i64().unwrap()+1}})).collect::<Vec<_>>();
            let h = serde_json::json!({"staff_headers":hs});
            let rows = clef_candidates::browse_rectangles(
                &observed_grid,
                &h,
                input["height"].as_u64().unwrap() as usize,
                11,
            )
            .unwrap();
            let row = rows
                .iter()
                .find(|s| s["staff_id"] == q["staff_id"])
                .unwrap();
            assert_eq!(row["outer"], q["outer"], "{} {q}", c["name"]);
            assert_eq!(row["inner"], q["inner"]);
        }
    }
    let c = &f["configurations"][0];
    assert!(
        clef_candidates::browse_rectangles(
            &c["input"]["grid"],
            &serde_json::json!({"staff_headers":[]}),
            1000,
            11
        )
        .is_err()
    );
    assert!(
        clef_candidates::browse_rectangles(&c["input"]["grid"], &serde_json::json!({}), 1000, 0)
            .is_err()
    );
}

#[test]
fn clef_browse_extreme_header_start_returns_error() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-clef-neighbor-rects.json"
    ))
    .unwrap();
    let g = &f["configurations"][0]["input"]["grid"];
    let h =
        serde_json::json!({"staff_headers":[{"staff_id":g["staffs"][0]["id"],"start":i64::MIN}]});
    assert!(clef_candidates::browse_rectangles(g, &h, 1000, 11).is_err());
}
