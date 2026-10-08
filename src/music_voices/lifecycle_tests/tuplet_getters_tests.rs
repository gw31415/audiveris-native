// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame_with_groups;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control};
use super::*;
use crate::music_xml::TupletRead;
use crate::shared_sig::{Key, NoteOwnerEdit};

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-getters/observed.json"
    ))
    .unwrap();
    let relations = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] =
            relations[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let frames = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = frames[p["state"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
pub(super) fn node(p: &Prepared, key: Key) -> Value {
    p.c.owner.owner.graph.snapshot()["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["origin"] == json!(key))
        .unwrap()
        .clone()
}
pub(super) fn link(p: &mut Prepared, chord: usize, tuplet: usize, manual: bool) {
    let source = chord_key(&p.c, &format!("chord{chord}"));
    let target = p.tuplets[tuplet];
    p.c.owner
        .owner
        .graph
        .relations(
            1,
            crate::shared_sig::Phase::Symbol,
            vec![(
                source,
                target,
                json!({"kind":"ChordTupletRelation","manual":manual}),
            )],
        )
        .unwrap();
    let snapshot = p.c.owner.owner.graph.snapshot();
    let id = snapshot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .rfind(|e| {
            e["kind"] == "ChordTupletRelation"
                && e["source"] == p.c.owner.owner.graph.id(source).unwrap()
                && e["target"] == p.c.owner.owner.graph.id(target).unwrap()
        })
        .unwrap()["id"]
        .as_u64()
        .unwrap() as usize;
    p.relations
        .push((id, format!("tuple_link{}", p.relations.len())));
}
fn rows(p: &mut Prepared, first: &mut [Option<TupletRead>]) -> Value {
    let mut rows = Vec::new();
    for (i, retained) in first.iter_mut().enumerate().take(p.tuplets.len()) {
        let key = p.tuplets[i];
        let n = node(p, key);
        let read = p.c.owner.read_rhythm_tuplet(key).unwrap();
        let previous = retained.as_ref();
        let same_factor = previous.is_none_or(|r| Arc::ptr_eq(&r.factor, &read.factor));
        let same_base = match (read.base.as_ref(), previous.and_then(|r| r.base.as_ref())) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            (Some(_), None) => true,
            _ => false,
        };
        let same_staff = match (read.staff.as_ref(), previous.and_then(|r| r.staff.as_ref())) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            (Some(_), None) => true,
            _ => false,
        };
        if let Some(s) = &read.staff {
            assert!(
                p.c.owner
                    .owner
                    .staffs
                    .iter()
                    .any(|actual| Arc::ptr_eq(actual, s))
            );
        }
        rows.push(json!({"label":format!("tuplet{i}"),"shape":n["shape"],"manual":n["manual"],"factor":format!("{}/{}",read.factor.actual_num,read.factor.actual_den),"same_factor":same_factor,"base":read.base.as_ref().map(|b|crate::music_time::format_fraction(**b)),"same_base":same_base,"staff":read.staff.as_ref().map(|s|s.id),"same_staff":same_staff,"part":read.part,"voice":read.voice.as_ref().map(|v|format!("voice{}",p.c.refs.iter().position(|r|Arc::ptr_eq(&r.identity,v)).unwrap())),"chords":read.chords.iter().map(|k|p.c.labels[&p.c.owner.owner.graph.id(*k).unwrap()].clone()).collect::<Vec<_>>()}));
        if let Some(previous) = retained.as_mut() {
            if previous.base.is_none() {
                previous.base = read.base;
            }
            if previous.staff.is_none() {
                previous.staff = read.staff;
            }
        } else {
            *retained = Some(read);
        }
    }
    json!(rows)
}
#[test]
fn original_tuplet_lazy_references_null_staff_and_first_incoming_voice_survive_genuine_reset() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let groups = p.c.owner.actual_measure_beam_groups(1).unwrap();
        let mut first = (0..3).map(|_| None).collect::<Vec<_>>();
        let physical = p.c.owner.chords.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let staffs = p.c.owner.owner.staffs.clone();
        for step in case["phases"].as_array().unwrap() {
            let phase = step["phase"].as_str().unwrap();
            match phase {
                "unlinked" => {}
                "linked" => {
                    for (ch, t, m) in [(0, 0, true), (1, 0, false), (4, 1, true), (0, 1, false)] {
                        link(&mut p, ch, t, m);
                    }
                }
                "declared_shape_changed" => {
                    for (i, shape) in [(0, "TUPLET_SIX"), (1, "TUPLET_THREE")] {
                        let key = p.tuplets[i];
                        let mut n = node(&p, key);
                        n["shape"] = json!(shape);
                        p.c.owner.owner.graph.vertex(key, n, false).unwrap();
                    }
                }
                "note_staff_null" | "notes_restored" => {
                    for i in [0, 1] {
                        let key = chord_key(&p.c, &format!("chord{i}"));
                        let mut n = node(&p, key);
                        n["staff_id"] = Value::Null;
                        p.c.owner.owner.graph.vertex(key, n, false).unwrap();
                        let members =
                            p.c.owner
                                .chords
                                .iter()
                                .find(|ch| ch.key == key)
                                .unwrap()
                                .members
                                .clone();
                        for member in members {
                            p.c.owner
                                .owner
                                .graph
                                .edit_note_owner(
                                    serde_json::from_value(member["origin"].clone()).unwrap(),
                                    NoteOwnerEdit::Staff(if phase == "notes_restored" {
                                        Some(1)
                                    } else {
                                        None
                                    }),
                                )
                                .unwrap();
                        }
                    }
                }
                "tuple_staff_cleared" => {
                    p.c.owner
                        .set_rhythm_tuplet_staff(p.tuplets[0], None)
                        .unwrap()
                }
                "first_link_removed" => {
                    p.c.owner
                        .remove_implicit_tuplet_link(p.relations[0].0)
                        .unwrap()
                }
                "late_faster_link" => {
                    link(&mut p, 2, 1, true);
                    link(&mut p, 0, 2, true);
                }
                "measure_reset" => reset_rhythm(
                    &mut p.c.owner.owner.graph,
                    &mut p.c.owner.voices,
                    &mut p.c.owner.measures[0],
                    &p.c.owner.chords,
                    0,
                )
                .unwrap(),
                "fresh_number_reused" => {
                    let key = chord_key(&p.c, "chord0");
                    let seed = p.c.owner.chords.iter().find(|ch| ch.key == key).unwrap();
                    let vi = create_voice(
                        &mut p.c.owner.owner.graph,
                        &mut p.c.owner.voices,
                        &p.c.owner.measures,
                        seed,
                    )
                    .unwrap();
                    assert_eq!(p.c.owner.voices[vi].id, 1);
                    assert!(!Arc::ptr_eq(
                        &p.c.owner.voices[vi].identity,
                        &p.c.refs[0].identity
                    ));
                    p.c.refs.push(p.c.owner.voices[vi].clone());
                }
                _ => panic!("unknown source boundary"),
            }
            assert_eq!(
                rows(&mut p, &mut first),
                step["getters"],
                "{} {phase}",
                case["name"]
            );
            let relations = json!(
                p.relations
                    .iter()
                    .map(|(id, label)| {
                        let mut v = p.c.owner.owner.graph.tuplet_relation_registration(*id);
                        v["label"] = json!(label);
                        v
                    })
                    .collect::<Vec<_>>()
            );
            assert_eq!(relations, step["relations"]);
            let mut expected = step["state"].clone();
            expected["owned"]["relations"] = json!([]);
            assert_eq!(
                frame_with_groups(&mut p.c, &groups),
                expected,
                "{} {phase}",
                case["name"]
            );
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            for (a, b) in staffs.iter().zip(&p.c.owner.owner.staffs) {
                assert!(Arc::ptr_eq(a, b));
                for (a, b) in a.lines.iter().zip(&b.lines) {
                    assert!(Arc::ptr_eq(a, b));
                }
            }
        }
    }
}
#[test]
fn late_tuplet_read_failure_restores_cold_and_warm_cache_references_real_chords_and_slots() {
    let original = original();
    let mut p = control(&original["cases"][0]);
    link(&mut p, 0, 0, true);
    link(&mut p, 1, 0, false);
    let key = p.tuplets[0];
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let second = chord_key(&p.c, "chord1");
    let ch = p.c.owner.chords.iter().find(|ch| ch.key == second).unwrap();
    let stem = ch.stem.unwrap();
    p.c.owner
        .owner
        .graph
        .invalidate_owned_head_chord_cache(second)
        .unwrap();
    let saved = node(&p, stem);
    let mut bad = saved.clone();
    bad["bounds"][2] = json!(0);
    p.c.owner.owner.graph.vertex(stem, bad, false).unwrap();
    let graph = p.c.owner.owner.graph.snapshot();
    let cache = p.c.owner.owner.graph.tuplet_cache_observation(key);
    assert!(p.c.owner.read_rhythm_tuplet(key).is_err());
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(p.c.owner.owner.graph.tuplet_cache_observation(key), cache);
    p.c.owner.owner.graph.vertex(stem, saved, false).unwrap();
    let first = p.c.owner.read_rhythm_tuplet(key).unwrap();
    let graph = p.c.owner.owner.graph.snapshot();
    let original_id =
        p.c.owner
            .chords
            .iter()
            .find(|ch| ch.key == second)
            .unwrap()
            .id;
    p.c.owner
        .chords
        .iter_mut()
        .find(|ch| ch.key == second)
        .unwrap()
        .id += 100000;
    assert_eq!(
        p.c.owner.read_rhythm_tuplet(key).unwrap_err(),
        "music_actual_tuplet_chord_owner_required"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    p.c.owner
        .chords
        .iter_mut()
        .find(|ch| ch.key == second)
        .unwrap()
        .id = original_id;
    let retry = p.c.owner.read_rhythm_tuplet(key).unwrap();
    assert!(Arc::ptr_eq(&first.factor, &retry.factor));
    assert!(Arc::ptr_eq(
        first.base.as_ref().unwrap(),
        retry.base.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        first.staff.as_ref().unwrap(),
        retry.staff.as_ref().unwrap()
    ));
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    assert!(p.c.owner.remove_implicit_tuplet_link(0).is_err());
    assert!(p.c.owner.set_rhythm_tuplet_staff(key, Some(999)).is_err());
}
