// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::GrayImage;
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
#[test]
fn original_horizontal_header_erasure_pixels_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let name = "horizontal-rendered-clefs";
    let expected: Value = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/official-horizontal-beam-header.json")).unwrap(),
    )
    .unwrap();
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
    let mut shared = audiveris_native::shared_sig::Graph::default();
    shared.import_grid(&grid).unwrap();
    let before = shared.snapshot();
    let headers = header_preparation::from_grid(&grid).unwrap();
    let candidates =
        clef_candidates::analyze(&residual, &grid, &headers, sc.interline.main).unwrap();
    let registered =
        clef_registration::analyze(&candidates, &grid, &headers, sc.interline.main).unwrap();

    let key =
        audiveris_native::key_column::analyze(&residual, &grid, &registered, sc.interline.main)
            .unwrap();
    let time = audiveris_native::time_column::analyze(
        &residual,
        &grid,
        &registered,
        &key,
        sc.interline.main,
    )
    .unwrap();
    let headers =
        audiveris_native::header_finalization::analyze(&grid, &time, sc.interline.main).unwrap();
    assert_eq!(headers["headers_complete"], true);
    let seeds =
        audiveris_native::stem_seeds::analyze(&residual, &grid, &headers, &sc, slope).unwrap();

    let (_prepared, got) =
        audiveris_native::beam_preparation::analyze(&residual, &grid, &headers, &seeds, &sc)
            .unwrap();
    assert_eq!(
        got["header_erase_rectangles"],
        expected["header_erase_rectangles"]
    );
    for key in ["no_staff", "gaussian", "header_erased"] {
        for field in ["width", "height", "black_count", "sha256"] {
            assert_eq!(
                got["masks"][key][field], expected[key][field],
                "{key}.{field}"
            );
        }
    }
    assert_ne!(
        expected["gaussian"]["sha256"],
        expected["header_erased"]["sha256"]
    );
    assert!(
        expected["header_erased"]["black_count"].as_u64().unwrap()
            < expected["gaussian"]["black_count"].as_u64().unwrap()
    );
    assert_eq!(shared.snapshot(), before);
}
#[test]
fn original_system_area_header_rectangles_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture: Value = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/official-beam-header-areas.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    for config in fixture["configurations"].as_array().unwrap() {
        let input = &config["input"];
        let mut grid = input["grid"].clone();
        let w = input["width"].as_u64().unwrap() as usize;
        let h = input["height"].as_u64().unwrap() as usize;
        let ends =
            audiveris_native::system_area_ends::discover(grid["systems"].as_array().unwrap(), w)
                .unwrap();
        for (s, e) in grid["systems"].as_array_mut().unwrap().iter_mut().zip(ends) {
            s["area"] = serde_json::json!({"x_range":e["x_range"]});
        }
        let image = GrayImage::new(w, h, vec![255; w * h]).unwrap();
        let headers = serde_json::json!({"headers_complete":true,"staffs":config["headers"]});
        let (_, got) =
            audiveris_native::beam_preparation::header_rectangles(&image, &grid, &headers, 11)
                .unwrap();
        assert_eq!(got, config["header_erase_rectangles"], "{}", config["name"]);
        let mut malformed = grid.clone();
        malformed["systems"][0]["area"]["x_range"] = serde_json::json!([1, w]);
        assert!(
            audiveris_native::beam_preparation::header_rectangles(&image, &malformed, &headers, 11)
                .is_err()
        );
    }
}
#[test]
fn certified_curved_neighbor_clip_bounds_enable_header_erasure() {
    let mut fixture: Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-header-areas.json")).unwrap();
    let config = fixture["configurations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "curved-clipped-staff-chain")
        .unwrap();
    let mut grid = config["input"]["grid"].clone();
    let mut staff = grid["staffs"][0].clone();
    staff["id"] = serde_json::json!(4);
    staff["system_id"] = serde_json::json!(3);
    for line in staff["lines"].as_array_mut().unwrap() {
        for point in line["points"].as_array_mut().unwrap() {
            point[1] = serde_json::json!(point[1].as_f64().unwrap() + 280.);
        }
    }
    grid["staffs"].as_array_mut().unwrap().push(staff);
    grid["systems"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"id":3,"staff_ids":[4],"coordinates":[20,330,121,394,0]}));
    let ends =
        audiveris_native::system_area_ends::discover(grid["systems"].as_array().unwrap(), 320)
            .unwrap();
    for (s, e) in grid["systems"].as_array_mut().unwrap().iter_mut().zip(ends) {
        s["area"] = serde_json::json!({"x_range":e["x_range"]});
    }
    let mut headers = serde_json::json!({"headers_complete":true,"staffs":config["headers"]});
    headers["staffs"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"staff_id":4,"system_id":3,"stop":50}));
    let image = GrayImage::new(320, 400, vec![255; 128000]).unwrap();
    let (_, rectangles) =
        audiveris_native::beam_preparation::header_rectangles(&image, &grid, &headers, 11).unwrap();
    assert_eq!(rectangles.as_array().unwrap().len(), 3);
}
