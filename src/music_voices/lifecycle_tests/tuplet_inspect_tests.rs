// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::control;
use super::tuplet_shrink_existing_tests::{clocks, set_time};
use super::*;
use crate::{music_font_symbols::TupletFontSymbols, music_time::Fraction};
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-inspect-end/observed.json"
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
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_inspect_uses_actual_first_implicit_last_any_successor_and_signed_exact_ratio_before_shrink()
 {
    for case in original()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"] != "later_voice_null_start")
    {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let tuplets = p.c.owner.measures[0].tuplets.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let mut candidates = p.c.owner.bind_rhythm_candidates(1).unwrap();
        let compound_storage = candidates.compounds.as_ptr();
        let first = chord_key(&p.c, "chord3");
        let last = chord_key(&p.c, "chord9");
        set_time(&mut p, first, &case["declared_start"]);
        set_time(&mut p, last, &case["declared_final_time"]);
        p.c.owner.measures[0].timing.expected = case["declared_expected"]
            .as_str()
            .map(Fraction::parse)
            .transpose()
            .unwrap();
        let expected = p.c.owner.measures[0].timing.expected;
        assert_eq!(
            frame(&mut p.c),
            normalized(case["phases"][0]["state"].clone()),
            "{} before",
            case["name"]
        );
        assert_eq!(clocks(&candidates), case["phases"][0]["narrow_clocks"]);
        let before = p.c.owner.owner.graph.snapshot();
        let candidate_before = candidates.observe().unwrap();
        let manual = p
            .relations
            .iter()
            .map(|(id, _)| p.c.owner.owner.graph.tuplet_relation_registration(*id))
            .collect::<Vec<_>>();
        let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
        p.c.owner.owner.graph.trace_tuplet_additions();
        let result = p.c.owner.inspect_rhythm_voice_ends(&mut candidates, &wrong);
        if case["error"].is_null() {
            let keys = result.unwrap();
            assert!(keys.iter().all(|k| p.tuplets.contains(k)));
        } else {
            assert_eq!(case["error"], "NullPointerException");
            let expected = if case["declared_expected"].is_null() {
                "music_tuplet_null_shrink_chord_time"
            } else {
                "music_tuplet_null_inspect_chord_start"
            };
            assert_eq!(result.unwrap_err(), expected);
            assert_eq!(p.c.owner.owner.graph.snapshot(), before);
            assert_eq!(candidates.observe().unwrap(), candidate_before);
        }
        assert_eq!(case["graph_changes"], 0);
        assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
        assert_eq!(
            frame(&mut p.c),
            normalized(case["phases"][1]["state"].clone()),
            "{} after",
            case["name"]
        );
        assert_eq!(clocks(&candidates), case["phases"][1]["narrow_clocks"]);
        assert_eq!(
            p.relations
                .iter()
                .map(|(id, _)| p.c.owner.owner.graph.tuplet_relation_registration(*id))
                .collect::<Vec<_>>(),
            manual
        );
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
        assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplets);
        assert_eq!(p.c.owner.measures[0].timing.expected, expected);
        assert_eq!(candidates.compounds.as_ptr(), compound_storage);
        assert_eq!(p.c.owner.owner.prepared.report, report);
    }
}

#[test]
fn later_voice_null_inspection_restores_earlier_voice_actual_chord_and_narrow_clock_changes() {
    let d = original();
    let case = d["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "later_voice_null_start")
        .unwrap();
    assert_eq!(case["error"], "NullPointerException");
    assert_ne!(case["phases"][0]["state"], case["phases"][1]["state"]);
    assert_ne!(
        case["phases"][0]["narrow_clocks"],
        case["phases"][1]["narrow_clocks"]
    );
    let mut p = control(case);
    let mut candidates = p.c.owner.bind_rhythm_candidates(1).unwrap();
    let first = chord_key(&p.c, "chord3");
    let last = chord_key(&p.c, "chord9");
    let null = chord_key(&p.c, "chord6");
    set_time(&mut p, first, &case["declared_start"]);
    set_time(&mut p, last, &case["declared_final_time"]);
    set_time(&mut p, null, &Value::Null);
    p.c.owner.measures[0].timing.expected = None;
    let before = frame(&mut p.c);
    assert_eq!(before, normalized(case["phases"][0]["state"].clone()));
    let graph = p.c.owner.owner.graph.snapshot();
    let clocks_before = candidates.observe().unwrap();
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let tuplets = p.c.owner.measures[0].tuplets.as_ptr();
    let compounds = candidates.compounds.as_ptr();
    let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
    p.c.owner.owner.graph.trace_tuplet_additions();
    assert_eq!(
        p.c.owner
            .inspect_rhythm_voice_ends(&mut candidates, &wrong)
            .unwrap_err(),
        "music_tuplet_null_shrink_chord_time"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(frame(&mut p.c), before);
    assert_eq!(candidates.observe().unwrap(), clocks_before);
    assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplets);
    assert_eq!(candidates.compounds.as_ptr(), compounds);
    assert!(p.c.owner.measures[0].timing.expected.is_none());
}
