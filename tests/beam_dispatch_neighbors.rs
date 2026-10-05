// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::GrayImage;
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
#[test]
fn original_two_column_spots_preserve_pixel_order_and_identity() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let name = "horizontal-beams-double";
    let expected: Value = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/official-horizontal-beam-dispatch.json")).unwrap(),
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

    let geometry = audiveris_native::beam_dispatch::Geometry::from_grid(
        image.width,
        image.height,
        &grid,
        sc.interline.main,
    )
    .unwrap();
    let mut registry = audiveris_native::beam_dispatch::Registry::default();
    let dispatched =
        audiveris_native::beam_dispatch::dispatch(&_prepared.glyphs, &geometry, &mut registry)
            .unwrap();
    assert_eq!(dispatched.len(), 2);
    let mut total = 0;
    for (a, b) in dispatched
        .iter()
        .zip(expected["systems"].as_array().unwrap())
    {
        assert_eq!(a.system_id, b["system_id"].as_u64().unwrap());
        let wanted = b["grouped_spots"].as_array().unwrap();
        assert_eq!(a.glyph_ids.len(), wanted.len());
        assert!(!wanted.is_empty());
        total += wanted.len();
        for (&id, spot) in a.glyph_ids.iter().zip(wanted) {
            assert_eq!(
                audiveris_native::beam_dispatch::glyph_json(&registry.get(id).unwrap().glyph),
                spot["glyph"]
            );
        }
    }
    assert!(total > 0);
    let first = registry.snapshot();
    let again =
        audiveris_native::beam_dispatch::dispatch(&_prepared.glyphs, &geometry, &mut registry)
            .unwrap();
    assert_eq!(first, registry.snapshot());
    assert_eq!(
        serde_json::to_value(dispatched).unwrap(),
        serde_json::to_value(again).unwrap()
    );
    for sys in grid["systems"].as_array().unwrap() {
        let sid = sys["id"].as_u64().unwrap();
        let mapping = audiveris_native::native_rest_mapping::SystemMapping::from_grid(
            &grid,
            image.width,
            image.height,
            sid,
        )
        .unwrap();
        assert_eq!(mapping.staffs.len(), 1);
        assert!(!mapping.sections.is_empty());
        let staff = &grid["staffs"][sid as usize - 1];
        let x = (staff["left"].as_f64().unwrap() + staff["right"].as_f64().unwrap()) / 2.;
        let y = mapping.staffs[0].middle[x as usize] as f64;
        assert_eq!(mapping.closest_staff(x, y).unwrap(), Some(0));
    }
    assert_eq!(shared.snapshot(), before);
}
#[test]
fn original_neighbor_dispatch_selection_and_beam_ends_match() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-dispatch-neighbors.json"
    ))
    .unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    let mut positive = 0;
    let mut absent = 0;
    for config in fixture["configurations"].as_array().unwrap() {
        let input = &config["input"];
        let mut grid = input["grid"].clone();
        grid["schema"] = serde_json::json!("audiveris-native.grid-ir.v1");
        let w = input["width"].as_u64().unwrap() as usize;
        let h = input["height"].as_u64().unwrap() as usize;
        for (system, oracle) in grid["systems"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(config["systems"].as_array().unwrap())
        {
            system["area"] = serde_json::json!({"x_range":oracle["x_range"],"vertical_margin":10.});
            for (field, key) in [
                ("north_neighbor_last_line", "north_staff_ids"),
                ("south_neighbor_first_line", "south_staff_ids"),
            ] {
                let ids = oracle[key].as_array().unwrap();
                system["area"][field] = if ids.len() == 1 {
                    ids[0].clone()
                } else {
                    Value::Null
                };
                if ids.len() > 1 {
                    system["area"][key] = oracle[key].clone();
                }
            }
        }
        let geometry =
            audiveris_native::beam_dispatch::Geometry::from_grid(w, h, &grid, 11).unwrap();
        for q in config["selections"].as_array().unwrap() {
            let p = [
                q["point"][0].as_i64().unwrap() as i32,
                q["point"][1].as_i64().unwrap() as i32,
            ];
            let got = geometry.eligible(p).unwrap();
            assert_eq!(
                serde_json::json!(got),
                q["eligible"],
                "{} {p:?}",
                config["name"]
            );
            if got.is_empty() {
                absent += 1;
            } else {
                positive += 1;
            }
            for row in q["beam_sides_height8"].as_array().unwrap() {
                assert_eq!(
                    geometry
                        .is_beam_side_in_system(
                            row["system_id"].as_u64().unwrap(),
                            p.map(|v| v as f64),
                            8.
                        )
                        .unwrap(),
                    row["inside"].as_bool().unwrap()
                );
            }
        }
        let mut malformed = grid.clone();
        malformed["systems"][0]["area"]["south_neighbor_first_line"] = serde_json::json!(u64::MAX);
        assert!(
            audiveris_native::beam_dispatch::Geometry::from_grid(w, h, &malformed, 11).is_err()
        );
        let mut malformed = grid.clone();
        malformed["systems"][0]["area"]["north_staff_ids"] = serde_json::json!([u64::MAX]);
        assert!(
            audiveris_native::beam_dispatch::Geometry::from_grid(w, h, &malformed, 11).is_err()
        );
    }
    assert!(positive > 0 && absent > 0);
}
