// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{register_clef, register_key};
use crate::{
    beam_dispatch::{self, Registry},
    curves::Curve,
    grid_ir::{StaffGeometry, SystemGeometry},
    native_headers::{HeaderStaffInput, NativeHeaders, NativeSystemHeaders},
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

struct DeclaredHeader {
    system: Arc<SystemGeometry>,
    staff: HeaderStaffInput,
    inters: Vec<Value>,
    relations: Vec<Value>,
    detached: BTreeMap<u64, Value>,
}
impl DeclaredHeader {
    fn finish(self) -> Result<NativeSystemHeaders, String> {
        NativeSystemHeaders::from_finalized(
            self.system,
            vec![self.staff],
            self.inters,
            self.relations,
            self.detached,
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::from([10, 20, 21, 22]),
        )
    }
}

fn glyph(x: i32, y: i32) -> Value {
    json!({"bounds":[x,y,1,3],"orientation":"VERTICAL","runs":[[0,0,3]]})
}

// Declared finalized values exercise the ownership boundary, not recognition quality.
fn declared_header() -> DeclaredHeader {
    let points = (0..5)
        .map(|line| {
            vec![
                [10., 100. + line as f64 * 11.],
                [500., 100. + line as f64 * 11.],
            ]
        })
        .collect::<Vec<_>>();
    let staff = Arc::new(StaffGeometry {
        id: 1,
        system: 1,
        specific_interline: 11,
        abscissae: [10, 500],
        lines: points
            .iter()
            .map(|p| Arc::new(Curve::interpolate(p.clone()).unwrap()))
            .collect(),
    });
    let geometry = json!({
        "systems":[{"id":1,"coordinates":[10,100,491,144,44],"staff_ids":[1]}],
        "staffs":[{"id":1,"system_id":1,"left":10,"right":500,
            "lines":points.iter().map(|p|json!({"points":p})).collect::<Vec<_>>()}]
    });
    let area = crate::system_area_geometry::build(&geometry, 512, 200, 11)
        .unwrap()
        .system_areas
        .remove(0);
    let system = Arc::new(SystemGeometry {
        id: 1,
        left: 10,
        profile: 0,
        area: Arc::new(area),
        staffs: vec![staff.clone()],
    });
    let clef = json!({"id":10,"kind":"ClefInter","shape":"G_CLEF",
        "staff_id":1,"bounds":[12,92,6,40],"glyph":null,
        "frozen":true,"manual":true,"removed":false});
    let first = json!({"id":21,"kind":"KeyAlterInter","shape":"SHARP",
        "staff_id":1,"bounds":[20,90,1,3],"glyph":glyph(20,90),
        "frozen":true,"manual":true,"removed":false});
    let second = json!({"id":22,"kind":"KeyAlterInter","shape":"SHARP",
        "staff_id":1,"bounds":[30,100,1,3],"glyph":glyph(30,100),
        "frozen":true,"manual":false,"removed":false});
    let key = json!({"id":20,"kind":"KeyInter","shape":"KEY_SHARP_2",
        "staff_id":1,"bounds":[20,90,11,13],"glyph":null,"fifths":2,
        "members":[first.clone(),second.clone()],
        "frozen":true,"manual":false,"removed":false});
    DeclaredHeader {
        system,
        staff: HeaderStaffInput {
            staff,
            start: 10,
            stop: 40,
            clef_id: Some(10),
            key_id: Some(20),
            time_id: None,
            clef_range: Value::Null,
            key_range: Value::Null,
            time_range: Value::Null,
        },
        inters: vec![clef, key, first, second],
        relations: vec![
            json!({"kind":"KeyAltersRelation","source":21,"target":22,"manual":true}),
            json!({"kind":"Containment","source":20,"target":21,"manual":true}),
            json!({"kind":"Containment","source":20,"target":22,"manual":false}),
            json!({"kind":"ClefKeyRelation","source":10,"target":20,
                "grade":1.0,"source_ratio":1.0,"target_ratio":1.0,"manual":true}),
        ],
        detached: BTreeMap::new(),
    }
}

fn source_key(id: u64) -> Key {
    Key {
        system: 1,
        phase: if id == 10 {
            Phase::MusicHeader
        } else {
            Phase::MusicKey
        },
        local: if id == 10 { 1 } else { id as usize },
    }
}

#[test]
fn finalized_canonical_members_relations_and_staff_arcs_survive_registration() {
    let system = declared_header().finish().unwrap();
    let header = &system.staffs()[0];
    let staff = header.staff().clone();
    assert!(Arc::ptr_eq(&staff, &system.system().staffs[0]));
    let clef = header.clef().unwrap();
    let key = header.key().unwrap();
    assert_eq!(
        key.members().iter().map(|n| n.id()).collect::<Vec<_>>(),
        [21, 22]
    );
    for source in [clef, key].into_iter().chain(key.members()) {
        let canonical = system
            .inters()
            .iter()
            .find(|n| n.id() == source.id())
            .unwrap();
        assert!(Arc::ptr_eq(source, canonical));
    }
    for relation in system.relations() {
        for endpoint in [relation.source(), relation.target()] {
            let canonical = system
                .inters()
                .iter()
                .find(|n| n.id() == endpoint.id())
                .unwrap();
            assert!(Arc::ptr_eq(endpoint, canonical));
        }
    }
    let mut graph = Graph::default();
    let mut registry = Registry::default();
    let registered_clef = register_clef(&mut graph, header).unwrap().unwrap();
    let registered_key = register_key(
        &mut graph,
        &mut registry,
        header,
        &system,
        registered_clef.key,
    )
    .unwrap()
    .unwrap();
    assert_eq!(registered_clef.key, source_key(10));
    assert_eq!(registered_key.key, source_key(20));
    assert_eq!(registry.len(), 2);
    for owner in [&graph, &graph.clone()] {
        for source in [clef, key].into_iter().chain(key.members()) {
            owner.assert_native_header_binding(source_key(source.id()), source, &staff);
        }
        for relation in system.relations() {
            owner.assert_native_header_relation(
                source_key(relation.source().id()),
                source_key(relation.target().id()),
                relation,
            );
        }
    }
    for member in key.members() {
        let body = graph.retained_inter(source_key(member.id())).unwrap().0;
        let glyph_id = body["glyph_id"].as_u64().unwrap() as usize;
        assert_eq!(
            beam_dispatch::glyph_json(&registry.get(glyph_id).unwrap().glyph),
            member.body()["glyph"]
        );
    }
    let foreign = Arc::new(StaffGeometry {
        id: staff.id,
        system: staff.system,
        specific_interline: staff.specific_interline,
        abscissae: staff.abscissae,
        lines: staff.lines.clone(),
    });
    let owners = NativeHeaders::new(vec![system]).unwrap();
    assert!(Arc::ptr_eq(
        owners.staff_header(&staff).unwrap().staff(),
        &staff
    ));
    assert_eq!(
        owners.staff_header(&foreign).err().unwrap(),
        "native_header_actual_staff_owner_required"
    );
    let mut invalid = declared_header();
    invalid.staff.staff = foreign;
    assert_eq!(
        invalid.finish().err().unwrap(),
        "native_header_actual_staff_range_required"
    );
}

#[test]
fn detached_or_raw_null_selected_clef_cannot_replace_a_held_binding() {
    let held = declared_header().finish().unwrap();
    let held_header = &held.staffs()[0];
    let mut graph = Graph::default();
    let registered = register_clef(&mut graph, held_header).unwrap().unwrap();
    let before = graph.snapshot();
    for detached in [true, false] {
        let mut declared = declared_header();
        if detached {
            let clef = declared.inters.remove(0);
            declared.detached.insert(10, clef);
            declared.relations.retain(|r| r["source"] != 10);
        } else {
            declared.inters[0]["staff_id"] = Value::Null;
        }
        let refused = declared.finish().unwrap();
        let header = &refused.staffs()[0];
        assert_eq!(header.clef().unwrap().in_final_sig(), !detached);
        if detached {
            assert!(Arc::ptr_eq(
                refused.detached().get(&10).unwrap(),
                header.clef().unwrap(),
            ));
        } else {
            assert!(refused.detached().is_empty());
        }
        assert_eq!(
            register_clef(&mut graph, header).err().unwrap(),
            "music_native_selected_frozen_Clef_required"
        );
        assert_eq!(graph.snapshot(), before);
        graph.assert_native_header_binding(
            registered.key,
            held_header.clef().unwrap(),
            held_header.staff(),
        );
        let selected = header.clef().unwrap().clone();
        let actual_staff = header.staff().clone();
        let retained = NativeHeaders::new(vec![refused]).unwrap();
        let lookup = retained.staff_header(&actual_staff).unwrap();
        assert!(Arc::ptr_eq(lookup.clef().unwrap(), &selected));
        assert_eq!(lookup.clef().unwrap().in_final_sig(), !detached);
    }
    let mut conflicting = declared_header();
    conflicting.inters[0]["staff_id"] = json!(2);
    assert_eq!(
        conflicting.finish().err().unwrap(),
        "native_header_inter_staff_conflict"
    );
    assert_eq!(graph.snapshot(), before);
}

#[test]
fn late_bad_key_glyph_preserves_graph_source_arcs_and_physical_registry() {
    let mut declared = declared_header();
    declared.inters[3]["glyph"]["runs"] = json!([]);
    declared.inters[1]["members"][1] = declared.inters[3].clone();
    let system = declared.finish().unwrap();
    let header = &system.staffs()[0];
    let mut graph = Graph::default();
    let registered = register_clef(&mut graph, header).unwrap().unwrap();
    let mut registry = Registry::default();
    let existing = registry
        .register(&beam_dispatch::glyph_from_json(&glyph(5, 5)).unwrap())
        .unwrap();
    let held_runs = registry.get(existing).unwrap().glyph.runs.as_ptr();
    let before_graph = graph.snapshot();
    let before_glyphs = registry.snapshot();
    assert_eq!(
        register_key(&mut graph, &mut registry, header, &system, registered.key)
            .err()
            .unwrap(),
        "invalid_beam_canonical_runs"
    );
    assert_eq!(graph.snapshot(), before_graph);
    assert_eq!(registry.snapshot(), before_glyphs);
    assert_eq!(
        registry.get(existing).unwrap().glyph.runs.as_ptr(),
        held_runs
    );
    graph.assert_native_header_binding(registered.key, header.clef().unwrap(), header.staff());
    for id in [20, 21, 22] {
        assert!(graph.id(source_key(id)).is_none());
        assert!(graph.native_header_source(source_key(id)).is_err());
    }
}

#[test]
fn null_selected_headers_do_not_fall_back_to_live_candidates() {
    let mut declared = declared_header();
    declared.staff.clef_id = None;
    declared.staff.key_id = None;
    let system = declared.finish().unwrap();
    let header = &system.staffs()[0];
    assert_eq!(system.inters().len(), 4);
    assert!(header.clef().is_none() && header.key().is_none() && header.time().is_none());
    let mut graph = Graph::default();
    let mut registry = Registry::default();
    let before_graph = graph.snapshot();
    let before_glyphs = registry.snapshot();
    assert!(register_clef(&mut graph, header).unwrap().is_none());
    assert!(
        register_key(&mut graph, &mut registry, header, &system, source_key(10))
            .unwrap()
            .is_none()
    );
    assert_eq!(graph.snapshot(), before_graph);
    assert_eq!(registry.snapshot(), before_glyphs);
    assert!(header.clef().is_none() && header.key().is_none() && header.time().is_none());
}

#[test]
fn refused_key_member_support_or_foreign_owner_preserves_held_registration() {
    for (case, expected) in [
        (0, "music_header_actual_Clef_Key_support_required"),
        (1, "music_header_actual_KeyAlter_required"),
        (2, "music_header_actual_Clef_Key_owners_required"),
    ] {
        let mut declared = declared_header();
        if case == 0 {
            declared
                .relations
                .retain(|r| r["kind"] != "ClefKeyRelation");
        } else if case == 1 {
            declared.inters[2]["shape"] = json!("NATURAL");
            declared.inters[1]["members"][0] = declared.inters[2].clone();
        }
        let held = declared.finish().unwrap();
        let header = &held.staffs()[0];
        let mut graph = Graph::default();
        let clef = register_clef(&mut graph, header).unwrap().unwrap();
        let foreign = (case == 2).then(|| declared_header().finish().unwrap());
        let requested = foreign.as_ref().unwrap_or(&held);
        let requested_header = &requested.staffs()[0];
        if case == 2 {
            assert_eq!(header.staff().id, requested_header.staff().id);
            assert!(!Arc::ptr_eq(header.staff(), requested_header.staff()));
            assert_eq!(
                header.clef().unwrap().id(),
                requested_header.clef().unwrap().id()
            );
            assert!(!Arc::ptr_eq(
                header.clef().unwrap(),
                requested_header.clef().unwrap()
            ));
        }
        let mut registry = Registry::default();
        let existing = registry
            .register(&beam_dispatch::glyph_from_json(&glyph(5, 5)).unwrap())
            .unwrap();
        let held_runs = registry.get(existing).unwrap().glyph.runs.as_ptr();
        let before_graph = graph.snapshot();
        let before_glyphs = registry.snapshot();
        assert_eq!(
            register_key(
                &mut graph,
                &mut registry,
                requested_header,
                requested,
                clef.key
            )
            .err()
            .unwrap(),
            expected
        );
        assert_eq!(graph.snapshot(), before_graph);
        assert_eq!(registry.snapshot(), before_glyphs);
        assert_eq!(
            registry.get(existing).unwrap().glyph.runs.as_ptr(),
            held_runs
        );
        graph.assert_native_header_binding(clef.key, header.clef().unwrap(), header.staff());
        for id in [20, 21, 22] {
            assert!(graph.id(source_key(id)).is_none());
            assert!(graph.native_header_source(source_key(id)).is_err());
        }
    }
}

#[test]
fn contradictory_final_ids_or_member_payloads_never_replace_held_owners() {
    let held = declared_header().finish().unwrap();
    let header = &held.staffs()[0];
    let mut graph = Graph::default();
    let clef = register_clef(&mut graph, header).unwrap().unwrap();
    let mut registry = Registry::default();
    register_key(&mut graph, &mut registry, header, &held, clef.key)
        .unwrap()
        .unwrap();
    let before_graph = graph.snapshot();
    let before_glyphs = registry.snapshot();
    let held_runs = registry.get(1).unwrap().glyph.runs.as_ptr();
    for (case, expected) in [
        (0, "native_header_duplicate_inter"),
        (1, "native_header_member_payload_conflict"),
        (2, "native_header_relation_target_required"),
        (3, "native_header_relation_source_required"),
    ] {
        let mut declared = declared_header();
        match case {
            0 => declared.inters.push(declared.inters[0].clone()),
            1 => declared.inters[1]["members"][0]["manual"] = json!(false),
            2 => {
                declared.inters.remove(3);
            }
            _ => {
                declared.inters.remove(2);
            }
        }
        assert_eq!(declared.finish().err().unwrap(), expected);
        assert_eq!(graph.snapshot(), before_graph);
        assert_eq!(registry.snapshot(), before_glyphs);
        assert_eq!(registry.get(1).unwrap().glyph.runs.as_ptr(), held_runs);
        for source in [header.clef().unwrap(), header.key().unwrap()]
            .into_iter()
            .chain(header.key().unwrap().members())
        {
            graph.assert_native_header_binding(source_key(source.id()), source, header.staff());
        }
        for relation in held.relations() {
            graph.assert_native_header_relation(
                source_key(relation.source().id()),
                source_key(relation.target().id()),
                relation,
            );
        }
    }
}

mod time;
