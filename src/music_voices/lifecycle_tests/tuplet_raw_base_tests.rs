// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::frame;
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_getters_tests::{link, node};
use super::tuplet_preparation_tests::control;
use super::*;
use crate::shared_sig::{Key, Phase};

fn inflate(text: &str, field: &str) -> Value {
    let mut d: Value = serde_json::from_str(text).unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    let rows = d["row_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for k in ["before", "after"] {
            c[k] = fs[c[k].as_u64().unwrap() as usize].clone();
        }
        c[field] = rows[c[field].as_u64().unwrap() as usize].clone();
    }
    d
}
fn original() -> Value {
    inflate(
        include_str!("../../../fixtures/official-tuplet-raw-base/observed.json"),
        "raw_rows",
    )
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_raw_shape_duration_ignores_dots_tuplets_and_measure_rest_wrapper_on_actual_chords() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        assert_eq!(frame(&mut p.c), normalized(case["before"].clone()));
        for row in case["raw_rows"].as_array().unwrap() {
            let key = chord_key(&p.c, row["chord"].as_str().unwrap());
            let ch = p.c.owner.chords.iter().find(|ch| ch.key == key).unwrap();
            let head: Key = serde_json::from_value(ch.members[0]["origin"].clone()).unwrap();
            let saved_head = node(&p, head);
            let saved_chord = node(&p, key);
            let mut note = saved_head.clone();
            note["shape"] = row["shape"].clone();
            let mut chord = saved_chord.clone();
            chord["dots_number"] = row["dots"].clone();
            p.c.owner.owner.graph.vertex(head, note, false).unwrap();
            p.c.owner.owner.graph.vertex(key, chord, false).unwrap();
            let before = p.c.owner.owner.graph.snapshot();
            match p.c.owner.raw_rhythm_chord_duration(key) {
                Ok(raw) => {
                    assert!(row["error"].is_null());
                    assert_eq!(json!(raw), row["raw"]);
                }
                Err(error) => {
                    assert_eq!(row["error"], "NullPointerException");
                    assert_eq!(error, "music_raw_null_duration_with_beams_flags");
                    assert_eq!(p.c.owner.owner.graph.snapshot(), before);
                }
            }
            p.c.owner
                .owner
                .graph
                .vertex(head, saved_head, false)
                .unwrap();
            p.c.owner
                .owner
                .graph
                .vertex(key, saved_chord, false)
                .unwrap();
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.owner.prepared.report, report);
        }
        assert_eq!(frame(&mut p.c), normalized(case["before"].clone()));
        for row in case["base_rows"].as_array().unwrap() {
            let ti = row["tuplet"]
                .as_str()
                .unwrap()
                .strip_prefix("tuplet")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let key = chord_key(&p.c, row["source"].as_str().unwrap());
            link(&mut p, ti, ti, ti == 1);
            let saved = node(&p, key);
            let mut declared = saved.clone();
            declared["dots_number"] = row["declared_first_dots"].clone();
            p.c.owner.owner.graph.vertex(key, declared, false).unwrap();
            let first =
                p.c.owner
                    .owner
                    .graph
                    .tuplet_base_duration(p.tuplets[ti])
                    .unwrap()
                    .unwrap();
            assert_eq!(
                crate::music_time::format_fraction(*first),
                row["first_base"].as_str().unwrap()
            );
            p.c.owner.owner.graph.vertex(key, saved, false).unwrap();
            let retry =
                p.c.owner
                    .owner
                    .graph
                    .tuplet_base_duration(p.tuplets[ti])
                    .unwrap()
                    .unwrap();
            assert!(Arc::ptr_eq(&first, &retry));
            assert_eq!(row["same_base"], true);
        }
        assert_eq!(frame(&mut p.c), normalized(case["after"].clone()));
        assert_eq!(p.c.owner.chords.as_ptr(), physical);
        assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    }
}
#[test]
fn original_raw_mirror_void_promotion_and_summed_live_flag_owners_keep_old_clocks() {
    let d = inflate(
        include_str!("../../../fixtures/official-chord-raw-owners/observed.json"),
        "rows",
    );
    for case in d["cases"].as_array().unwrap() {
        let mut p = control(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        let key = chord_key(&p.c, "chord1");
        let other = chord_key(&p.c, "chord0");
        let ch = p.c.owner.chords.iter().find(|c| c.key == key).unwrap();
        let stem = ch.stem.unwrap();
        let head: Key = serde_json::from_value(ch.members[0]["origin"].clone()).unwrap();
        let other_head: Key = serde_json::from_value(
            p.c.owner
                .chords
                .iter()
                .find(|c| c.key == other)
                .unwrap()
                .members[0]["origin"]
                .clone(),
        )
        .unwrap();
        let voice =
            p.c.owner
                .owner
                .graph
                .chord_voice_owner(key, &p.c.owner.voices)
                .unwrap()
                .unwrap()
                .identity
                .clone();
        let slot =
            p.c.owner
                .owner
                .graph
                .chord_slot_identity(key)
                .unwrap()
                .clone();
        let baseline = p.c.owner.owner.graph.clone();
        let start = baseline.snapshot()["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|n| serde_json::from_value::<Key>(n["origin"].clone()).ok())
            .filter(|k| k.system == 1 && k.phase == Phase::Symbol)
            .map(|k| k.local)
            .max()
            .unwrap()
            + 1;
        assert_eq!(frame(&mut p.c), normalized(case["before"].clone()));
        let mut mirror = None;
        let mut flags = Vec::new();
        for row in case["rows"].as_array().unwrap() {
            let mut note = node(&p, head);
            note["shape"] = row["shape"].clone();
            p.c.owner.owner.graph.vertex(head, note, false).unwrap();
            let mut chord = node(&p, key);
            chord["dots_number"] = row["declared_dots"].clone();
            p.c.owner.owner.graph.vertex(key, chord, false).unwrap();
            if row["mirror_registered"] == true && mirror.is_none() {
                p.c.owner
                    .owner
                    .graph
                    .relations(
                        1,
                        Phase::Symbol,
                        vec![(
                            head,
                            other_head,
                            json!({"kind":"MirrorRelation","manual":true}),
                        )],
                    )
                    .unwrap();
                let g = p.c.owner.owner.graph.snapshot();
                mirror = Some(
                    g["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|e| e["kind"] == "MirrorRelation")
                        .unwrap()["id"]
                        .as_u64()
                        .unwrap() as usize,
                );
            }
            let values = row["declared_flag_values"].as_array().unwrap();
            while flags.len() < values.len() {
                let value = values[flags.len()].as_u64().unwrap();
                let flag = Key {
                    system: 1,
                    phase: Phase::Symbol,
                    local: start + flags.len(),
                };
                p.c.owner.owner.graph.vertex(flag,json!({"kind":"FlagInter","shape":format!("FLAG_{value}"),"value":value,"grade":0.8,"bounds":[200,150,15,35],"staff_id":1,"manual":true}),false).unwrap();
                p.c.owner
                    .owner
                    .graph
                    .relations(
                        1,
                        Phase::Symbol,
                        vec![(flag, stem, json!({"kind":"FlagStemRelation","manual":true}))],
                    )
                    .unwrap();
                let g = p.c.owner.owner.graph.snapshot();
                let id = g["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| {
                        e["kind"] == "FlagStemRelation"
                            && e["source"] == p.c.owner.owner.graph.id(flag).unwrap()
                    })
                    .unwrap()["id"]
                    .as_u64()
                    .unwrap() as usize;
                flags.push((flag, id));
            }
            assert_eq!(
                json!(p.c.owner.raw_rhythm_chord_duration(key).unwrap()),
                row["raw"]
            );
            if let Some(id) = mirror {
                assert_eq!(
                    p.c.owner.owner.graph.edge_manual_status(id).unwrap(),
                    row["mirror_manual"].as_bool().unwrap()
                );
            }
            assert_eq!(
                json!(
                    flags
                        .iter()
                        .map(|(_, id)| p.c.owner.owner.graph.edge_manual_status(*id).unwrap())
                        .collect::<Vec<_>>()
                ),
                row["flag_relation_manual"]
            );
            assert_eq!(row["same_chord"], true);
            assert_eq!(row["same_voice"], true);
            assert_eq!(row["same_slot"], true);
            assert!(Arc::ptr_eq(
                &voice,
                &p.c.owner
                    .owner
                    .graph
                    .chord_voice_owner(key, &p.c.owner.voices)
                    .unwrap()
                    .unwrap()
                    .identity
            ));
            assert!(Arc::ptr_eq(
                &slot,
                p.c.owner.owner.graph.chord_slot_identity(key).unwrap()
            ));
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.owner.prepared.report, report);
        }
        // Restore only declared test dependencies from the held actual Graph.
        // This does not claim Flag/Mirror removal callback or task admission.
        p.c.owner.owner.graph = baseline;
        assert_eq!(frame(&mut p.c), normalized(case["after"].clone()));
    }
}

#[test]
fn original_partial_base_cache_and_part_before_staff_differ_from_outer_rollback() {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-base-failure/observed.json"
    ))
    .unwrap();
    let relations = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] =
            relations[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let frames = d["frame_sets"].as_array().unwrap().clone();
    for case in d["cases"].as_array().unwrap() {
        let mut declarations = case.clone();
        let spec = case["declared_tuplets"][0].clone();
        declarations["declared_tuplets"] = json!(
            (0..3)
                .map(|i| {
                    let mut v = spec.clone();
                    v["label"] = json!(format!("tuplet{i}"));
                    v
                })
                .collect::<Vec<_>>()
        );
        let mut p = control(&declarations);
        let before = frames[case["before"].as_u64().unwrap() as usize].clone();
        let after = frames[case["after"].as_u64().unwrap() as usize].clone();
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let report = p.c.owner.owner.prepared.report.clone();
        assert_eq!(frame(&mut p.c), normalized(before));
        for (i, row) in case["scenarios"].as_array().unwrap().iter().enumerate() {
            for (j, name) in row["incoming"].as_array().unwrap().iter().enumerate() {
                let index = name
                    .as_str()
                    .unwrap()
                    .strip_prefix("chord")
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                link(&mut p, index, i, j == 0);
            }
            let bad = chord_key(&p.c, row["bad_source"].as_str().unwrap());
            let head: Key = serde_json::from_value(
                p.c.owner
                    .chords
                    .iter()
                    .find(|ch| ch.key == bad)
                    .unwrap()
                    .members[0]["origin"]
                    .clone(),
            )
            .unwrap();
            let saved = node(&p, head);
            let mut note = saved.clone();
            note["shape"] = row["declared_shape"].clone();
            p.c.owner.owner.graph.vertex(head, note, false).unwrap();
            let key = p.tuplets[i];
            if row["error"] == "NullPointerException" {
                let graph = p.c.owner.owner.graph.snapshot();
                let cache = p.c.owner.owner.graph.tuplet_cache_observation(key);
                assert!(p.c.owner.read_rhythm_tuplet(key).is_err());
                assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
                assert_eq!(p.c.owner.owner.graph.tuplet_cache_observation(key), cache);
            }
            let result = p.c.owner.owner.graph.tuplet_base_duration(key);
            match result {
                Ok(raw) => {
                    assert!(row["error"].is_null());
                    assert_eq!(
                        json!(raw.map(|b| crate::music_time::format_fraction(*b))),
                        row["result"]
                    );
                }
                Err(e) => {
                    assert_eq!(row["error"], "NullPointerException");
                    assert_eq!(
                        e,
                        if i == 0 {
                            "music_raw_null_duration_with_beams_flags"
                        } else {
                            "music_tuplet_null_raw_duration_after_base"
                        }
                    );
                }
            }
            assert_eq!(
                p.c.owner.owner.graph.tuplet_cache_observation(key)["base"],
                row["cached_after_first"]
            );
            let cached =
                p.c.owner
                    .owner
                    .graph
                    .tuplet_base_duration(key)
                    .unwrap()
                    .unwrap();
            let retry =
                p.c.owner
                    .owner
                    .graph
                    .tuplet_base_duration(key)
                    .unwrap()
                    .unwrap();
            assert_eq!(
                json!(crate::music_time::format_fraction(*retry)),
                row["retry"]
            );
            assert!(Arc::ptr_eq(&cached, &retry));
            assert_eq!(row["same_cached_retry"], true);
            assert_eq!(
                json!(p.c.owner.owner.graph.tuplet_part(key).unwrap()),
                row["part_before_staff"]
            );
            let staff =
                p.c.owner
                    .owner
                    .graph
                    .tuplet_staff(key, &p.c.owner.owner.staffs)
                    .unwrap()
                    .unwrap();
            assert_eq!(json!(staff.id), row["staff"]);
            assert!(
                p.c.owner
                    .owner
                    .staffs
                    .iter()
                    .any(|actual| Arc::ptr_eq(actual, &staff))
            );
            assert_eq!(
                json!(p.c.owner.owner.graph.tuplet_part(key).unwrap()),
                row["part_after_staff"]
            );
            p.c.owner.set_rhythm_tuplet_staff(key, None).unwrap();
            assert_eq!(
                json!(p.c.owner.owner.graph.tuplet_part(key).unwrap()),
                row["part_after_clear_staff"]
            );
            assert_eq!(
                p.c.owner.owner.graph.tuplet_status(key).unwrap(),
                (true, row["manual"].as_bool().unwrap())
            );
            let links = &p.relations[p.relations.len() - 2..];
            assert_eq!(
                json!(
                    links
                        .iter()
                        .map(|(id, _)| p.c.owner.owner.graph.edge_manual_status(*id).unwrap())
                        .collect::<Vec<_>>()
                ),
                row["relation_manual"]
            );
            p.c.owner.owner.graph.vertex(head, saved, false).unwrap();
            p.c.owner.measures[0].tuplets.retain(|k| *k != key);
            let measure = p.c.owner.measures[0].tuplets.clone();
            p.c.owner
                .owner
                .graph
                .remove_owned_implicit_tuplet(key, &measure, &measure)
                .unwrap();
            assert_eq!(
                p.c.owner.owner.graph.tuplet_registration(key)["removed"],
                true
            );
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.owner.prepared.report, report);
        }
        assert_eq!(frame(&mut p.c), normalized(after));
    }
}
