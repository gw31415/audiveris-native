// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_force_tests::prepare;
use super::tuplet_getters_tests::node;
use super::*;
use crate::{music_font_symbols::TupletFontSymbols, music_time::Fraction};
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-find/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for s in c["scenarios"].as_array_mut().unwrap() {
            for k in ["before", "generated", "removed"] {
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
fn original_find_zero_start_exact_sans_sum_null_expected_and_force_keep_actual_voice_clocks() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = prepare(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let font =
            TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
                .unwrap();
        for s in case["scenarios"].as_array().unwrap() {
            let saved_expected = p.c.owner.measures[0].timing.expected;
            p.c.owner.measures[0].timing.expected = s["expected"]
                .as_str()
                .map(Fraction::parse)
                .transpose()
                .unwrap();
            let expected = p.c.owner.measures[0].timing.expected;
            let mut saved = BTreeMap::new();
            for row in s["firsts"].as_array().unwrap() {
                let key = chord_key(&p.c, row["chord"].as_str().unwrap());
                let old = node(&p, key);
                saved.insert(key, old.clone());
                let mut n = old;
                n["time_offset"] = row["time"].clone();
                p.c.owner.owner.graph.vertex(key, n, false).unwrap();
            }
            if s["dots"] == 1 {
                for name in s["dot_chords"].as_array().unwrap() {
                    let key = chord_key(&p.c, name.as_str().unwrap());
                    let old = node(&p, key);
                    saved.entry(key).or_insert(old.clone());
                    let mut n = old;
                    n["dots_number"] = json!(1);
                    p.c.owner.owner.graph.vertex(key, n, false).unwrap();
                }
            }
            let before = frame(&mut p.c);
            assert_eq!(before, normalized(s["before"].clone()));
            let graph = p.c.owner.owner.graph.snapshot();
            p.c.owner.owner.graph.trace_tuplet_additions();
            match p.c.owner.find_rhythm_implicit_tuplets(1, "1/4", &font) {
                Ok(keys) => {
                    assert!(s["error"].is_null());
                    assert_eq!(keys.len() as u64, s["returned_count"].as_u64().unwrap());
                    let events = p.c.owner.owner.graph.take_tuplet_additions();
                    assert_eq!(
                        events
                            .iter()
                            .filter(|cb| cb.phase == "vertex_registered")
                            .count() as u64,
                        s["fresh_count"].as_u64().unwrap()
                    );
                    let mut selected = Vec::<Arc<()>>::new();
                    for key in keys {
                        let identity =
                            p.c.owner
                                .owner
                                .graph
                                .tuplet_voice_identity(key)
                                .unwrap()
                                .unwrap();
                        if !selected.iter().any(|v| Arc::ptr_eq(v, &identity)) {
                            selected.push(identity);
                        }
                    }
                    assert_eq!(
                        json!(
                            selected
                                .iter()
                                .map(|v| format!(
                                    "voice{}",
                                    p.c.refs
                                        .iter()
                                        .position(|r| Arc::ptr_eq(&r.identity, v))
                                        .unwrap()
                                ))
                                .collect::<Vec<_>>()
                        ),
                        s["selected"]
                    );
                }
                Err(error) => {
                    assert_eq!(
                        error,
                        if s["error"] == "NullPointerException" {
                            "music_tuplet_expected_duration_required"
                        } else {
                            assert_eq!(s["error"], "IllegalArgumentException");
                            "music_tuplet_zero_expected_duration"
                        }
                    );
                    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
                    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
                }
            }
            assert_eq!(frame(&mut p.c), normalized(s["generated"].clone()));
            p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
            assert_eq!(frame(&mut p.c), normalized(s["removed"].clone()));
            assert_eq!(p.c.owner.measures[0].timing.expected, expected);
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            for (key, n) in saved {
                p.c.owner.owner.graph.vertex(key, n, false).unwrap();
            }
            p.c.owner.measures[0].timing.expected = saved_expected;
        }
    }
}
#[test]
fn skipped_null_starts_bypass_later_duration_expected_beat_and_font_without_fabricated_voice() {
    let d = original();
    let mut p = prepare(&d["cases"][0]);
    let firsts = p.c.refs.iter().map(|v| v.chord_ids[0]).collect::<Vec<_>>();
    for id in firsts {
        let key = p.c.owner.chords.iter().find(|ch| ch.id == id).unwrap().key;
        let mut n = node(&p, key);
        n["time_offset"] = Value::Null;
        p.c.owner.owner.graph.vertex(key, n, false).unwrap();
    }
    let later = chord_key(&p.c, "chord2");
    let ch = p.c.owner.chords.iter().find(|ch| ch.key == later).unwrap();
    let note = serde_json::from_value(ch.members[0]["origin"].clone()).unwrap();
    let mut n = node(&p, note);
    n["shape"] = json!("G_CLEF");
    p.c.owner.owner.graph.vertex(note, n, false).unwrap();
    p.c.owner.measures[0].timing.expected = None;
    let graph = p.c.owner.owner.graph.snapshot();
    let identities =
        p.c.owner
            .voices
            .iter()
            .map(|v| v.identity.clone())
            .collect::<Vec<_>>();
    let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
    assert!(
        p.c.owner
            .find_rhythm_implicit_tuplets(1, "invalid-beat", &wrong)
            .unwrap()
            .is_empty()
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    for (v, id) in p.c.owner.voices.iter().zip(identities) {
        assert!(Arc::ptr_eq(&v.identity, &id));
    }
}
