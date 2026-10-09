// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::GrayImage;

#[test]
fn real_image_tuplet_factory_stack_lookup_and_orphan_lifecycle_match_original() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-explicit-tuplets/factory-observed.json"
    ))
    .unwrap();
    let lookup_source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-explicit-tuplets/lookup-observed.json"
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
    let factories =
        music.owner.prepared.report["measures_head_chords"]["explicit_tuplet_factories"].clone();
    let original = source["factory_candidates"].as_array().unwrap();
    assert_eq!(factories.as_array().unwrap().len(), original.len());
    for (native, expected) in factories.as_array().unwrap().iter().zip(original) {
        for field in ["system", "shape", "bounds", "lookup_bounds", "returns_null"] {
            assert_eq!(native[field], expected[field], "{field}");
        }
        let expected_bounds = expected["factory_system_chords"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["selected_by_original_intersection"] == true)
            .map(|c| c["bounds"].clone())
            .collect::<Vec<_>>();
        let actual_bounds = native["nearby"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["bounds"].clone())
            .collect::<Vec<_>>();
        assert_eq!(actual_bounds, expected_bounds);
    }
    let keys = music
        .owner
        .systems
        .iter()
        .flat_map(|s| music.owner.graph.live_inter_keys(s.id, "TupletInter"))
        .collect::<Vec<_>>();
    assert_eq!(keys.len(), 1);
    let key = keys[0];
    let original_tuplet = &original[0]["result"];
    let node = music.owner.graph.retained_inter(key).unwrap().0.clone();
    for field in ["shape", "bounds", "manual", "abnormal"] {
        assert_eq!(node[field], original_tuplet[field]);
    }
    assert!(node["staff_id"].is_null());
    assert!(
        (node["grade"].as_f64().unwrap() - original_tuplet["grade"].as_f64().unwrap()).abs()
            < 1e-10
    );
    assert!(music.owner.graph.tuplet_chords(key).unwrap().is_empty());
    assert_eq!(
        music
            .measures
            .iter()
            .filter(|m| m.stack_tuplets.contains(&key))
            .count(),
        1
    );
    assert!(music.measures.iter().all(|m| m.tuplets.is_empty()));
    let mi = music
        .measures
        .iter()
        .position(|m| m.stack_tuplets.contains(&key))
        .unwrap();
    let stack = music.measures[mi].geometry.as_ref().unwrap().stack.clone();
    assert_eq!(
        json!(stack.index),
        original_tuplet["unassigned_stack_owners"][0]["index"]
    );
    assert!(
        music
            .measure_layout
            .stacks
            .iter()
            .any(|s| Arc::ptr_eq(&s.stack, &stack))
    );
    let mut graph = music.owner.graph.clone();
    for chord in &music.chords {
        if !graph.retained_inter(chord.key).unwrap().1 {
            graph.count_owned_augmentation(chord.key).unwrap();
        }
    }
    let (links, observation) = lookup(&music, &mut graph, key, &stack).unwrap();
    let reference = &lookup_source["rows"][0];
    assert!(links.is_empty());
    assert_eq!(
        observation["search_links"],
        reference["actual_lookup_links"]
    );
    assert_eq!(observation["target_staff"], reference["target_staff"]);
    let source_around = reference["around"].as_array().unwrap();
    let native_around = observation["around"].as_array().unwrap();
    assert_eq!(native_around.len(), source_around.len());
    for (a, b) in native_around.iter().zip(source_around) {
        assert_eq!(a["bounds"], b["bounds"]);
        for location in ["head", "tail"] {
            let expected = b[location]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as i64)
                .collect::<Vec<_>>();
            assert_eq!(a[location], json!(expected));
        }
    }
    let replay = reference["collector_replay"].as_array().unwrap();
    let trace = observation["collector"].as_array().unwrap();
    assert_eq!(trace.len(), replay.len());
    for (actual, expected) in trace.iter().zip(replay) {
        for field in ["base", "total", "expected_total", "status"] {
            assert_eq!(actual[field], expected["after"][field], "{field}");
        }
    }
    assert_eq!(observation["status"], "OUTSIDE");
    let warmed = graph.snapshot();
    let chord_storage = music.chords.as_ptr();
    let geometries = music
        .measures
        .iter()
        .map(|m| m.geometry.clone().unwrap())
        .collect::<Vec<_>>();
    let (removed, plan) = link_and_remove(&music, &mut graph).unwrap();
    assert_eq!(removed["tuplets"][0]["removed"], true);
    assert!(graph.retained_inter(key).unwrap().1);
    assert_eq!(graph.retained_inter(key).unwrap().0, &node);
    assert_eq!(graph.id(key), music.owner.graph.id(key));
    assert!(plan.iter().all(|(_, assigned, unassigned)| !assigned.contains(&key) && !unassigned.contains(&key)));
    let after = graph.snapshot();
    assert_eq!(warmed["relations"], after["relations"]);
    for before in warmed["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] != "TupletInter")
    {
        assert_eq!(
            Some(before),
            after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == before["id"])
        );
    }

    // This declared failure fixture adds exact clocks to the same retained
    // owners; the ordinary source control above has NULL pre-LINKS clocks.
    let mut failed = music.owner.graph.clone();
    let seed = music
        .chords
        .iter()
        .find(|c| c.kind == ChordKind::Head && c.key.system == key.system)
        .unwrap()
        .clone();
    let exact = Fraction::new(7, 64).unwrap();
    let voice =
        crate::music_voices::create_voice(&mut failed, &mut music.voices, &music.measures, &seed)
            .unwrap();
    let slot = crate::music_voices::Slot {
        identity: Arc::new(()),
        id: 1,
        time_offset: Some(exact),
        x_offset: seed.bounds[0],
        incoming_ids: vec![seed.id],
    };
    failed
        .set_chord_slot(seed.key, slot.id, &slot.identity)
        .unwrap();
    let mut head = failed.retained_inter(seed.key).unwrap().0.clone();
    head["time_offset"] = json!(crate::music_time::format_fraction(exact));
    head["manual"] = json!(true);
    failed.vertex(seed.key, head, false).unwrap();
    music.measures[seed.measure].timing.expected = Some(exact);
    music.measures[seed.measure].timing.slots.push(slot.clone());
    let mut frozen = failed.retained_inter(key).unwrap().0.clone();
    frozen["frozen"] = json!(true);
    failed.vertex(key, frozen, false).unwrap();
    let before_failure = failed.snapshot();
    assert_eq!(
        failed.actual_chord_time_offset(seed.key).unwrap(),
        Some(exact)
    );
    let end_time = failed.chord_end_time(seed.key).unwrap();
    assert!(end_time.is_some());
    assert_eq!(
        link_and_remove(&music, &mut failed).unwrap_err(),
        "music_removal_frozen_inter"
    );
    assert_eq!(failed.snapshot(), before_failure);
    assert_eq!(
        failed.actual_chord_time_offset(seed.key).unwrap(),
        Some(exact)
    );
    assert_eq!(failed.chord_end_time(seed.key).unwrap(), end_time);
    assert!(failed.chord_has_voice(seed.key, &music.voices[voice]));
    assert!(Arc::ptr_eq(
        failed.chord_slot_identity(seed.key).unwrap(),
        &slot.identity
    ));
    assert_eq!(music.measures[seed.measure].timing.expected, Some(exact));
    assert!(Arc::ptr_eq(
        &music.measures[seed.measure].timing.slots[0].identity,
        &slot.identity
    ));
    assert_eq!(music.chords.as_ptr(), chord_storage);
    assert!(
        music
            .measures
            .iter()
            .zip(&geometries)
            .all(|(m, g)| Arc::ptr_eq(m.geometry.as_ref().unwrap(), g))
    );
    assert!(music.measures[mi].stack_tuplets.contains(&key));
    std::fs::write("/tmp/real-tuplets-native-observed.json", serde_json::to_vec_pretty(&json!({"factories":factories,"lookup":observation,"cleanup":removed,"actual_stack_and_chord_owners_retained":true,"frozen_failure_exact_7_64_voice_slot_expected_rollback":true,"manual_and_linked_positive_and_six_source_comparison_complete":false})).unwrap()).unwrap();
}
