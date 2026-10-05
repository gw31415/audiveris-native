// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::stem_head_neighborhood::{self, Bounds, Neighborhood, Request};
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
pub fn request(o: &Value) -> Value {
    json!({"schema":stem_head_neighborhood::REQUEST_SCHEMA,"systems":o["systems"].as_array().unwrap().iter().map(|s|json!({"system":s["system"],"system_bounds":s["system_bounds"],"vicinity_margin":s["vicinity_margin"],"beam_ids":s["ordered_beams"],"beams":s["beams"],"beam_bounds":s["beam_bounds"],"groups":s["groups"],"cases":s["cases"].as_array().unwrap().iter().map(|c|json!({"head_bounds":c["head_bounds"],"head":c["head"],"min_distance":c["min_distance"],"small_head":c["small_head"]})).collect::<Vec<_>>()})).collect::<Vec<_>>()})
}
pub fn compare(o: &Value, r: &Value) {
    let systems = o["systems"].as_array().unwrap();
    assert_eq!(systems.len(), r["systems"].as_array().unwrap().len());
    for (s, n) in systems.iter().zip(r["systems"].as_array().unwrap()) {
        assert_eq!(s["system"], n["system"]);
        assert_eq!(
            s["cases"].as_array().unwrap().len(),
            n["cases"].as_array().unwrap().len()
        );
        for (i, (c, p)) in s["cases"]
            .as_array()
            .unwrap()
            .iter()
            .zip(n["cases"].as_array().unwrap())
            .enumerate()
        {
            assert_eq!(c["neighbors"], p["neighbors"], "case{i} neighbors");
            assert_eq!(
                c["candidates"], p["candidates"],
                "case{i} initial candidates"
            );
            let t = &p["target"];
            for k in [
                "selected_beams",
                "selected_groups",
                "target_beam",
                "initial_line",
                "target",
            ] {
                assert_eq!(c[k], t[k], "case{i} {k}");
            }
            assert_eq!(c["area_line"], t["area"]["theoretical_line"]);
            let points = c["area_path"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|p| match p["type"].as_u64().unwrap() {
                    0 | 1 => Some(serde_json::from_value(p["coordinates"].clone()).unwrap()),
                    4 => None,
                    _ => panic!("uncertified area"),
                })
                .collect::<Vec<[f64; 2]>>();
            let native: Vec<[f64; 2]> =
                serde_json::from_value(t["area"]["polygon"].clone()).unwrap();
            assert_eq!(canonical(&points), canonical(&native), "case{i} area");
        }
    }
    for k in [
        "automatic_raster_pipeline_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(r[k], false);
    }
}
pub fn observe(o: &Value) -> Value {
    let r: Request = serde_json::from_value(request(o)).unwrap();
    let report = serde_json::to_value(stem_head_neighborhood::evaluate(&r).unwrap()).unwrap();
    compare(o, &report);
    let mut boundaries = Vec::new();
    for s in o["systems"].as_array().unwrap() {
        let bounds: Vec<Bounds> = serde_json::from_value(s["beam_bounds"].clone()).unwrap();
        for c in s["rectangle_boundaries"].as_array().unwrap() {
            let mut b = bounds.clone();
            if c["removed_override"] == true {
                let id = c["ordered_beams"][0].as_u64().unwrap() as usize;
                b.iter_mut().find(|b| b.id == id).unwrap().removed = true;
            }
            let n = Neighborhood::new(&b).unwrap();
            let ids = serde_json::from_value::<Vec<usize>>(c["ordered_beams"].clone()).unwrap();
            let found = n
                .neighbors(
                    &ids,
                    serde_json::from_value(c["head_bounds"].clone()).unwrap(),
                    serde_json::from_value(s["system_bounds"].clone()).unwrap(),
                    s["vicinity_margin"].as_i64().unwrap() as i32,
                )
                .unwrap();
            assert_eq!(json!(found), c["neighbors"]);
            boundaries.push(json!({"neighbors":found}));
        }
    }
    json!({"report":report,"rectangle_boundaries":boundaries})
}
