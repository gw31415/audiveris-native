// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_groups::{self, GroupBeam},
    beam_structure::BeamItem,
};
use serde_json::Value;
#[test]
fn original_uninterrupted_group_controls() {
    let v: Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-groups.json")).unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 103);
    for (index, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let beams = c["beams"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, b)| GroupBeam {
                id: i + 1,
                bounds: std::array::from_fn(|n| b["bounds"][n].as_i64().unwrap() as i32),
                item: BeamItem {
                    median: std::array::from_fn(|n| b["median"][n].as_f64().unwrap()),
                    height: b["height"].as_f64().unwrap(),
                },
            })
            .collect::<Vec<_>>();
        let exclusions = c["input"]["exclusions"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|e| {
                        (
                            e[0].as_u64().unwrap() as usize,
                            e[1].as_u64().unwrap() as usize,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let got = beam_groups::populate(
            &beams,
            &exclusions,
            c["input"]["interline"].as_u64().unwrap_or(16) as usize,
        )
        .unwrap();
        let groups = got
            .groups
            .iter()
            .map(|g| serde_json::json!({"members":g.members,"bounds":g.bounds}))
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::json!(groups),
            c["groups"],
            "case {index} groups"
        );
        assert_eq!(
            serde_json::json!(got.supports),
            c["supports"],
            "case {index} supports"
        );
    }
}
#[test]
fn rejects_invalid_before_grouping() {
    assert!(beam_groups::populate(&[], &[], 0).is_err());
    let b = GroupBeam {
        id: 1,
        bounds: [0, 0, 10, 8],
        item: BeamItem {
            median: [0., 4., 10., 4.],
            height: 8.,
        },
    };
    assert!(beam_groups::populate(&[b.clone(), b], &[], 16).is_err());
}

#[test]
fn original_group_removal_preserves_survivors_and_incident_support_order() {
    let v: Value =
        serde_json::from_str(include_str!("../fixtures/official-beam-group-cleanup.json")).unwrap();
    assert_eq!(v["native_outputs_used"], false);
    for (i, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let beams = c["beams"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(n, b)| GroupBeam {
                id: n + 1,
                bounds: std::array::from_fn(|n| b["bounds"][n].as_i64().unwrap() as i32),
                item: BeamItem {
                    median: std::array::from_fn(|n| b["median"][n].as_f64().unwrap()),
                    height: b["height"].as_f64().unwrap(),
                },
            })
            .collect::<Vec<_>>();
        let pass = beam_groups::populate(&beams, &[], 16).unwrap();
        let removed = c["input"]["remove"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let got = beam_groups::remove_members(&pass, &beams, &removed).unwrap();
        let groups = got
            .groups
            .iter()
            .map(|g| serde_json::json!({"members":g.members,"bounds":g.bounds}))
            .collect::<Vec<_>>();
        assert_eq!(serde_json::json!(groups), c["groups"], "case {i}");
        assert_eq!(serde_json::json!(got.supports), c["supports"], "case {i}");
    }
}
