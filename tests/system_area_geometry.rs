// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::system_area_geometry;
use serde_json::Value;
#[test]
fn original_horizontal_chains_vertical_neighbors_and_area_points_match() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-neighbor-system-area.json"
    ))
    .unwrap();
    assert_eq!(f["native_outputs_used"], false);
    for c in f["configurations"].as_array().unwrap() {
        let i = &c["input"];
        let m = system_area_geometry::build(
            &i["grid"],
            i["width"].as_u64().unwrap() as usize,
            i["height"].as_u64().unwrap() as usize,
            11,
        )
        .unwrap();
        for (kind, plans, areas) in [
            ("systems", &m.system_neighbors, &m.system_areas),
            ("staffs", &m.staff_neighbors, &m.staff_areas),
        ] {
            let expected = c[kind].as_array().unwrap();
            assert_eq!(plans.len(), expected.len());
            for ((p, a), e) in plans.iter().zip(areas).zip(expected) {
                let mut plan = e.clone();
                plan.as_object_mut().unwrap().remove("cases");
                assert_eq!(*p, plan, "{} {kind}", c["name"]);
                for q in e["cases"].as_array().unwrap() {
                    assert_eq!(
                        a.contains([q["x"].as_f64().unwrap(), q["y"].as_f64().unwrap()])
                            .unwrap(),
                        q["contains"].as_bool().unwrap(),
                        "{} {kind} {q}",
                        c["name"]
                    );
                }
            }
        }
    }
}

#[test]
fn incomplete_membership_and_nonfinite_points_reject() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-neighbor-system-area.json"
    ))
    .unwrap();
    let mut g = f["configurations"][0]["input"]["grid"].clone();
    let m = system_area_geometry::build(&g, 1000, 1000, 11).unwrap();
    assert!(m.system_areas[0].contains([f64::NAN, 40.]).is_err());
    g["systems"][0]["staff_ids"] = serde_json::json!([999]);
    assert!(system_area_geometry::build(&g, 1000, 1000, 11).is_err());
    assert!(system_area_geometry::build(&g, 0, 1000, 11).is_err());
}
