// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_getters_tests::node;
use super::tuplet_preparation_tests::control;
use super::*;
use crate::shared_sig::Key;

fn inflate(text: &str) -> Value {
    let mut d: Value = serde_json::from_str(text).unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for k in ["before", "after"] {
            if c.get(k).is_some() {
                c[k] = fs[c[k].as_u64().unwrap() as usize].clone();
            }
        }
        if let Some(ph) = c.get_mut("phases") {
            for p in ph.as_array_mut().unwrap() {
                p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
            }
        }
    }
    d
}
fn original() -> Value {
    inflate(include_str!(
        "../../../fixtures/official-tuplet-sans/observed.json"
    ))
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_sans_tuplet_raw_dots_and_measure_rest_shape_ignore_real_manual_six_relation() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let relations = p.c.owner.owner.graph.snapshot()["relations"].clone();
        assert_eq!(frame(&mut p.c), normalized(case["before"].clone()));
        for row in case["rows"].as_array().unwrap() {
            let key = chord_key(&p.c, row["chord"].as_str().unwrap());
            let ch = p.c.owner.chords.iter().find(|ch| ch.key == key).unwrap();
            let note: Key = serde_json::from_value(ch.members[0]["origin"].clone()).unwrap();
            let saved_note = node(&p, note);
            let saved_chord = node(&p, key);
            let mut n = saved_note.clone();
            n["shape"] = row["shape"].clone();
            let mut c = saved_chord.clone();
            c["dots_number"] = row["dots"].clone();
            p.c.owner.owner.graph.vertex(note, n, false).unwrap();
            p.c.owner.owner.graph.vertex(key, c, false).unwrap();
            let before = p.c.owner.owner.graph.snapshot();
            match p.c.owner.sans_tuplet_rhythm_chord_duration(key) {
                Ok(duration) => {
                    assert!(row["error"].is_null());
                    assert_eq!(json!(duration), row["duration"]);
                }
                Err(e) => {
                    assert_eq!(row["error"], "NullPointerException");
                    assert_eq!(e, "music_raw_null_duration_with_beams_flags");
                    assert_eq!(p.c.owner.owner.graph.snapshot(), before);
                }
            }
            p.c.owner
                .owner
                .graph
                .vertex(note, saved_note, false)
                .unwrap();
            p.c.owner
                .owner
                .graph
                .vertex(key, saved_chord, false)
                .unwrap();
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            assert_eq!(p.c.owner.owner.graph.snapshot()["relations"], relations);
        }
        for row in case["voice_rows"].as_array().unwrap() {
            let vi = row["voice"]
                .as_str()
                .unwrap()
                .strip_prefix("voice")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let identity = p.c.refs[vi].identity.clone();
            let ids = p.c.refs[vi].chord_ids.clone();
            let mut saved = Vec::new();
            for id in &ids {
                let key = p.c.owner.chords.iter().find(|ch| ch.id == *id).unwrap().key;
                let n = node(&p, key);
                let mut changed = n.clone();
                changed["dots_number"] = row["dots"].clone();
                p.c.owner.owner.graph.vertex(key, changed, false).unwrap();
                saved.push((key, n));
            }
            assert_eq!(
                json!(
                    ids.iter()
                        .map(|id| p.c.labels[id].clone())
                        .collect::<Vec<_>>()
                ),
                row["chords"]
            );
            assert_eq!(
                json!(
                    p.c.owner
                        .sans_tuplet_rhythm_voice_duration(1, &identity)
                        .unwrap()
                ),
                row["sum"]
            );
            for (key, n) in saved {
                p.c.owner.owner.graph.vertex(key, n, false).unwrap();
            }
            assert_eq!(p.c.owner.owner.graph.snapshot()["relations"], relations);
        }
        assert_eq!(case["same_tuplet"], true);
        assert_eq!(case["same_manual_relation"], true);
        assert_eq!(frame(&mut p.c), normalized(case["after"].clone()));
    }
}
fn retired_frame(
    p: &mut super::tuplet_preparation_tests::Prepared,
    baseline: &BTreeMap<Key, Option<Arc<()>>>,
) -> Value {
    let old_count = p.c.refs.len();
    for v in &p.c.owner.voices {
        if v.measure == 0
            && !p
                .c
                .refs
                .iter()
                .any(|r| Arc::ptr_eq(&r.identity, &v.identity))
        {
            p.c.refs.push(v.clone());
        }
    }
    let mut observed = frame(&mut p.c);
    p.c.refs.truncate(old_count);
    observed["owned"]["voices"]
        .as_array_mut()
        .unwrap()
        .truncate(old_count);
    for label in observed["owned"]["measure_voices"].as_array_mut().unwrap() {
        if label
            .as_str()
            .unwrap()
            .strip_prefix("voice")
            .unwrap()
            .parse::<usize>()
            .unwrap()
            >= old_count
        {
            *label = Value::Null;
        }
    }
    for row in observed["owned"]["chords"].as_array_mut().unwrap() {
        if row["voice"].as_str().is_some_and(|s| {
            s.strip_prefix("voice").unwrap().parse::<usize>().unwrap() >= old_count
        }) {
            row["voice"] = Value::Null;
        }
        let key = chord_key(&p.c, row["label"].as_str().unwrap());
        let current = p.c.owner.owner.graph.chord_slot_identity(key);
        // Labels and reused slot numbers cannot stand in for the actual
        // baseline Slot reference held by the original observer.
        row["same_baseline_slot"] = json!(match (current, baseline[&key].as_ref()) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        });
    }
    observed
}
#[test]
fn original_retired_voice_sum_keeps_same_actual_chords_after_reset_and_number_reuse() {
    let d = inflate(include_str!(
        "../../../fixtures/official-tuplet-sans-retired/observed.json"
    ));
    for case in d["cases"].as_array().unwrap() {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let baseline =
            p.c.owner
                .chords
                .iter()
                .filter(|ch| ch.measure == 0)
                .map(|ch| {
                    (
                        ch.key,
                        p.c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
        for step in case["phases"].as_array().unwrap() {
            match step["phase"].as_str().unwrap() {
                "before" => {}
                "measure_reset" => reset_rhythm(
                    &mut p.c.owner.owner.graph,
                    &mut p.c.owner.voices,
                    &mut p.c.owner.measures[0],
                    &p.c.owner.chords,
                    0,
                )
                .unwrap(),
                "reprocessed" => p.c.owner.reprocess_page_stack_rhythm(1).unwrap(),
                _ => panic!("unknown source boundary"),
            }
            let mut old = Vec::new();
            for i in 0..p.c.refs.len() {
                let voice = p.c.refs[i].clone();
                let sum =
                    p.c.owner
                        .sans_tuplet_rhythm_voice_duration(1, &voice.identity)
                        .unwrap();
                old.push(json!({"voice":format!("voice{i}"),"musical_id":voice.id,"chords":voice.chord_ids.iter().map(|id|p.c.labels[id].clone()).collect::<Vec<_>>(),"sum":sum,"registered":p.c.owner.voices.iter().any(|v|Arc::ptr_eq(&v.identity,&voice.identity))}));
            }
            assert_eq!(json!(old), step["old_voices"]);
            let current =
                p.c.owner
                    .voices
                    .iter()
                    .filter(|v| v.measure == 0)
                    .cloned()
                    .collect::<Vec<_>>();
            let rows=current.iter().map(|v|{
                let sum=p.c.owner.sans_tuplet_rhythm_voice_duration(1,&v.identity).unwrap();
                json!({"musical_id":v.id,"old_object":p.c.refs.iter().any(|old|Arc::ptr_eq(&old.identity,&v.identity)),"sum":sum})
            }).collect::<Vec<_>>();
            assert_eq!(json!(rows), step["current_voices"]);
            assert_eq!(
                retired_frame(&mut p, &baseline),
                normalized(step["state"].clone())
            );
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
        }
    }
}
#[test]
fn late_sans_tuplet_sum_failure_rolls_back_and_unknown_voice_never_reuses_a_number() {
    let d = original();
    let mut p = control(&d["cases"][0]);
    let identity = p.c.refs[0].identity.clone();
    let key = chord_key(&p.c, "chord2");
    let ch = p.c.owner.chords.iter().find(|ch| ch.key == key).unwrap();
    let note: Key = serde_json::from_value(ch.members[0]["origin"].clone()).unwrap();
    let old = node(&p, note);
    let mut bad = old.clone();
    bad["shape"] = json!("G_CLEF");
    p.c.owner.owner.graph.vertex(note, bad, false).unwrap();
    let graph = p.c.owner.owner.graph.snapshot();
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    assert_eq!(
        p.c.owner
            .sans_tuplet_rhythm_voice_duration(1, &identity)
            .unwrap_err(),
        "music_raw_null_duration_with_beams_flags"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    p.c.owner.owner.graph.vertex(note, old, false).unwrap();
    assert_eq!(
        p.c.owner
            .sans_tuplet_rhythm_voice_duration(1, &identity)
            .unwrap(),
        Some("1".into())
    );
    assert_eq!(
        p.c.owner
            .sans_tuplet_rhythm_voice_duration(1, &Arc::new(()))
            .unwrap_err(),
        "music_actual_tuplet_voice_owner_required"
    );
}
