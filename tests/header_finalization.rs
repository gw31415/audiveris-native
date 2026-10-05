// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::header_finalization::finalize_system;
use serde_json::{Value, json};
fn fixture() -> (Value, Value) {
    let grid = json!({"bar_semantics":{"left_end_peaks":[[1,30,31]],"systems":[{"inters":[]}]}});
    let nodes = json!([
    {"id":1,"kind":"ClefInter","staff_id":1,"bounds":[20,20,10,40],"grade":0.6,"contextual_grade":null},
    {"id":2,"kind":"ClefInter","staff_id":1,"bounds":[20,20,10,40],"grade":0.5,"contextual_grade":null},
    {"id":3,"kind":"KeyInter","staff_id":1,"bounds":[40,20,10,40],"grade":0.8,"members":[]},
    {"id":4,"kind":"KeyAlterInter","staff_id":1,"bounds":[40,20,10,40],"grade":0.8,"frozen":false},
    {"id":5,"kind":"BarlineInter","staff_id":1,"bounds":[10,20,1,40],"source_peak":[1,10,10]},
    {"id":6,"kind":"BarlineInter","staff_id":1,"bounds":[70,20,1,40],"source_peak":[1,70,70]},
    {"id":7,"kind":"BarlineInter","staff_id":1,"bounds":[30,20,2,40],"source_peak":[1,30,31]},
    {"id":8,"kind":"BarlineInter","staff_id":1,"bounds":[50,20,2,40],"source_peak":[1,50,51]}
    ]);
    let post = json!({"staffs":[{"staff_id":1,"header_start":10,"header_stop":70,"clef_stop":30,"header_key":{"id":3},"header_time":null}],"sig_inters":nodes,"relations":[{"kind":"Exclusion","source":1,"target":2},{"kind":"ClefKeyRelation","source":2,"target":3},{"kind":"Containment","source":3,"target":4}]});
    (grid, post)
}
#[test]
fn context_selects_supported_clef_and_freezes_key_members() {
    let (g, p) = fixture();
    let a = finalize_system(&g, &p, 10).unwrap();
    assert_eq!(a["staffs"][0]["clef"]["id"], 2);
    assert_eq!(a["staffs"][0]["clef"]["contextual_grade"], json!(2.5 / 3.0));
    assert_eq!(a["removed_clef_ids"], json!([1]));
    assert_eq!(a["frozen_ids"], json!([2, 3, 4]));
    assert_eq!(a["staffs"][0]["key"]["members"][0]["frozen"], true);
    assert!(p["sig_inters"][1]["contextual_grade"].is_null());
}
#[test]
fn purge_uses_strict_integer_center_boundary_and_left_end_protection() {
    let (g, p) = fixture();
    let a = finalize_system(&g, &p, 10).unwrap();
    assert_eq!(a["purged_barline_ids"], json!([8]));
    for id in [5, 6, 7] {
        assert!(
            a["sig_inters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["id"] == id)
        );
    }
}
#[test]
fn rejects_dangling_relations_without_mutating_input() {
    let (g, mut p) = fixture();
    p["relations"][0]["target"] = json!(999);
    assert!(finalize_system(&g, &p, 10).is_err());
    assert!(p["sig_inters"][0]["contextual_grade"].is_null());
}

#[test]
fn contextual_partition_maximizes_each_rounded_grade_not_contribution() {
    let controls: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/context-partition-controls.json"
    ))
    .unwrap();
    let cases = controls["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let mut nodes = vec![
            json!({"id":1,"kind":"ClefInter","staff_id":1,"bounds":[20,20,10,40],"grade":case["clef_grade"],"contextual_grade":null}),
        ];
        let mut relations = Vec::new();
        for (i, g) in case["key_grades"].as_array().unwrap().iter().enumerate() {
            nodes.push(json!({"id":i+2,"kind":"KeyInter","staff_id":1,"bounds":[40,20,10,40],"grade":g,"members":[]}));
            relations.push(json!({"kind":"ClefKeyRelation","source":1,"target":i+2}));
        }
        for e in case["exclusions"].as_array().unwrap() {
            relations.push(json!({"kind":"Exclusion","source":e[0].as_u64().unwrap()+2,"target":e[1].as_u64().unwrap()+2}));
        }
        let g = json!({"bar_semantics":{"left_end_peaks":[],"systems":[]}});
        let p = json!({"staffs":[{"staff_id":1,"header_start":10,"header_stop":70,"clef_stop":30,"header_key":null,"header_time":null}],"sig_inters":nodes,"relations":relations});
        let a = finalize_system(&g, &p, 10).unwrap();
        assert_eq!(
            a["staffs"][0]["clef"]["contextual_grade"]
                .as_f64()
                .unwrap()
                .to_bits(),
            case["original_final_contextual_grade"]
                .as_f64()
                .unwrap()
                .to_bits(),
            "{}",
            case["name"]
        );
    }
}
#[test]
fn malformed_header_range_and_member_bounds_are_errors_not_panics() {
    let (g, mut p) = fixture();
    p["staffs"][0]["header_start"] = json!(i64::MAX);
    assert!(finalize_system(&g, &p, 10).is_err());
    let (g, mut p) = fixture();
    p["sig_inters"][3]["bounds"] = json!([0]);
    assert!(finalize_system(&g, &p, 10).is_err());
}
#[test]
fn rejects_duplicate_staffs_and_empty_complete_pipeline() {
    let (g, mut p) = fixture();
    let staff = p["staffs"][0].clone();
    p["staffs"].as_array_mut().unwrap().push(staff);
    assert!(finalize_system(&g, &p, 10).is_err());
    assert!(
        audiveris_native::header_finalization::analyze(
            &g,
            &json!({"time_column_complete":true,"systems":[]}),
            10
        )
        .is_err()
    );
}
