// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::GrayImage;
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
#[test]
fn original_beam_spots_match_complete_native_pipeline() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = std::fs::read_dir(root.join("fixtures/official-beams"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 13);
    for path in paths {
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
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

        let seeds =
            audiveris_native::stem_seeds::analyze(&residual, &grid, &headers, &sc, slope).unwrap();
        let (prepared, got) =
            audiveris_native::beam_preparation::analyze(&residual, &grid, &headers, &seeds, &sc)
                .unwrap();
        assert_eq!(got["pixel_preparation_complete"], true);
        assert_eq!(got["beams_complete"], false);
        assert_eq!(
            got["header_erase_rectangles"], expected["header_erase_rectangles"],
            "{name} header ROI"
        );
        for key in [
            "interline",
            "fore",
            "stem_max",
            "beam_thickness",
            "small_beam_thickness",
            "small_beams_switch",
            "effective_spot_beam",
        ] {
            assert_eq!(
                got["scale"][key], expected["scale"][key],
                "{name} scale {key}"
            );
        }
        for key in [
            "no_staff",
            "filtered_horizontal",
            "median",
            "gaussian",
            "header_erased",
            "closed",
            "binarized",
        ] {
            for field in ["width", "height", "black_count", "sha256"] {
                assert_eq!(
                    got["masks"][key][field], expected["masks"][key][field],
                    "{name} {key}.{field}"
                );
            }
        }
        for field in ["width", "height", "black_count", "sha256"] {
            assert_eq!(
                got["masks"]["head_spots"][field], expected["final"]["head_spots_mask"][field],
                "{name} head_spots.{field}"
            );
            assert_eq!(
                got["masks"]["no_staff"][field], expected["final"]["no_staff_after"][field],
                "{name} no_staff unchanged.{field}"
            );
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
            audiveris_native::beam_dispatch::dispatch(&prepared.glyphs, &geometry, &mut registry)
                .unwrap();
        assert_eq!(
            dispatched.len(),
            expected["systems"].as_array().unwrap().len()
        );
        for (a, b) in dispatched
            .iter()
            .zip(expected["systems"].as_array().unwrap())
        {
            assert_eq!(a.system_id, b["system_id"].as_u64().unwrap());
            let wanted = b["grouped_spots_before_beams"].as_array().unwrap();
            assert_eq!(a.glyph_ids.len(), wanted.len(), "{name} dispatch count");
            for (&id, g) in a.glyph_ids.iter().zip(wanted) {
                assert_eq!(
                    audiveris_native::beam_dispatch::glyph_json(&registry.get(id).unwrap().glyph),
                    g["glyph"],
                    "{name} dispatch pixels/order"
                );
            }
        }
        let faithful: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-beam-prolog/{name}.json")))
                .unwrap(),
        )
        .unwrap();
        let mut prolog =
            audiveris_native::beam_prolog::build(&prepared, &grid, &seeds, &sc).unwrap();
        let snapshot = prolog.report().unwrap();
        assert_eq!(
            snapshot["beam_prolog_complete"], true,
            "{name} native prolog complete"
        );
        assert_eq!(
            snapshot["head_scale"], faithful["prolog"]["head_scale"],
            "{name} head scale"
        );
        for (a, b) in snapshot["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(faithful["prolog"]["systems"].as_array().unwrap())
        {
            assert_eq!(a["system_id"], b["system_id"]);
            assert_eq!(
                a["grouped_spots"], b["grouped_spots"],
                "{name} native prolog registered group identity/order"
            );
        }
        let initial_expected: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-beam-candidates/{name}.json")))
                .unwrap(),
        )
        .unwrap();
        let registry_before = prolog.registry.snapshot();
        let state =
            audiveris_native::beam_candidates::analyze_typed(&residual, &mut prolog, &sc).unwrap();
        let initial = state.report();
        let registry_after = prolog.registry.snapshot();
        for (before, after) in registry_before
            .as_array()
            .unwrap()
            .iter()
            .zip(registry_after.as_array().unwrap())
        {
            assert_eq!(
                before, after,
                "{name} existing glyph identity/groups changed"
            );
        }
        for entry in registry_after
            .as_array()
            .unwrap()
            .iter()
            .skip(registry_before.as_array().unwrap().len())
        {
            assert_eq!(
                entry["groups"],
                serde_json::json!([]),
                "{name} candidate registration adds no glyph group"
            );
        }
        for system in state.systems() {
            let mut ids = std::collections::BTreeSet::new();
            for (offset, inter) in system.inters().iter().enumerate() {
                assert_eq!(inter.id(), offset + 1, "{name} native registration order");
                assert!(ids.insert((system.system_id(), inter.id())));
                assert_eq!(
                    prolog.registry.get(inter.glyph_id()).unwrap().glyph,
                    inter.glyph
                );
                let count = prolog.registry.len();
                assert_eq!(
                    prolog.registry.register(&inter.glyph).unwrap(),
                    inter.glyph_id()
                );
                assert_eq!(
                    prolog.registry.len(),
                    count,
                    "{name} duplicate glyph identity"
                );
            }
            for &(a, b) in system.exclusions() {
                assert!(
                    ids.contains(&(system.system_id(), a))
                        && ids.contains(&(system.system_id(), b))
                );
            }
            assert!(system.assigned_spot_ids().iter().all(|id| {
                system
                    .ordered_spots()
                    .iter()
                    .any(|v| v["spot_id"].as_u64() == Some(*id as u64))
            }));
        }
        assert_eq!(
            state.report(),
            initial,
            "{name} stable serialization after registry reuse"
        );
        assert_eq!(initial["initial_beam_candidates_complete"], true);
        assert_eq!(initial["beams_complete"], false);
        assert_eq!(initial["musicxml_complete"], false);
        assert_eq!(
            initial["systems"].as_array().unwrap().len(),
            initial_expected["systems"].as_array().unwrap().len()
        );
        for (a, b) in initial["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(initial_expected["systems"].as_array().unwrap())
        {
            assert_eq!(a["system_id"], b["system_id"]);
            let actual_order = a["ordered_spots"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["glyph"].clone())
                .collect::<Vec<_>>();
            let expected_order = b["ordered_spots"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["glyph"].clone())
                .collect::<Vec<_>>();
            assert_eq!(actual_order, expected_order, "{name} initial spot order");
            let assigned = a["assigned_spot_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| {
                    audiveris_native::beam_dispatch::glyph_json(
                        &prolog
                            .registry
                            .get(id.as_u64().unwrap() as usize)
                            .unwrap()
                            .glyph,
                    )
                })
                .collect::<Vec<_>>();
            let expected_assigned = b["assigned_spots"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["glyph"].clone())
                .collect::<Vec<_>>();
            assert_eq!(assigned, expected_assigned, "{name} assigned spots");
            for key in ["inters", "relations"] {
                let mut aa = a[key].as_array().unwrap().clone();
                let mut bb = b["after_create_beams"][key].as_array().unwrap().clone();
                let order = |v: &Value| {
                    serde_json::json!([
                        v["kind"],
                        v["bounds"],
                        v["median"],
                        v["source"]["kind"],
                        v["source"]["bounds"],
                        v["target"]["kind"],
                        v["target"]["bounds"]
                    ])
                    .to_string()
                };
                aa.sort_by_key(&order);
                bb.sort_by_key(&order);
                compare_initial(
                    &Value::Array(aa),
                    &Value::Array(bb),
                    &format!("{name} initial {key}"),
                );
            }
        }
        let actual = got["unassigned_spots"].as_array().unwrap();
        let original = expected["sheet_spots"].as_array().unwrap();
        assert_eq!(actual.len(), original.len(), "{name} spot count");
        for (index, (a, b)) in actual.iter().zip(original).enumerate() {
            for field in ["glyph", "weight", "centroid"] {
                assert_eq!(a[field], b[field], "{name} spot {index}.{field}");
            }
        }
    }
}

fn empty_input() -> (GrayImage, Value, Value, Value, scale::Scale) {
    use serde_json::json;
    let image = GrayImage::new(100, 100, vec![255; 10000]).unwrap();
    let grid = json!({"staffs":[{"id":1,"system_id":1,"lines":(0..5).map(|i|json!({"points":[[10,20+5*i],[90,20+5*i]]})).collect::<Vec<_>>()}],"systems":[{"id":1,"staff_ids":[1],"area":{"x_range":[0,100]}}]});
    let headers =
        json!({"headers_complete":true,"staffs":[{"staff_id":1,"system_id":1,"stop":20}]});
    let seeds = json!({"stem_seeds_complete":true,"stem_scaling":{"stem_scale":{"max":2}}});
    let scale = scale::Scale {
        line: scale::Range {
            min: 1,
            main: 1,
            max: 1,
        },
        interline: scale::Range {
            min: 5,
            main: 5,
            max: 5,
        },
        small_interline: None,
        beam: scale::Beam {
            thickness: 3,
            extrapolated: false,
        },
        small_beam: None,
    };
    (image, grid, headers, seeds, scale)
}
#[test]
fn native_adapter_has_explicit_stage_and_scale_boundary() {
    let (image, grid, headers, seeds, mut sc) = empty_input();
    let (_, out) =
        audiveris_native::beam_preparation::analyze(&image, &grid, &headers, &seeds, &sc).unwrap();
    assert_eq!(out["spot_count"], 0);
    assert_eq!(out["beams_complete"], false);
    assert_eq!(
        out["header_erase_rectangles"][0]["rect"],
        serde_json::json!([0, 10, 21, 41])
    );
    sc.small_beam = Some(scale::Beam {
        thickness: 2,
        extrapolated: false,
    });
    assert_eq!(
        audiveris_native::beam_preparation::effective_beam(&sc).unwrap(),
        2
    );
    sc.small_beam = Some(scale::Beam {
        thickness: 4,
        extrapolated: false,
    });
    assert_eq!(
        audiveris_native::beam_preparation::effective_beam(&sc).unwrap(),
        3
    );
    sc.small_beam = Some(scale::Beam {
        thickness: 0,
        extrapolated: false,
    });
    assert!(audiveris_native::beam_preparation::effective_beam(&sc).is_err());
}
#[test]
fn native_adapter_rejects_incomplete_or_inconsistent_dependencies() {
    use serde_json::json;
    let (image, grid, headers, seeds, sc) = empty_input();
    let run = |g: &Value, h: &Value, s: &Value| {
        audiveris_native::beam_preparation::analyze(&image, g, h, s, &sc)
    };
    let mut changed = headers.clone();
    changed["headers_complete"] = json!(false);
    assert!(run(&grid, &changed, &seeds).is_err());
    let mut changed = seeds.clone();
    changed["stem_seeds_complete"] = json!(false);
    assert!(run(&grid, &headers, &changed).is_err());
    for val in [json!(0), json!(129), json!(-1), json!(null)] {
        let mut changed = seeds.clone();
        changed["stem_scaling"]["stem_scale"]["max"] = val;
        assert!(run(&grid, &headers, &changed).is_err());
    }
    for stop in [
        json!(-1),
        json!(i64::MIN),
        json!(i64::MAX),
        json!(0.1),
        json!(null),
    ] {
        let mut changed = headers.clone();
        changed["staffs"][0]["stop"] = stop;
        assert!(run(&grid, &changed, &seeds).is_err());
    }
    let mut changed = grid.clone();
    changed["systems"][0]["area"]["x_range"] = json!([10, 100]);
    assert!(run(&changed, &headers, &seeds).is_err());
    let mut changed = grid.clone();
    changed["systems"][0]["staff_ids"] = json!([1, 1]);
    assert!(run(&changed, &headers, &seeds).is_err());
    let mut changed = grid.clone();
    changed["staffs"][0]["system_id"] = json!(2);
    assert!(run(&changed, &headers, &seeds).is_err());
    let mut changed = grid.clone();
    changed["staffs"][0]["lines"][0]["points"] = json!([[10, 20], [10, 30]]);
    assert!(run(&changed, &headers, &seeds).is_err());
}
#[test]
fn beam_spots_cli_is_native_and_refuses_existing_outputs() {
    use sha2::{Digest, Sha256};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!("native-beam-spots-cli-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let mask = dir.join("spots.pgm");
    let report = dir.join("report.json");
    let run = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_audiveris-native"))
            .env_clear()
            .env("PATH", "/nonexistent")
            .args([
                "beam-spots",
                root.join("fixtures/official-grid/beams-double.png")
                    .to_str()
                    .unwrap(),
                mask.to_str().unwrap(),
                report.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let output = run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let r: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    assert_eq!(r["schema"], "audiveris-native.experimental-beam-spots.v1");
    assert_eq!(r["last_completed_stage"], "LEDGERS");
    assert_eq!(r["last_completed_substage"], "LEDGERS.EPILOG");
    assert_eq!(r["ledger_post_analysis"]["ledgers_complete"], true);
    assert_eq!(r["ledger_post_analysis"]["post_analysis_complete"], true);
    for boundary in [
        "extensions_complete",
        "hooks_complete",
        "groups_complete",
        "multiple_rest_boundary_complete",
        "epilog_complete",
        "beams_complete",
    ] {
        assert_eq!(r["later_beams"][boundary], true, "{boundary}");
    }
    assert_eq!(r["musicxml_complete"], false);
    assert_eq!(r["later_beams"]["epilog"]["graph_mutation"], false);
    assert!(
        r["shared_sig"]["covered_substages"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("BEAMS.EPILOG"))
    );
    let live: std::collections::BTreeSet<_> = r["shared_sig"]["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_u64().unwrap())
        .collect();
    assert_eq!(
        live.len(),
        r["shared_sig"]["inters"].as_array().unwrap().len()
    );
    for relation in r["shared_sig"]["relations"].as_array().unwrap() {
        assert!(live.contains(&relation["source"].as_u64().unwrap()));
        assert!(live.contains(&relation["target"].as_u64().unwrap()));
    }
    for tombstone in r["shared_sig"]["tombstones"].as_array().unwrap() {
        assert!(!live.contains(&tombstone.as_u64().unwrap()));
    }
    assert_eq!(
        r["beam_candidates"]["initial_beam_candidates_complete"],
        true
    );
    assert_eq!(r["beam_candidates"]["beams_complete"], false);
    assert_eq!(r["beam_preparation"]["beams_complete"], false);
    assert_eq!(r["beam_prolog"]["beam_prolog_complete"], true);
    assert_eq!(r["beam_prolog"]["beams_complete"], false);
    let bytes = std::fs::read(&mask).unwrap();
    let image = GrayImage::decode(&bytes).unwrap();
    assert_eq!(
        r["beam_preparation"]["masks"]["binarized"]["sha256"],
        format!("{:x}", Sha256::digest(&image.pixels))
    );
    assert_eq!(run().status.code(), Some(1));
    assert_eq!(std::fs::read(&mask).unwrap(), bytes);
    std::fs::remove_dir_all(dir).unwrap();
}

fn compare_initial(actual: &Value, expected: &Value, path: &str) {
    match expected {
        Value::Object(fields) => {
            for (k, v) in fields {
                compare_initial(&actual[k], v, &format!("{path}.{k}"));
            }
        }
        Value::Array(values) => {
            let a = actual.as_array().unwrap();
            assert_eq!(a.len(), values.len(), "{path}");
            for (i, (x, y)) in a.iter().zip(values).enumerate() {
                compare_initial(x, y, &format!("{path}[{i}]"));
            }
        }
        Value::Number(_) => {
            let a = actual.as_f64().unwrap();
            let b = expected.as_f64().unwrap();
            assert!((a - b).abs() <= 1e-8, "{path}: {actual} != {expected}");
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}

#[test]
fn initial_candidates_reject_partial_prolog_without_mutating_it() {
    let (image, mut grid, headers, mut seeds, sc) = empty_input();
    grid["systems"][0]["coordinates"] = serde_json::json!([10, 20, 81, 40, 0]);
    grid["systems"][0]["area"]["vertical_margin"] = serde_json::json!(4.0);
    seeds["systems"] = serde_json::json!([{"system_id":1,"accepted_seeds":[]}]);
    let (prepared, _) =
        audiveris_native::beam_preparation::analyze(&image, &grid, &headers, &seeds, &sc).unwrap();
    let mut prolog = audiveris_native::beam_prolog::build(&prepared, &grid, &seeds, &sc).unwrap();
    prolog.head_sizing.music_font_unsupported = Some("review_partial_font_state".into());
    let before = prolog.report().unwrap();
    assert_eq!(before["beam_prolog_complete"], false);
    assert_eq!(
        audiveris_native::beam_candidates::analyze(&image, &mut prolog, &sc).unwrap_err(),
        "incomplete_beam_prolog_dependency"
    );
    assert_eq!(before, prolog.report().unwrap());
}
