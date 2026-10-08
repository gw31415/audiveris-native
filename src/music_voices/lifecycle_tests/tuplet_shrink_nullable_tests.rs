// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::control;
use super::tuplet_shrink_existing_tests::clocks;
use super::*;
use crate::music_font_symbols::TupletFontSymbols;
fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-shrink-nullable/observed.json"
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
fn original_nullable_shrink_follows_actual_stop_voice_or_returns_and_preserves_null_empty_retired_ranges()
 {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let old = p.c.refs[0].identity.clone();
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let expected = p.c.owner.measures[0].timing.expected;
        let report = p.c.owner.owner.prepared.report.clone();
        let mut candidates = p.c.owner.bind_rhythm_candidates(1).unwrap();
        let name = case["name"].as_str().unwrap();
        let stop = case["stop"].as_str().map(|s| chord_key(&p.c, s));
        if name == "null_voice_unowned_stop" {
            let key = stop.unwrap();
            let snapshot = p.c.owner.owner.graph.snapshot();
            let mut n = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == p.c.owner.owner.graph.id(key).unwrap())
                .unwrap()
                .clone();
            n["voice"] = Value::Null;
            p.c.owner.owner.graph.vertex(key, n, false).unwrap();
        }
        if name == "live_reset_null_chords" {
            p.c.owner.reset_rhythm_voice_chords(&old).unwrap();
        }
        if name == "live_removed_empty_chords" || name == "null_voice_retired_stop" {
            let keys = p.c.refs[0]
                .chord_ids
                .iter()
                .map(|id| p.c.owner.chords.iter().find(|ch| ch.id == *id).unwrap().key)
                .collect::<Vec<_>>();
            for key in keys {
                assert!(p.c.owner.remove_rhythm_voice_chord(&old, key).unwrap());
            }
        }
        if name == "null_voice_retired_stop" {
            p.c.owner.purge_rhythm_measure_voices(1).unwrap();
            assert!(p.c.owner.owner.graph.retained_voice_owner(&old).is_some());
        }
        assert_eq!(
            frame(&mut p.c),
            normalized(case["phases"][0]["state"].clone()),
            "{name} before"
        );
        assert_eq!(clocks(&candidates), case["phases"][0]["narrow_clocks"]);
        let manual_relations = p
            .relations
            .iter()
            .map(|(id, _)| p.c.owner.owner.graph.tuplet_relation_registration(*id))
            .collect::<Vec<_>>();
        let graph = p.c.owner.owner.graph.snapshot();
        let clocks_before = candidates.observe().unwrap();
        let identity = case["requested_voice"].as_str().map(|_| &old);
        let wrong = TupletFontSymbols::load("Leland", p.c.owner.owner.interline + 1).unwrap();
        p.c.owner.owner.graph.trace_tuplet_additions();
        let result = p.c.owner.shrink_rhythm_voice_from(
            &mut candidates,
            identity,
            stop,
            case["last_sync"].as_str(),
            &wrong,
        );
        if case["error"].is_null() {
            let keys = result.unwrap();
            assert!(keys.iter().all(|k| p.tuplets.contains(k)));
        } else {
            assert_eq!(case["error"], "IllegalArgumentException");
            assert_eq!(
                result.unwrap_err(),
                "music_tuplet_shrink_group_range_required"
            );
            assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
            assert_eq!(candidates.observe().unwrap(), clocks_before);
        }
        assert_eq!(
            p.relations
                .iter()
                .map(|(id, _)| p.c.owner.owner.graph.tuplet_relation_registration(*id))
                .collect::<Vec<_>>(),
            manual_relations
        );
        assert_eq!(case["graph_changes"], 0);
        assert!(p.c.owner.owner.graph.take_tuplet_additions().is_empty());
        assert_eq!(
            frame(&mut p.c),
            normalized(case["phases"][1]["state"].clone()),
            "{name} after"
        );
        assert_eq!(clocks(&candidates), case["phases"][1]["narrow_clocks"]);
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
        assert_eq!(p.c.owner.measures[0].timing.expected, expected);
        assert_eq!(p.c.owner.owner.prepared.report, report);
    }
}
