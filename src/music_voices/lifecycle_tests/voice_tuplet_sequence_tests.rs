// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::control;
use super::*;

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-voice-tuplet-sequence/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
#[test]
fn original_voice_tuplet_getters_keep_first_relation_null_list_absent_successor_and_purge_order() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let old = p.c.refs[0].identity.clone();
        let keys = p.c.refs[0]
            .chord_ids
            .iter()
            .map(|id| p.c.owner.chords.iter().find(|ch| ch.id == *id).unwrap().key)
            .collect::<Vec<_>>();
        let foreign =
            p.c.owner
                .chords
                .iter()
                .find(|ch| ch.measure == 1)
                .unwrap()
                .key;
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let expected = p.c.owner.measures[0].timing.expected;
        let report = p.c.owner.owner.prepared.report.clone();
        // Own labels remain stable when old registry/live list membership changes.
        let chord_labels =
            p.c.owner
                .chords
                .iter()
                .filter_map(|ch| p.c.labels.get(&ch.id).map(|l| (ch.key, l.clone())))
                .collect::<BTreeMap<_, _>>();
        for phase in case["phases"].as_array().unwrap() {
            match phase["phase"].as_str().unwrap() {
                "before" => {}
                "allocated_empty" => {
                    for &key in &keys {
                        assert!(p.c.owner.remove_rhythm_voice_chord(&old, key).unwrap());
                    }
                }
                "reset_null" => {
                    for &key in &keys {
                        p.c.owner.add_rhythm_voice_chord(&old, key).unwrap();
                    }
                    p.c.owner.reset_rhythm_voice_chords(&old).unwrap();
                }
                "purged_null" => p.c.owner.purge_rhythm_measure_voices(1).unwrap(),
                "restored" => {
                    for &key in &keys {
                        p.c.owner.add_rhythm_voice_chord(&old, key).unwrap();
                    }
                    let held =
                        p.c.owner
                            .owner
                            .graph
                            .retained_voice_owner(&old)
                            .unwrap()
                            .clone();
                    p.c.owner.voices.push(held);
                }
                _ => panic!("unknown source phase"),
            }
            let state = frame(&mut p.c);
            let graph = p.c.owner.owner.graph.snapshot();
            let selected =
                p.c.owner
                    .rhythm_voices_with_implicit_tuplets(1)
                    .unwrap()
                    .iter()
                    .map(|id| {
                        format!(
                            "voice{}",
                            p.c.refs
                                .iter()
                                .position(|v| Arc::ptr_eq(id, &v.identity))
                                .unwrap()
                        )
                    })
                    .collect::<Vec<_>>();
            let mut queries = Vec::new();
            for (vi, reference) in p.c.refs.iter().enumerate() {
                let identity = &reference.identity;
                let (tuplets, error) = match p.c.owner.rhythm_voice_tuplets(1, identity) {
                    Ok(keys) => (
                        json!(
                            keys.iter()
                                .map(|key| p.c.labels[&p.c.owner.owner.graph.id(*key).unwrap()]
                                    .clone())
                                .collect::<Vec<_>>()
                        ),
                        Value::Null,
                    ),
                    Err(error) => {
                        assert_eq!(error, "music_tuplet_null_voice_chord_list");
                        (Value::Null, json!("NullPointerException"))
                    }
                };
                let last =
                    p.c.owner
                        .last_rhythm_voice_tuplet_chord(1, identity)
                        .unwrap()
                        .map(|key| chord_labels[&key].clone());
                let after = phase["queries"][vi]["after"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|q| {
                        let input = q["input"].as_str().map(|name| {
                            if name == "foreign_chord" {
                                foreign
                            } else {
                                chord_key(&p.c, name)
                            }
                        });
                        let next =
                            p.c.owner
                                .rhythm_voice_chord_after(1, identity, input)
                                .unwrap()
                                .map(|key| chord_labels[&key].clone());
                        json!({"input":q["input"],"after":next})
                    })
                    .collect::<Vec<_>>();
                queries.push(json!({"voice":format!("voice{vi}"),"tuplets":tuplets,"tuplets_error":error,"last_with_tuplet":last,"after":after}));
            }
            assert_eq!(json!(selected), phase["selected"]);
            assert_eq!(json!(queries), phase["queries"]);
            let mut source = phase["state"].clone();
            source["owned"]["relations"] = json!([]);
            assert_eq!(state, source);
            assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.measures[0].timing.expected, expected);
            assert_eq!(p.c.owner.owner.prepared.report, report);
        }
    }
}
