// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame, prepare_control};
use super::*;
use crate::{
    music_font_symbols::TupletFontSymbols,
    music_measures::ChordKind,
    shared_sig::{Key, Phase},
};

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-find-inputs/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for k in ["before", "allocated_empty", "empty"] {
            c[k] = fs[c[k].as_u64().unwrap() as usize].clone();
        }
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
fn normalized(mut f: Value) -> Value {
    f["owned"]["relations"] = json!([]);
    f
}
fn create_rest(c: &mut Control, declaration: &Value) -> (Key, usize) {
    let snap = c.owner.owner.graph.snapshot();
    let local = |phase| {
        snap["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| serde_json::from_value::<Key>(n["origin"].clone()).unwrap())
            .filter(|k| k.system == 1 && k.phase == phase)
            .map(|k| k.local)
            .max()
            .unwrap_or(0)
            + 1
    };
    let rest = Key {
        system: 1,
        phase: Phase::Symbol,
        local: local(Phase::Symbol),
    };
    let key = Key {
        system: 1,
        phase: Phase::Chord,
        local: local(Phase::Chord),
    };
    let bounds: [i64; 4] = serde_json::from_value(declaration["bounds"].clone()).unwrap();
    let id = c.owner.owner.graph.vertex(rest, json!({"kind":"RestInter","shape":declaration["shape"],"bounds":bounds,"grade":0.8,"staff_id":1}), false).unwrap();
    let cid = c.owner.owner.graph.vertex(key, json!({"kind":"RestChordInter","staff_id":1,"part_id":1,"measure_index":c.owner.measures[0].index,"bounds":bounds,"voice":null,"member_ids":[id]}), false).unwrap();
    c.owner
        .owner
        .graph
        .relations(
            1,
            Phase::Chord,
            vec![(key, rest, json!({"kind":"Containment"}))],
        )
        .unwrap();
    let member = c.owner.owner.graph.snapshot()["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap()
        .clone();
    let chord = Chord {
        kind: ChordKind::Rest,
        key,
        id: cid,
        stem: None,
        members: vec![member],
        measure: 0,
        bounds,
    };
    c.owner.chords.push(chord.clone());
    c.labels.insert(id, "rest0".into());
    c.labels.insert(cid, "rest_chord0".into());
    c.rests.push(key);
    c.baseline
        .push(c.owner.owner.graph.rest_chord_beams(key).unwrap());
    let vi = create_voice(
        &mut c.owner.owner.graph,
        &mut c.owner.voices,
        &c.owner.measures,
        &chord,
    )
    .unwrap();
    c.refs.push(c.owner.voices[vi].clone());
    (key, vi)
}
fn rest_frame(c: &mut Control) -> Value {
    let mut f = frame(c);
    for row in f["rests"].as_array_mut().unwrap() {
        let key = c
            .owner
            .chords
            .iter()
            .find(|ch| {
                c.labels
                    .get(&ch.id)
                    .is_some_and(|label| json!(label) == row["label"])
            })
            .unwrap()
            .key;
        row["voice"] = c
            .owner
            .owner
            .graph
            .chord_voice_owner(key, &c.owner.voices)
            .unwrap()
            .map(|v| {
                format!(
                    "voice{}",
                    c.refs
                        .iter()
                        .position(|r| Arc::ptr_eq(&r.identity, &v.identity))
                        .unwrap()
                )
            })
            .into();
    }
    f
}
#[test]
fn original_null_reset_list_and_allocated_empty_list_keep_live_voice_clocks_and_backrefs() {
    for case in original()["cases"].as_array().unwrap() {
        let mut c = prepare_control(case);
        let old = c.refs[0].identity.clone();
        let keys = c.refs[0]
            .chord_ids
            .iter()
            .map(|id| c.owner.chords.iter().find(|ch| ch.id == *id).unwrap().key)
            .collect::<Vec<_>>();
        let physical = c.owner.chords.as_ptr();
        let slots = c.owner.measures[0].timing.slots.as_ptr();
        let report = c.owner.owner.prepared.report.clone();
        let graph = c.owner.owner.graph.snapshot();
        let font = TupletFontSymbols::load("Bravura", c.owner.owner.interline).unwrap();
        assert_eq!(frame(&mut c), normalized(case["before"].clone()));
        for &key in &keys {
            assert!(c.owner.remove_rhythm_voice_chord(&old, key).unwrap());
        }
        assert!(!c.owner.remove_rhythm_voice_chord(&old, keys[0]).unwrap());
        assert_eq!(frame(&mut c), normalized(case["allocated_empty"].clone()));
        assert_eq!(
            json!(c.owner.sans_tuplet_rhythm_voice_duration(1, &old).unwrap()),
            case["allocated_empty_sum"]
        );
        assert_eq!(
            json!(!actual_voice(&c.owner, &c.refs[0]).chords_initialized),
            case["allocated_chords_null"]
        );
        for &key in keys.iter().rev() {
            c.owner.add_rhythm_voice_chord(&old, key).unwrap();
        }
        c.owner.add_rhythm_voice_chord(&old, keys[0]).unwrap();
        assert_eq!(frame(&mut c), normalized(case["before"].clone()));
        c.owner.reset_rhythm_voice_chords(&old).unwrap();
        assert_eq!(
            json!(!actual_voice(&c.owner, &c.refs[0]).chords_initialized),
            case["empty_chords_null"]
        );
        assert_eq!(case["empty_sum_error"], "NullPointerException");
        assert_eq!(
            c.owner
                .sans_tuplet_rhythm_voice_duration(1, &old)
                .unwrap_err(),
            "music_tuplet_null_voice_chord_list"
        );
        assert_eq!(case["empty_error"], "IndexOutOfBoundsException");
        assert_eq!(
            c.owner
                .find_rhythm_implicit_tuplets(1, "invalid-beat", &font)
                .unwrap_err(),
            "music_tuplet_empty_voice_chord_list"
        );
        assert_eq!(frame(&mut c), normalized(case["empty"].clone()));
        assert_eq!(c.owner.owner.graph.snapshot(), graph);
        assert_eq!(c.owner.chords.as_ptr(), physical);
        assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slots);
        assert_eq!(c.owner.owner.prepared.report, report);
        assert!(!c.owner.remove_rhythm_voice_chord(&old, keys[0]).unwrap());
        for &key in &keys {
            c.owner.add_rhythm_voice_chord(&old, key).unwrap();
        }
        assert_eq!(frame(&mut c), normalized(case["before"].clone()));
    }
}
#[test]
fn original_real_measure_rest_constructor_and_reset_bypass_find_before_missing_expected() {
    for case in original()["cases"].as_array().unwrap() {
        let mut c = prepare_control(case);
        for reference in &c.refs {
            let key = c
                .owner
                .chords
                .iter()
                .find(|ch| ch.id == reference.chord_ids[0])
                .unwrap()
                .key;
            let snapshot = c.owner.owner.graph.snapshot();
            let mut n = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == c.owner.owner.graph.id(key).unwrap())
                .unwrap()
                .clone();
            n["time_offset"] = Value::Null;
            c.owner.owner.graph.vertex(key, n, false).unwrap();
        }
        c.owner.measures[0].timing.expected = None;
        let (key, vi) = create_rest(&mut c, &case["rest_declaration"]);
        let identity = c.owner.voices[vi].identity.clone();
        let physical = c.owner.chords.as_ptr();
        let slots = c.owner.measures[0].timing.slots.as_ptr();
        let graph = c.owner.owner.graph.snapshot();
        let wrong = TupletFontSymbols::load("Leland", c.owner.owner.interline + 1).unwrap();
        for phase in case["phases"].as_array().unwrap() {
            if phase["phase"] == "rest_chords_reset" {
                c.owner.reset_rhythm_voice_chords(&identity).unwrap();
            }
            assert!(
                c.owner
                    .find_rhythm_implicit_tuplets(1, "invalid-beat", &wrong)
                    .unwrap()
                    .is_empty()
            );
            let voice = &c.owner.voices[vi];
            let row = json!({"phase":phase["phase"],"id":voice.id,"measure_rest":voice.measure_rest_chord.is_some(),"chords_null":!voice.chords_initialized,"chords":voice.chord_ids.iter().map(|id|c.labels[id].clone()).collect::<Vec<_>>(),"same_rest_chord":voice.measure_rest_chord==Some(c.owner.owner.graph.id(key).unwrap()),"rest_chord_voice":Arc::ptr_eq(&c.owner.owner.graph.chord_voice_owner(key,&c.owner.voices).unwrap().unwrap().identity,&identity),"registered":true,"found_count":0,"sum":c.owner.sans_tuplet_rhythm_voice_duration(1,&identity).unwrap(),"state":rest_frame(&mut c)});
            let mut expected = phase.clone();
            expected["state"] = normalized(expected["state"].clone());
            assert_eq!(row, expected);
            assert_eq!(c.owner.owner.graph.snapshot(), graph);
            assert_eq!(c.owner.chords.as_ptr(), physical);
            assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slots);
        }
    }
}
