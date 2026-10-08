// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use serde_json::{Value, json};
fn frame(page: &crate::page_runtime::OwnedPage) -> Value {
    let mut systems = Vec::new();
    for system in page
        .staffs
        .iter()
        .map(|s| s.system)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let mut names = BTreeMap::<Key, usize>::new();
        let mut bodies = Vec::new();
        let mut maps = Vec::new();
        for map in page
            .ledger_memberships
            .maps
            .iter()
            .filter(|m| m.staff.system == system)
        {
            for members in map.lines.values() {
                for key in members {
                    if !names.contains_key(key) {
                        let label = names.len() + 1;
                        names.insert(*key, label);
                        let (n, removed) = page.graph.retained_inter(*key).unwrap();
                        bodies.push(json!({"label":label,"bounds":n["bounds"],"index":n["index"],"owner_staff":n["staff_id"],"removed":removed,"glyph":crate::beam_dispatch::glyph_json(&crate::beam_dispatch::glyph_from_json(&n["glyph"]).unwrap())}));
                    }
                }
            }
        }
        for map in page
            .ledger_memberships
            .maps
            .iter()
            .filter(|m| m.staff.system == system)
        {
            maps.push(json!({"staff":map.staff.id,"lines":map.lines.iter().map(|(index,members)|json!({"index":index,"members":members.iter().map(|k|names[k]).collect::<Vec<_>>()})).collect::<Vec<_>>()}));
        }
        systems.push(json!({"system":system,"bodies":bodies,"maps":maps}));
    }
    json!(systems)
}
#[test]
fn real_image_constructor_post_analysis_memberships_match_original_before_stems_and_keep_actual_references()
 {
    let original: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ledger-memberships/observed.json"
    ))
    .unwrap();
    assert_eq!(original["ledger_stage"], original["head_stage"]);
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-grid/laura-gray.png"
    ))
    .unwrap();
    let mut page = crate::page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    let graph = page.graph.snapshot();
    page.ledger_memberships.validate(&page.graph).unwrap();
    assert_eq!(frame(&page), original["head_stage"]);
    for map in &page.ledger_memberships.maps {
        assert!(page.staffs.iter().any(|s| Arc::ptr_eq(s, &map.staff)));
    }
    // Corrupting the observation cannot replace registered body or membership owners.
    page.report["ledger_post_analysis"] = json!({"systems":[]});
    page.report["grid_ir"]["staffs"] = json!([]);
    assert_eq!(frame(&page), original["head_stage"]);
    assert_eq!(page.graph.snapshot(), graph);
}

#[test]
fn original_shared_duplicate_and_retired_ledger_references_keep_different_map_indices_and_last_staff_only_removal()
 {
    let original: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ledger-memberships/references.json"
    ))
    .unwrap();
    let staff = |id| {
        Arc::new(crate::grid_ir::StaffGeometry {
            abscissae: [0, 500],
            id,
            system: 1,
            lines: vec![Arc::new(
                crate::curves::Curve::interpolate(vec![[0., 0.], [500., 0.]]).unwrap(),
            )],
        })
    };
    let staffs = vec![staff(2), staff(3)];
    let mut owners = LedgerMemberships::default();
    let mut graph = Graph::default();
    let key = Key {
        system: 1,
        phase: Phase::Ledger,
        local: 1,
    };
    graph.vertex(key,json!({"kind":"LedgerInter","shape":"LEDGER","index":1,"staff_id":3,"bounds":[350,572,16,2],"grade":0.8,"manual":false}),false).unwrap();
    let state = |owners: &LedgerMemberships, graph: &Graph| {
        let (n, removed) = graph.retained_inter(key).unwrap();
        json!({"maps":owners.maps.iter().map(|m|json!({"staff":m.staff.id,"lines":m.lines.iter().map(|(i,ids)|json!({"index":i,"members":ids.iter().map(|k|{assert_eq!(*k,key);1}).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"index":n["index"],"owner_staff":n["staff_id"],"removed":removed,"registered":!removed})
    };
    owners
        .record(
            1,
            StaffMembership {
                staff_id: 2,
                lines: vec![(1, vec![1])],
            },
            &staffs,
        )
        .unwrap();
    owners
        .record(
            1,
            StaffMembership {
                staff_id: 3,
                lines: vec![(-6, vec![1])],
            },
            &staffs,
        )
        .unwrap();
    assert_eq!(state(&owners, &graph), original["frames"][0]["state"]);
    owners
        .record(
            1,
            StaffMembership {
                staff_id: 2,
                lines: vec![(1, vec![1, 1])],
            },
            &staffs,
        )
        .unwrap();
    assert_eq!(state(&owners, &graph), original["frames"][1]["state"]);
    let before = state(&owners, &graph);
    assert!(
        owners
            .record(
                1,
                StaffMembership {
                    staff_id: 2,
                    lines: vec![(1, vec![1])]
                },
                &[staff(2), staffs[1].clone()]
            )
            .is_err()
    );
    assert_eq!(state(&owners, &graph), before);
    owners.remove_from_owner(key, None);
    assert_eq!(state(&owners, &graph), before);
    owners.remove_from_owner(key, Some(3));
    graph.remove_music_inter(key).unwrap();
    owners.validate(&graph).unwrap();
    assert_eq!(state(&owners, &graph), original["frames"][2]["state"]);
    assert!(Arc::ptr_eq(&owners.maps[0].staff, &staffs[0]));
    assert!(Arc::ptr_eq(&owners.maps[1].staff, &staffs[1]));
    assert_eq!(graph.id(key), Some(1));
}
