// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame};
use super::beam_rest_competition_tests::{observed_callback, registrations};
use super::beam_rest_linked_tests::{chord_key, declare_links};
use super::beam_rest_polygon_tests::{declare, prepare};
use super::*;
use crate::shared_sig::Key;

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-detect/observed.json"
    ))
    .unwrap();
    let clocks = d["clock_sets"].as_array().unwrap().clone();
    let frames = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for k in ["before", "after"] {
            c[k] = frames[c[k].as_u64().unwrap() as usize].clone();
        }
        for cb in c["callbacks"].as_array_mut().unwrap() {
            cb["clocks"] = clocks[cb["clocks"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
fn control(case: &Value) -> (Control, Vec<(usize, String)>) {
    let (mut c, _) = prepare(case);
    declare(&mut c, case, &case["geometry"]);
    declare_links(&mut c, case);
    let mut tracked = Vec::new();
    if let Some(index) = case["preexisting_rest"].as_u64() {
        let chord = c
            .owner
            .chords
            .iter()
            .find(|ch| ch.key == c.rests[index as usize])
            .unwrap();
        let note: Key = serde_json::from_value(chord.members[0]["origin"].clone()).unwrap();
        let snap = c.owner.owner.graph.snapshot();
        let beam: Key = serde_json::from_value(
            snap["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| {
                    c.labels
                        .get(&(n["id"].as_u64().unwrap() as usize))
                        .is_some_and(|l| l == "competition_beam0")
                })
                .unwrap()["origin"]
                .clone(),
        )
        .unwrap();
        let id = c
            .owner
            .owner
            .graph
            .add_owned_beam_rest(
                beam,
                note,
                true,
                c.owner.owner.sheet_skew,
                c.owner.owner.interline,
            )
            .unwrap();
        tracked.push((id, "old0".into()));
        c.owner
            .owner
            .graph
            .invalidate_owned_rest_cache(chord.key)
            .unwrap();
        c.baseline[index as usize] = c.owner.owner.graph.rest_chord_beams(chord.key).unwrap();
    }
    (c, tracked)
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
fn candidates(c: &mut Control) -> Value {
    let rows = c.owner.interleaved_rest_candidates(1).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    let names = |k: &str| {
        rows[0][k]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| c.labels[&(id.as_u64().unwrap() as usize)].clone())
            .collect::<Vec<_>>()
    };
    json!({"seeds":names("seed_chord_ids"),"black":names("black_rest_ids"),"white":names("white_rest_ids"),"plain":names("plain_rest_ids")})
}
#[test]
fn original_white_then_plain_detect_holds_captured_chords_clocks_and_actual_owners() {
    for case in original()["cases"].as_array().unwrap() {
        let (mut c, mut tracked) = control(case);
        assert_eq!(
            candidates(&mut c),
            case["candidates"],
            "{} candidates",
            case["name"]
        );
        assert_eq!(frame(&mut c), normalized(case["before"].clone()));
        assert_eq!(registrations(&c, &tracked), case["before_relations"]);
        let chords = c.owner.chords.as_ptr();
        let slots_storage = c.owner.measures[0].timing.slots.as_ptr();
        let staffs = c.owner.owner.staffs.clone();
        let slots = c
            .owner
            .chords
            .iter()
            .map(|ch| {
                (
                    ch.key,
                    c.owner.owner.graph.chord_slot_identity(ch.key).cloned(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let before = c.owner.owner.graph.snapshot()["relations"].clone();
        let report = c.owner.owner.prepared.report.clone();
        c.owner.owner.graph.trace_beam_rest_callbacks();
        let added = c
            .owner
            .detect_interleaved_rests(1)
            .unwrap_or_else(|e| panic!("{}: {e}", case["name"]));
        tracked.extend(
            added
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, format!("added{i}"))),
        );
        assert_eq!(
            registrations(&c, &tracked),
            case["after_relations"],
            "{} relations",
            case["name"]
        );
        let trace = c.owner.owner.graph.take_beam_rest_callbacks();
        let mut observed = Vec::new();
        for cb in &trace {
            let row = observed_callback(&c, cb, &tracked, &slots);
            assert_eq!(row["clocks"], case["callbacks"][0]["clocks"]);
            if cb.phase == "entered" {
                assert_eq!(row["registered"], true);
                assert_eq!(row["manual"], false);
                assert_eq!(row["dy"], 0);
                assert_eq!(row["portion"], Value::Null);
            } else if cb.phase != "remove_entered" {
                observed.push(row);
            }
        }
        assert_eq!(
            json!(observed),
            case["callbacks"],
            "{} callbacks",
            case["name"]
        );
        assert_eq!(frame(&mut c), normalized(case["after"].clone()));
        assert_eq!(c.owner.chords.as_ptr(), chords);
        assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slots_storage);
        assert_eq!(c.owner.owner.prepared.report, report);
        for (old, now) in staffs.iter().zip(&c.owner.owner.staffs) {
            assert!(Arc::ptr_eq(old, now));
            for (a, b) in old.lines.iter().zip(&now.lines) {
                assert!(Arc::ptr_eq(a, b));
            }
        }
        for (key, old) in c.rests.iter().zip(&c.baseline) {
            assert!(Arc::ptr_eq(
                &c.owner.owner.graph.rest_chord_beams(*key).unwrap(),
                old
            ));
        }
        for e in before.as_array().unwrap() {
            assert!(
                c.owner.owner.graph.snapshot()["relations"]
                    .as_array()
                    .unwrap()
                    .contains(e)
            );
        }
    }
}
#[test]
fn detect_late_plain_member_failure_rolls_back_prior_white_callback_and_retries_same_chords() {
    let d = original();
    let case = &d["cases"][0];
    let (mut c, _) = control(case);
    let key = chord_key(&c, "rest_chord1");
    let index = c.owner.chords.iter().position(|ch| ch.key == key).unwrap();
    let member = c.owner.chords[index].members[0].clone();
    c.owner.chords[index].members[0]["origin"]["local"] = json!(999999);
    let snapshot = c.owner.owner.graph.snapshot();
    let before = frame(&mut c);
    let chords = c.owner.chords.as_ptr();
    let layout = c.owner.layout();
    c.owner.owner.graph.trace_beam_rest_callbacks();
    assert_eq!(
        c.owner.detect_interleaved_rests(1).unwrap_err(),
        "music_rest_actual_note_required"
    );
    assert_eq!(c.owner.owner.graph.snapshot(), snapshot);
    assert!(c.owner.owner.graph.take_beam_rest_callbacks().is_empty());
    assert_eq!(frame(&mut c), before);
    assert_eq!(c.owner.layout(), layout);
    assert_eq!(c.owner.chords.as_ptr(), chords);
    c.owner.chords[index].members[0] = member;
    let added = c.owner.detect_interleaved_rests(1).unwrap();
    let tracked = added
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, format!("added{i}")))
        .collect::<Vec<_>>();
    assert_eq!(registrations(&c, &tracked), case["after_relations"]);
    assert_eq!(frame(&mut c), normalized(case["after"].clone()));
    assert_eq!(c.owner.chords.as_ptr(), chords);
}
