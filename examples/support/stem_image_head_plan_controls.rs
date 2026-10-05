// SPDX-License-Identifier: AGPL-3.0-or-later
use serde_json::Value;
use std::collections::BTreeMap;
pub fn compare(o: &Value, n: &Value) {
    assert_eq!(o["corners"], n["corner_count"]);
    assert_eq!(
        o["systems"].as_array().unwrap().len(),
        n["systems"].as_array().unwrap().len()
    );
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        let c = &b["image_context"];
        assert_eq!(a["system"], b["system"]);
        assert_eq!(a["parts"], c["parts"]);
        assert_eq!(a["profile"], c["profile"]);
        let mut beams = BTreeMap::new();
        for v in c["beams"].as_array().unwrap() {
            let found = a["beams"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|x| {
                    ["median", "top", "bottom", "height"]
                        .iter()
                        .all(|k| x[k] == v[k])
                })
                .collect::<Vec<_>>();
            assert_eq!(found.len(), 1, "physical beam {v}");
            beams.insert(v["id"].as_u64().unwrap(), found[0]["id"].as_u64().unwrap());
        }
        assert_eq!(beams.len(), a["beams"].as_array().unwrap().len());
        for v in c["beam_bounds"].as_array().unwrap() {
            let found = a["beam_bounds"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == beams[&v["id"].as_u64().unwrap()])
                .unwrap();
            assert_eq!(found["bounds"], v["bounds"]);
        }
        let mut groups = BTreeMap::new();
        for v in c["groups"].as_array().unwrap() {
            let mut members = v["members"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| beams[&x.as_u64().unwrap()])
                .collect::<Vec<_>>();
            members.sort();
            let found = a["groups"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|x| {
                    let mut m = x["members"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap())
                        .collect::<Vec<_>>();
                    m.sort();
                    m == members
                })
                .collect::<Vec<_>>();
            assert_eq!(found.len(), 1);
            groups.insert(v["id"].as_u64().unwrap(), found[0]["id"].as_u64().unwrap());
        }
        assert_eq!(groups.len(), a["groups"].as_array().unwrap().len());
        let key = |x: &Value| {
            serde_json::to_string(&(&x["head_bounds"], &x["shape"], &x["side"], &x["vertical"]))
                .unwrap()
        };
        let expected = a["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| (key(x), x))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(expected.len(), a["cases"].as_array().unwrap().len());
        assert_eq!(expected.len(), b["cases"].as_array().unwrap().len());
        for (meta, p) in c["corners"]
            .as_array()
            .unwrap()
            .iter()
            .zip(b["cases"].as_array().unwrap())
        {
            let e = expected[&key(meta)];
            let t = &p["target"];
            assert_eq!(e["head"]["reference"], meta["reference"]);
            assert_eq!(e["staff_id"], meta["staff_id"]);
            for (k, value, map) in [
                ("neighbors", &p["neighbors"], &beams),
                ("candidates", &p["candidates"], &beams),
                ("selected_beams", &t["selected_beams"], &beams),
                ("selected_groups", &t["selected_groups"], &groups),
            ] {
                let ids = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| map[&v.as_u64().unwrap()])
                    .collect::<Vec<_>>();
                assert_eq!(e[k], serde_json::json!(ids), "{k} {meta}");
            }
            let target = t["target_beam"].as_u64().map(|x| beams[&x]);
            assert_eq!(e["target_beam"], serde_json::json!(target));
            assert_eq!(e["initial_line"], t["initial_line"]);
            assert_eq!(e["target"], t["target"]);
            assert_eq!(e["area_line"], t["area"]["theoretical_line"]);
            let mut path = e["area_path"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| matches!(v["type"].as_u64(), Some(0 | 1)))
                .map(|v| serde_json::from_value::<[f64; 2]>(v["coordinates"].clone()).unwrap())
                .collect::<Vec<_>>();
            let mut native: Vec<[f64; 2]> =
                serde_json::from_value(t["area"]["polygon"].clone()).unwrap();
            let order =
                |a: &[f64; 2], b: &[f64; 2]| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1]));
            path.sort_by(order);
            native.sort_by(order);
            assert_eq!(path, native);
        }
    }
    assert_eq!(n["image_head_target_selection_complete"], true);
    for k in [
        "automatic_raster_pipeline_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(n[k], false);
    }
}
