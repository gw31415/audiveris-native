// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::system_area_ends::{discover, native_grid_ends};
use serde_json::{Value, json};
#[test]
fn original_horizontal_neighbor_and_area_end_controls_match() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-horizontal-system-ends.json"
    ))
    .unwrap();
    assert_eq!(f["native_outputs_used"], false);
    for c in f["configurations"].as_array().unwrap() {
        let systems = c["input"]["grid"]["systems"].as_array().unwrap();
        let w = c["input"]["width"].as_u64().unwrap() as usize;
        assert_eq!(
            discover(systems, w).unwrap(),
            c["expected"].as_array().unwrap().clone(),
            "{}",
            c["name"]
        );
        let horizontal = c["expected"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| !s["left_neighbor"].is_null() || !s["right_neighbor"].is_null());
        if horizontal {
            assert_eq!(
                native_grid_ends(systems, w).err().unwrap(),
                "unsupported_grid_horizontal_staff_chains"
            );
        } else {
            assert_eq!(
                native_grid_ends(systems, w).unwrap(),
                discover(systems, w).unwrap()
            );
        }
    }
}
#[test]
fn malformed_and_duplicate_native_system_coordinates_reject() {
    let s = json!({"id":1,"coordinates":[10,20,30,60,0]});
    assert!(discover(&[], 100).is_err());
    assert!(discover(&[s.clone(), s.clone()], 100).is_err());
    for coords in [
        json!([10, 20, 0, 60, 0]),
        json!([10, 60, 30, 20, 0]),
        json!([-1, 20, 30, 60, 0]),
        json!([10, 20, 200, 60, 0]),
    ] {
        let mut v = s.clone();
        v["coordinates"] = coords;
        assert!(discover(&[v], 100).is_err());
    }
}
