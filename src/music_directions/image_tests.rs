// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{GrayImage, music_measures::OwnedMusic};
fn rows(music: &OwnedMusic) -> Vec<Value> {
    let snapshot = music.owner.graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut rows=nodes.iter().filter(|n|matches!(n["kind"].as_str(),Some("DynamicsInter"|"PedalInter"))).map(|n| {
        let relations=snapshot["relations"].as_array().unwrap().iter().filter(|e|matches!(e["kind"].as_str(),Some("ChordDynamicsRelation"|"ChordPedalRelation")) && e["target"]==n["id"]).map(|e| {
            let ch=nodes.iter().find(|c|c["id"]==e["source"]).unwrap();
            json!({"kind":e["kind"],"manual":e["manual"],"opposite_kind":ch["kind"],"bounds":ch["bounds"],"source_ratio":e["source_ratio"],"target_ratio":e["target_ratio"]})
        }).collect::<Vec<_>>();
        json!({"system":n["origin"]["system"],"kind":n["kind"],"shape":n["shape"],"bounds":n["bounds"],"staff":n["staff_id"],"manual":n["manual"],"abnormal":n["abnormal"],"grade":n["grade"],"contextual_grade":n["contextual_grade"],"relations":relations})
    }).collect::<Vec<_>>();
    rows.sort_by_key(|r| format!("{}{}{}{}", r["system"], r["bounds"], r["shape"], r["grade"]));
    rows
}
fn expected(source: &Value, index: usize) -> Vec<Value> {
    let mut rows = Vec::new();
    for system in source["stages"][index]["systems"].as_array().unwrap() {
        for n in system["directions"].as_array().unwrap() {
            let mut n = n.clone();
            n["system"] = system["system"].clone();
            rows.push(n);
        }
    }
    rows.sort_by_key(|r| format!("{}{}{}{}", r["system"], r["bounds"], r["shape"], r["grade"]));
    rows
}
fn compare_numeric(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10)
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                compare_numeric(a, b);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, v) in a {
                compare_numeric(v, &b[k]);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}
#[test]
fn unchanged_real_page_dynamics_pedal_factory_lookup_and_export_match_original() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-directions/image-observed.json"
    ))
    .unwrap();
    let image = GrayImage::decode(include_bytes!(
        "../../fixtures/official-grid/laura-gray.png"
    ))
    .unwrap();
    let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
    let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
    let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
    let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
    let owner = crate::music_links::finalize_stems(owner).unwrap();
    let header_owners = crate::music_headers_owned::image_tests::hold(&owner);
    let music = crate::music_measures::continue_measures(owner).unwrap();
    crate::music_headers_owned::image_tests::verify(&music, &header_owners, "an_laura", "MEASURES");
    let mut music = crate::music_symbols::continue_symbols(music).unwrap();
    crate::music_symbols::assert_real_checked_rejections(&music, "an_laura");
    let header_filler = crate::shared_sig::Graph::assert_real_owned_filler(&music);
    // The original observer calls Pedal.getStaff at SYMBOLS; compare that lazy
    // read separately without assigning those source IDs into the native graph.

    let mut before = rows(&music);
    for n in &mut before {
        if n["kind"] == "PedalInter" {
            let b: [i64; 4] = serde_json::from_value(n["bounds"].clone()).unwrap();
            let system = music
                .owner
                .systems
                .iter()
                .find(|s| n["system"] == s.id)
                .unwrap();
            n["staff"] = json!(
                sided_staff(
                    &music,
                    system,
                    [(b[0] + b[2] / 2) as f64, (b[1] + b[3] / 2) as f64],
                    true
                )
                .unwrap()
                .map(|s| s.id)
            );
        }
    }
    let expect_before = expected(&source, 0);
    std::fs::write(
        "/tmp/real-directions-current.json",
        serde_json::to_vec_pretty(&json!({"before":before,"expected_before":expect_before}))
            .unwrap(),
    )
    .unwrap();
    let mut residuals: Vec<Value> = Vec::new();
    let bound: Value = serde_json::from_str(include_str!(
        "../../evidence/real-symbol-ensemble/VALIDATION.json"
    ))
    .unwrap();
    let bound = bound["numeric_grade_residuals"]["maximum_absolute_difference"]
        .as_f64()
        .unwrap();
    let stripped = |rows: Vec<Value>| {
        rows.into_iter()
            .map(|mut n| {
                n["contextual_grade"] = Value::Null;
                for r in n["relations"].as_array_mut().unwrap() {
                    r.as_object_mut().unwrap().remove("partner_grade");
                }
                n
            })
            .collect::<Vec<_>>()
    };
    compare_numeric(
        &json!(stripped(before.clone())),
        &json!(stripped(expect_before.clone())),
    );
    let actual_owners = music
        .chords
        .iter()
        .filter(|c| c.kind == ChordKind::Head)
        .map(|c| {
            (
                c.key,
                c.id,
                c.measure,
                c.stem,
                c.members
                    .iter()
                    .map(|n| n["id"].clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let frames = music.owner.graph.snapshot();
    crate::music_symbol_reduction::owned_image_tests::begin(&music);
    music = crate::music_links::continue_links(music).unwrap();
    crate::music_symbol_reduction::owned_image_tests::finish(&music, "an_laura");
    crate::music_headers_owned::image_tests::verify(&music, &header_owners, "an_laura", "LINKS");
    crate::shared_sig::Graph::assert_completed_header_filler(&music, &header_filler);
    let after = rows(&music);
    let expect_after = expected(&source, 1);
    assert_eq!(before.len(), 29);
    assert_eq!(after.len(), 6);
    compare_numeric(
        &json!(stripped(after.clone())),
        &json!(stripped(expect_after.clone())),
    );
    for (a, b) in after.iter().zip(&expect_after) {
        let g = a["grade"].as_f64().unwrap();
        let original_cg = b["contextual_grade"].as_f64().unwrap();
        let cg = a["contextual_grade"].as_f64().unwrap();
        if a["kind"] == "DynamicsInter" {
            let rel = &a["relations"][0];
            let chord = music
                .chords
                .iter()
                .find(|c| json!(c.bounds) == rel["bounds"] && a["system"] == c.key.system)
                .unwrap();
            let p = music.owner.graph.retained_inter(chord.key).unwrap().0["grade"]
                .as_f64()
                .unwrap();
            assert!((cg - (1. + 2.5 * p) * g / (1. + 2.5 * p * g)).abs() < 1e-12);
            assert!((cg - original_cg).abs() <= 0.625 * bound + 1e-9);
            residuals.push(json!({"bounds":a["bounds"],"native_contextual_grade":cg,"original_contextual_grade":original_cg,"absolute_residual":(cg-original_cg).abs()}));
        } else {
            assert!((cg - g).abs() < 1e-12);
        }
    }
    let retained_owners = music
        .chords
        .iter()
        .filter(|c| c.kind == ChordKind::Head)
        .map(|c| {
            (
                c.key,
                c.id,
                c.measure,
                c.stem,
                c.members
                    .iter()
                    .map(|n| n["id"].clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_owners, retained_owners);
    let actual_chords = music.chords.as_ptr();
    let after_snapshot = music.owner.graph.snapshot();
    for chord in &music.chords {
        let before = frames["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == chord.id)
            .unwrap();
        assert_eq!(music.owner.graph.id(chord.key), Some(chord.id));
        let after = after_snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == chord.id)
            .unwrap();
        for field in [
            "origin",
            "member_ids",
            "stem_id",
            "part_id",
            "measure_index",
            "time_offset",
            "voice",
            "slot_id",
            "manual",
        ] {
            assert_eq!(before[field], after[field]);
        }
    }
    music.process_recognized_page_rhythm().unwrap();
    let callbacks = music.owner.graph.observe_actual_pedal_callbacks().unwrap();
    let original_callbacks: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-pedal-callbacks/image-observed.json"
    ))
    .unwrap();
    assert_eq!(callbacks["stages"], original_callbacks["stages"]);
    assert_eq!(
        callbacks["actual_identity_retained"],
        original_callbacks["actual_identity_retained"]
    );
    let baseline = music.owner.graph.snapshot();
    music.fix_recognized_page_measures().unwrap();
    let (xml, observation) = music.write_recognized_musicxml("An Laura").unwrap();
    std::fs::write("/tmp/real-directions-native.musicxml", xml).unwrap();
    std::fs::write("/tmp/real-directions-native-observed.json",serde_json::to_vec_pretty(&json!({"before":before,"after":after,"cached_confidence_residuals":residuals,"pedal_callbacks":callbacks,"export":observation,"actual_Chord_vector_retained":music.chords.as_ptr()==actual_chords,"read_does_not_change_graph_clocks_relations_manual_status":music.owner.graph.snapshot()==baseline})).unwrap()).unwrap();
    assert_eq!(baseline, music.owner.graph.snapshot());
    assert_eq!(observation["events"].as_array().unwrap().len(), 92);
}
