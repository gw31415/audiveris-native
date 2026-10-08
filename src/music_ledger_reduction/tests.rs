// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{
    ledger_membership_owners::LedgerMemberships,
    ledger_registration::StaffMembership,
    shared_sig::{Graph, Phase},
};
use std::{collections::BTreeMap, sync::Arc};
fn staffs(ids: &[u64]) -> Vec<Arc<crate::grid_ir::StaffGeometry>> {
    ids.iter()
        .map(|id| {
            Arc::new(crate::grid_ir::StaffGeometry {
                id: *id,
                system: 1,
                lines: (0..5)
                    .map(|i| {
                        Arc::new(
                            crate::curves::Curve::interpolate(vec![
                                [0., i as f64 * 11.],
                                [500., i as f64 * 11.],
                            ])
                            .unwrap(),
                        )
                    })
                    .collect(),
            })
        })
        .collect()
}
fn frame(g: &Graph, o: &LedgerMemberships, names: &BTreeMap<Key, String>) -> Value {
    let mut bodies = names.iter().collect::<Vec<_>>();
    bodies.sort_by_key(|(_, name)| {
        name.strip_prefix("ledger")
            .unwrap()
            .parse::<usize>()
            .unwrap()
    });
    json!({"maps":o.maps.iter().map(|m|json!({"staff":m.staff.id,"lines":m.lines.iter().map(|(i,ks)|json!({"index":i,"members":ks.iter().map(|k|&names[k]).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"bodies":bodies.into_iter().map(|(k,name)|{let(n,r)=g.retained_inter(*k).unwrap();json!({"label":name,"removed":r,"registered":!r,"index":n["index"],"owner_staff":n["staff_id"]})}).collect::<Vec<_>>()})
}
#[test]
fn original_multistaff_map_indices_duplicate_retirement_next_body_and_head_contacts_match() {
    let original: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-ledger-reduction/observed.json"
    ))
    .unwrap();
    for c in original["cases"].as_array().unwrap() {
        let staffs = staffs(&[1, 2, 3]);
        let mut graph = Graph::default();
        let mut owners = LedgerMemberships::default();
        let mut names = BTreeMap::new();
        for (i, n) in c["nodes"].as_array().unwrap().iter().enumerate() {
            let key = Key {
                system: 1,
                phase: Phase::Ledger,
                local: i + 1,
            };
            let mut body = n.clone();
            body["kind"] = json!("LedgerInter");
            body["shape"] = json!("LEDGER");
            body["grade"] = json!(0.8);
            body["manual"] = json!(false);
            graph.vertex(key, body, false).unwrap();
            names.insert(key, n["label"].as_str().unwrap().to_owned());
            if n["removed"] == true {
                graph.remove_music_inter(key).unwrap();
            }
        }
        for m in c["before"]["maps"].as_array().unwrap() {
            owners
                .record(
                    1,
                    StaffMembership {
                        staff_id: m["staff"].as_u64().unwrap(),
                        lines: m["lines"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|l| {
                                (
                                    l["index"].as_i64().unwrap() as i32,
                                    l["members"]
                                        .as_array()
                                        .unwrap()
                                        .iter()
                                        .map(|s| {
                                            names
                                                .iter()
                                                .find(|(_, n)| s.as_str() == Some(n.as_str()))
                                                .unwrap()
                                                .0
                                                .local
                                        })
                                        .collect(),
                                )
                            })
                            .collect(),
                    },
                    &staffs,
                )
                .unwrap();
        }
        for (i, h) in c["heads"].as_array().unwrap().iter().enumerate() {
            let mut h = h.clone();
            h["shape"] = json!("NOTEHEAD_BLACK");
            h["grade"] = json!(0.8);
            graph
                .vertex(
                    Key {
                        system: 1,
                        phase: Phase::Head,
                        local: i + 1,
                    },
                    h,
                    false,
                )
                .unwrap();
        }
        assert_eq!(frame(&graph, &owners, &names), c["before"], "{}", c["name"]);
        let removed = reduce(
            &mut graph,
            &mut owners,
            &staffs,
            original["interline"].as_f64().unwrap(),
        )
        .unwrap();
        assert_eq!(
            removed.len() as u64,
            c["removed_count"].as_u64().unwrap(),
            "{}",
            c["name"]
        );
        assert_eq!(frame(&graph, &owners, &names), c["after"], "{}", c["name"]);
        assert_eq!(
            reduce(&mut graph, &mut owners, &staffs, 11.).unwrap().len() as u64,
            c["repeat_count"].as_u64().unwrap()
        );
        for (m, s) in owners.maps.iter().zip(&staffs) {
            assert!(Arc::ptr_eq(&m.staff, s));
        }
    }
}
#[test]
fn late_invalid_head_failure_keeps_actual_maps_bodies_and_references_for_retry() {
    let staffs = staffs(&[1]);
    let mut g = Graph::default();
    let mut owners = LedgerMemberships::default();
    for i in 1..=2 {
        g.vertex(Key{system:1,phase:Phase::Ledger,local:i},json!({"kind":"LedgerInter","shape":"LEDGER","bounds":[i*100,200,11,2],"index":1,"staff_id":1,"grade":0.8}),false).unwrap();
    }
    owners
        .record(
            1,
            StaffMembership {
                staff_id: 1,
                lines: vec![(1, vec![1, 2])],
            },
            &staffs,
        )
        .unwrap();
    let head = Key {
        system: 1,
        phase: Phase::Head,
        local: 1,
    };
    g.vertex(head,json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK","bounds":[205,205,8,8],"pitch":null,"staff_id":1,"grade":0.8}),false).unwrap();
    let before = g.snapshot();
    let maps = owners.maps[0].lines.clone();
    assert_eq!(
        reduce(&mut g, &mut owners, &staffs, 11.).unwrap_err(),
        "music_ledger_head_pitch_required"
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(owners.maps[0].lines, maps);
    assert!(Arc::ptr_eq(&owners.maps[0].staff, &staffs[0]));
}
#[test]
fn real_head_stage_multistaff_pruning_matches_original_with_retained_staff_and_ledger_bodies() {
    let original: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-ledger-reduction/image-observed.json"
    ))
    .unwrap();
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-grid/laura-gray.png"
    ))
    .unwrap();
    let mut page = crate::page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    let mut observations = Vec::new();
    for c in original["systems"].as_array().unwrap() {
        let system = c["system"].as_u64().unwrap();
        let mut names = BTreeMap::new();
        for m in page
            .ledger_memberships
            .maps
            .iter()
            .filter(|m| m.staff.system == system)
        {
            for k in m.lines.values().flatten() {
                if !names.contains_key(k) {
                    let name = format!("ledger{}", names.len());
                    names.insert(*k, name);
                }
            }
        }
        let owners = LedgerMemberships {
            maps: page
                .ledger_memberships
                .maps
                .iter()
                .filter(|m| m.staff.system == system)
                .cloned()
                .collect(),
        };
        assert_eq!(frame(&page.graph, &owners, &names), c["before"]);
        observations.push((c, names));
    }
    let removed = reduce(
        &mut page.graph,
        &mut page.ledger_memberships,
        &page.staffs,
        11.,
    )
    .unwrap();
    assert_eq!(
        removed.len() as u64,
        original["systems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["removed_count"].as_u64().unwrap())
            .sum::<u64>()
    );
    for (c, names) in observations {
        let system = c["system"].as_u64().unwrap();
        let owners = LedgerMemberships {
            maps: page
                .ledger_memberships
                .maps
                .iter()
                .filter(|m| m.staff.system == system)
                .cloned()
                .collect(),
        };
        assert_eq!(frame(&page.graph, &owners, &names), c["after"]);
        for m in owners.maps {
            assert!(page.staffs.iter().any(|s| Arc::ptr_eq(s, &m.staff)));
        }
    }
}
