// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control, state};
use super::*;
use crate::{
    music_font_symbols::TupletFontSymbols,
    music_xml::TupletPlan,
    shared_sig::{Key, Phase},
};

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-placement/observed.json"
    ))
    .unwrap();
    let relations = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] =
            relations[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    for f in d["clock_sets"].as_array_mut().unwrap() {
        f["relations"] = relations[f["relations"].as_u64().unwrap() as usize].clone();
    }
    let frames = d["frame_sets"].as_array().unwrap().clone();
    let clocks = d["clock_sets"].as_array().unwrap().clone();
    let sets = ["spec", "geometry", "callbacks", "result"]
        .map(|k| (k, d[format!("{k}_sets")].as_array().unwrap().clone()));
    for c in d["cases"].as_array_mut().unwrap() {
        for s in c["scenarios"].as_array_mut().unwrap() {
            for (k, rows) in &sets {
                s[*k] = rows[s[*k].as_u64().unwrap() as usize].clone();
            }
            for f in ["before", "generated", "removed"] {
                s[f] = frames[s[f].as_u64().unwrap() as usize].clone();
            }
            for cb in s["callbacks"].as_array_mut().unwrap() {
                cb["clocks"] = clocks[cb["clocks"].as_u64().unwrap() as usize].clone();
            }
        }
    }
    d
}
fn declare(p: &mut Prepared, scenario: &Value) -> Vec<Key> {
    for row in scenario["geometry"].as_array().unwrap() {
        let key = chord_key(&p.c, row["chord"].as_str().unwrap());
        p.c.owner
            .owner
            .graph
            .declare_chord_geometry(
                key,
                serde_json::from_value(row["bounds"].clone()).unwrap(),
                serde_json::from_value(row["head"].clone()).unwrap(),
                serde_json::from_value(row["tail"].clone()).unwrap(),
            )
            .unwrap();
    }
    scenario["spec"]["indices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            i.as_u64()
                .map(|i| chord_key(&p.c, &format!("chord{i}")))
                .unwrap_or(Key {
                    system: 1,
                    phase: Phase::Chord,
                    local: 999999,
                })
        })
        .collect()
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_tuplet_shape_font_bounds_direction_votes_preserve_real_chord_and_clock_owners() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        assert_eq!(
            p.c.owner.owner.interline,
            case["interline"].as_u64().unwrap() as usize
        );
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let staffs = p.c.owner.owner.staffs.clone();
        let report = p.c.owner.owner.prepared.report.clone();
        let voices = projection(&p.c.owner.voices, &p.c.owner.measures);
        for s in case["scenarios"].as_array().unwrap() {
            let group = declare(&mut p, s);
            let font =
                TupletFontSymbols::load(s["family"].as_str().unwrap(), p.c.owner.owner.interline)
                    .unwrap();
            let before = state(&mut p);
            assert_eq!(
                before["state"],
                normalized(s["before"].clone()),
                "{} {} before",
                case["name"],
                s["spec"]["name"]
            );
            let graph = p.c.owner.owner.graph.snapshot();
            match p.c.owner.plan_rhythm_tuplet(1, &group, &font) {
                Ok(TupletPlan::New { shape, bounds }) => {
                    assert!(s["error"].is_null());
                    assert_eq!(shape.as_str(), s["result"]["shape"].as_str().unwrap());
                    assert_eq!(json!(bounds), s["result"]["bounds"]);
                    assert_eq!(json!(font.dimension(shape)), s["result"]["dimension"]);
                }
                Ok(TupletPlan::Existing(_)) => panic!("unexpected existing object"),
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
                }
            }
            assert_eq!(state(&mut p), before);
            assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
            assert_eq!(before["state"], normalized(s["removed"].clone()));
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(projection(&p.c.owner.voices, &p.c.owner.measures), voices);
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
fn late_tuplet_bounds_failure_restores_staged_getters_and_existing_winner_precedes_font_scale() {
    let d = original();
    let case = &d["cases"][0];
    let mut p = control(case);
    let s = &case["scenarios"][1];
    let group = declare(&mut p, s);
    let font = TupletFontSymbols::load("Bravura", p.c.owner.owner.interline).unwrap();
    let before = state(&mut p);
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let stem =
        p.c.owner
            .chords
            .iter()
            .find(|ch| ch.key == group[1])
            .unwrap()
            .stem
            .unwrap();
    let snapshot = p.c.owner.owner.graph.snapshot();
    let mut node = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["origin"] == json!(stem))
        .unwrap()
        .clone();
    let saved = node.clone();
    for &key in &group {
        p.c.owner
            .owner
            .graph
            .invalidate_owned_head_chord_cache(key)
            .unwrap();
    }
    node["bounds"][2] = json!(0);
    p.c.owner.owner.graph.vertex(stem, node, false).unwrap();
    let graph = p.c.owner.owner.graph.snapshot();
    assert_eq!(
        p.c.owner.plan_rhythm_tuplet(1, &group, &font).unwrap_err(),
        "invalid_head_chord_member_bounds"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    p.c.owner.owner.graph.vertex(stem, saved, false).unwrap();
    let group = declare(&mut p, s);
    assert_eq!(
        p.c.owner.plan_rhythm_tuplet(1, &group, &font).unwrap(),
        TupletPlan::New {
            shape: crate::music_font_symbols::TupletShape::Three,
            bounds: serde_json::from_value(s["result"]["bounds"].clone()).unwrap()
        }
    );
    assert_eq!(state(&mut p), before);
    let reuse: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-reuse/observed.json"
    ))
    .unwrap();
    let mut p = control(&reuse["cases"][0]);
    let seed = chord_key(&p.c, "chord1");
    let null = Key {
        system: 1,
        phase: Phase::Chord,
        local: 999999,
    };
    let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
    let graph = p.c.owner.owner.graph.snapshot();
    assert_eq!(
        p.c.owner
            .plan_rhythm_tuplet(1, &[seed, null], &wrong)
            .unwrap(),
        TupletPlan::Existing(p.tuplets[0])
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    let empty_case = &d["cases"][0];
    let mut p = control(empty_case);
    let group = declare(&mut p, &empty_case["scenarios"][0]);
    let graph = p.c.owner.owner.graph.snapshot();
    assert_eq!(
        p.c.owner.plan_rhythm_tuplet(1, &group, &wrong).unwrap_err(),
        "music_tuplet_actual_sheet_scale_required"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert!(TupletFontSymbols::load("Bravura", 0).is_err());
    assert!(TupletFontSymbols::load("Bravura", 129).is_err());
    assert!(TupletFontSymbols::load("unknown", 11).is_err());
}
