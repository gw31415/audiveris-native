// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::GrayImage;
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
#[test]
fn original_stem_seeds_match_complete_native_pipeline() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = std::fs::read_dir(root.join("fixtures/official-stem-seeds"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 12);
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".json")
            .unwrap()
            .to_owned();
        println!("checking {name}");
        let expected: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
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
            audiveris_native::header_finalization::analyze(&grid, &time, sc.interline.main)
                .unwrap();
        assert_eq!(headers["headers_complete"], true);
        let got =
            audiveris_native::stem_seeds::analyze(&residual, &grid, &headers, &sc, slope).unwrap();
        assert_eq!(got["stem_seeds_complete"], true);
        assert_eq!(shared.snapshot(), before);
        assert_eq!(
            got["systems"].as_array().unwrap().len(),
            expected["systems"].as_array().unwrap().len()
        );
        for (a, b) in got["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["systems"].as_array().unwrap())
        {
            for key in ["system_id", "left", "right", "profile"] {
                compare(&a[key], &b[key], &format!("{name}.{key}"));
            }
            let mut actual_seeds = a["accepted_seeds"].as_array().unwrap().clone();
            let mut expected_seeds = b["accepted_seeds"].as_array().unwrap().clone();
            actual_seeds.sort_by_key(Value::to_string);
            expected_seeds.sort_by_key(Value::to_string);
            compare(
                &Value::Array(actual_seeds),
                &Value::Array(expected_seeds),
                &format!("{name}.accepted_seed_membership"),
            );
            let aa = a["candidates"].as_array().unwrap();
            let bb = b["candidates"].as_array().unwrap();
            assert_eq!(aa.len(), bb.len(), "{name} candidates");
            for (i, (a, b)) in aa.iter().zip(bb).enumerate() {
                for key in [
                    "bounds",
                    "members",
                    "center",
                    "start",
                    "stop",
                    "line",
                    "mean_distance",
                    "glyph",
                    "closest_staff_id",
                    "skipped_header_or_staff",
                ] {
                    compare(&a[key], &b[key], &format!("{name}.{i}.{key}"));
                }
                if b["skipped_header_or_staff"] == false {
                    let c = &a["check"];
                    let e = &b["replayed_original_check"];
                    for k in ["context_black", "context_white", "context_gap"] {
                        assert_eq!(c[k], e[k], "{name}.{i}.{k}");
                    }
                    compare(
                        &c["grade"],
                        &e["impacts"]["grade"],
                        &format!("{name}.{i}.grade"),
                    );
                    assert_eq!(c["accepted"], b["accepted_seed"]);
                    assert_eq!(
                        c["checks"].as_array().unwrap().len(),
                        e["raw_checks"].as_array().unwrap().len()
                    );
                    for (x, y) in c["checks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(e["raw_checks"].as_array().unwrap())
                    {
                        compare(x, y, &format!("{name}.{i}.rawcheck"));
                    }
                }
            }
        }
    }
}
fn compare(a: &Value, b: &Value, path: &str) {
    match b {
        Value::Object(m) => {
            for (k, v) in m {
                compare(&a[k], v, &format!("{path}.{k}"));
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len(), "{path}");
            for (i, (x, y)) in a.as_array().unwrap().iter().zip(v).enumerate() {
                compare(x, y, &format!("{path}[{i}]"));
            }
        }
        Value::Number(_) => {
            if b.as_i64().is_some() {
                assert_eq!(a, b, "{path}");
            } else {
                assert!(
                    (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-8,
                    "{path}: {a} != {b}"
                );
            }
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}

fn empty_valid_input() -> (GrayImage, Value, Value, audiveris_native::scale::Scale) {
    use audiveris_native::scale::{Beam, Range, Scale};
    use serde_json::json;
    let image = GrayImage::new(100, 100, vec![255; 10_000]).unwrap();
    let staffs=(0..2).map(|i|json!({"id":i+1,"system_id":1,"left":0,"right":99,"lines":(0..5).map(|j|json!({"points":[[0,10+40*i+5*j],[99,10+40*i+5*j]]})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let grid = json!({"staffs":staffs,"vlag":[],"hlag":[],"systems":[{"id":1,"staff_ids":[1,2],"coordinates":[0,10,100,70,0],"area":{"x_range":[0,100]},"vertical_section_ids":[],"horizontal_section_ids":[]}]});
    let headers = json!({"headers_complete":true,"staffs":[{"staff_id":1,"system_id":1,"stop":5},{"staff_id":2,"system_id":1,"stop":5}]});
    let sc = Scale {
        line: Range {
            min: 1,
            main: 1,
            max: 1,
        },
        interline: Range {
            min: 5,
            main: 5,
            max: 5,
        },
        small_interline: None,
        beam: Beam {
            thickness: 2,
            extrapolated: true,
        },
        small_beam: None,
    };
    (image, grid, headers, sc)
}
#[test]
fn seed_scope_and_identity_rejections_also_apply_without_candidates() {
    use serde_json::json;
    let (image, grid, headers, sc) = empty_valid_input();
    assert!(audiveris_native::stem_seeds::analyze(&image, &grid, &headers, &sc, 0.).is_ok());
    for (kind, value) in [
        ("duplicate", json!([1, 1])),
        ("reordered", json!([2, 1])),
        ("missing", json!([1])),
        ("unknown", json!([1, 3])),
    ] {
        let mut g = grid.clone();
        g["systems"][0]["staff_ids"] = value;
        assert!(
            audiveris_native::stem_seeds::analyze(&image, &g, &headers, &sc, 0.).is_err(),
            "{kind}"
        );
    }
    let mut g = grid.clone();
    g["systems"][0]["area"]["x_range"] = json!([1, 100]);
    assert!(audiveris_native::stem_seeds::analyze(&image, &g, &headers, &sc, 0.).is_err());
    let mut h = headers.clone();
    h["staffs"][0]["system_id"] = json!(2);
    assert!(audiveris_native::stem_seeds::analyze(&image, &grid, &h, &sc, 0.).is_err());
}
#[test]
fn vertical_lag_run_cap_precedes_deserialization() {
    use serde_json::json;
    let (_, mut grid, _, _) = empty_valid_input();
    grid["vlag"] = json!([{"id":1,"first_pos":1,"runs":vec![[0,1];8193]}]);
    assert_eq!(
        audiveris_native::vertical_sticks::analyze(&grid, 5, 2).unwrap_err(),
        "invalid_stem_runs"
    );
    let section = audiveris_native::vertical_sticks::SeedSection {
        id: 1,
        vertical: true,
        first_pos: 4_000_000,
        runs: vec![[0, 1]],
    };
    assert!(audiveris_native::vertical_sticks::retrieve(&[section], &[], 2, 2).is_err());
}

#[test]
fn original_clipped_neighbor_chain_staff_selection_matches() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture: Value = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/official-seed-staff-area.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    let mut positive = 0;
    let mut absent = 0;
    for config in fixture["configurations"].as_array().unwrap() {
        let input = &config["input"];
        let mut grid = input["grid"].clone();
        let ends = audiveris_native::system_area_ends::discover(
            grid["systems"].as_array().unwrap(),
            input["width"].as_u64().unwrap() as usize,
        )
        .unwrap();
        for (s, e) in grid["systems"].as_array_mut().unwrap().iter_mut().zip(ends) {
            s["area"] = serde_json::json!({"x_range":e["x_range"]});
        }
        let mapping = audiveris_native::stem_seeds::StaffMapping::from_grid(
            &grid,
            input["width"].as_u64().unwrap() as usize,
            input["height"].as_u64().unwrap() as usize,
            11,
        )
        .unwrap();
        for q in config["selections"].as_array().unwrap() {
            let got = mapping
                .closest(
                    q["system_id"].as_u64().unwrap(),
                    [q["x"].as_f64().unwrap(), q["y"].as_f64().unwrap()],
                )
                .unwrap()
                .unwrap_or(0);
            assert_eq!(
                got,
                q["closest"].as_u64().unwrap(),
                "{} {}",
                config["name"],
                q
            );
            if got == 0 {
                absent += 1;
            } else {
                positive += 1;
            }
        }
        assert!(mapping.closest(u64::MAX, [0., 0.]).is_err());
        assert!(
            mapping
                .closest(grid["systems"][0]["id"].as_u64().unwrap(), [f64::NAN, 0.])
                .is_err()
        );
    }
    assert!(positive > 0 && absent > 0);
}
