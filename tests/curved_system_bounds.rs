// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{native_rest_mapping::SystemMapping, shared_sig, system_area_geometry};
use serde_json::{Value, json};
#[test]
fn original_curved_clipped_bounds_selection_and_lag_rectangles_match() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-curved-system-bounds.json"
    ))
    .unwrap();
    assert_eq!(f["native_outputs_used"], false);
    let mut accepted = 0;
    let mut rejected = 0;
    let mut positive = 0;
    let mut absent = 0;
    for c in f["configurations"].as_array().unwrap() {
        let mut grid = c["input"]["grid"].clone();
        grid["schema"] = json!("audiveris-native.grid-ir.v1");
        for (sys, oracle) in grid["systems"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(c["systems"].as_array().unwrap())
        {
            sys["area"] = json!({"x_range":oracle["x_range"],"vertical_margin":10.});
            for (field, key) in [
                ("north_neighbor_last_line", "north_staff_ids"),
                ("south_neighbor_first_line", "south_staff_ids"),
            ] {
                let ids = oracle[key].as_array().unwrap();
                sys["area"][field] = if ids.len() == 1 {
                    ids[0].clone()
                } else {
                    Value::Null
                };
                if ids.len() > 1 {
                    sys["area"][key] = oracle[key].clone();
                }
            }
        }
        let w = 320;
        let h = 400;
        let layout = system_area_geometry::build(&grid, w, h, 11).unwrap();
        let oracle = &c["systems"][0];
        let name = c["name"].as_str().unwrap();
        let unsupported = name == "crosses-canvas" || name == "non-monotone-controls";
        if unsupported {
            rejected += 1;
            assert!(layout.system_areas[0].bounds().is_err());
            assert!(layout.system_areas[0].header_left().is_err());
            assert_eq!(
                SystemMapping::from_grid(&grid, w, h, 1).err().unwrap(),
                "unsupported_rest_native_system_area_bounds"
            );
            continue;
        }
        accepted += 1;
        assert_eq!(
            json!(layout.system_areas[0].bounds().unwrap()),
            oracle["bounds"],
            "{name}"
        );
        assert_eq!(
            layout.system_areas[0].header_left().unwrap(),
            oracle["bounds"][0].as_u64().unwrap() as usize
        );
        grid["vlag"] = json!(
            oracle["section_rectangles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| json!({"id":p["id"],"first_pos":p["x"],"runs":[[p["y"],1]]}))
                .collect::<Vec<_>>()
        );
        grid["bar_semantics"] = json!({"systems":[{"id":1,"inters":[{"id":1,"kind":"StaffBarlineInter"},{"id":2,"kind":"StaffBarlineInter"}],"relations":[{"id":1,"source":1,"target":2,"kind":"BarConnectionRelation"}]}]});
        let mut graph = shared_sig::Graph::default();
        graph.import_grid(&grid).unwrap();
        let before = graph.snapshot();
        let mapping = SystemMapping::from_grid(&grid, w, h, 1).unwrap();
        let ids = mapping.sections.iter().map(|s| s.id.0).collect::<Vec<_>>();
        let expected = oracle["section_rectangles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["inside"] == true)
            .map(|p| p["id"].as_u64().unwrap() as u32)
            .collect::<Vec<_>>();
        assert_eq!(ids, expected, "{name}");
        assert!(!ids.is_empty());
        for p in oracle["selections"].as_array().unwrap() {
            let x = p["x"].as_f64().unwrap();
            let y = p["y"].as_f64().unwrap();
            let id = |n: Option<usize>| n.map(|i| mapping.staffs[i].id).unwrap_or(0);
            let closest = id(mapping.closest_staff(x, y).unwrap());
            assert_eq!(closest, p["closest"].as_u64().unwrap(), "{name} {p}");
            assert_eq!(
                id(mapping.staff_at_or_above(x, y).unwrap()),
                p["above"].as_u64().unwrap(),
                "{name} {p}"
            );
            if closest == 0 {
                absent += 1
            } else {
                positive += 1
            }
        }
        assert_eq!(graph.snapshot(), before);
    }
    assert_eq!(accepted, 6);
    assert_eq!(rejected, 2);
    assert!(positive > 0 && absent > 0);
}
#[test]
fn bounded_curve_clip_rejects_invalid_or_non_monotone_parameters() {
    use audiveris_area::monotone_clip_y_bounds;
    for p in [
        vec![],
        vec![[0., 0.]],
        vec![[0., 0.], [2., 0.], [1., 1.]],
        vec![[0., 0.], [f64::NAN, 1.]],
    ] {
        assert!(monotone_clip_y_bounds(&p, 0., 1.).is_err());
    }
    assert!(monotone_clip_y_bounds(&[[0., 0.], [2., 1.]], 1., 1.).is_err());
    assert!(monotone_clip_y_bounds(&[[0., 0.], [2., 1.]], 3., 4.).is_err());
    assert_eq!(
        monotone_clip_y_bounds(&[[0., 3.], [2., 3.]], 0.5, 1.5).unwrap(),
        [3., 3.]
    );
}
