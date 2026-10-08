// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{Control, frame, prepare_control};
use super::*;
use crate::shared_sig::{Key, Phase};
use crate::sheet_skew::SheetSkew;

fn original() -> Value {
    let mut observed: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-beam-rest-nearest/observed.json"
    ))
    .unwrap();
    let relations = observed["relation_sets"].as_array().unwrap().clone();
    for case in observed["cases"].as_array_mut().unwrap() {
        for phase in ["state_before", "state_after"] {
            let index = case[phase]["owned"]["relations"].as_u64().unwrap() as usize;
            case[phase]["owned"]["relations"] = relations[index].clone();
        }
    }
    observed
}
fn key(c: &Control, id: usize) -> Key {
    serde_json::from_value(
        c.owner.owner.graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap()["origin"]
            .clone(),
    )
    .unwrap()
}
fn groups(c: &mut Control, case: &Value) -> BTreeMap<String, Key> {
    let source = c.owner.actual_measure_beam_groups(1).unwrap()[0];
    let mut groups = BTreeMap::from([("source_group".into(), key(c, source))]);
    let snapshot = c.owner.owner.graph.snapshot();
    let next = |phase| {
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| serde_json::from_value::<Key>(n["origin"].clone()).unwrap())
            .filter(|k| k.phase == phase && k.system == 1)
            .map(|k| k.local)
            .max()
            .unwrap_or(0)
            + 1
    };
    let mut beam_local = next(Phase::Beam);
    let mut group_local = next(Phase::Group);
    for row in case["groups"].as_array().unwrap() {
        if row["actual_source"] == true {
            let beam = c.owner.owner.graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == c.beam)
                .unwrap()
                .clone();
            let raw: [f64; 4] = serde_json::from_value(beam["median"].clone()).unwrap();
            assert_eq!(
                json!(raw.map(|n| (n * 10.).round_ties_even() / 10.)),
                row["beams"][0]["median"]
            );
            continue;
        }
        if row["original_fresh_geometry"] == true {
            let beam = c.owner.owner.graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == c.beam)
                .unwrap()
                .clone();
            assert_eq!(beam["median"], row["beams"][0]["median"]);
            groups.insert("fresh_source".into(), groups["source_group"]);
            continue;
        }
        let group = Key {
            system: 1,
            phase: Phase::Group,
            local: group_local,
        };
        group_local += 1;
        c.owner
            .owner
            .graph
            .vertex(group, json!({"kind":"BeamGroupInter"}), false)
            .unwrap();
        groups.insert(row["label"].as_str().unwrap().into(), group);
        for member in row["beams"].as_array().unwrap() {
            let beam = Key {
                system: 1,
                phase: Phase::Beam,
                local: beam_local,
            };
            beam_local += 1;
            let id = c
                .owner
                .owner
                .graph
                .vertex(
                    beam,
                    json!({"kind":member["kind"],"median":member["median"],"height":4.}),
                    false,
                )
                .unwrap();
            c.labels
                .insert(id, member["label"].as_str().unwrap().into());
            c.owner
                .owner
                .graph
                .relations(
                    1,
                    Phase::Group,
                    vec![(group, beam, json!({"kind":"Containment"}))],
                )
                .unwrap();
        }
    }
    groups
}
fn owned_equal(c: &mut Control, expected: &Value) {
    let actual = frame(c);
    let mut original = expected.clone();
    // Original's pre-existing relation objects are observed unchanged rather
    // than assigning their serialized labels as native ownership identities.
    original["owned"]["relations"] = json!([]);
    assert_eq!(actual, original);
}

#[test]
fn original_nearest_rest_beam_keeps_real_skew_full_only_endpoints_ties_and_clock_objects() {
    for case in original()["cases"].as_array().unwrap() {
        let mut c = prepare_control(case);
        owned_equal(&mut c, &case["state_before"]);
        assert_eq!(case["state_before"], case["state_after"]);
        let groups = groups(&mut c, case);
        // The saved book quantizes medians via JAXB. Compare that input view
        // in a graph copy; the actual image owner keeps its measured precision
        // and is compared to independently measured original fresh geometry.
        let mut stored = c.owner.owner.graph.clone();
        let source_key = key(&c, c.beam);
        let mut beam = c.owner.owner.graph.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == c.beam)
            .unwrap()
            .clone();
        beam["median"] = case["groups"][0]["beams"][0]["median"].clone();
        stored.vertex(source_key, beam, false).unwrap();
        let graph = c.owner.owner.graph.snapshot();
        let chord_storage = c.owner.chords.as_ptr();
        let slot_storage = c.owner.measures[0].timing.slots.as_ptr();
        let report = c.owner.owner.prepared.report.clone();
        let upstream = c.owner.owner.upstream_report.clone();
        let original_skew = c.owner.owner.sheet_skew;
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
        for query in case["queries"].as_array().unwrap() {
            let skew = SheetSkew::new(query["slope"].as_f64().unwrap()).unwrap();
            let point: [i32; 2] = serde_json::from_value(query["point"].clone()).unwrap();
            let group = groups[query["group"].as_str().unwrap()];
            let result = if query["group"] == "fresh_source" {
                c.owner.owner.sheet_skew = skew;
                c.owner
                    .nearest_interleaved_rest_beam(1, group, point)
                    .unwrap()
            } else if query["group"] == "source_group" {
                stored.nearest_rest_beam(group, point, skew).unwrap()
            } else {
                c.owner
                    .owner
                    .graph
                    .nearest_rest_beam(group, point, skew)
                    .unwrap()
            };
            let label =
                result.map(|(key, _)| c.labels[&c.owner.owner.graph.id(key).unwrap()].clone());
            let expected_label = if query["beam"] == "fresh_source_beam" {
                json!("source_beam")
            } else {
                query["beam"].clone()
            };
            assert_eq!(json!(label), expected_label, "{} {query}", case["name"]);
            if let Some((_, distance)) = result {
                assert!(
                    (distance - query["distance"].as_f64().unwrap()).abs() < 1e-10,
                    "{query} distance {distance}"
                );
            } else {
                assert!(query["distance"].is_null());
            }
        }
        c.owner.owner.sheet_skew = original_skew;
        assert_eq!(c.owner.owner.graph.snapshot(), graph);
        assert_eq!(c.owner.chords.as_ptr(), chord_storage);
        assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slot_storage);
        assert_eq!(c.owner.owner.prepared.report, report);
        assert_eq!(c.owner.owner.upstream_report, upstream);
        for (key, old) in slots {
            match (old, c.owner.owner.graph.chord_slot_identity(key)) {
                (Some(old), Some(now)) => assert!(Arc::ptr_eq(&old, now)),
                (None, None) => {}
                _ => panic!("nearest query changed actual Chord Slot"),
            }
        }
        owned_equal(&mut c, &case["state_after"]);
    }
}

#[test]
fn nearest_rest_beam_uses_retained_skew_and_rolls_back_later_invalid_member_without_clock_reset() {
    let observed = original();
    let mut c = prepare_control(&observed["cases"][0]);
    let group_id = c.owner.actual_measure_beam_groups(1).unwrap()[0];
    let group = key(&c, group_id);
    assert_eq!(c.owner.owner.sheet_skew.slope(), 0.);
    c.owner.owner.upstream_report["line_completion"]["global_slope"] = json!(999.);
    let result = c
        .owner
        .nearest_interleaved_rest_beam(1, group, [359, 250])
        .unwrap()
        .unwrap();
    assert_eq!(c.owner.owner.graph.id(result.0), Some(c.beam));
    let expected = observed["cases"][0]["queries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|q| {
            q["group"] == "fresh_source" && q["slope"] == 0. && q["point"] == json!([359, 250])
        })
        .unwrap()["distance"]
        .as_f64()
        .unwrap();
    assert!((result.1 - expected).abs() < 1e-10);
    let graph = c.owner.owner.graph.snapshot();
    let local = graph["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| serde_json::from_value::<Key>(n["origin"].clone()).unwrap())
        .filter(|k| k.system == 1 && k.phase == Phase::Beam)
        .map(|k| k.local)
        .max()
        .unwrap()
        + 1;
    let invalid = Key {
        system: 1,
        phase: Phase::Beam,
        local,
    };
    c.owner
        .owner
        .graph
        .vertex(invalid, json!({"kind":"BeamInter","median":null}), false)
        .unwrap();
    c.owner
        .owner
        .graph
        .relations(
            1,
            Phase::Group,
            vec![(group, invalid, json!({"kind":"Containment"}))],
        )
        .unwrap();
    let graph = c.owner.owner.graph.snapshot();
    let clocks = projection(&c.owner.voices, &c.owner.measures);
    let frame_before = frame(&mut c);
    let storage = (
        c.owner.chords.as_ptr(),
        c.owner.measures[0].timing.slots.as_ptr(),
    );
    let report = c.owner.owner.prepared.report.clone();
    let cache = c.owner.owner.graph.rest_chord_beams(c.rests[0]).unwrap();
    assert_eq!(
        c.owner
            .nearest_interleaved_rest_beam(1, group, [359, 250])
            .unwrap_err(),
        "music_rest_actual_beam_median_required"
    );
    assert_eq!(c.owner.owner.graph.snapshot(), graph);
    assert_eq!(projection(&c.owner.voices, &c.owner.measures), clocks);
    assert_eq!(frame(&mut c), frame_before);
    assert_eq!(
        (
            c.owner.chords.as_ptr(),
            c.owner.measures[0].timing.slots.as_ptr()
        ),
        storage
    );
    assert_eq!(c.owner.owner.prepared.report, report);
    assert!(Arc::ptr_eq(
        &cache,
        &c.owner.owner.graph.rest_chord_beams(c.rests[0]).unwrap()
    ));
    c.owner
        .owner
        .graph
        .vertex(
            invalid,
            json!({"kind":"BeamInter","median":[300.,100.,450.,100.]}),
            false,
        )
        .unwrap();
    let result = c
        .owner
        .nearest_interleaved_rest_beam(1, group, [359, 250])
        .unwrap()
        .unwrap();
    assert_eq!(c.owner.owner.graph.id(result.0), Some(c.beam));
    assert_eq!(projection(&c.owner.voices, &c.owner.measures), clocks);
    assert!(SheetSkew::new(f64::NAN).is_err());
}
