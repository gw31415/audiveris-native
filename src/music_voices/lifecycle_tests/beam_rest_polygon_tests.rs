// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame, prepare_control};
use super::beam_rest_linked_tests::chord_key;
use super::*;
use crate::shared_sig::{Key, Phase};
fn original() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-polygon/observed.json"
    ))
    .unwrap()
}
fn node(c: &Control, id: usize) -> Value {
    c.owner.owner.graph.snapshot()["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap()
        .clone()
}
pub(super) fn prepare(case: &Value) -> (Control, Key) {
    let mut c = prepare_control(case);
    let gid = c.owner.actual_measure_beam_groups(1).unwrap()[0];
    let group: Key = serde_json::from_value(node(&c, gid)["origin"].clone()).unwrap();
    let local = c.owner.owner.graph.snapshot()["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| serde_json::from_value::<Key>(n["origin"].clone()).ok())
        .filter(|k| k.system == 1 && k.phase == Phase::Beam)
        .map(|k| k.local)
        .max()
        .unwrap()
        + 1;
    for (i, b) in case["declared_beams"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let key = Key {
            system: 1,
            phase: Phase::Beam,
            local: local + i,
        };
        let id = c
            .owner
            .owner
            .graph
            .vertex(
                key,
                json!({"kind":b["kind"],"median":b["median"],"height":b["height"],"abnormal":true}),
                false,
            )
            .unwrap();
        c.labels.insert(id, b["label"].as_str().unwrap().into());
        c.owner
            .owner
            .graph
            .relations(
                1,
                Phase::Group,
                vec![(group, key, json!({"kind":"Containment"}))],
            )
            .unwrap();
    }
    (c, group)
}
pub(super) fn declare(c: &mut Control, case: &Value, geometry: &Value) {
    let points = geometry["input"].as_array().unwrap();
    for (label, bounds, h, t) in [
        (case["left"].as_str().unwrap(), "left_bounds", 0, 1),
        (case["right"].as_str().unwrap(), "right_bounds", 2, 3),
    ] {
        let key = chord_key(c, label);
        c.owner
            .owner
            .graph
            .declare_chord_geometry(
                key,
                serde_json::from_value(geometry[bounds].clone()).unwrap(),
                serde_json::from_value(points[h].clone()).unwrap(),
                serde_json::from_value(points[t].clone()).unwrap(),
            )
            .unwrap();
    }
}
#[test]
fn original_polygon_even_odd_rectangle_contact_and_degenerate_boundaries_match() {
    for row in original()["rectangle_controls"].as_array().unwrap() {
        let points: Vec<[i32; 2]> = serde_json::from_value(row["polygon"].clone()).unwrap();
        let bounds = serde_json::from_value(row["bounds"].clone()).unwrap();
        assert_eq!(
            audiveris_area::polygon_intersects_rect(&points, bounds).unwrap(),
            row["intersects"].as_bool().unwrap(),
            "{row}"
        );
    }
}
#[test]
fn original_private_lookup_acceptance_matches_real_cached_chords_and_retained_staff_middle_line() {
    for case in original()["cases"].as_array().unwrap() {
        let (mut c, group) = prepare(case);
        let left = chord_key(&c, case["left"].as_str().unwrap());
        let right = chord_key(&c, case["right"].as_str().unwrap());
        let mut expected = case["before"].clone();
        assert_eq!(expected, case["after"]);
        expected["owned"]["relations"] = json!([]);
        assert_eq!(frame(&mut c), expected);
        let storage = c.owner.chords.as_ptr();
        let slot_storage = c.owner.measures[0].timing.slots.as_ptr();
        let report = c.owner.owner.prepared.report.clone();
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
            .collect::<Vec<_>>();
        let staffs = c.owner.owner.staffs.clone();
        let baseline = frame(&mut c);
        let rest_keys = c.rests.clone();
        for geometry in case["geometry"].as_array().unwrap() {
            declare(&mut c, case, geometry);
            let result = c
                .owner
                .interleaved_rest_lookup_geometry(1, group, left, right, &rest_keys)
                .unwrap();
            assert_eq!(result["polygon"], geometry["polygon"]);
            assert_eq!(result["main_median"], geometry["main_median"]);
            assert_eq!(result["rest_detection_complete"], false);
            for (actual, expected) in result["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .zip(geometry["queries"].as_array().unwrap())
            {
                assert_eq!(
                    c.labels[&(actual["chord_id"].as_u64().unwrap() as usize)],
                    expected["rest"].as_str().unwrap()
                );
                assert_eq!(actual["intersects"], expected["intersects"]);
                assert_eq!(actual["x_overlap"], expected["x_overlap"]);
                assert_eq!(actual["eligible"], expected["accepted"]);
                if actual["eligible"] == true {
                    assert_eq!(actual["nearest_beam_id"], c.beam);
                }
            }
            assert_eq!(frame(&mut c), baseline);
            assert_eq!(c.owner.chords.as_ptr(), storage);
            assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
            assert_eq!(c.owner.owner.prepared.report, report);
            for (old, now) in staffs.iter().zip(&c.owner.owner.staffs) {
                assert!(Arc::ptr_eq(old, now));
                for (old, now) in old.lines.iter().zip(&now.lines) {
                    assert!(Arc::ptr_eq(old, now));
                }
            }
            for (key, old) in &slots {
                match (old, c.owner.owner.graph.chord_slot_identity(*key)) {
                    (Some(old), Some(now)) => assert!(Arc::ptr_eq(old, now)),
                    (None, None) => {}
                    _ => panic!("lookup changed actual Slot"),
                }
            }
        }
    }
}
#[test]
fn lookup_geometry_late_missing_actual_member_rolls_back_and_same_owner_retry_matches() {
    let d = original();
    let case = &d["cases"][0];
    let (mut c, group) = prepare(case);
    declare(&mut c, case, &case["geometry"][0]);
    let left = chord_key(&c, case["left"].as_str().unwrap());
    let right = chord_key(&c, case["right"].as_str().unwrap());
    let index = c
        .owner
        .chords
        .iter()
        .position(|ch| ch.key == c.rests[18])
        .unwrap();
    let saved = c.owner.chords[index].members.clone();
    c.owner.chords[index].members[0]["origin"]["local"] = json!(999999);
    let snapshot = c.owner.owner.graph.snapshot();
    let layout = c.owner.layout();
    let voices = projection(&c.owner.voices, &c.owner.measures);
    let storage = c.owner.chords.as_ptr();
    let before = frame(&mut c);
    let keys = c.rests.clone();
    assert_eq!(
        c.owner
            .interleaved_rest_lookup_geometry(1, group, left, right, &keys)
            .unwrap_err(),
        "music_rest_actual_note_required"
    );
    assert_eq!(c.owner.owner.graph.snapshot(), snapshot);
    assert_eq!(c.owner.layout(), layout);
    assert_eq!(projection(&c.owner.voices, &c.owner.measures), voices);
    assert_eq!(c.owner.chords.as_ptr(), storage);
    assert_eq!(frame(&mut c), before);
    c.owner.chords[index].members = saved;
    let result = c
        .owner
        .interleaved_rest_lookup_geometry(1, group, left, right, &keys)
        .unwrap();
    assert_eq!(result["polygon"], case["geometry"][0]["polygon"]);
    for (key, old) in c.rests.iter().zip(&c.baseline) {
        assert!(Arc::ptr_eq(
            &c.owner.owner.graph.rest_chord_beams(*key).unwrap(),
            old
        ));
    }
}
