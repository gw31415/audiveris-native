// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static OBSERVATIONS: RefCell<Option<Vec<Value>>> = const { RefCell::new(None) };
    static PURGE_FRAMES: RefCell<Option<Vec<Value>>> = const { RefCell::new(None) };
}

pub(super) fn record_purge(stage: &str, system: u64, graph: &Graph) {
    PURGE_FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        let Some(frames) = frames.as_mut() else {
            return;
        };
        let snapshot = graph.snapshot();
        let mut ornaments = Vec::new();
        for node in snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .chain(snapshot["tombstones"].as_array().unwrap())
        {
            let key: Key = serde_json::from_value(node["origin"].clone()).unwrap();
            if key.system != system {
                continue;
            }
            let (body, removed) = graph.retained_inter(key).unwrap();
            if body["kind"] == "OrnamentInter" {
                ornaments.push(json!({"body":body,"removed":removed}));
            }
        }
        frames.push(json!({"stage":stage,"system":system,"ornaments":ornaments}));
    });
}

fn close(actual: &Value, expected: &Value) {
    assert!(
        (actual.as_f64().unwrap() - expected.as_f64().unwrap()).abs() < 1e-10,
        "actual={actual} expected={expected}"
    );
}

pub(crate) fn assert_der_winter_owned_lifecycle(music: &OwnedMusic) -> Value {
    let factory: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ornaments/der-winter-factory-observed.json"
    ))
    .unwrap();
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ornaments/der-winter-lifecycle-observed.json"
    ))
    .unwrap();
    let confidence: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ornaments/der-winter-chord-confidence-observed.json"
    ))
    .unwrap();
    assert!(source["observer_errors"].as_array().unwrap().is_empty());
    let before = music.owner.graph.snapshot();
    let ornaments = before["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "OrnamentInter")
        .collect::<Vec<_>>();
    let input_rows =
        ornaments
            .iter()
            .map(|ornament| {
                let sources = before["relations"].as_array().unwrap().iter()
            .filter(|r| r["kind"] == "ChordOrnamentRelation" && r["target"] == ornament["id"])
            .map(|relation| {
                let chord = music.chords.iter().find(|c| relation["source"] == c.id).unwrap();
                let body = music.owner.graph.retained_inter(chord.key).unwrap().0;
                let members = before["relations"].as_array().unwrap().iter()
                    .filter(|r| r["kind"] == "Containment" && r["source"] == chord.id)
                    .map(|r| before["inters"].as_array().unwrap().iter()
                        .find(|n| n["id"] == r["target"]).unwrap()).collect::<Vec<_>>();
                json!({"relation":relation,"chord_key":chord.key,"chord_bounds":chord.bounds,
                    "cached_grade":body["grade"],"member_heads":members})
            }).collect::<Vec<_>>();
                json!({"ornament":ornament,"sources":sources})
            })
            .collect::<Vec<_>>();
    std::fs::write("/tmp/real-rest-ornament-native-inputs.json", serde_json::to_vec_pretty(&json!({
        "scope":"Measured actual DerWinter SYMBOLS Ornament and Chord inputs before comparison assertions; no source-grade substitution in recognition.",
        "rows":input_rows
    })).unwrap()).unwrap();
    assert_eq!(ornaments.len(), 3);
    let trials = music.owner.prepared.report["classifier_trials"]
        .as_array()
        .unwrap();
    let mut factory_results = Vec::new();
    for expected in factory["factory_candidates"].as_array().unwrap() {
        let trial = trials
            .iter()
            .find(|trial| {
                trial["system_id"] == expected["system"]
                    && trial["bounds"] == expected["bounds"]
                    && trial["evaluations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|evaluation| {
                            evaluation["shape"] == expected["shape"]
                                && (evaluation["score"].as_f64().unwrap()
                                    - expected["evaluation_grade"].as_f64().unwrap())
                                .abs()
                                    < 1e-10
                        })
            })
            .expect("original checked Ornament candidate must be present in actual image trials");
        let glyph_id = trial["glyph_id"].as_u64().unwrap() as usize;
        let glyph = &music.owner.prepared.registry.get(glyph_id).unwrap().glyph;
        let lookup = create(
            &music.owner.graph,
            music,
            expected["system"].as_u64().unwrap(),
            glyph_id,
            crate::beam_dispatch::glyph_json(glyph),
            expected["shape"].as_str().unwrap(),
            expected["intrinsic_grade"].as_f64().unwrap(),
        )
        .unwrap();
        assert_eq!(
            lookup.is_none(),
            expected["returns_null"].as_bool().unwrap()
        );
        if let Some((chord_key, body, relation)) = lookup {
            let chord = music.chords.iter().find(|c| c.key == chord_key).unwrap();
            assert_eq!(
                json!(chord.bounds),
                expected["separate_original_lookup"]["partner_bounds"]
            );
            for field in ["grade", "dx", "dy", "source_ratio", "target_ratio"] {
                close(
                    &relation[field],
                    &expected["separate_original_lookup"][field],
                );
            }
            assert_eq!(body["staff_id"], Value::Null);
            assert_eq!(body["manual"], false);
            factory_results
                .push(json!({"bounds":body["bounds"],"chord_key":chord_key,"relation":relation}));
        }
    }
    assert_eq!(factory["factory_candidates"].as_array().unwrap().len(), 24);
    assert_eq!(factory_results.len(), 3);
    assert_eq!(music.owner.graph.snapshot(), before);

    let mut held = Vec::new();
    let mut confidence_residuals = Vec::new();
    for ornament in &ornaments {
        let key: Key = serde_json::from_value(ornament["origin"].clone()).unwrap();
        let expected = source["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["stage"] == "after_ordinary_SYMBOLS" && f["system"] == key.system)
            .flat_map(|f| f["retained_ornaments"].as_array().unwrap())
            .find(|o| o["bounds"] == ornament["bounds"])
            .unwrap();
        close(&ornament["grade"], &expected["grade"]);
        for field in ["manual", "abnormal", "frozen", "contextual_grade"] {
            assert_eq!(ornament[field], expected[field]);
        }
        assert_eq!(ornament["staff_id"], expected["raw_staff"]);
        let relation = before["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"] == "ChordOrnamentRelation" && r["target"] == ornament["id"])
            .unwrap();
        let original = &expected["held_actual_relations"][0];
        let chord = music
            .chords
            .iter()
            .find(|c| relation["source"] == c.id)
            .unwrap();
        let (body, removed) = music.owner.graph.retained_inter(chord.key).unwrap();
        assert!(!removed);
        assert_eq!(json!(chord.bounds), original["source_chord"]["bounds"]);
        let source_chord = confidence["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["stage"] == "after_ordinary_SYMBOLS" && f["system"] == key.system)
            .flat_map(|f| f["head_chords"].as_array().unwrap())
            .find(|c| c["bounds"] == json!(chord.bounds))
            .unwrap();
        let source_at_creation = confidence["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["stage"] == "after_ordinary_CHORDS" && f["system"] == key.system)
            .flat_map(|f| f["head_chords"].as_array().unwrap())
            .find(|c| c["object"] == source_chord["object"])
            .unwrap();
        assert_eq!(source_chord["members"], source_at_creation["members"]);
        assert_eq!(
            source_chord["containment"],
            source_at_creation["containment"]
        );
        close(&source_chord["raw_grade"], &source_at_creation["raw_grade"]);
        close(
            &source_chord["raw_grade"],
            &original["source_chord"]["raw_grade"],
        );
        let member_ids = body["member_ids"].as_array().unwrap();
        assert_eq!(
            member_ids.len(),
            source_chord["members"].as_array().unwrap().len()
        );
        let mut member_best_sum = 0.;
        let mut member_residuals = Vec::new();
        for id in member_ids {
            let member = before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == *id)
                .unwrap();
            assert!(
                before["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["kind"] == "Containment"
                        && r["source"] == chord.id
                        && r["target"] == *id)
            );
            let expected_member = source_chord["members"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["bounds"] == member["bounds"] && n["shape"] == member["shape"])
                .unwrap();
            assert_eq!(member["staff_id"], expected_member["staff"]);
            let best = member["contextual_grade"]
                .as_f64()
                .unwrap_or_else(|| member["grade"].as_f64().unwrap());
            member_best_sum += best;
            member_residuals.push(json!({"bounds":member["bounds"],"native_best_grade":best,
                "source_best_grade":expected_member["best_grade"],
                "difference":best-expected_member["best_grade"].as_f64().unwrap()}));
        }
        close(
            &body["grade"],
            &json!(member_best_sum / member_ids.len() as f64),
        );
        confidence_residuals.push(json!({"ornament_bounds":ornament["bounds"],"chord_bounds":chord.bounds,
            "native_cached_grade":body["grade"],"source_cached_grade":source_chord["raw_grade"],
            "difference":body["grade"].as_f64().unwrap()-source_chord["raw_grade"].as_f64().unwrap(),
            "actual_member_geometry_matches":true,"members":member_residuals}));
        for (field, original_field) in [
            ("voice", "voice"),
            ("slot_id", "slot"),
            ("time_offset", "time_offset"),
        ] {
            assert_eq!(body[field], original["source_chord"][original_field]);
        }
        let measure = &music.measures[chord.measure];
        let layout = measure.geometry.as_ref().unwrap();
        assert!(
            music
                .measure_layout
                .parts
                .iter()
                .any(|part| part.measures.iter().any(|m| Arc::ptr_eq(m, layout)))
        );
        assert_eq!(
            json!(measure.part),
            original["source_chord"]["measure"]["part"]
        );
        assert_eq!(
            json!(measure.index),
            original["source_chord"]["measure"]["measure_index"]
        );
        held.push((
            key,
            (*ornament).clone(),
            relation.clone(),
            chord.key,
            body.clone(),
        ));
    }
    let mut graph = music.owner.graph.clone();
    PURGE_FRAMES.with(|frames| *frames.borrow_mut() = Some(Vec::new()));
    let result = contextualize_and_purge(&mut graph, music).unwrap();
    let frames = PURGE_FRAMES.with(|frames| frames.borrow_mut().take().unwrap());
    assert_eq!(
        result["removed_weak_ornament_ids"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    for frame in frames
        .iter()
        .filter(|frame| frame["stage"] == "contextualized")
    {
        assert!(
            frame["ornaments"]
                .as_array()
                .unwrap()
                .iter()
                .all(|o| o["removed"] == false && o["body"]["contextual_grade"].is_number())
        );
    }
    for (key, body, relation, chord_key, chord_body) in &held {
        let expected = source["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["stage"] == "after_ordinary_LINKS" && f["system"] == key.system)
            .flat_map(|f| f["retained_ornaments"].as_array().unwrap())
            .find(|o| o["bounds"] == body["bounds"])
            .unwrap();
        let (retained, removed) = graph.retained_inter(*key).unwrap();
        assert!(removed);
        let native_grade = body["grade"].as_f64().unwrap();
        let source_grade = expected["grade"].as_f64().unwrap();
        let coefficient = relation["target_ratio"].as_f64().unwrap() - 1.;
        let native_support = coefficient * chord_body["grade"].as_f64().unwrap();
        let source_support = (expected["held_actual_relations"][0]["target_ratio"]
            .as_f64()
            .unwrap()
            - 1.)
            * expected["held_actual_relations"][0]["source_chord"]["raw_grade"]
                .as_f64()
                .unwrap();
        let equation = |grade: f64, support: f64| grade * (1. + support) / (1. + grade * support);
        close(
            &retained["contextual_grade"],
            &json!(equation(native_grade, native_support)),
        );
        close(
            &expected["contextual_grade"],
            &json!(equation(source_grade, source_support)),
        );
        let bound = 0.25 * (native_support - source_support).abs()
            + (1. + native_support.max(source_support)) * (native_grade - source_grade).abs();
        assert!(
            (retained["contextual_grade"].as_f64().unwrap()
                - expected["contextual_grade"].as_f64().unwrap())
            .abs()
                <= bound + 1e-10
        );
        assert_eq!(
            retained["contextual_grade"].as_f64().unwrap() < 0.5,
            expected["contextual_grade"].as_f64().unwrap() < 0.5
        );
        for field in ["id", "bounds", "manual", "abnormal", "staff_id", "grade"] {
            assert_eq!(retained[field], body[field]);
        }
        let (edge, removed) = graph
            .retained_ornament_relation(relation["id"].as_u64().unwrap() as usize)
            .unwrap();
        assert!(removed);
        assert_eq!(edge, relation);
        assert_eq!(
            graph.retained_inter(*chord_key).unwrap(),
            (chord_body, false)
        );
    }
    // This separate declared control substitutes measured original cache
    // grades only in a clone. Recognition above keeps every native grade.
    let mut exact_control = music.owner.graph.clone();
    for (key, body, _, chord_key, chord_body) in &held {
        let original = source["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["stage"] == "after_ordinary_SYMBOLS" && f["system"] == key.system)
            .flat_map(|f| f["retained_ornaments"].as_array().unwrap())
            .find(|o| o["bounds"] == body["bounds"])
            .unwrap();
        let mut declared = chord_body.clone();
        declared["grade"] =
            original["held_actual_relations"][0]["source_chord"]["raw_grade"].clone();
        exact_control.vertex(*chord_key, declared, false).unwrap();
    }
    let exact_contexts = contextualize_and_purge(&mut exact_control, music).unwrap();
    for (key, body, _, _, _) in &held {
        let original = source["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["stage"] == "after_ordinary_LINKS" && f["system"] == key.system)
            .flat_map(|f| f["retained_ornaments"].as_array().unwrap())
            .find(|o| o["bounds"] == body["bounds"])
            .unwrap();
        close(
            &exact_control.retained_inter(*key).unwrap().0["contextual_grade"],
            &original["contextual_grade"],
        );
    }
    // A late frozen survivor is still an explicit output frontier. Earlier
    // staged removals must not leak through that failure or clear real clocks.
    let mut failure = music.owner.graph.clone();
    let (key, body, relation, chord_key, _) = held.last().unwrap();
    let mut frozen = body.clone();
    frozen["frozen"] = json!(true);
    frozen["manual"] = json!(true);
    failure.vertex(*key, frozen, false).unwrap();
    let exact = crate::music_time::Fraction::new(7, 64).unwrap();
    let mut chord_body = failure.retained_inter(*chord_key).unwrap().0.clone();
    chord_body["time_offset"] = json!(crate::music_time::format_fraction(exact));
    chord_body["manual"] = json!(true);
    failure.vertex(*chord_key, chord_body, false).unwrap();
    assert_eq!(
        failure.actual_chord_time_offset(*chord_key).unwrap(),
        Some(exact)
    );
    let end = failure.chord_end_time(*chord_key).unwrap();
    assert!(end.is_some());
    let saved = failure.snapshot();
    let expected_durations = music
        .measures
        .iter()
        .map(|m| m.timing.expected)
        .collect::<Vec<_>>();
    assert_eq!(
        contextualize_and_purge(&mut failure, music).unwrap_err(),
        "music_ornament_survivor_export_pending"
    );
    assert_eq!(failure.snapshot(), saved);
    assert_eq!(
        failure.actual_chord_time_offset(*chord_key).unwrap(),
        Some(exact)
    );
    assert_eq!(failure.chord_end_time(*chord_key).unwrap(), end);
    assert_eq!(
        music
            .measures
            .iter()
            .map(|m| m.timing.expected)
            .collect::<Vec<_>>(),
        expected_durations
    );
    assert_eq!(
        failure
            .retained_ornament_relation(relation["id"].as_u64().unwrap() as usize)
            .unwrap(),
        (relation, false)
    );
    assert_eq!(music.owner.graph.snapshot(), before);
    json!({"actual_symbols_ornaments":3,"original_checked_candidates":24,"positive_factories":factory_results,
        "context_and_purge":result,"per_system_context_before_removal":frames,
        "measured_cached_HeadChord_residuals":confidence_residuals,"numeric_parity_claimed":false,
        "declared_source_cache_grade_clone_control":exact_contexts,
        "retained_relation_identity_and_raw_staff_null":true,"late_survivor_rollback_keeps_actual_chord_7_64":true,
        "scope":"Actual image factory owners and weak Ornament purge only; original overlap Exclusion event sequence and surviving ornament export remain separate frontiers."})
}

pub(super) fn record(
    graph: &Graph,
    music: &OwnedMusic,
    system: u64,
    bounds: [i64; 4],
    shape: &str,
    grade: f64,
    link: &Option<(Key, Value)>,
) {
    OBSERVATIONS.with(|observations| {
        let mut observations = observations.borrow_mut();
        let Some(rows) = observations.as_mut() else {
            return;
        };
        let chords = music
            .chords
            .iter()
            .filter(|c| c.key.system == system && c.kind == ChordKind::Head)
            .map(|chord| {
                let (body, removed) = graph.retained_inter(chord.key).unwrap();
                assert!(!removed);
                assert_eq!(graph.id(chord.key), Some(chord.id));
                let measure = &music.measures[chord.measure];
                let layout = measure.geometry.as_ref().unwrap();
                assert!(
                    music
                        .measure_layout
                        .parts
                        .iter()
                        .any(|part| part.measures.iter().any(|m| Arc::ptr_eq(m, layout)))
                );
                json!({"key":chord.key,"id":chord.id,"bounds":chord.bounds,
                    "staff":body["staff_id"],"manual":body["manual"],
                    "voice":body["voice"],"slot":body["slot_id"],"time_offset":body["time_offset"],
                    "measure_part":measure.part,"measure_index":measure.index,
                    "actual_measure_retained":true})
            })
            .collect::<Vec<_>>();
        rows.push(
            json!({"system":system,"bounds":bounds,"shape":shape,"grade":grade,
            "returns_null":link.is_none(),"lookup":link,"actual_chords":chords}),
        );
    });
}

#[test]
fn an_laura_checked_ornaments_preserve_original_null_factory_without_registration() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ornaments/factory-observed.json"
    ))
    .unwrap();
    let stages: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ornaments/stages-observed.json"
    ))
    .unwrap();
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-grid/laura-gray.png"
    ))
    .unwrap();
    let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
    let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
    let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
    let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
    let owner = crate::music_links::finalize_stems(owner).unwrap();
    let music = crate::music_measures::continue_measures(owner).unwrap();
    let measures = music
        .measures
        .iter()
        .map(|m| m.geometry.clone().unwrap())
        .collect::<Vec<_>>();
    OBSERVATIONS.with(|rows| *rows.borrow_mut() = Some(Vec::new()));
    let result = crate::music_symbols::build_symbols(music);
    let rows = OBSERVATIONS.with(|rows| rows.borrow_mut().take().unwrap());
    let output = std::env::var("AUDIVERIS_REAL_ORNAMENT_OBSERVATIONS")
        .unwrap_or_else(|_| "/tmp/real-ornament-native-symbols.json".into());
    let report = result
        .as_ref()
        .map(|music| music.owner.prepared.report.clone())
        .unwrap_or_else(|error| json!({"error":error}));
    std::fs::write(&output, serde_json::to_vec_pretty(&json!({
        "scope":"Actual unchanged AnLaura image through native SYMBOLS. Candidate rows inspect retained Graph/Chord/Measure objects; reports do not construct ownership.",
        "ornament_candidates":rows,"native_symbols_report":report
    })).unwrap()).unwrap();
    let music = result.unwrap();
    let expected = source["factory_candidates"].as_array().unwrap();
    assert_eq!(expected.len(), 8);
    assert_eq!(rows.len(), expected.len());
    for original in expected {
        let row = rows
            .iter()
            .find(|row| {
                row["system"] == original["system"]
                    && row["shape"] == original["shape"]
                    && row["bounds"] == original["bounds"]
            })
            .expect("every original checked candidate must reach the native factory");
        assert_eq!(row["returns_null"], original["returns_null"]);
        assert_eq!(row["lookup"], original["separate_original_lookup"]);
        assert_eq!(original["vertex_edge_identity_retained"], true);
        assert!(
            (row["grade"].as_f64().unwrap() - original["intrinsic_grade"].as_f64().unwrap()).abs()
                < 1e-10
        );
        for reference in original["nearby_actual_head_chords"].as_array().unwrap() {
            let chord = row["actual_chords"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["bounds"] == reference["bounds"])
                .unwrap();
            assert_eq!(chord["staff"], reference["raw_staff"]);
            assert_eq!(chord["measure_part"], reference["measure"]["part"]);
            assert_eq!(
                chord["measure_index"],
                reference["measure"]["measure_index"]
            );
            assert_eq!(chord["voice"], reference["voice"]);
            assert_eq!(chord["slot"], reference["slot"]);
            assert_eq!(chord["time_offset"], reference["time_offset"]);
            assert_eq!(reference["grade"], 0.);
        }
    }
    for (measure, held) in music.measures.iter().zip(measures) {
        assert!(Arc::ptr_eq(measure.geometry.as_ref().unwrap(), &held));
    }
    for system in &music.owner.systems {
        assert!(
            music
                .owner
                .graph
                .live_inter_keys(system.id, "OrnamentInter")
                .is_empty()
        );
    }
    assert!(
        stages["frames"]
            .as_array()
            .unwrap()
            .iter()
            .all(|frame| frame["ornaments"].as_array().unwrap().is_empty())
    );
    assert!(
        music.owner.graph.snapshot()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|relation| relation["kind"] != "ChordOrnamentRelation")
    );
}
