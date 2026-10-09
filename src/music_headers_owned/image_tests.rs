// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{
    grid_ir::{StaffGeometry, SystemGeometry},
    music_measures::OwnedMusic,
    native_headers::{NativeHeaderInter, NativeHeaderRelation},
    stem_image_owned::OwnedItems,
};

struct InterRef {
    key: Key,
    source: Arc<NativeHeaderInter>,
    staff: Arc<StaffGeometry>,
}
struct HeldSystemRelations {
    system: Arc<SystemGeometry>,
    relations: Vec<Arc<NativeHeaderRelation>>,
}
pub(crate) struct HeldHeaders {
    inters: Vec<InterRef>,
    relations: Vec<(Key, Key, Arc<NativeHeaderRelation>)>,
    carrier_relations: Vec<HeldSystemRelations>,
    null_clefs: Vec<Arc<StaffGeometry>>,
    projection: Value,
}
/// Called before MEASURES. Only native-finalized objects provide these handles.
pub(crate) fn hold(owner: &OwnedItems) -> HeldHeaders {
    let projection = owner.native_headers.report();
    assert_eq!(projection, owner.upstream_report["headers"]);
    let mut inters = Vec::new();
    let mut relations = Vec::new();
    let mut carrier_relations = Vec::new();
    let mut null_clefs = Vec::new();
    for system in owner.native_headers.systems() {
        assert!(
            owner
                .systems
                .iter()
                .any(|s| Arc::ptr_eq(s, system.system()))
        );
        for header in system.staffs() {
            assert!(owner.staffs.iter().any(|s| Arc::ptr_eq(s, header.staff())));
            assert!(std::ptr::eq(
                owner.native_headers.staff_header(header.staff()).unwrap(),
                header
            ));
            assert_eq!(
                owner.native_headers.header_stop(header.staff()).unwrap(),
                header.stop()
            );
            assert!(header.stop() >= header.start());
            if let Some(source) = header.clef() {
                inters.push(InterRef {
                    key: Key {
                        system: system.system().id,
                        phase: Phase::MusicHeader,
                        local: header.staff().id as usize,
                    },
                    source: source.clone(),
                    staff: header.staff().clone(),
                });
            } else {
                null_clefs.push(header.staff().clone());
            }
            if let Some(source) = header.key() {
                for member in source.members() {
                    inters.push(InterRef {
                        key: Key {
                            system: system.system().id,
                            phase: Phase::MusicKey,
                            local: member.id() as usize,
                        },
                        source: member.clone(),
                        staff: header.staff().clone(),
                    });
                }
                inters.push(InterRef {
                    key: Key {
                        system: system.system().id,
                        phase: Phase::MusicKey,
                        local: source.id() as usize,
                    },
                    source: source.clone(),
                    staff: header.staff().clone(),
                });
            }
        }
        carrier_relations.push(HeldSystemRelations {
            system: system.system().clone(),
            relations: system.relations().to_vec(),
        });
        for relation in system.relations() {
            let a = inters.iter().find(|r| {
                r.key.system == system.system().id && Arc::ptr_eq(&r.source, relation.source())
            });
            let b = inters.iter().find(|r| {
                r.key.system == system.system().id && Arc::ptr_eq(&r.source, relation.target())
            });
            if let (Some(a), Some(b)) = (a, b) {
                match relation.body()["kind"].as_str() {
                    Some("Containment") => {
                        assert!(system.staffs().iter().any(|header| {
                            header.key().is_some_and(|key| {
                                Arc::ptr_eq(key, relation.source())
                                    && key
                                        .members()
                                        .iter()
                                        .any(|member| Arc::ptr_eq(member, relation.target()))
                            })
                        }));
                        relations.push((a.key, b.key, relation.clone()));
                    }
                    Some("ClefKeyRelation") => {
                        assert!(system.staffs().iter().any(|header| {
                            header
                                .clef()
                                .is_some_and(|clef| Arc::ptr_eq(clef, relation.source()))
                                && header
                                    .key()
                                    .is_some_and(|key| Arc::ptr_eq(key, relation.target()))
                        }));
                        relations.push((a.key, b.key, relation.clone()));
                    }
                    Some("KeyAltersRelation") => {
                        assert!(system.staffs().iter().any(|header| {
                            header.key().is_some_and(|key| {
                                [relation.source(), relation.target()]
                                    .iter()
                                    .all(|endpoint| {
                                        endpoint.body()["kind"] == "KeyAlterInter"
                                            && key
                                                .members()
                                                .iter()
                                                .any(|member| Arc::ptr_eq(member, endpoint))
                                    })
                            })
                        }));
                        relations.push((a.key, b.key, relation.clone()));
                    }
                    other => panic!("uncompared selected header relation: {other:?}"),
                }
            }
        }
    }
    assert!(!inters.is_empty());
    HeldHeaders {
        inters,
        relations,
        carrier_relations,
        null_clefs,
        projection,
    }
}
pub(crate) fn verify(music: &OwnedMusic, held: &HeldHeaders, page: &str, stage: &str) {
    let before = music.owner.graph.snapshot();
    let clone = music.owner.graph.clone();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for reference in &held.inters {
        let header = music
            .owner
            .native_headers
            .staff_header(&reference.staff)
            .unwrap();
        assert!(
            music
                .owner
                .staffs
                .iter()
                .any(|s| Arc::ptr_eq(s, &reference.staff))
        );
        assert!(
            [header.clef(), header.key()]
                .into_iter()
                .flatten()
                .any(|s| Arc::ptr_eq(s, &reference.source)
                    || s.members()
                        .iter()
                        .any(|m| Arc::ptr_eq(m, &reference.source)))
        );
        nodes.push(music.owner.graph.assert_native_header_binding(
            reference.key,
            &reference.source,
            &reference.staff,
        ));
        clone.assert_native_header_binding(reference.key, &reference.source, &reference.staff);
    }
    let mut carrier_kinds = std::collections::BTreeMap::<String, usize>::new();
    for held_system in &held.carrier_relations {
        let actual = music
            .owner
            .native_headers
            .systems()
            .iter()
            .find(|system| Arc::ptr_eq(system.system(), &held_system.system))
            .unwrap();
        assert_eq!(actual.relations().len(), held_system.relations.len());
        for (relation, before) in actual.relations().iter().zip(&held_system.relations) {
            assert!(Arc::ptr_eq(relation, before));
            for endpoint in [relation.source(), relation.target()] {
                let canonical = actual
                    .inters()
                    .iter()
                    .find(|inter| inter.id() == endpoint.id())
                    .unwrap();
                assert!(Arc::ptr_eq(endpoint, canonical));
            }
            *carrier_kinds
                .entry(relation.body()["kind"].as_str().unwrap().to_owned())
                .or_default() += 1;
        }
    }
    for (source, target, relation) in &held.relations {
        edges.push(
            music
                .owner
                .graph
                .assert_native_header_relation(*source, *target, relation),
        );
        clone.assert_native_header_relation(*source, *target, relation);
    }
    for staff in &held.null_clefs {
        assert!(
            music
                .owner
                .native_headers
                .staff_header(staff)
                .unwrap()
                .clef()
                .is_none()
        );
        let key = Key {
            system: staff.system,
            phase: Phase::MusicHeader,
            local: staff.id as usize,
        };
        assert!(music.owner.graph.id(key).is_none());
    }
    assert_eq!(held.projection, music.owner.native_headers.report());
    assert_eq!(before, music.owner.graph.snapshot());
    let path = match (page, stage) {
        ("an_laura", "MEASURES") => "/tmp/real-laura-header-bindings-MEASURES.json",
        ("an_laura", "LINKS") => "/tmp/real-laura-header-bindings-LINKS.json",
        ("der_winter", "MEASURES") => "/tmp/real-rest-header-bindings-MEASURES.json",
        ("der_winter", "LINKS") => "/tmp/real-rest-header-bindings-LINKS.json",
        _ => panic!("unknown actual-owner observation"),
    };
    std::fs::write(path,serde_json::to_vec_pretty(&json!({"stage":stage,"nodes":nodes,"relations":edges,"null_selected_clef_count":held.null_clefs.len(),
        "all_carrier_relation_kinds":carrier_kinds,"all_carrier_relation_order_and_endpoint_Arcs_retained":true,
        "selected_KeyAlters_supports_registered_from_same_Arcs":true,
        "native_finalized_source_Staff_member_relation_Arcs_retained":true,"transaction_clone_keeps_same_Arcs":true,"read_only_observation":true,
        "Measure_membership_or_prefreeze_identity_claimed":false,"quality_accepted":false})).unwrap()).unwrap();
}
