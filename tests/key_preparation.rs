// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, key_preparation, line_completion, page_glyphs, scale, sections,
    staff_projection, system_graph, vertical,
};
use key_preparation::{Barline, StaffInput, analyze, prepare_staff};
use serde_json::{Value, json};

fn staff() -> StaffInput {
    StaffInput {
        staff_id: 7,
        header_start: 10,
        header_stop: 40,
        clef_stop: Some(30),
        specific_interline: 10,
        lines: (0..5)
            .map(|i| vec![[0., (40 + 10 * i) as f64], [499., (40 + 10 * i) as f64]])
            .collect(),
        barlines: vec![],
    }
}
fn image() -> GrayImage {
    GrayImage::new(500, 200, vec![255; 100_000]).unwrap()
}
fn bar(id: u64, x: i64, grade: f64, connected: bool) -> Barline {
    Barline {
        id,
        bounds: [x, 40, 2, 41],
        intrinsic_grade: grade,
        has_bar_connection: connected,
    }
}

#[test]
fn projects_real_ink_including_measure_columns_before_browse_start() {
    let mut image = image();
    for (x, y) in [
        (10, 20),
        (10, 90),
        (30, 45),
        (31, 40),
        (160, 90),
        (161, 40),
        (31, 19),
        (31, 91),
    ] {
        image.pixels[y * image.width + x] = 0;
    }
    let p = prepare_staff(&image, 10, &staff()).unwrap();
    assert_eq!(p.browse_rect, [31, 20, 130, 71]);
    assert_eq!(p.safe_rect, p.browse_rect);
    assert_eq!(p.projection_start, 10);
    assert_eq!(p.values.len(), 151);
    assert_eq!(p.values[0], 2);
    assert_eq!(p.values[20], 1);
    assert_eq!(p.values[21], 1);
    assert_eq!(p.values[150], 1);
    assert_eq!(p.values.iter().sum::<i32>(), 5);
}

#[test]
fn uses_header_stop_fallback_and_separate_sheet_and_staff_interlines() {
    let mut s = staff();
    s.clef_stop = None;
    let p = prepare_staff(&image(), 16, &s).unwrap();
    assert_eq!(p.browse_start, 41);
    assert_eq!(p.browse_stop, 250);
    assert_eq!(p.browse_rect, [41, 20, 210, 71]);
}

#[test]
fn connected_bar_cutoff_is_inclusive_point_six_and_strictly_after_start() {
    let mut s = staff();
    s.barlines = vec![
        bar(4, 100, 0.6, true),
        bar(2, 50, 0.599999, true),
        bar(3, 70, 1., false),
        bar(1, 31, 1., true),
    ];
    let p = prepare_staff(&image(), 10, &s).unwrap();
    assert_eq!(p.browse_stop, 99);
    assert_eq!(p.stopping_barline, Some(4));
    assert_eq!(p.values.len(), 90);
}

#[test]
fn bar_at_last_browse_column_clips_but_bar_after_it_does_not() {
    let mut s = staff();
    s.barlines.push(bar(1, 161, 1., true));
    assert_eq!(prepare_staff(&image(), 10, &s).unwrap().browse_stop, 160);
    s.barlines[0].bounds[0] = 160;
    assert_eq!(prepare_staff(&image(), 10, &s).unwrap().browse_stop, 159);
}

#[test]
fn barline_iteration_matches_integer_center_order_not_left_order() {
    let mut s = staff();
    let mut wide = bar(1, 60, 1., true);
    wide.bounds[2] = 100;
    s.barlines = vec![wide, bar(2, 80, 1., true)];
    let p = prepare_staff(&image(), 10, &s).unwrap();
    assert_eq!(p.stopping_barline, Some(2));
    assert_eq!(p.browse_stop, 79);
}

#[test]
fn follows_upstream_xmin_only_geometry_and_ties_even_rounding() {
    let mut s = staff();
    s.lines = (0..5)
        .map(|i| vec![[31., 40.5 + 10. * i as f64], [160., 100.5 + 10. * i as f64]])
        .collect();
    let p = prepare_staff(&image(), 10, &s).unwrap();
    assert_eq!(p.browse_rect, [31, 20, 130, 71]);
}

#[test]
fn clamps_pixel_rectangle_without_rewriting_original_browse_range() {
    let mut s = staff();
    s.header_start = 450;
    s.header_stop = 460;
    s.clef_stop = Some(460);
    s.lines = (0..5)
        .map(|i| vec![[0., 5. + 10. * i as f64], [499., 5. + 10. * i as f64]])
        .collect();
    let p = prepare_staff(&image(), 10, &s).unwrap();
    assert_eq!(p.browse_rect, [461, -15, 140, 71]);
    assert_eq!(p.safe_rect, [461, 0, 39, 56]);
    assert_eq!(p.browse_stop, 600);
    assert_eq!(p.values.len(), 50);
    assert!(p.values.iter().all(|&v| v == 0));
}

#[test]
fn rejects_invalid_or_unsupported_inputs_without_a_partial_result() {
    let mut s = staff();
    assert!(prepare_staff(&image(), 0, &s).is_err());
    s.lines.pop();
    assert!(prepare_staff(&image(), 10, &s).is_err());
    s = staff();
    s.lines[0][1][0] = s.lines[0][0][0];
    assert!(prepare_staff(&image(), 10, &s).is_err());
    s = staff();
    s.clef_stop = Some(300);
    assert_eq!(
        prepare_staff(&image(), 10, &s).unwrap_err(),
        "unsupported_empty_key_browse_range"
    );
    let mut im = image();
    im.pixels[0] = 127;
    assert_eq!(
        prepare_staff(&im, 10, &staff()).unwrap_err(),
        "unsupported_key_nonbinary_no_staff"
    );
    s = staff();
    s.barlines.push(bar(1, 100, f64::NAN, true));
    assert_eq!(
        prepare_staff(&image(), 10, &s).unwrap_err(),
        "invalid_key_barline"
    );
}

#[test]
fn adapter_requires_explicit_supported_interline_and_unique_inputs() {
    let s = staff();
    let mut grid = json!({"staffs":[{"id":7,"system_id":1,"specific_interline":10,
        "lines":s.lines.iter().map(|points|json!({"points":points})).collect::<Vec<_>>() }],
        "bar_semantics":{"systems":[{"id":1,"inters":[],"relations":[]}]}});
    let mut registration =
        json!({"staffs":[{"staff_id":7,"header_start":10,"header_stop":40,"clef_stop":30}]});
    let result = analyze(&image(), &grid, &registration, 10).unwrap();
    assert_eq!(result["key_recognition_complete"], false);
    assert_eq!(result["headers_complete"], false);
    grid["staffs"][0]["specific_interline"] = Value::Null;
    assert_eq!(
        analyze(&image(), &grid, &registration, 10).unwrap_err(),
        "unsupported_key_unknown_specific_interline"
    );
    grid["staffs"][0]["specific_interline"] = json!(11);
    assert_eq!(
        analyze(&image(), &grid, &registration, 10).unwrap_err(),
        "unsupported_key_mixed_specific_interline"
    );
    grid["staffs"][0]["specific_interline"] = json!(10);
    let duplicate = registration["staffs"][0].clone();
    registration["staffs"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert_eq!(
        analyze(&image(), &grid, &registration, 10).unwrap_err(),
        "missing_or_duplicate_key_registration"
    );
}

#[test]
fn parser_rejects_oversized_inputs_before_collecting_geometry() {
    let s = staff();
    let base = json!({"staffs":[{"id":7,"system_id":1,"specific_interline":10,
        "lines":s.lines.iter().map(|points|json!({"points":points})).collect::<Vec<_>>() }],
        "bar_semantics":{"systems":[{"id":1,"inters":[],"relations":[]}]}});
    let registration =
        json!({"staffs":[{"staff_id":7,"header_start":10,"header_stop":40,"clef_stop":30}]});
    let mut grid = base.clone();
    grid["staffs"][0]["lines"][0]["points"] = json!(vec![[0, 0]; 4097]);
    assert_eq!(
        key_preparation::staff_inputs(&grid, &registration, 10).unwrap_err(),
        "unsupported_key_curve_budget"
    );
    grid = base.clone();
    grid["staffs"][0]["lines"] = json!(vec![Value::Null; 6]);
    assert_eq!(
        key_preparation::staff_inputs(&grid, &registration, 10).unwrap_err(),
        "unsupported_key_staff_configuration"
    );
    grid = base;
    grid["bar_semantics"]["systems"][0]["relations"] = json!(vec![Value::Null; 65_537]);
    assert_eq!(
        key_preparation::staff_inputs(&grid, &registration, 10).unwrap_err(),
        "unsupported_key_graph_budget"
    );
}

fn exact_fields(actual: &Value, expected: &Value) {
    match expected {
        Value::Object(map) => {
            for (key, value) in map {
                exact_fields(&actual[key], value);
            }
        }
        Value::Array(values) => {
            assert_eq!(actual.as_array().unwrap().len(), values.len());
            for (a, b) in actual.as_array().unwrap().iter().zip(values) {
                exact_fields(a, b);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

#[test]
fn native_image_pipeline_matches_all_ten_independent_key_projections() {
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
        let candidates =
            clef_candidates::analyze(&residual, &grid, &headers, sc.interline.main).unwrap();
        let registered =
            clef_registration::analyze(&candidates, &grid, &headers, sc.interline.main).unwrap();
        let actual = analyze(&residual, &grid, &registered, sc.interline.main).unwrap();
        let expected: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!(
                "fixtures/official-grid/{name}-key-preparation.json"
            )))
            .unwrap(),
        )
        .unwrap();
        println!("checking {name}");
        exact_fields(&actual["staffs"], &expected["staffs"]);
        assert_eq!(actual["key_recognition_complete"], false);
        assert_eq!(actual["headers_complete"], false);
        if name == "synthetic-system-bars" {
            check_independent_controls(&residual);
        }
        let peaks = audiveris_native::key_peaks::analyze(&actual, sc.interline.main).unwrap();
        let peak_oracle: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-key-peaks.json")))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            peaks["staffs"].as_array().unwrap().len(),
            peak_oracle["staffs"].as_array().unwrap().len()
        );
        for (actual, expected) in peaks["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(peak_oracle["staffs"].as_array().unwrap())
        {
            assert_eq!(actual["staff_id"], expected["staff_id"]);
            exact_fields(&actual["peaks"], &expected["peaks"]);
            for (native, official) in [
                ("min_value", "min_value"),
                ("min_derivative", "min_peak_derivative"),
                ("quorum", "min_peak_cumul"),
                ("min_gain_ratio", "min_gain_ratio"),
            ] {
                assert_eq!(actual["parameters"][native], expected["params"][official]);
            }
        }
    }
}

fn check_independent_controls(residual: &GrayImage) {
    let oracle: Value = serde_json::from_slice(include_bytes!(
        "../fixtures/official-grid/key-preparation-controlled.json"
    ))
    .unwrap();
    for c in oracle["browse_stop_cases"].as_array().unwrap() {
        let bar = bar(
            1,
            c["bar_start"].as_i64().unwrap(),
            c["grade"].as_f64().unwrap(),
            c["connected"].as_bool().unwrap(),
        );
        let actual = key_preparation::browse_stop(
            c["x_min"].as_i64().unwrap(),
            c["x_max"].as_i64().unwrap(),
            &[bar],
        )
        .unwrap();
        assert_eq!(actual.0, c["observed_stop"].as_i64().unwrap());
    }
    for c in oracle["projection_cases"].as_array().unwrap() {
        let raw: [i64; 4] = std::array::from_fn(|i| c["raw_rect"][i].as_i64().unwrap());
        let p =
            key_preparation::project_rectangle(residual, c["measure_start"].as_i64().unwrap(), raw)
                .unwrap();
        assert_eq!(json!(p.safe_rect), c["safe_rect"]);
        assert_eq!(json!(p.projection_start), c["x_min"]);
        assert_eq!(
            json!(p.projection_start + p.values.len() as i64 - 1),
            c["x_max"]
        );
        assert_eq!(json!(p.values), c["values"]);
    }
}
