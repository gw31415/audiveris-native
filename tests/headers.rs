// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, time_column};
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::{Value, json};
#[test]
fn complete_headers_match_live_original() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = std::fs::read_dir(root.join("fixtures/official-headers-live"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 71);
    for path in paths {
        let name = path.file_stem().unwrap().to_str().unwrap();
        println!("headers {name}");
        let expected: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
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

        let column =
            audiveris_native::key_column::analyze(&residual, &grid, &registered, sc.interline.main)
                .unwrap();
        let actual =
            time_column::analyze(&residual, &grid, &registered, &column, sc.interline.main)
                .unwrap();

        let mut staffs = Vec::new();
        let mut systems = Vec::new();
        for system in actual["systems"].as_array().unwrap() {
            let finalized = audiveris_native::header_finalization::finalize_system(
                &grid,
                &system["after_system_time_stop"],
                sc.interline.main,
            )
            .unwrap();
            for staff in finalized["staffs"].as_array().unwrap() {
                let mut s = staff.clone();
                s["system_id"] = system["system_id"].clone();
                staffs.push(s);
            }
            systems.push(json!({"id":system["system_id"],"inters":finalized["sig_inters"],"relations":finalized["relations"]}));
        }
        let got = json!({"staffs":staffs,"systems":systems});
        fields(&canonical(&got), &canonical(&expected), name);
    }
}
fn identity(n: &Value) -> String {
    json!([
        n["kind"],
        n["shape"],
        n["bounds"],
        strip_ids(&n["glyph"]),
        n["numerator"],
        n["denominator"],
        n["fifths"],
        n["clef_kind"]
    ])
    .to_string()
}
fn strip_ids(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            m.remove("id");
            for v in m.values_mut() {
                *v = strip_ids(v);
            }
            Value::Object(m)
        }
        Value::Array(a) => Value::Array(a.iter().map(strip_ids).collect()),
        _ => v.clone(),
    }
}
fn canonical(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            m.remove("live_header_builder_complete");
            m.remove("persisted_or_reloaded");
            m.remove("id");
            if m.contains_key("inters") && m.contains_key("relations") {
                let ids = m["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| (n["id"].as_u64().unwrap(), identity(n)))
                    .collect::<std::collections::BTreeMap<_, _>>();
                for e in m.get_mut("relations").unwrap().as_array_mut().unwrap() {
                    for k in ["source", "target"] {
                        e[k] = json!(ids.get(&e[k].as_u64().unwrap()).unwrap());
                    }
                }
                m.get_mut("inters")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .sort_by_key(identity);
                m.get_mut("relations")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .sort_by_key(|e| json!([e["kind"], e["source"], e["target"]]).to_string());
            }
            for v in m.values_mut() {
                *v = canonical(v);
            }
            Value::Object(m)
        }
        Value::Array(a) => Value::Array(a.iter().map(canonical).collect()),
        _ => v.clone(),
    }
}
fn fields(a: &Value, b: &Value, path: &str) {
    match b {
        Value::Object(m) => {
            for (k, v) in m {
                fields(&a[k], v, &format!("{path}.{k}"));
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len(), "{path}: {a} != {b}");
            for (i, (x, y)) in a.as_array().unwrap().iter().zip(v).enumerate() {
                fields(x, y, &format!("{path}[{i}]"));
            }
        }
        Value::Number(_) => {
            if b.as_i64().is_some() {
                assert_eq!(a, b, "{path}");
            } else {
                let x = a.as_f64().unwrap();
                let y = b.as_f64().unwrap();
                assert!((x - y).abs() <= 1e-9, "{path}: {x:?} != {y:?}");
            }
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}
