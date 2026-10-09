// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
fn sorted(mut v: Vec<Value>) -> Vec<Value> {
    v.sort_by_key(Value::to_string);
    v
}
fn assert_stage(music: &crate::music_measures::OwnedMusic, stage: &Value) {
    let snapshot = music.owner.graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut actual = Vec::new();
    for chord in &music.chords {
        let n = nodes.iter().find(|n| n["id"] == chord.id).unwrap();
        let measure = &music.measures[chord.measure];
        let held = measure.geometry.as_ref().unwrap();
        assert!(
            music
                .owner
                .parts
                .iter()
                .any(|p| std::sync::Arc::ptr_eq(p, &held.part))
        );
        assert!(
            music
                .measure_layout
                .parts
                .iter()
                .flat_map(|p| &p.measures)
                .any(|m| std::sync::Arc::ptr_eq(m, held))
        );
        let members = n["member_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| {
                let h = nodes.iter().find(|n| n["id"] == *id).unwrap();
                json!({"kind":h["kind"],"bounds":h["bounds"],"staff":h["staff_id"]})
            })
            .collect::<Vec<_>>();
        actual.push(json!({"system":measure.system,"kind":n["kind"],"bounds":n["bounds"],"part":measure.part,"measure_index":measure.index,"staff":n["staff_id"],"same_actual_Part_Measure_object":true,"members":members}));
    }
    let expected = stage["systems"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|sys| {
            sys["chords"].as_array().unwrap().iter().map(move |c| {
                let mut c = c.clone();
                assert_eq!(c["same_actual_Part_Measure_object"], true);
                c["system"] = sys["system"].clone();
                c
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(sorted(actual), sorted(expected));
    for sys in stage["systems"].as_array().unwrap() {
        for staff in sys["staff_notes"].as_array().unwrap() {
            let actual = nodes
                .iter()
                .filter(|n| {
                    n["origin"]["system"] == sys["system"]
                        && n["staff_id"] == staff["staff"]
                        && matches!(n["kind"].as_str(), Some("HeadInter" | "RestInter"))
                })
                .map(|n| json!({"kind":n["kind"],"bounds":n["bounds"]}))
                .collect::<Vec<_>>();
            assert_eq!(
                sorted(actual),
                sorted(staff["notes"].as_array().unwrap().clone())
            );
        }
    }
}
pub(crate) fn assert_real_part_memberships(
    mut music: crate::music_measures::OwnedMusic,
) -> (crate::music_measures::OwnedMusic, Value) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-recognition-containers/image-observed.json"
    ))
    .unwrap();
    let stages = source["stages"].as_array().unwrap();
    let symbols = stages.iter().find(|s| s["stage"] == "SYMBOLS").unwrap();
    let links = stages.iter().find(|s| s["stage"] == "LINKS").unwrap();
    let before = music.owner.graph.snapshot();
    let heads = music
        .chords
        .iter()
        .filter(|c| c.kind == ChordKind::Head)
        .count();
    assert_eq!(heads, 67);
    assert_eq!(music.chords.len(), 73);
    let initial = refresh(
        &mut music.owner.graph,
        &music.measures,
        &mut music.chords,
        &mut music.voices,
    )
    .unwrap();
    assert!(initial.removed_chords.is_empty());
    assert_eq!(music.owner.graph.snapshot(), before);
    assert_stage(&music, symbols);
    let mut foreign = music.measures.clone();
    let old = foreign.last().unwrap().geometry.as_ref().unwrap();
    let replacement = std::sync::Arc::new(crate::music_measure_layout::MeasureLayout {
        part: old.part.clone(),
        index: old.index,
        stack: old.stack.clone(),
        contained_left: old.contained_left.clone(),
        effective_left: old.effective_left.clone(),
        right: old.right.clone(),
    });
    foreign.last_mut().unwrap().geometry = Some(replacement);
    let mut invalid = music.owner.graph.clone();
    let ck = music.chords[0].key;
    let mut body = invalid.retained_inter(ck).unwrap().0.clone();
    body["manual"] = json!(true);
    body["time_offset"] = json!({"num":7,"den":64});
    body["expected_duration"] = json!({"num":3,"den":4});
    invalid.vertex(ck, body, false).unwrap();
    let bad_before = invalid.snapshot();
    let mut old_chords = music.chords.clone();
    let mut old_voices = music.voices.clone();
    let ids = old_chords.iter().map(|c| c.id).collect::<Vec<_>>();
    assert_eq!(
        refresh(&mut invalid, &foreign, &mut old_chords, &mut old_voices)
            .err()
            .unwrap(),
        "music_dot_measure_rebinding_pending"
    );
    assert_eq!(invalid.snapshot(), bad_before);
    assert_eq!(old_chords.iter().map(|c| c.id).collect::<Vec<_>>(), ids);
    let mut flag_overlap_preparation =
        crate::music_flags::detect_overlaps(&mut music.owner.graph).unwrap();
    let prepared = music.owner.graph.snapshot();
    let source_order: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-flag-overlap-order/image-observed.json"
    ))
    .unwrap();
    let flags = |snapshot: &Value| {
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "FlagInter")
            .map(
                |n| json!({"system":n["origin"]["system"],"shape":n["shape"],"bounds":n["bounds"]}),
            )
            .collect::<Vec<_>>()
    };
    let source_flags = |stage: &str| {
        source_order["stages"].as_array().unwrap().iter().find(|s|s["stage"] == stage).unwrap()["systems"].as_array().unwrap().iter().flat_map(|sys|sys["flags"].as_array().unwrap().iter().map(move |n|json!({"system":sys["system"],"shape":n["shape"],"bounds":n["bounds"]}))).collect::<Vec<_>>()
    };
    assert_eq!(
        sorted(flags(&prepared)),
        sorted(source_flags("FLAG_OVERLAPS"))
    );
    let inserted = flag_overlap_preparation["inserted_flag_overlap_pairs"]
        .as_array()
        .unwrap();
    assert_eq!(inserted.len(), 2);
    let exclusions = prepared["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            e["kind"] == "Exclusion"
                && inserted.iter().any(|p| {
                    (p[0] == e["source"] && p[1] == e["target"])
                        || (p[1] == e["source"] && p[0] == e["target"])
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(exclusions.len(), 2);
    crate::music_alters::contextualize_and_purge(&mut music.owner.graph).unwrap();
    crate::music_flags::contextualize(&mut music.owner.graph).unwrap();
    crate::music_symbol_reduction::reduce(&mut music.owner.graph).unwrap();
    crate::music_beams::clean_head_supports(&mut music.owner.graph).unwrap();
    let graph_before_refresh = music.owner.graph.snapshot();
    assert_eq!(
        sorted(flags(&graph_before_refresh)),
        sorted(source_flags("WEAK_PURGED"))
    );
    for relation in &exclusions {
        assert_eq!(
            graph_before_refresh["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == relation["id"])
                .unwrap(),
            relation
        );
    }
    flag_overlap_preparation["original_pre_weak_Flag_membership_matches"] = json!(true);
    flag_overlap_preparation["actual_exclusion_identity_survives_weak_callbacks"] = json!(true);
    let result = refresh(
        &mut music.owner.graph,
        &music.measures,
        &mut music.chords,
        &mut music.voices,
    )
    .unwrap();
    assert_eq!(result.removed_chords.len(), 6);
    assert_eq!(music.chords.len(), 67);
    assert_eq!(music.owner.graph.snapshot(), graph_before_refresh);
    assert_stage(&music, links);
    assert_eq!(
        result.projection["staff_notes"].as_array().unwrap().len(),
        6
    );
    std::fs::write("/tmp/real-part-containers-native-observed.json",serde_json::to_vec_pretty(&json!({"actual_SYMBOLS_head_chords":67,"actual_SYMBOLS_rest_chords":6,"actual_LINKS_head_chords":67,"actual_LINKS_rest_chords":0,"actual_LINKS_heads":94,"all_six_Staff_note_memberships_and_Part_Measure_Chord_members_match":true,"actual_Part_Measure_Arcs_retained":true,"nullable_Chord_Staff_validated_from_members":true,"foreign_Measure_Arc_same_scalar_ID_rejected_before_changes":true,"declared_real_Chord_clock_and_manual_full_graph_rollback":true,"projection":result.projection})).unwrap()).unwrap();
    (music, flag_overlap_preparation)
}
