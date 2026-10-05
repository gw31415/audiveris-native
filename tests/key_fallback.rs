// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, key_components};
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
fn near(a: &Value, b: &Value) {
    match b {
        Value::Object(map) => {
            for (k, v) in map {
                near(&a[k], v);
            }
        }
        Value::Array(values) => {
            assert_eq!(a.as_array().unwrap().len(), values.len());
            for (x, y) in a.as_array().unwrap().iter().zip(values) {
                near(x, y);
            }
        }
        Value::Number(_) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10,
            "{a} != {b}"
        ),
        _ => assert_eq!(a, b),
    }
}
#[test]
fn all_original_rasters() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names = std::fs::read_dir(root.join("fixtures/official-grid"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with("-key-fallback.json")
        })
        .map(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .trim_end_matches("-key-fallback.json")
                .to_owned()
        })
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(names.len(), 22);
    for name in names {
        println!("checking {name}");
        let image_path = root.join(format!("fixtures/official-grid/{name}.png"));
        let image = GrayImage::decode(&std::fs::read(image_path).unwrap()).unwrap();
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
        let actual = key_components::analyze_with_empty_slice_fallback(
            &residual,
            &grid,
            &registered,
            sc.interline.main,
        )
        .unwrap();

        let expected: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-key-fallback.json")))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            actual["staffs"].as_array().unwrap().len(),
            expected["staffs"].as_array().unwrap().len()
        );
        for (native, official) in actual["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["staffs"].as_array().unwrap())
        {
            assert_eq!(native["staff_id"], official["staff_id"]);
            assert_eq!(
                native["shapes"].as_array().unwrap().len(),
                official["shapes"].as_array().unwrap().len()
            );
            for (a, b) in native["shapes"]
                .as_array()
                .unwrap()
                .iter()
                .zip(official["shapes"].as_array().unwrap())
            {
                println!(
                    "compare {name} staff {} shape {}",
                    native["staff_id"], a["shape"]
                );
                assert_eq!(a["shape"], b["shape"]);
                let c = &b["components"];
                let target = if c["hard_slice_fallback"].is_object() {
                    &c["hard_slice_fallback"]["slice_assignments_after_fallback"]
                } else {
                    &c["slice_assignments"]
                };
                assert_eq!(
                    a["slices"].as_array().unwrap().len(),
                    target.as_array().unwrap().len()
                );
                for (got, want) in a["slices"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(target.as_array().unwrap())
                {
                    assert_eq!(got["bounds"], want["rect"]);
                    near(&got["alter"], &want["alter"]);
                }
                if let Some(wanted) = c["hard_slice_fallback"]["ordered_attempts"].as_array() {
                    let got = a["fallback_attempts"].as_array().unwrap();
                    assert_eq!(got.len(), wanted.len());
                    for (x, y) in got.iter().zip(wanted) {
                        assert_eq!(x["slice_index"], y["slice_index"]);
                        assert_eq!(
                            x["assigned"].as_bool().unwrap(),
                            !y["after"]["alter"].is_null()
                        );
                        for key in ["raw_sha256", "cropped_sha256", "neighbor_erased_pixels"] {
                            assert_eq!(x[key], y["pixels_before_attempt"][key], "{name} {key}");
                        }
                    }
                } else {
                    assert!(a["fallback_attempts"].as_array().unwrap().is_empty());
                }
            }
        }
        assert_eq!(actual["headers_complete"], false);
        assert_eq!(actual["hard_slice_fallback_complete"], true);
    }
}
