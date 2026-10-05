// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, time_column};
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::{Value, json};
#[test]
fn complete_time_column_matches_original() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = std::fs::read_dir(root.join("fixtures/official-grid"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with("-time-column.json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 71);
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix("-time-column.json")
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
        assert_eq!(
            actual["systems"].as_array().unwrap().len(),
            expected["systems"].as_array().unwrap().len()
        );
        for (a, b) in actual["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["systems"].as_array().unwrap())
        {
            for k in [
                "system_id",
                "clef_offset",
                "key_offset",
                "returned_time_offset",
            ] {
                fields(&a[k], &b[k], &format!("{name}.{k}"));
            }
            for stage in ["after_retrieve_time", "after_system_time_stop"] {
                fields(
                    &canonical(&a[stage]),
                    &canonical(&b[stage]),
                    &format!("{name}.{stage}"),
                );
                assert_eq!(
                    time_relations(&a[stage]),
                    time_relations(&b[stage]),
                    "{name}.{stage}.ordered_time_relations"
                );
            }
        }
    }
}

fn time_relations(state: &Value) -> Value {
    let ids = state["sig_inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap(), n))
        .collect::<std::collections::BTreeMap<_, _>>();
    json!(
        state["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| ids[&e["source"].as_u64().unwrap()]["kind"]
                .as_str()
                .unwrap()
                .starts_with("Time"))
            .map(|e| json!([
                e["kind"],
                identity(ids[&e["source"].as_u64().unwrap()]),
                identity(ids[&e["target"].as_u64().unwrap()])
            ]))
            .collect::<Vec<_>>()
    )
}
fn identity(n: &Value) -> String {
    serde_json::json!([n["kind"], n["shape"], n["bounds"], n["glyph"]]).to_string()
}
fn canonical(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            if m.contains_key("kind") && m.contains_key("grade") {
                m.remove("id");
            }
            m.remove("links");
            if let Some(a) = m.get_mut("sig_inters").and_then(Value::as_array_mut) {
                a.sort_by_key(identity);
            }
            if m.contains_key("sig_inters") {
                let ids = m["sig_inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| (n["id"].as_u64().unwrap(), identity(n)))
                    .collect::<std::collections::BTreeMap<_, _>>();
                for edge in m.get_mut("relations").unwrap().as_array_mut().unwrap() {
                    for k in ["source", "target"] {
                        edge[k] = json!(ids.get(&edge[k].as_u64().unwrap()).unwrap());
                    }
                }
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
