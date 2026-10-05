// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::stem_beam_lookup_area::{self, Request};
use serde_json::{Value, json};
/// Normalize ring start/winding and exactly repeated consecutive vertices.
/// Java Area may emit zero-length edges on a sloped border. No rounding.
fn canonical(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut unique = Vec::new();
    for &point in points {
        if unique.last() != Some(&point) {
            unique.push(point);
        }
    }
    if unique.len() > 1 && unique.first() == unique.last() {
        unique.pop();
    }
    let points = unique.as_slice();
    if points.is_empty() {
        return vec![];
    }
    let mut candidates = Vec::new();
    for reverse in [false, true] {
        let mut ring = points.to_vec();
        if reverse {
            ring.reverse();
        }
        for i in 0..ring.len() {
            candidates.push(
                ring[i..]
                    .iter()
                    .chain(&ring[..i])
                    .copied()
                    .collect::<Vec<_>>(),
            );
        }
    }
    candidates.sort_by(|a, b| {
        for (p, q) in a.iter().zip(b) {
            let c = p[0].total_cmp(&q[0]).then(p[1].total_cmp(&q[1]));
            if !c.is_eq() {
                return c;
            }
        }
        a.len().cmp(&b.len())
    });
    candidates.remove(0)
}
pub fn observe(original: &Value) -> Value {
    let mut rows = Vec::new();
    for (i, c) in original["cases"].as_array().unwrap().iter().enumerate() {
        let mut input = c.as_object().unwrap().clone();
        for key in ["mode", "profile", "area_path", "theoretical_line"] {
            input.remove(key);
        }
        let r: Request = serde_json::from_value(Value::Object(input)).unwrap();
        let geometry =
            stem_beam_lookup_area::generate(&r).unwrap_or_else(|e| panic!("case{i}: {e}"));
        let mut expected = Vec::new();
        let mut closed = false;
        for segment in c["area_path"].as_array().unwrap() {
            assert!(!closed);
            match segment["type"].as_u64().unwrap() {
                0 => {
                    assert!(expected.is_empty());
                    expected.push(serde_json::from_value(segment["coordinates"].clone()).unwrap());
                }
                1 => {
                    assert!(!expected.is_empty());
                    expected.push(serde_json::from_value(segment["coordinates"].clone()).unwrap());
                }
                4 => closed = true,
                _ => panic!("uncertified area path"),
            }
        }
        let native_ring = canonical(&geometry.polygon);
        assert_eq!(
            native_ring,
            canonical(&expected),
            "case{i} vertices and adjacent edges"
        );
        assert_eq!(
            json!(geometry.theoretical_line),
            c["theoretical_line"],
            "case{i} theoretical line"
        );
        rows.push(json!({"polygon":native_ring,"theoretical_line":geometry.theoretical_line}));
    }
    json!({"cases":rows,"beam_neighborhood_selection_complete":false,"closer_limit_selection_complete":false,"heads_complete":false,"stems_complete":false})
}
