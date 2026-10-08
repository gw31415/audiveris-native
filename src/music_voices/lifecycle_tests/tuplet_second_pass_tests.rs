// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_merge_tests::{compare, name, normalized, observed_callbacks, prepare, state};
use super::tuplet_preparation_tests::Prepared;
use super::*;
use crate::{music_font_symbols::TupletFontSymbols, music_time::Fraction};
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-second-pass/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    for f in d["clock_sets"].as_array_mut().unwrap() {
        f["relations"] = rs[f["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    let cs = d["clock_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
        }
        for cb in c["callbacks"].as_array_mut().unwrap() {
            cb["clocks"] = cs[cb["clocks"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
fn control(case: &Value) -> Prepared {
    let mut p = prepare(case);
    p.c.owner.measures[0].timing.expected = case["expected"]
        .as_str()
        .map(Fraction::parse)
        .transpose()
        .unwrap();
    for label in case["dotted_chords"].as_array().unwrap() {
        let key = chord_key(&p.c, label.as_str().unwrap());
        let snapshot = p.c.owner.owner.graph.snapshot();
        let mut node = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == p.c.owner.owner.graph.id(key).unwrap())
            .unwrap()
            .clone();
        node["dots_number"] = json!(1);
        p.c.owner.owner.graph.vertex(key, node, false).unwrap();
    }
    p
}
fn reset(p: &mut Prepared) {
    let owner = &mut p.c.owner;
    crate::music_voices::reset_rhythm(
        &mut owner.owner.graph,
        &mut owner.voices,
        &mut owner.measures[0],
        &owner.chords,
        0,
    )
    .unwrap();
}
#[test]
fn original_second_pass_keeps_old_and_new_signs_at_find_callbacks_then_removes_captured_refs_before_actual_reset()
 {
    for case in original()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["error"].is_null() || c["error"] == "IllegalArgumentException")
    {
        let mut p = control(case);
        let before = state(&mut p, &[]);
        compare(
            before.clone(),
            normalized(case["phases"][0].clone()),
            "second-before",
        );
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let expected = p.c.owner.measures[0].timing.expected;
        let report = p.c.owner.owner.prepared.report.clone();
        let font =
            TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
                .unwrap();
        p.c.owner.owner.graph.trace_tuplet_additions();
        p.c.owner.owner.graph.trace_tuplet_removals();
        let result = p.c.owner.recheck_rhythm_implicit_tuplets(
            1,
            case["pass"].as_u64().unwrap() as usize,
            case["enabled"] == true,
            "1/4",
            &font,
        );
        let (new, callbacks) = observed_callbacks(&mut p);
        let mut source = case["callbacks"].clone();
        for cb in source.as_array_mut().unwrap() {
            cb["clocks"]["relations"] = json!([]);
        }
        compare(
            callbacks,
            source,
            &format!("{}-second-callbacks", case["name"].as_str().unwrap()),
        );
        match result {
            Ok(returned) => {
                assert!(case["error"].is_null());
                assert_eq!(
                    json!(
                        returned
                            .iter()
                            .map(|&k| name(&p, k, &new))
                            .collect::<Vec<_>>()
                    ),
                    case["returned"]
                );
                assert_eq!(!returned.is_empty(), case["repeat"] == true);
                compare(
                    state(&mut p, &new),
                    normalized(case["phases"][1].clone()),
                    &format!("{}-second-after", case["name"].as_str().unwrap()),
                );
                if !returned.is_empty() {
                    reset(&mut p);
                }
                compare(
                    state(&mut p, &new),
                    normalized(case["phases"][2].clone()),
                    &format!("{}-second-reset", case["name"].as_str().unwrap()),
                );
            }
            Err(error) => {
                assert_eq!(error, "music_tuplet_zero_expected_duration");
                assert_eq!(case["error"], "IllegalArgumentException");
                assert_eq!(state(&mut p, &[]), before);
            }
        }
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
        assert_eq!(p.c.owner.measures[0].timing.expected, expected);
        assert_eq!(p.c.owner.owner.prepared.report, report);
    }
}
#[test]
fn second_pass_registration_and_relation_failures_restore_old_signs_actual_storage_clocks_and_allow_retry()
 {
    let d = original();
    let success = &d["cases"][0];
    for case in d["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["error"] == "IllegalStateException")
    {
        assert_ne!(case["phases"][0], case["phases"][1]);
        assert_eq!(case["phases"][1]["tuplets"][0]["registered"], true);
        assert_eq!(case["phases"][1]["tuplets"][1]["registered"], true);
        assert_eq!(case["phases"][1]["tuplets"][3]["registered"], true);
        let mut p = control(case);
        let before = state(&mut p, &[]);
        let graph = p.c.owner.owner.graph.snapshot();
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let containers =
            p.c.owner
                .measures
                .iter()
                .map(|m| (m.tuplets.as_ptr(), m.stack_tuplets.as_ptr()))
                .collect::<Vec<_>>();
        let font =
            TupletFontSymbols::load(case["family"].as_str().unwrap(), p.c.owner.owner.interline)
                .unwrap();
        p.c.owner.owner.graph.trace_tuplet_additions();
        p.c.owner.owner.graph.trace_tuplet_removals();
        if case["name"] == "late_registration_failure" {
            p.c.owner
                .owner
                .graph
                .fail_tuplet_registration_after(Some(1));
        } else {
            p.c.owner.owner.graph.fail_tuplet_addition_after(Some(2));
        }
        assert_eq!(
            p.c.owner
                .recheck_rhythm_implicit_tuplets(1, 1, true, "1/4", &font)
                .unwrap_err(),
            "music_tuplet_addition_observer_failure"
        );
        assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
        assert_eq!(state(&mut p, &[]), before);
        assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
        assert!(p.c.owner.owner.graph.take_tuplet_removals().is_empty());
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
        for (m, (tuplets, stack)) in p.c.owner.measures.iter().zip(containers) {
            assert_eq!(m.tuplets.as_ptr(), tuplets);
            assert_eq!(m.stack_tuplets.as_ptr(), stack);
        }
        p.c.owner.owner.graph.fail_tuplet_registration_after(None);
        p.c.owner.owner.graph.fail_tuplet_addition_after(None);
        let returned =
            p.c.owner
                .recheck_rhythm_implicit_tuplets(1, 1, true, "1/4", &font)
                .unwrap();
        let (new, _) = observed_callbacks(&mut p);
        assert_eq!(
            json!(
                returned
                    .iter()
                    .map(|&k| name(&p, k, &new))
                    .collect::<Vec<_>>()
            ),
            success["returned"]
        );
        compare(
            state(&mut p, &new),
            normalized(success["phases"][1].clone()),
            "second-retry",
        );
        reset(&mut p);
        compare(
            state(&mut p, &new),
            normalized(success["phases"][2].clone()),
            "second-retry-reset",
        );
    }
}
