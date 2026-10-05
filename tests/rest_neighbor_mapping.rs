// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{native_rest_mapping::SystemMapping, system_area_geometry};
use serde_json::{Value, json};
#[test]
fn original_horizontal_closest_above_and_certified_bounds_match() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-rest-neighbor-areas.json"
    ))
    .unwrap();
    assert_eq!(f["native_outputs_used"], false);
    let mut positive = 0;
    let mut absent = 0;
    let mut supported = 0;

    for c in f["configurations"].as_array().unwrap() {
        let input = &c["input"];
        let mut grid = input["grid"].clone();
        grid["schema"] = json!("audiveris-native.grid-ir.v1");
        for staff in grid["staffs"].as_array_mut().unwrap() {
            staff["specific_interline"] = json!(11);
        }
        grid["vlag"] = json!([]);
        grid["hlag"] = json!([]);
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
        let w = input["width"].as_u64().unwrap() as usize;
        let h = input["height"].as_u64().unwrap() as usize;
        let layout = system_area_geometry::build(&grid, w, h, 11).unwrap();
        for (index, sys) in c["systems"].as_array().unwrap().iter().enumerate() {
            let sid = sys["id"].as_u64().unwrap();
            let Ok(bounds) = layout.system_areas[index].bounds() else {
                assert_eq!(
                    SystemMapping::from_grid(&grid, w, h, sid).err().unwrap(),
                    "unsupported_rest_native_system_area_bounds"
                );
                continue;
            };
            assert_eq!(json!(bounds), sys["bounds"], "{} system {}", c["name"], sid);
            let m = SystemMapping::from_grid(&grid, w, h, sid).unwrap();
            supported += 1;
            for q in sys["selections"].as_array().unwrap() {
                let x = q["x"].as_f64().unwrap();
                let y = q["y"].as_f64().unwrap();
                let id = |n: Option<usize>| n.map(|i| m.staffs[i].id).unwrap_or(0);
                let closest = id(m.closest_staff(x, y).unwrap());
                assert_eq!(
                    closest,
                    q["closest"].as_u64().unwrap(),
                    "{} {sid} {q}",
                    c["name"]
                );
                assert_eq!(
                    id(m.staff_at_or_above(x, y).unwrap()),
                    q["above"].as_u64().unwrap(),
                    "{} {sid} {q}",
                    c["name"]
                );
                if closest == 0 {
                    absent += 1
                } else {
                    positive += 1
                }
            }
            let mut bad = grid.clone();
            bad["systems"][index]["area"]["north_staff_ids"] = json!([u64::MAX]);
            assert!(SystemMapping::from_grid(&bad, w, h, sid).is_err());
            let mut bad = grid.clone();
            bad["schema"] = json!("unknown");
            assert!(SystemMapping::from_grid(&bad, w, h, sid).is_err());
            assert!(m.closest_staff(f64::NAN, 0.).is_err());
        }
    }
    assert!(positive > 0 && absent > 0 && supported > 0);
}

#[test]
fn certified_curved_clipped_system_bounds_enable_native_rest_mapping() {
    let mut grid = json!({"schema":"audiveris-native.grid-ir.v1","staffs":[],"systems":[],"vlag":[],"hlag":[]});
    for (id, left, right, top, bottom) in [
        (1, 20, 140, 40, 84),
        (2, 180, 300, 40, 84),
        (3, 20, 140, 220, 264),
    ] {
        let lines=(0..5).map(|n|json!({"points":if id==3 {json!([[left,top+11*n],[(left+right)/2,top+11*n+4],[right,top+11*n]])}else{json!([[left,top+11*n],[right,top+11*n]])},"thickness":1.})).collect::<Vec<_>>();
        grid["staffs"].as_array_mut().unwrap().push(json!({"id":id,"system_id":id,"specific_interline":11,"left":left,"right":right,"lines":lines,"ledger_lines":[]}));
        grid["systems"].as_array_mut().unwrap().push(json!({"id":id,"staff_ids":[id],"coordinates":[left,top,right-left+1,bottom,bottom-top]}));
    }
    let layout = system_area_geometry::build(&grid, 320, 400, 11).unwrap();
    for (sys, plan) in grid["systems"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(&layout.system_neighbors)
    {
        sys["area"] = json!({"x_range":plan["x_range"],"vertical_margin":10.});
        for (field, key) in [
            ("north_neighbor_last_line", "north_staff_ids"),
            ("south_neighbor_first_line", "south_staff_ids"),
        ] {
            let ids = plan[key].as_array().unwrap();
            sys["area"][field] = if ids.len() == 1 {
                ids[0].clone()
            } else {
                Value::Null
            };
            if ids.len() > 1 {
                sys["area"][key] = plan[key].clone();
            }
        }
    }
    let mapping = SystemMapping::from_grid(&grid, 320, 400, 1).unwrap();
    assert_eq!(mapping.closest_staff(60., 60.).unwrap(), Some(0));
    assert_eq!(mapping.closest_staff(160., 60.).unwrap(), None);
}
