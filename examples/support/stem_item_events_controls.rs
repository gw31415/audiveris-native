// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    stem_item_events::{self, Prepared, Source},
    stem_items_cli::{self, Case, Request},
    stem_length_map,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn near(a: &Value, b: &Value) {
    match b {
        Value::Number(n) if n.is_f64() => assert!(
            (a.as_f64().unwrap() - n.as_f64().unwrap()).abs() < 1e-12,
            "{a} != {b}"
        ),
        Value::Object(m) => {
            for (k, v) in m {
                near(&a[k], v)
            }
        }
        Value::Array(v) => {
            assert_eq!(a.as_array().unwrap().len(), v.len());
            for (x, y) in a.as_array().unwrap().iter().zip(v) {
                near(x, y)
            }
        }
        _ => assert_eq!(a, b),
    }
}
fn prepared(v: &Value) -> Prepared {
    Prepared {
        token: serde_json::from_value(v["token"].clone()).unwrap(),
        kind: serde_json::from_value(v["kind"].clone()).unwrap(),
        reference: serde_json::from_value(v["reference"].clone()).unwrap(),
        contrib: v["contrib"].as_i64().unwrap() as i32,
        line: serde_json::from_value(v["line"].clone()).unwrap(),
        head_bounds: serde_json::from_value(v["head_bounds"].clone()).unwrap(),
    }
}
fn observed(i: &Prepared) -> Value {
    json!({"kind":i.kind,"token":i.token,"reference":i.reference,"gap":i.kind==stem_item_events::Kind::GapItem,"contrib":i.contrib,"line":i.line,"line_bounds":stem_length_map::line_bounds(i.line).unwrap(),"head_bounds":i.head_bounds})
}
pub fn request(o: &Value) -> Request {
    let mut cases = Vec::new();
    for g in o["groups"].as_array().unwrap() {
        for c in g["cases"].as_array().unwrap() {
            cases.push(Case {
                direction: g["direction"].as_i64().unwrap() as i8,
                start_y: g["start_y"].as_f64().unwrap(),
                y_range: serde_json::from_value(g["y_range"].clone()).unwrap(),
                sources: serde_json::from_value(c["sources"].clone()).unwrap(),
                gap_map: serde_json::from_value(o["gap_map"].clone()).unwrap(),
                stem_profile: c["stem_profile"].as_i64().unwrap() as i32,
                max_profile: 4,
                initial_lengths: BTreeMap::new(),
                skew: 0.,
            });
        }
    }
    Request {
        schema: stem_items_cli::REQUEST_SCHEMA.into(),
        cases,
    }
}
pub fn compare_report(o: &Value, native: &Value) {
    let mut index = 0;
    for g in o["groups"].as_array().unwrap() {
        for c in g["cases"].as_array().unwrap() {
            let n = &native["cases"][index];
            near(&n["events"]["sorted_tokens"], &c["sorted_tokens"]);
            assert_eq!(n["prefix_lengths"], c["prefix_lengths"]);
            assert_eq!(n["lengths"], c["lengths"]);
            let v = n["events"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| observed(&serde_json::from_value(v.clone()).unwrap()))
                .collect::<Vec<_>>();
            near(&json!(v), &c["items"]);
            let kept = v.iter().filter(|v| !v["token"].is_null()).count();
            assert_eq!(
                n["events"]["truncated"],
                json!(kept < c["sources"].as_array().unwrap().len())
            );
            index += 1;
        }
    }
    assert_eq!(native["cases"].as_array().unwrap().len(), index);
    assert!(
        native["registered_to_sig"] == false
            && native["epilog_complete"] == false
            && native["heads_complete"] == false
            && native["stems_complete"] == false
    );
}
pub fn observe(o: &Value) -> Value {
    let req = request(o);
    let native = stem_items_cli::evaluate(&req).unwrap();
    let report = serde_json::to_value(&native).unwrap();
    compare_report(o, &report);
    let mut boundaries = Vec::new();
    let gaps: BTreeMap<i32, i32> = serde_json::from_value(o["gap_map"].clone()).unwrap();
    for g in o["groups"].as_array().unwrap() {
        let dir = g["direction"].as_i64().unwrap() as i8;
        for c in g["cases"].as_array().unwrap() {
            let sources: Vec<Source> = serde_json::from_value(c["sources"].clone()).unwrap();
            let constructed = stem_item_events::construct(
                &sources,
                serde_json::from_value(g["y_range"].clone()).unwrap(),
            )
            .unwrap();
            let v = constructed.iter().map(observed).collect::<Vec<_>>();
            near(&json!(v), &c["prepared_input"]);
        }
        for c in g["declared_line_boundaries"].as_array().unwrap() {
            let items = c["prepared_input"]
                .as_array()
                .unwrap()
                .iter()
                .map(prepared)
                .collect::<Vec<_>>();
            let events = stem_item_events::events(
                &items,
                dir,
                gaps[&(c["stem_profile"].as_i64().unwrap() as i32)],
            )
            .unwrap();
            let out = events.items.iter().map(observed).collect::<Vec<_>>();
            near(&json!(out), &c["items"]);
            assert_eq!(json!(events.sorted_tokens), c["sorted_tokens"]);
            let length_items = events
                .items
                .iter()
                .map(|i| Some(i.length_item()))
                .collect::<Vec<_>>();
            let prefix =
                stem_length_map::prefix_lengths(&length_items, dir, g["start_y"].as_f64().unwrap())
                    .unwrap();
            assert_eq!(json!(prefix), c["prefix_lengths"]);
            let lengths = stem_length_map::retrieve_lengths(
                &length_items,
                dir,
                g["start_y"].as_f64().unwrap(),
                &gaps,
                4,
                &BTreeMap::new(),
            )
            .unwrap();
            assert_eq!(json!(lengths), c["lengths"]);
            boundaries.push(json!({"items":out,"prefix_lengths":prefix,"lengths":lengths}));
        }
    }
    json!({"report":report,"declared_line_boundaries":boundaries,"automatic_image_sources_complete":false,"page_cache_SIG_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false})
}
