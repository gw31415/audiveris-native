// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::stem_head_targets::{Beam, Catalog, Group, Request};
use serde_json::{Value, json};
fn canonical(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    if points.is_empty() {
        return vec![];
    }
    let mut rings = Vec::new();
    for reverse in [false, true] {
        let mut p = points.to_vec();
        if reverse {
            p.reverse();
        }
        for i in 0..p.len() {
            rings.push(p[i..].iter().chain(&p[..i]).copied().collect::<Vec<_>>());
        }
    }
    rings.sort_by(|a, b| {
        for (p, q) in a.iter().zip(b) {
            let c = p[0].total_cmp(&q[0]).then(p[1].total_cmp(&q[1]));
            if !c.is_eq() {
                return c;
            }
        }
        a.len().cmp(&b.len())
    });
    rings.remove(0)
}
pub fn request(c: &Value) -> Request {
    serde_json::from_value(json!({"head":c["head"],"min_distance":c["min_distance"],"small_head":c["small_head"],"candidates":c["candidates"]})).unwrap()
}
pub fn observe(o: &Value) -> Value {
    let beams: Vec<Beam> = serde_json::from_value(o["beams"].clone()).unwrap();
    let groups: Vec<Group> = serde_json::from_value(o["groups"].clone()).unwrap();
    let catalog = Catalog::new(&beams, &groups).unwrap();
    let mut cases = Vec::new();
    for (i, c) in o["cases"].as_array().unwrap().iter().enumerate() {
        let s = catalog
            .select(&request(c))
            .unwrap_or_else(|e| panic!("case{i}: {e}"));
        assert_eq!(
            json!(s.selected_beams),
            c["selected_beams"],
            "case{i} beams"
        );
        assert_eq!(
            json!(s.selected_groups),
            c["selected_groups"],
            "case{i} groups"
        );
        assert_eq!(
            json!(s.target_beam),
            c["target_beam"],
            "case{i} target beam"
        );
        assert_eq!(json!(s.initial_line), c["initial_line"], "case{i} initial");
        assert_eq!(json!(s.target), c["target"], "case{i} target");
        assert_eq!(
            json!(s.area.theoretical_line),
            c["area_line"],
            "case{i} area line"
        );
        let mut ring = Vec::new();
        let mut closed = false;
        for p in c["area_path"].as_array().unwrap() {
            assert!(!closed);
            match p["type"].as_u64().unwrap() {
                0 | 1 => ring.push(serde_json::from_value(p["coordinates"].clone()).unwrap()),
                4 => closed = true,
                _ => panic!("unobserved target path"),
            }
        }
        assert_eq!(
            canonical(&s.area.polygon),
            canonical(&ring),
            "case{i} area edges"
        );
        cases.push(serde_json::to_value(s).unwrap());
    }
    let mut boundaries = Vec::new();
    for (i, c) in o["boundaries"].as_array().unwrap().iter().enumerate() {
        let l = catalog
            .lookup(
                serde_json::from_value(c["reference"].clone()).unwrap(),
                c["direction"].as_i64().unwrap() as i8,
                c["min_distance"].as_i64().unwrap() as i32,
                &serde_json::from_value::<Vec<usize>>(c["candidates"].clone()).unwrap(),
                0.,
            )
            .unwrap();
        assert_eq!(json!(l.selected_beams), c["selected_beams"], "boundary{i}");
        assert_eq!(
            json!(l.selected_groups),
            c["selected_groups"],
            "boundary{i}"
        );
        boundaries.push(serde_json::to_value(l).unwrap());
    }
    json!({"cases":cases,"boundaries":boundaries,"image_neighborhood_selection_complete":false,"page_cache_SIG_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false})
}
