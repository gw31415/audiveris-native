// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_placement_tests::{declare, original};
use super::tuplet_preparation_tests::{Prepared, control};
use super::*;
use crate::{
    music_font_symbols::TupletFontSymbols,
    shared_sig::{Key, TupletAdditionObservation},
};

fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
fn callback(p: &mut Prepared, cb: &TupletAdditionObservation) -> Value {
    // A clone of the actual observed Graph retains its real Voice/Slot Arcs;
    // snapshots/reports never recreate these owners for the comparison.
    let saved = std::mem::replace(&mut p.c.owner.owner.graph, cb.graph.clone());
    let clocks = frame(&mut p.c)["owned"].clone();
    p.c.owner.owner.graph = saved;
    let graph = cb.graph.snapshot();
    let id = cb.graph.id(cb.tuplet).unwrap();
    let node = graph["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap();
    let mut row = json!({"phase":cb.phase,"registered":true,"sig_attached":cb.sig_attached,"abnormal":node["abnormal"]==true,"measure_contains":cb.measure_tuplets.contains(&cb.tuplet),"stack_contains":cb.stack_tuplets.contains(&cb.tuplet),"clocks":clocks});
    if let Some(edge) = cb.edge {
        let relation = graph["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == edge)
            .unwrap();
        row["source"] = json!(p.c.labels[&(relation["source"].as_u64().unwrap() as usize)]);
        row["manual"] = json!(relation["manual"] == true);
        row["implicit"] = node["implicit"].clone();
    }
    row
}
fn result(p: &mut Prepared, key: Key, font: &TupletFontSymbols, first: Key) -> Value {
    let graph = &mut p.c.owner.owner.graph;
    let snapshot = graph.snapshot();
    let id = graph.id(key).unwrap();
    let n = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap();
    let mut chords = Vec::new();
    for ch in graph.tuplet_chords(key).unwrap() {
        chords.push((ch, graph.retained_or_live_chord_bounds(ch).unwrap()[0]));
    }
    chords.sort_by_key(|(_, x)| *x);
    let staff = graph.tuplet_staff(key, &p.c.owner.owner.staffs).unwrap();
    let voice = graph.tuplet_voice_identity(key).unwrap();
    let actual = graph
        .chord_voice_owner(first, &p.c.owner.voices)
        .unwrap()
        .map(|v| v.identity.clone());
    let same_voice = match (voice, actual) {
        (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
        (None, None) => true,
        _ => false,
    };
    let base = graph.tuplet_base_duration(key).unwrap();
    let factor = graph.tuplet_duration_factor(key).unwrap();
    assert_eq!(graph.tuplet_cache_observation(key)["part"], Value::Null);
    if let Some(staff) = &staff {
        assert!(
            p.c.owner
                .owner
                .staffs
                .iter()
                .any(|actual| Arc::ptr_eq(staff, actual))
        );
    }
    let shape = if n["shape"] == "TUPLET_THREE" {
        crate::music_font_symbols::TupletShape::Three
    } else {
        crate::music_font_symbols::TupletShape::Six
    };
    json!({"shape":n["shape"],"bounds":n["bounds"],"dimension":font.dimension(shape),"implicit":n["implicit"],"abnormal":n["abnormal"],"grade":n["grade"],"manual":n["manual"],"chords":chords.iter().map(|(k,_)|p.c.labels[&graph.id(*k).unwrap()].clone()).collect::<Vec<_>>(),"staff":staff.as_ref().map(|s|s.id),"same_voice":same_voice,"base_duration":base.as_ref().map(|b|crate::music_time::format_fraction(**b)),"factor":format!("{}/{}",factor.actual_num,factor.actual_den)})
}
#[test]
fn original_fresh_tuplet_callbacks_parallel_objects_and_remove_keep_real_owners_and_old_clocks() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let staffs = p.c.owner.owner.staffs.clone();
        let mut allocated = Vec::new();
        for s in case["scenarios"].as_array().unwrap() {
            let group = declare(&mut p, s);
            let font =
                TupletFontSymbols::load(s["family"].as_str().unwrap(), p.c.owner.owner.interline)
                    .unwrap();
            let before = frame(&mut p.c);
            assert_eq!(before, normalized(s["before"].clone()));
            p.c.owner.owner.graph.trace_tuplet_additions();
            match p.c.owner.generate_rhythm_tuplet(1, &group, &font) {
                Ok(key) => {
                    assert!(s["error"].is_null());
                    assert!(!allocated.contains(&key));
                    allocated.push(key);
                    assert!(p.c.owner.measures[0].tuplets.contains(&key));
                    assert!(p.c.owner.measures[0].stack_tuplets.contains(&key));
                    let callbacks = p.c.owner.owner.graph.take_tuplet_additions();
                    let got = json!(
                        callbacks
                            .iter()
                            .map(|cb| callback(&mut p, cb))
                            .collect::<Vec<_>>()
                    );
                    let mut expected = s["callbacks"].clone();
                    for cb in expected.as_array_mut().unwrap() {
                        cb["clocks"]["relations"] = json!([]);
                    }
                    assert_eq!(
                        got, expected,
                        "{} {} callbacks",
                        case["name"], s["spec"]["name"]
                    );
                    let graph = p.c.owner.owner.graph.snapshot();
                    let target = p.c.owner.owner.graph.id(key).unwrap();
                    let relations = graph["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|e| e["kind"] == "ChordTupletRelation" && e["target"] == target)
                        .collect::<Vec<_>>();
                    assert_eq!(relations.len(), group.len());
                    let ids = relations
                        .iter()
                        .map(|e| e["id"].as_u64().unwrap())
                        .collect::<std::collections::BTreeSet<_>>();
                    assert_eq!(ids.len(), group.len());
                    for edge in &relations {
                        assert_eq!(edge["manual"], false);
                        assert_eq!(edge["source_ratio"], 1.0);
                        let coeff = if s["result"]["shape"] == "TUPLET_THREE" {
                            2.0 * 0.33
                        } else {
                            2.0 * 0.17
                        };
                        assert_eq!(edge["target_ratio"], json!(1.0 + coeff));
                    }
                    assert_eq!(result(&mut p, key, &font, group[0]), s["result"]);
                    assert_eq!(frame(&mut p.c), normalized(s["generated"].clone()));
                    let factor = p.c.owner.owner.graph.tuplet_duration_factor(key).unwrap();
                    let base = p.c.owner.owner.graph.tuplet_base_duration(key).unwrap();
                    p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
                    assert!(!p.c.owner.measures[0].tuplets.contains(&key));
                    assert!(!p.c.owner.measures[0].stack_tuplets.contains(&key));
                    assert_eq!(
                        p.c.owner.owner.graph.tuplet_registration(key)["removed"],
                        true
                    );
                    assert_eq!(
                        p.c.owner.owner.graph.tuplet_cache_observation(key)["part"],
                        1
                    );
                    assert_eq!(
                        p.c.owner.owner.graph.tuplet_cache_observation(key)["base"],
                        json!(base.map(|b| crate::music_time::format_fraction(*b)))
                    );
                    assert_eq!(
                        p.c.owner.owner.graph.tuplet_cache_observation(key)["factor"],
                        json!([factor.actual_num, factor.actual_den])
                    );
                }
                Err(error) => {
                    assert_eq!(s["error"], "NullPointerException");
                    assert_eq!(
                        error,
                        if group.is_empty() {
                            "music_tuplet_actual_group_bounds_required"
                        } else {
                            "music_actual_tuplet_chord_owner_required"
                        }
                    );
                    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
                }
            }
            assert_eq!(frame(&mut p.c), normalized(s["removed"].clone()));
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
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
fn late_tuplet_addition_failure_restores_actual_containers_caches_clock_owners_and_manual_winner() {
    let d = original();
    let case = &d["cases"][0];
    let reuse: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-reuse/observed.json"
    ))
    .unwrap();
    let mut declaration = case.clone();
    declaration["declared_tuplets"] = json!([
        reuse["cases"][0]["declared_tuplets"][0],
        reuse["cases"][0]["declared_tuplets"][2]
    ]);
    let mut p = control(&declaration);
    let s = &case["scenarios"][5];
    let group = declare(&mut p, s);
    let font = TupletFontSymbols::load("Bravura", p.c.owner.owner.interline).unwrap();
    let before = frame(&mut p.c);
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let tuplets = p.c.owner.measures[0].tuplets.as_ptr();
    let stack = p.c.owner.measures[0].stack_tuplets.as_ptr();
    p.c.owner.owner.graph.trace_tuplet_additions();
    p.c.owner.owner.graph.fail_tuplet_addition_after(Some(2));
    let graph = p.c.owner.owner.graph.snapshot();
    let voices = projection(&p.c.owner.voices, &p.c.owner.measures);
    assert_eq!(
        p.c.owner
            .generate_rhythm_tuplet(1, &group, &font)
            .unwrap_err(),
        "music_tuplet_addition_observer_failure"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
    assert_eq!(frame(&mut p.c), before);
    assert_eq!(projection(&p.c.owner.voices, &p.c.owner.measures), voices);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplets);
    assert_eq!(p.c.owner.measures[0].stack_tuplets.as_ptr(), stack);
    p.c.owner.owner.graph.fail_tuplet_addition_after(None);
    let key = p.c.owner.generate_rhythm_tuplet(1, &group, &font).unwrap();
    assert_eq!(result(&mut p, key, &font, group[0]), s["result"]);
    p.c.owner.remove_implicit_rhythm_tuplets(1).unwrap();
    assert_eq!(frame(&mut p.c), before);
    let mut p = control(&reuse["cases"][0]);
    let first = chord_key(&p.c, "chord1");
    let null = Key {
        system: 1,
        phase: crate::shared_sig::Phase::Chord,
        local: 999999,
    };
    let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
    let graph = p.c.owner.owner.graph.snapshot();
    assert_eq!(
        p.c.owner
            .generate_rhythm_tuplet(1, &[first, null], &wrong)
            .unwrap(),
        p.tuplets[0]
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
}
