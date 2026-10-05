// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::GrayImage;
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
#[test]
fn original_uninterrupted_loop_matches_thirteen_stage_boundaries() {
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
        let mut state =
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
        let original_loop: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-beam-extension-loop/{name}.json")))
                .unwrap(),
        )
        .unwrap();
        let extension_scale = audiveris_native::beam_extend_to_beam::ExtensionScale {
            interline: sc.interline.main,
            standard_height: sc.beam.thickness as f64,
            small_height: sc.small_beam.map(|b| b.thickness as f64),
        };
        for owner in &prolog.systems {
            let system = state.system_mut(owner.system_id).unwrap();
            let original = original_loop["systems"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["system_id"] == owner.system_id)
                .unwrap();
            let initial_ids = system
                .inters()
                .iter()
                .filter(|v| !v.is_removed())
                .map(|v| v.id())
                .collect::<Vec<_>>();
            let report = audiveris_native::beam_extension_loop::extend_beams(
                system,
                &residual,
                &mut prolog.registry,
                extension_scale,
                audiveris_native::beam_extend_in_parallel::ParallelOwner {
                    free_glyphs: owner,
                    geometry: &prolog.geometry,
                },
            )
            .unwrap();
            assert_eq!(report.initial_ids, initial_ids, "{name} snapshot capture");
            assert!(
                report
                    .sides
                    .iter()
                    .all(|(id, _, _)| initial_ids.contains(id)),
                "{name} appended revisit"
            );
            let raw = original["raw_after"].as_array().unwrap();
            assert_eq!(
                system.inters().len(),
                raw.len(),
                "{name} raw registration count"
            );
            for (a, b) in system.inters().iter().zip(raw) {
                assert_eq!(
                    a.is_removed(),
                    b["removed"].as_bool().unwrap(),
                    "{name} tombstones/order"
                );
                compare_initial(
                    &serde_json::json!(a.item.median),
                    &b["beam"]["median"],
                    &format!("{name} raw median"),
                );
                compare_initial(
                    &audiveris_native::beam_dispatch::glyph_json(&a.glyph),
                    &b["beam"]["glyph"],
                    &format!("{name} raw glyph"),
                );
            }
        }
        let extended = state.report();
        for (actual, original) in extended["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(original_loop["systems"].as_array().unwrap())
        {
            for key in ["inters", "relations"] {
                let mut aa = actual[key].as_array().unwrap().clone();
                let mut bb = original["after_extend_beams"][key]
                    .as_array()
                    .unwrap()
                    .clone();
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
                    &format!("{name} extension {key}"),
                );
            }
        }
    }
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
fn original_whole_loop_controls_preserve_snapshot_removal_and_replacement_order() {
    use audiveris_native::{
        beam_candidates::SystemCandidates,
        beam_dispatch::{Geometry, Registry, SystemSpots},
        beam_extend_in_parallel::ParallelOwner,
        beam_extend_to_beam::ExtensionScale,
        beam_impacts::Impacts,
        beam_merge::MergeCandidate,
        beam_structure::BeamItem,
    };
    use serde_json::json;
    let data: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extension-loop-controls.json"
    ))
    .unwrap();
    assert_eq!(data["native_outputs_used"], false);
    assert_eq!(data["cases"].as_array().unwrap().len(), 37);
    let grid = json!({"staffs":[{"id":1,"system_id":1,"lines":(0..5).map(|i|json!({"points":[[0,10+4*i],[160,10+4*i]]})).collect::<Vec<_>>()}],"systems":[{"id":1,"staff_ids":[1],"coordinates":[0,10,160,30,0],"area":{"x_range":[0,160],"vertical_margin":14.0,"north_neighbor_last_line":null,"south_neighbor_first_line":null}}]});
    let geometry = Geometry::from_grid(160, 96, &grid, 16).unwrap();
    let mut changed = 0;
    for (i, c) in data["cases"].as_array().unwrap().iter().enumerate() {
        let input = &c["input"];
        let seed = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
        let mut pixels = vec![255; 160 * 96];
        for p in input["black_pixels"].as_array().unwrap() {
            pixels[p[1].as_u64().unwrap() as usize * 160 + p[0].as_u64().unwrap() as usize] = 0;
        }
        let image = GrayImage::new(160, 96, pixels).unwrap();
        let mut state = SystemCandidates::empty(1);
        let mut registry = Registry::default();
        for b in input["beams"].as_array().unwrap() {
            state
                .register_merge(
                    &seed,
                    &mut registry,
                    &MergeCandidate {
                        item: BeamItem {
                            median: std::array::from_fn(|i| b["median"][i].as_f64().unwrap()),
                            height: 8.,
                        },
                        is_small: false,
                        impacts: Impacts::new([1.; 6]).unwrap(),
                    },
                )
                .unwrap();
        }
        if let Some(id) = input["pre_removed"].as_u64() {
            state.remove(id as usize).unwrap();
        }
        let owner = SystemSpots {
            system_id: 1,
            glyph_ids: vec![],
        };
        let report = audiveris_native::beam_extension_loop::extend_beams(
            &mut state,
            &image,
            &mut registry,
            ExtensionScale {
                interline: 16,
                standard_height: 8.,
                small_height: None,
            },
            ParallelOwner {
                free_glyphs: &owner,
                geometry: &geometry,
            },
        )
        .unwrap();
        assert_eq!(
            json!(report.initial_ids),
            c["initial_ids"],
            "case {i} initial snapshot"
        );
        assert!(
            report
                .sides
                .iter()
                .all(|(id, _, _)| report.initial_ids.contains(id)),
            "case {i} appended revisit"
        );
        assert_eq!(
            json!(
                state
                    .inters()
                    .iter()
                    .filter(|v| !v.is_removed())
                    .map(|v| v.id())
                    .collect::<Vec<_>>()
            ),
            c["after"]["active_ids"],
            "case {i} active order"
        );
        assert_eq!(
            json!(
                state
                    .inters()
                    .iter()
                    .map(|v| v.is_removed())
                    .collect::<Vec<_>>()
            ),
            c["after"]["removed"],
            "case {i} raw removal order"
        );
        assert_eq!(
            json!(
                state
                    .inters()
                    .iter()
                    .map(|v| state.active(v.id()).is_some())
                    .collect::<Vec<_>>()
            ),
            c["after"]["index_present"],
            "case {i} active index"
        );
        assert_eq!(
            state.exclusions().len(),
            c["after"]["edge_count"].as_u64().unwrap() as usize
        );
        compare_initial(
            &json!(
                state
                    .inters()
                    .iter()
                    .map(|v| v.item.median)
                    .collect::<Vec<_>>()
            ),
            &c["raw_medians"],
            &format!("case {i} raw medians"),
        );
        compare_initial(
            &json!(
                state
                    .inters()
                    .iter()
                    .map(|v| v.impacts.grade)
                    .collect::<Vec<_>>()
            ),
            &c["raw_grades"],
            &format!("case {i} raw grades"),
        );
        if state.inters().iter().any(|v| v.is_removed()) {
            changed += 1;
        }
    }
    assert_eq!(changed, 29);
}
