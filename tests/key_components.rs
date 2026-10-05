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
    let mut names = vec![
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
        "rendered-header-one-sharp",
        "rendered-header-three-sharps",
        "rendered-header-one-flat",
        "rendered-header-four-flats",
        "rendered-header-empty",
        "rendered-header-noise",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let mut damaged = std::fs::read_dir(root.join("fixtures/official-grid"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("damaged-header-")
                && p.extension().is_some_and(|e| e == "png")
        })
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    damaged.sort();
    names.extend(damaged);
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
        let actual =
            key_components::analyze(&residual, &grid, &registered, sc.interline.main).unwrap();

        let expected: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!(
                "fixtures/official-grid/{name}-{}.json",
                if name.starts_with("damaged-header-") {
                    "key-component-branches"
                } else {
                    "key-components"
                }
            )))
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
                assert_eq!(a["parts"], c["retained_parts"]);
                let candidates = c["candidates_after_overlap_purge"].as_array().unwrap();
                assert_eq!(a["candidates"].as_array().unwrap().len(), candidates.len());
                for (got, want) in a["candidates"].as_array().unwrap().iter().zip(candidates) {
                    assert_eq!(got["glyph"], want["glyph"]);
                    assert_eq!(got["part_indices"], want["part_indices"]);
                    assert_eq!(got["shape"], want["eval"]["shape"]);
                    assert!(
                        (got["eval_grade"].as_f64().unwrap()
                            - want["eval"]["grade"].as_f64().unwrap())
                        .abs()
                            < 1e-12
                    );
                }
                let assigned = c["slice_assignments"].as_array().unwrap();
                assert_eq!(a["slices"].as_array().unwrap().len(), assigned.len());
                for (got, want) in a["slices"].as_array().unwrap().iter().zip(assigned) {
                    assert_eq!(got["bounds"], want["rect"]);
                    near(&got["alter"], &want["alter"]);
                }
            }
        }
        assert_eq!(actual["headers_complete"], false);
    }
}
