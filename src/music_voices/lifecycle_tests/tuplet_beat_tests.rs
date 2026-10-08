// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_getters_tests::node;
use super::tuplet_reuse_tests::prepare;
use super::*;

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-beat/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for s in c["scenarios"].as_array_mut().unwrap() {
            for k in ["before", "after"] {
                s[k] = fs[s[k].as_u64().unwrap() as usize].clone();
            }
        }
    }
    d
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_real_voice_beat_groups_keep_order_single_gap_step_null_failure_and_half_fallback() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = prepare(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let voices = projection(&p.c.owner.voices, &p.c.owner.measures);
        for s in case["scenarios"].as_array().unwrap() {
            let mut saved = Vec::new();
            for row in s["times"].as_array().unwrap() {
                let key = chord_key(&p.c, row["chord"].as_str().unwrap());
                let old = node(&p, key);
                let mut next = old.clone();
                next["time_offset"] = row["time"].clone();
                p.c.owner.owner.graph.vertex(key, next, false).unwrap();
                saved.push((key, old));
            }
            let before = frame(&mut p.c);
            assert_eq!(before, normalized(s["before"].clone()));
            let vi = s["voice"]
                .as_str()
                .unwrap()
                .strip_prefix("voice")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let identity = p.c.refs[vi].identity.clone();
            let graph = p.c.owner.owner.graph.snapshot();
            match p
                .c
                .owner
                .plan_rhythm_voice_beats(1, &identity, s["beat"].as_str().unwrap())
            {
                Ok(plan) => {
                    assert!(s["error"].is_null());
                    assert!(Arc::ptr_eq(&plan.voice, &identity));
                    assert_eq!(
                        json!(
                            plan.groups
                                .iter()
                                .map(|g| g
                                    .iter()
                                    .map(|k| p.c.labels[&p.c.owner.owner.graph.id(*k).unwrap()]
                                        .clone())
                                    .collect::<Vec<_>>())
                                .collect::<Vec<_>>()
                        ),
                        s["groups"]
                    );
                    assert_eq!(
                        json!(
                            plan.pivot
                                .map(|k| p.c.labels[&p.c.owner.owner.graph.id(k).unwrap()].clone())
                        ),
                        s["pivot"]
                    );
                    assert_eq!(plan.fallback, s["fallback"] == true);
                }
                Err(e) => {
                    assert_eq!(s["error"], "NullPointerException");
                    assert_eq!(e, "music_tuplet_null_group_chord_time");
                    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
                }
            }
            assert_eq!(frame(&mut p.c), normalized(s["after"].clone()));
            assert_eq!(projection(&p.c.owner.voices, &p.c.owner.measures), voices);
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            for (key, old) in saved {
                p.c.owner.owner.graph.vertex(key, old, false).unwrap();
            }
        }
    }
}
#[test]
fn null_clock_precedes_later_invalid_owner_and_actual_voice_identity_is_required() {
    let d = original();
    let mut p = prepare(&d["cases"][0]);
    let voice = p.c.refs[0].identity.clone();
    let first = chord_key(&p.c, "chord1");
    let second = chord_key(&p.c, "chord2");
    let saved_first = node(&p, first);
    let mut null = saved_first.clone();
    null["time_offset"] = Value::Null;
    p.c.owner.owner.graph.vertex(first, null, false).unwrap();
    let saved_second = node(&p, second);
    let mut bad = saved_second.clone();
    bad["part_id"] = json!(999);
    p.c.owner.owner.graph.vertex(second, bad, false).unwrap();
    let snapshot = p.c.owner.owner.graph.snapshot();
    let physical = p.c.owner.chords.as_ptr();
    assert_eq!(
        p.c.owner
            .plan_rhythm_voice_beats(1, &voice, "1/2")
            .unwrap_err(),
        "music_tuplet_null_group_chord_time"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), snapshot);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    p.c.owner
        .owner
        .graph
        .vertex(first, saved_first, false)
        .unwrap();
    assert_eq!(
        p.c.owner
            .plan_rhythm_voice_beats(1, &voice, "1/2")
            .unwrap_err(),
        "music_actual_tuplet_chord_owner_required"
    );
    p.c.owner
        .owner
        .graph
        .vertex(second, saved_second, false)
        .unwrap();
    p.c.owner.plan_rhythm_voice_beats(1, &voice, "1/2").unwrap();
    assert_eq!(
        p.c.owner
            .plan_rhythm_voice_beats(1, &Arc::new(()), "1/2")
            .unwrap_err(),
        "music_actual_tuplet_voice_owner_required"
    );
}
