// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, key_slices};
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
fn fields(a: &Value, b: &Value) {
    match b {
        Value::Object(m) => {
            for (k, v) in m {
                fields(&a[k], v);
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                fields(x, y)
            }
        }
        _ => assert_eq!(a, b),
    }
}
#[test]
fn all_original_rasters() {
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
        "rendered-header-one-sharp",
        "rendered-header-three-sharps",
        "rendered-header-one-flat",
        "rendered-header-four-flats",
        "rendered-header-empty",
        "rendered-header-noise",
    ] {
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
        let actual = key_slices::analyze(&residual, &grid, &registered, sc.interline.main).unwrap();
        let expected: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-key-slicing.json")))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            actual["staffs"].as_array().unwrap().len(),
            expected["staffs"].as_array().unwrap().len()
        );
        assert_eq!(actual["key_recognition_complete"], false);
        assert_eq!(actual["headers_complete"], false);
        for (actual, oracle) in actual["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["staffs"].as_array().unwrap())
        {
            assert_eq!(actual["staff_id"], oracle["staff_id"]);
            assert_eq!(actual["shapes"].as_array().unwrap().len(), 2);
            assert_eq!(oracle["shapes"].as_array().unwrap().len(), 2);
            for (a, b) in actual["shapes"]
                .as_array()
                .unwrap()
                .iter()
                .zip(oracle["shapes"].as_array().unwrap())
            {
                assert_eq!(a["shape"], b["shape"]);
                assert_eq!(a["signature"], b["refined_signature"]);
                assert_eq!(a["checkpoints"][4]["signature"], b["inferred_signature"]);
                fields(&a["range"], &b["final_state"]["range"]);
                fields(&a["peaks"], &b["final_state"]["peaks"]);
                fields(&a["starts"], &b["starts"]);
                fields(&a["slices"], &b["slices"]);
                let native_checkpoints = a["checkpoints"].as_array().unwrap();
                let official_checkpoints = b["checkpoints"].as_array().unwrap();
                assert_eq!(native_checkpoints.len(), 7);
                assert_eq!(official_checkpoints.len(), 6);
                for (index, (native, official)) in [
                    ("browse_area", "browseArea"),
                    ("refine_area_start", "refineAreaStart"),
                    ("merge_peaks", "mergePeaks"),
                    ("purge_light_peaks", "purgeLightPeaks"),
                    ("infer_signature", "inferSignature"),
                    ("refine_signature", "refineSignature"),
                ]
                .iter()
                .enumerate()
                {
                    let ac = &native_checkpoints[index];
                    let bc = &official_checkpoints[index];
                    assert_eq!(ac["phase"], *native);
                    assert_eq!(bc["phase"], *official);
                    fields(&ac["range"], &bc["range"]);
                    fields(&ac["peaks"], &bc["peaks"]);
                }
                assert_eq!(native_checkpoints[6]["phase"], "compute_starts");
                assert_eq!(b["final_state"]["phase"], "allocateSlices");
                fields(&native_checkpoints[6]["range"], &b["final_state"]["range"]);
                fields(&native_checkpoints[6]["peaks"], &b["final_state"]["peaks"]);
            }
        }
    }
}
