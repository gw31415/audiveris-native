// SPDX-License-Identifier: AGPL-3.0-or-later
// Development observer: evaluate original query coordinates on native page IR.
use audiveris_native::native_rest_mapping::SystemMapping;
use serde_json::{Value, json};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 4);
    assert!(!std::path::Path::new(&args[3]).exists());
    let native: Value = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let fixture: Value = serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let mut systems = Vec::new();
    for s in fixture["systems"].as_array().unwrap() {
        let mapping = SystemMapping::from_grid_with_main(
            &native["grid_ir"],
            fixture["width"].as_u64().unwrap() as usize,
            fixture["height"].as_u64().unwrap() as usize,
            s["system_id"].as_u64().unwrap(),
            fixture["main_interline"].as_u64().unwrap() as usize,
        )
        .unwrap();
        let mut queries = Vec::new();
        for q in s["queries"].as_array().unwrap() {
            let x = q["x"].as_f64().unwrap();
            let y = q["y"].as_f64().unwrap();
            queries.push(json!({"x":x,"y":y,"closest":mapping.closest_staff(x,y).unwrap().map(|i|mapping.staffs[i].id),"above":mapping.staff_at_or_above(x,y).unwrap().map(|i|mapping.staffs[i].id),"pitches":mapping.staffs.iter().map(|staff|json!({"staff_id":staff.id,"pitch":staff.pitch_at(x,y).unwrap()})).collect::<Vec<_>>()}));
        }
        systems.push(json!({"system_id":s["system_id"],"staffs":mapping.staffs.iter().map(|staff|json!({"id":staff.id,"specific_interline":staff.interline})).collect::<Vec<_>>(),"queries":queries}));
    }
    std::fs::write(
        &args[3],
        serde_json::to_vec_pretty(&json!({"systems":systems})).unwrap(),
    )
    .unwrap();
}
