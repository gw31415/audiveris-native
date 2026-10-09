// SPDX-License-Identifier: AGPL-3.0-or-later
//! Declared finalized owners; these controls do not perform image recognition.
use super::*;
use crate::music_headers_owned::register_time;

fn declared_time(whole: bool) -> DeclaredHeader {
    let mut declared = declared_header();
    if whole {
        declared.staff.time_id = Some(33);
        declared.inters.push(
            json!({"id":33,"kind":"TimeWholeInter","shape":"COMMON_TIME",
            "staff_id":1,"bounds":[40,100,1,3],"glyph":glyph(40,100),"numerator":4,
            "denominator":4,"value":"COMMON_TIME","frozen":true,"manual":true}),
        );
    } else {
        let top = json!({"id":31,"kind":"TimeNumberInter","shape":"TIME_THREE",
            "staff_id":1,"bounds":[40,100,1,3],"glyph":glyph(40,100),"side":"TOP",
            "value":3,"frozen":true,"manual":true});
        let bottom = json!({"id":32,"kind":"TimeNumberInter","shape":"TIME_FOUR",
            "staff_id":1,"bounds":[40,110,1,3],"glyph":glyph(40,110),"side":"BOTTOM",
            "value":4,"frozen":true,"manual":false});
        let pair = json!({"id":33,"kind":"TimePairInter","staff_id":1,
            "bounds":[40,100,1,13],"glyph":null,"numerator":3,"denominator":4,
            "value":"3/4","members":[top.clone(),bottom.clone()],"frozen":true,"manual":true});
        declared.staff.time_id = Some(33);
        declared.inters.extend([top, bottom, pair]);
        declared.relations.extend([
            json!({"kind":"TimeTopBottomRelation","source":31,"target":32,"manual":true}),
            json!({"kind":"Containment","source":33,"target":31,"manual":false}),
            json!({"kind":"Containment","source":33,"target":32,"manual":true}),
        ]);
    }
    declared
}
fn finish_time(declared: DeclaredHeader) -> NativeSystemHeaders {
    let frozen = declared
        .inters
        .iter()
        .filter(|n| n["frozen"] == true)
        .map(|n| n["id"].as_u64().unwrap())
        .collect();
    NativeSystemHeaders::from_finalized(
        declared.system,
        vec![declared.staff],
        declared.inters,
        declared.relations,
        declared.detached,
        BTreeSet::new(),
        BTreeSet::new(),
        frozen,
    )
    .unwrap()
}
fn time_key(id: u64) -> Key {
    Key {
        system: 1,
        phase: Phase::MusicTime,
        local: id as usize,
    }
}
fn replace_time_member(declared: &mut DeclaredHeader, id: u64, field: &str, value: Value) {
    let member = declared.inters.iter_mut().find(|n| n["id"] == id).unwrap();
    member[field] = value;
    let changed = member.clone();
    let pair = declared.inters.iter_mut().find(|n| n["id"] == 33).unwrap();
    let position = if id == 31 { 0 } else { 1 };
    pair["members"][position] = changed;
}
#[test]
fn selected_pair_and_whole_time_retain_actual_sources_physical_glyphs_and_relation_order() {
    for whole in [false, true] {
        let system = finish_time(declared_time(whole));
        let header = &system.staffs()[0];
        let mut graph = Graph::default();
        let mut registry = Registry::default();
        let existing = registry
            .register(&beam_dispatch::glyph_from_json(&glyph(5, 5)).unwrap())
            .unwrap();
        let held_runs = registry.get(existing).unwrap().glyph.runs.as_ptr();
        let registered = register_time(&mut graph, &mut registry, header, &system)
            .unwrap()
            .unwrap();
        assert_eq!(registered.key, time_key(33));
        let source = header.time().unwrap();
        for actual in std::iter::once(source).chain(source.members()) {
            graph.assert_native_header_binding(time_key(actual.id()), actual, header.staff());
            graph.clone().assert_native_header_binding(
                time_key(actual.id()),
                actual,
                header.staff(),
            );
            if actual.body()["kind"] != "TimePairInter" {
                let body = graph.retained_inter(time_key(actual.id())).unwrap().0;
                let glyph_id = body["glyph_id"].as_u64().unwrap() as usize;
                assert_eq!(
                    beam_dispatch::glyph_json(&registry.get(glyph_id).unwrap().glyph),
                    actual.body()["glyph"]
                );
            }
        }
        assert_eq!(
            registry.get(existing).unwrap().glyph.runs.as_ptr(),
            held_runs
        );
        assert_eq!(registry.len(), if whole { 2 } else { 3 });
        if !whole {
            let time_edges = system
                .relations()
                .iter()
                .filter(|r| r.source().id() >= 31)
                .collect::<Vec<_>>();
            for relation in &time_edges {
                graph.assert_native_header_relation(
                    time_key(relation.source().id()),
                    time_key(relation.target().id()),
                    relation,
                );
            }
            let snapshot = graph.snapshot();
            assert_eq!(
                snapshot["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| r["kind"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["TimeTopBottomRelation", "Containment", "Containment"]
            );
            assert_eq!(
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n["source_header_id"].as_u64().unwrap())
                    .collect::<Vec<_>>(),
                [31, 32, 33]
            );
            let body = graph.retained_inter(registered.key).unwrap().0;
            assert_eq!(
                body["member_ids"],
                json!([graph.id(time_key(31)), graph.id(time_key(32))])
            );
        }
        let before = graph.snapshot();
        let physical = registry.snapshot();
        assert_eq!(
            register_time(&mut graph, &mut registry, header, &system)
                .err()
                .unwrap(),
            "music_header_fresh_Time_binding_required"
        );
        assert_eq!(graph.snapshot(), before);
        assert_eq!(registry.snapshot(), physical);
        graph.assert_native_header_binding(registered.key, source, header.staff());
    }
}
#[test]
fn invalid_time_members_support_and_late_physical_failure_leave_both_owners_unchanged() {
    for variant in 0..8 {
        let mut declared = declared_time(variant == 7);
        match variant {
            0 => replace_time_member(&mut declared, 32, "staff_id", Value::Null),
            1 => replace_time_member(&mut declared, 32, "frozen", json!(false)),
            2 => declared
                .relations
                .retain(|r| r["kind"] != "TimeTopBottomRelation"),
            3 => declared
                .relations
                .push(json!({"kind":"Containment","source":33,"target":31})),
            4 => replace_time_member(&mut declared, 32, "value", json!(8)),
            5 => {
                let mut invalid = glyph(40, 110);
                invalid["runs"] = json!([]);
                replace_time_member(&mut declared, 32, "glyph", invalid);
            }
            6 => {
                let mut candidate = declared
                    .inters
                    .iter()
                    .find(|n| n["id"] == 32)
                    .unwrap()
                    .clone();
                candidate["id"] = json!(34);
                declared.inters.push(candidate);
                declared.relations.push(
                    json!({"kind":"TimeTopBottomRelation","source":31,"target":34,"manual":true}),
                );
            }
            7 => declared
                .relations
                .push(json!({"kind":"NoExclusion","source":33,"target":10,"manual":true})),
            _ => unreachable!(),
        }
        let system = finish_time(declared);
        let header = &system.staffs()[0];
        let mut graph = Graph::default();
        let clef = register_clef(&mut graph, header).unwrap().unwrap();
        let mut registry = Registry::default();
        let existing = registry
            .register(&beam_dispatch::glyph_from_json(&glyph(5, 5)).unwrap())
            .unwrap();
        let held_runs = registry.get(existing).unwrap().glyph.runs.as_ptr();
        let before = graph.snapshot();
        let physical = registry.snapshot();
        let expected_error = match variant {
            0 | 1 | 4 => "music_header_actual_TimeNumber_required",
            2 => "music_header_actual_Time_number_support_required",
            3 => "music_header_actual_Time_member_relation_required",
            5 => "invalid_beam_canonical_runs",
            6 | 7 => "music_header_Time_incident_relation_pending",
            _ => unreachable!(),
        };
        assert_eq!(
            register_time(&mut graph, &mut registry, header, &system)
                .err()
                .unwrap(),
            expected_error,
            "variant {variant}"
        );
        assert_eq!(graph.snapshot(), before);
        assert_eq!(registry.snapshot(), physical);
        assert_eq!(
            registry.get(existing).unwrap().glyph.runs.as_ptr(),
            held_runs
        );
        graph.assert_native_header_binding(clef.key, header.clef().unwrap(), header.staff());
        for id in [31, 32, 33] {
            assert!(graph.id(time_key(id)).is_none());
            assert!(graph.native_header_source(time_key(id)).is_err());
        }
    }
}
#[test]
fn null_foreign_detached_and_raw_null_time_do_not_substitute_equal_ids_or_candidates() {
    let actual = finish_time(declared_time(false));
    let foreign = finish_time(declared_time(false));
    let mut graph = Graph::default();
    let mut registry = Registry::default();
    let before = graph.snapshot();
    let physical = registry.snapshot();
    assert_eq!(
        register_time(&mut graph, &mut registry, &actual.staffs()[0], &foreign)
            .err()
            .unwrap(),
        "music_header_actual_Time_owners_required"
    );
    for variant in 0..3 {
        let mut declared = declared_time(false);
        match variant {
            0 => declared.staff.time_id = None,
            1 => {
                let index = declared.inters.iter().position(|n| n["id"] == 33).unwrap();
                let pair = declared.inters.remove(index);
                declared.detached.insert(33, pair);
                declared.relations.retain(|r| r["source"] != 33);
            }
            2 => {
                declared.inters.iter_mut().find(|n| n["id"] == 33).unwrap()["staff_id"] =
                    Value::Null
            }
            _ => unreachable!(),
        }
        let system = finish_time(declared);
        let header = &system.staffs()[0];
        if variant == 0 {
            assert!(header.time().is_none());
            assert!(
                system
                    .inters()
                    .iter()
                    .any(|n| n.body()["kind"] == "TimePairInter")
            );
            assert!(
                register_time(&mut graph, &mut registry, header, &system)
                    .unwrap()
                    .is_none()
            );
        } else {
            assert_eq!(
                register_time(&mut graph, &mut registry, header, &system)
                    .err()
                    .unwrap(),
                "music_header_actual_frozen_Time_required"
            );
        }
        assert_eq!(graph.snapshot(), before);
        assert_eq!(registry.snapshot(), physical);
    }
}
