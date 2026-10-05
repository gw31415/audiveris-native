use audiveris_native::{GrayImage, key_column};
use audiveris_native::{
    bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
use std::collections::BTreeMap;
fn identity_map(stage: &Value) -> BTreeMap<u64, String> {
    let mut ids = BTreeMap::new();
    for (i, c) in stage["remaining_sig_clefs"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        ids.insert(c["id"].as_u64().unwrap(), format!("clef:{i}"));
    }
    for sh in stage["shape_states"].as_array().unwrap() {
        let name = sh["shape"].as_str().unwrap();
        for (i, s) in sh["slices"].as_array().unwrap().iter().enumerate() {
            if let Some(id) = s["alter"]["id"].as_u64() {
                ids.insert(id, format!("alter:{name}:{i}"));
            }
        }
        if let Some(id) = sh["key_inter"]["id"].as_u64() {
            ids.insert(id, format!("key:{name}"));
        }
        for (i, a) in sh["key_inter"]["members"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            if let Some(id) = a["id"].as_u64() {
                ids.entry(id).or_insert(format!("member:{name}:{i}"));
            }
        }
    }
    ids
}
fn canonical(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            if m.contains_key("kind") {
                m.remove("id");
            }
            for val in m.values_mut() {
                *val = canonical(val);
            }
            Value::Object(m)
        }
        Value::Array(a) => Value::Array(a.iter().map(canonical).collect()),
        _ => v.clone(),
    }
}
fn canonical_stage(v: &Value) -> Value {
    let ids = identity_map(v);
    let mut out = canonical(v);
    for r in out["relations"].as_array_mut().unwrap() {
        for endpoint in ["source", "target"] {
            let id = r[endpoint].as_u64().unwrap();
            r[endpoint] = Value::String(
                ids.get(&id)
                    .unwrap_or_else(|| panic!("unmapped {id}: {v}"))
                    .clone(),
            );
        }
    }
    out
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
#[test]
fn original_column_rasters() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut cases = std::fs::read_dir(root.join("fixtures/official-grid"))
        .unwrap()
        .map(|d| d.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with("-key-column.json")
        })
        .collect::<Vec<_>>();
    cases.sort();
    assert_eq!(cases.len(), 51);
    for oracle_path in cases {
        let name = oracle_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches("-key-column.json");
        println!("checking {name}");
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
        let actual = key_column::analyze(&residual, &grid, &registered, sc.interline.main).unwrap();
        let expected: Value =
            serde_json::from_slice(&std::fs::read(&oracle_path).unwrap()).unwrap();
        assert_eq!(
            actual["systems"].as_array().unwrap().len(),
            expected["systems"].as_array().unwrap().len()
        );
        let time: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("fixtures/official-grid/{name}-time-column.json")))
                .unwrap(),
        )
        .unwrap();
        for (a, b) in actual["systems"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["systems"].as_array().unwrap())
        {
            let original_time = time["systems"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["system_id"] == a["system_id"])
                .unwrap();
            let after = &original_time["chronology"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["operation"] == "before_builders_values_1")
                .unwrap()["state"]["staffs"];
            assert_eq!(a["returned_key_column_offset"], original_time["key_offset"]);
            assert_eq!(
                a["after_system_key_stop"].as_array().unwrap().len(),
                after.as_array().unwrap().len()
            );
            for (s, t) in a["after_system_key_stop"]
                .as_array()
                .unwrap()
                .iter()
                .zip(after.as_array().unwrap())
            {
                for k in [
                    "staff_id",
                    "header_start",
                    "header_stop",
                    "clef_stop",
                    "key_stop",
                ] {
                    fields(&s[k], &t[k], &format!("{name}.after_system_key_stop.{k}"));
                }
            }
            for k in [
                "system_id",
                "multi_staff",
                "max_header_width",
                "clef_column_offset",
                "max_slice_dist",
                "returned_key_column_offset",
            ] {
                fields(&a[k], &b[k], &format!("{name}.{k}"));
            }
            for k in ["global_offsets", "parts"] {
                fields(
                    &a["final_state"][k],
                    &b["final_state"][k],
                    &format!("{name}.{k}"),
                );
            }
            let sa = a["final_state"]["staffs"].as_array().unwrap();
            let sb = b["final_state"]["staffs"].as_array().unwrap();
            assert_eq!(sa.len(), sb.len());
            for (a, b) in sa.iter().zip(sb) {
                for k in [
                    "staff_id",
                    "header_start",
                    "header_stop",
                    "clef_stop",
                    "key_stop",
                    "header_key",
                    "header_clef",
                    "alter_starts",
                    "key_range",
                ] {
                    fields(
                        &canonical(&a[k]),
                        &canonical(&b[k]),
                        &format!("{name}.staff{}.{}", a["staff_id"], k),
                    );
                }
                fields(
                    &canonical_stage(&a["builder_state"]),
                    &canonical_stage(&b["builder_state"]),
                    &format!("{name}.staff{}.builder", a["staff_id"]),
                );
                let original_ids = identity_map(&b["builder_state"]);
                let native_ids = identity_map(&a["builder_state"]);
                let ef = b["frozen_inter_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|id| original_ids.get(&id.as_u64().unwrap()))
                    .collect::<std::collections::BTreeSet<_>>();
                let af = a["frozen_key_alter_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| native_ids.get(&id.as_u64().unwrap()).unwrap())
                    .collect::<std::collections::BTreeSet<_>>();
                assert_eq!(af, ef, "{name} frozen");
            }
        }
    }
}
