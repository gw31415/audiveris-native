// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{GrayImage, music_measures::OwnedMusic};
fn rows(music: &OwnedMusic) -> Vec<Value> {
    let snapshot = music.owner.graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut rows=nodes.iter().filter(|n|matches!(n["kind"].as_str(),Some("CaesuraInter"|"BreathMarkInter"))).map(|n| {
        let relations=snapshot["relations"].as_array().unwrap().iter().filter(|e|e["kind"]=="ChordPauseRelation" && e["target"]==n["id"]).map(|e| {
            let ch=nodes.iter().find(|c|c["id"]==e["source"]).unwrap();
            json!({"kind":e["kind"],"manual":e["manual"],"opposite_kind":ch["kind"],"bounds":ch["bounds"],"source_ratio":e["source_ratio"],"target_ratio":e["target_ratio"],"relation_grade":e["grade"],"partner_grade":ch["grade"]})
        }).collect::<Vec<_>>();
        json!({"system":n["origin"]["system"],"kind":n["kind"],"shape":n["shape"],"bounds":n["bounds"],"staff":n["staff_id"],"manual":n["manual"],"abnormal":n["abnormal"],"grade":n["grade"],"contextual_grade":n["contextual_grade"],"relations":relations})
    }).collect::<Vec<_>>();
    rows.sort_by_key(|r| format!("{}{}{}", r["system"], r["bounds"], r["shape"]));
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
    rows.sort_by_key(|r| format!("{}{}{}", r["system"], r["bounds"], r["shape"]));
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
fn unchanged_real_page_pause_part_staff_lookup_and_notation_match_original() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-pause-part-staff/image-observed.json"
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
    let music = crate::music_measures::continue_measures(owner).unwrap();
    let mut music = crate::music_symbols::continue_symbols(music).unwrap();
    let before = rows(&music);
    let expect_before = expected(&source, 0);
    std::fs::write(
        "/tmp/real-pause-part-staff-current.json",
        serde_json::to_vec_pretty(&json!({"before":before,"expected_before":expect_before}))
            .unwrap(),
    )
    .unwrap();
    let mut residuals = Vec::new();
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
    for (a, b) in before.iter().zip(&expect_before) {
        let p = a["relations"][0]["partner_grade"].as_f64().unwrap();
        let q = b["relations"][0]["partner_grade"].as_f64().unwrap();
        assert!((p - q).abs() <= bound + 1e-10);
        residuals.push(json!({"bounds":a["bounds"],"native_cached_Chord_grade":p,"original_cached_Chord_grade":q,"absolute_residual":(p-q).abs()}));
    }
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
    music = crate::music_links::continue_links(music).unwrap();
    let after = rows(&music);
    let expect_after = expected(&source, 1);
    assert_eq!(before.len(), 15);
    assert_eq!(after.len(), 3);
    compare_numeric(
        &json!(stripped(after.clone())),
        &json!(stripped(expect_after.clone())),
    );
    for (a, b) in after.iter().zip(&expect_after) {
        let p = a["relations"][0]["partner_grade"].as_f64().unwrap();
        let q = b["relations"][0]["partner_grade"].as_f64().unwrap();
        let g = a["grade"].as_f64().unwrap();
        let r = a["relations"][0]["relation_grade"].as_f64().unwrap();
        let cg = a["contextual_grade"].as_f64().unwrap();
        let original_cg = b["contextual_grade"].as_f64().unwrap();
        assert!((cg - (1. + 3. * r * p) * g / (1. + 3. * r * p * g)).abs() < 1e-12);
        assert!((original_cg - (1. + 3. * r * q) * g / (1. + 3. * r * q * g)).abs() < 1e-10);
        assert!((cg - original_cg).abs() <= 0.75 * (p - q).abs() + 1e-9);
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
    let baseline = music.owner.graph.snapshot();
    music.fix_recognized_page_measures().unwrap();
    let (xml, observation) = music.write_recognized_musicxml("An Laura").unwrap();
    std::fs::write("/tmp/real-pause-part-staff-native.musicxml", xml).unwrap();
    std::fs::write("/tmp/real-pause-part-staff-native-observed.json",serde_json::to_vec_pretty(&json!({"before":before,"after":after,"cached_confidence_residuals":residuals,"export":observation,"actual_Chord_vector_retained":music.chords.as_ptr()==actual_chords,"read_does_not_change_graph_clocks_relations_manual_status":music.owner.graph.snapshot()==baseline})).unwrap()).unwrap();
    assert_eq!(baseline, music.owner.graph.snapshot());
    assert_eq!(observation["events"].as_array().unwrap().len(), 92);
}
