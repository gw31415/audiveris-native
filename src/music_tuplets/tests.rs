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

#[test]
fn real_image_linked_six_callbacks_and_later_weak_purge_match_original() {
    let factory: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-linked-explicit-tuplets/factory-observed.json"
    ))
    .unwrap();
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-linked-explicit-tuplets/support-inputs-observed.json"
    ))
    .unwrap();
    let stages: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-linked-explicit-tuplets/stages-observed.json"
    ))
    .unwrap();
    let source_members: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-ornaments/der-winter-chord-confidence-observed.json"
    ))
    .unwrap();
    let image = GrayImage::decode(include_bytes!(
        "../../fixtures/real-rest-page/der-winter-gray.png"
    ))
    .unwrap();
    let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
    let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
    let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
    let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
    let owner = crate::music_links::finalize_stems(owner).unwrap();
    let music = crate::music_measures::continue_measures(owner).unwrap();
    crate::music_arpeggiato::tests::begin_factory_observation();
    let mut music = crate::music_symbols::continue_symbols(music).unwrap();
    crate::music_arpeggiato::tests::assert_der_winter_null_factory(&music);
    std::fs::write("/tmp/real-rest-symbol-candidates.json",serde_json::to_vec(&json!({"symbol_glyphs":music.owner.prepared.report["symbol_glyphs"],"classifier_trials":music.owner.prepared.report["classifier_trials"]})).unwrap()).unwrap();
    let actual_sig = music.owner.graph.snapshot();
    let native_inputs = music.chords.iter().map(|chord| {
        let (node,removed)=music.owner.graph.retained_inter(chord.key).unwrap();
        let members=actual_sig["relations"].as_array().unwrap().iter().filter(|r|r["kind"]=="Containment"&&r["source"]==chord.id).map(|r| {
            let note=actual_sig["inters"].as_array().unwrap().iter().find(|n|n["id"]==r["target"]).unwrap();
            json!({"id":note["id"],"bounds":note["bounds"],"shape":note["shape"],"staff_id":note["staff_id"],"grade":note["grade"],"contextual_grade":note["contextual_grade"]})
        }).collect::<Vec<_>>();
        json!({"id":chord.id,"key":chord.key,"bounds":node["bounds"],"cached_grade":node["grade"],"removed":removed,"members_at_SYMBOLS":members})
    }).collect::<Vec<_>>();
    std::fs::write("/tmp/real-linked-tuplet-native-inputs.json",serde_json::to_vec_pretty(&json!({"stage":"after ordinary native SYMBOLS before assertions","actual_chords":native_inputs,"original_lookup":source["rows"],"qualification":"Cached Chord confidence is retained from membership invalidation; current member confidences are separate observations, not replacement inputs."})).unwrap()).unwrap();
    let ornaments = crate::music_ornaments::assert_der_winter_owned_lifecycle(&music);
    let keys = music
        .owner
        .systems
        .iter()
        .flat_map(|s| music.owner.graph.live_inter_keys(s.id, "TupletInter"))
        .collect::<Vec<_>>();
    assert_eq!(keys.len(), 1);
    let key = keys[0];
    let source_factory = &factory["factory_candidates"][0];
    let node = music.owner.graph.retained_inter(key).unwrap().0;
    assert_eq!(node["bounds"], source_factory["bounds"]);
    assert_eq!(node["shape"], "TUPLET_SIX");
    assert!(
        (node["grade"].as_f64().unwrap() - source_factory["result"]["grade"].as_f64().unwrap())
            .abs()
            < 1e-10
    );
    assert!(node["staff_id"].is_null());
    assert!(node["part_id"].is_null());
    let mi = music
        .measures
        .iter()
        .position(|m| m.stack_tuplets.contains(&key))
        .unwrap();
    let stack = music.measures[mi].geometry.as_ref().unwrap().stack.clone();
    let mut graph = music.owner.graph.clone();
    for c in &music.chords {
        if !graph.retained_inter(c.key).unwrap().1 {
            graph.count_owned_augmentation(c.key).unwrap();
        }
    }
    let (links, lookup_row) = lookup(&music, &mut graph, key, &stack).unwrap();
    let reference = &source["rows"][0];
    assert_eq!(
        links.len(),
        reference["actual_lookup_links"].as_array().unwrap().len()
    );
    assert_eq!(lookup_row["status"], "OK");
    assert_eq!(lookup_row["target_staff"], reference["target_staff"]);
    let around = lookup_row["around"].as_array().unwrap();
    let expected_around = reference["around"].as_array().unwrap();
    assert_eq!(around.len(), expected_around.len());
    let mut confidence_residuals = Vec::new();
    for (native, original) in around.iter().zip(expected_around) {
        assert_eq!(native["bounds"], original["bounds"]);
        for field in ["head", "tail"] {
            assert_eq!(
                native[field],
                json!(
                    original[field]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|n| n.as_f64().unwrap() as i64)
                        .collect::<Vec<_>>()
                )
            );
        }
        let c = music.chords.iter().find(|c| native["id"] == c.id).unwrap();
        let member_source = source_members["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|frame| {
                frame["stage"] == "after_ordinary_SYMBOLS" && frame["system"] == c.key.system
            })
            .flat_map(|frame| frame["head_chords"].as_array().unwrap())
            .find(|chord| chord["bounds"] == original["bounds"])
            .unwrap();
        assert_eq!(member_source["raw_grade"], original["intrinsic_grade"]);
        let mut native_member_geometry =
            native_inputs.iter().find(|row| row["id"] == c.id).unwrap()["members_at_SYMBOLS"]
                .as_array()
                .unwrap()
                .iter()
                .map(|member| json!({"bounds":member["bounds"],"shape":member["shape"],"staff":member["staff_id"]}))
                .collect::<Vec<_>>();
        let mut original_member_geometry = member_source["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|member| json!({"bounds":member["bounds"],"shape":member["shape"],"staff":member["staff"]}))
            .collect::<Vec<_>>();
        native_member_geometry.sort_by_key(Value::to_string);
        original_member_geometry.sort_by_key(Value::to_string);
        assert_eq!(native_member_geometry, original_member_geometry);
        assert_eq!(
            member_source["members"].as_array().unwrap().len(),
            original["members"].as_array().unwrap().len()
        );
        assert_eq!(
            graph
                .sans_tuplet_chord_duration(c.key, false)
                .unwrap()
                .unwrap()
                .to_string(),
            original["duration_sans_tuplet"].as_str().unwrap()
        );
        let native_grade = graph.retained_inter(c.key).unwrap().0["grade"]
            .as_f64()
            .unwrap();
        let original_grade = original["intrinsic_grade"].as_f64().unwrap();
        confidence_residuals.push(json!({"id":c.id,"bounds":native["bounds"],"native_cached_grade":native_grade,"original_cached_grade":original_grade,"absolute_delta":(native_grade-original_grade).abs()}));
    }
    let native_trace = lookup_row["collector"].as_array().unwrap();
    let source_trace = reference["collector_replay"].as_array().unwrap();
    assert_eq!(native_trace.len(), source_trace.len());
    for (native, original) in native_trace.iter().zip(source_trace) {
        for field in ["base", "total", "expected_total", "status"] {
            assert_eq!(native[field], original["after"][field]);
        }
    }
    let initial_frame = raw_callback_frame(&music, &graph, key).unwrap();
    let chord_storage = music.chords.as_ptr();
    let geometry = music
        .measures
        .iter()
        .map(|m| m.geometry.clone().unwrap())
        .collect::<Vec<_>>();
    let (linked, mut plan) = link_and_remove(&music, &mut graph).unwrap();
    assert_eq!(linked["tuplets"][0]["removed"], false);
    let callbacks = linked["tuplets"][0]["additions"].as_array().unwrap();
    let events = stages["events"].as_array().unwrap();
    assert_eq!(callbacks.len(), 3);
    let mut relations = Vec::new();
    for (callback, original) in callbacks.iter().zip(&events[..3]) {
        assert_eq!(
            callback["after"]["tuplet"]["abnormal"],
            original["retained_tuplets"][0]["abnormal"]
        );
        assert!(callback["after"]["tuplet"]["staff_id"].is_null());
        assert!(callback["after"]["tuplet"]["part_id"].is_null());
        assert_eq!(callback["after"]["chords"], initial_frame["chords"]);
        assert_eq!(callback["after"]["expected"], initial_frame["expected"]);
        let eid = callback["relation_id"].as_u64().unwrap() as usize;
        let (edge, removed) = graph.retained_explicit_tuplet_relation(eid).unwrap();
        assert!(!removed);
        assert_eq!(edge["manual"], original["relation_manual"]);
        assert_eq!(edge["grade"], original["relation_grade"]);
        assert_eq!(edge["source_ratio"], 1.0);
        assert_eq!(edge["target_ratio"], 1.34);
        relations.push((eid, edge.clone()));
    }
    assert!(plan[mi].2.contains(&key));
    assert!(graph.retained_inter(key).unwrap().0["contextual_grade"].is_null());
    graph.trace_tuplet_removals();
    let context = contextualize_and_purge(&music, &mut graph, &mut plan).unwrap();
    let original_removed = &events.last().unwrap()["retained_tuplets"][0];
    let retained = graph.retained_inter(key).unwrap();
    assert!(retained.1);
    assert_eq!(retained.0["abnormal"], false);
    assert!(retained.0["staff_id"].is_null());
    assert!(retained.0["part_id"].is_null());
    let native_context = retained.0["contextual_grade"].as_f64().unwrap();
    let original_context = original_removed["contextual_grade"].as_f64().unwrap();
    let native_grade = retained.0["grade"].as_f64().unwrap();
    let original_grade = reference["tuplet_before"]["grade"].as_f64().unwrap();
    let mut native_partners = links
        .iter()
        .map(|key| {
            let id = graph.id(*key).unwrap();
            let row = confidence_residuals.iter().find(|r| r["id"] == id).unwrap();
            (
                row["native_cached_grade"].as_f64().unwrap(),
                row["original_cached_grade"].as_f64().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let coefficient = 1.34 - 1.0;
    let input_delta = native_partners
        .iter()
        .map(|(n, o)| (n - o).abs() * coefficient)
        .sum::<f64>();
    native_partners.sort_by(|a, b| b.0.total_cmp(&a.0));
    let native_support = native_partners
        .iter()
        .map(|(n, _)| n * coefficient)
        .sum::<f64>();
    native_partners.sort_by(|a, b| b.1.total_cmp(&a.1));
    let original_support = native_partners
        .iter()
        .map(|(_, o)| o * coefficient)
        .sum::<f64>();
    let equation = |g: f64, s: f64| (1.0 + s) * g / (1.0 + s * g);
    // For the source support equation, sensitivity to support is <=1/4;
    // sensitivity to intrinsic grade is <=1+s. Use this image's measured
    // input deltas, without importing another page's confidence tolerance.
    let propagated_bound = 0.25 * input_delta
        + (1.0 + native_support.max(original_support)) * (native_grade - original_grade).abs();
    let contextual_residual = json!({"native_contextual_grade":native_context,"original_contextual_grade":original_context,"absolute_delta":(native_context-original_context).abs(),"native_support":native_support,"original_support":original_support,"measured_support_input_delta_bound":input_delta,"propagated_context_bound":propagated_bound,"native_equation":equation(native_grade,native_support),"original_equation":equation(original_grade,original_support),"numeric_parity_claimed":false});
    std::fs::write("/tmp/real-linked-tuplet-confidence-observed.json",serde_json::to_vec_pretty(&json!({"cached_chord_residuals":confidence_residuals,"contextual_residual":contextual_residual})).unwrap()).unwrap();
    assert!((native_context - equation(native_grade, native_support)).abs() < 1e-12);
    assert!((original_context - equation(original_grade, original_support)).abs() < 1e-12);
    assert!((native_context - original_context).abs() <= propagated_bound + 1e-12);
    assert_eq!(native_context < 0.5, original_context < 0.5);
    assert!(
        plan.iter()
            .all(|(_, a, u)| !a.contains(&key) && !u.contains(&key))
    );
    let removals = graph.take_tuplet_removals();
    let finished = removals
        .iter()
        .filter(|r| r.phase == "finished")
        .collect::<Vec<_>>();
    assert_eq!(finished.len(), 3);
    for (index, (row, ((eid, edge), original))) in finished
        .iter()
        .zip(relations.iter().zip(&events[3..6]))
        .enumerate()
    {
        assert_eq!(row.edge, *eid);
        assert_eq!(
            row.state["registered"],
            original["retained_tuplets"][0]["in_actual_SIG"]
        );
        assert_eq!(
            row.state["removed"],
            original["retained_tuplets"][0]["removed"]
        );
        assert!(row.measure_tuplets.is_empty());
        assert!(row.stack_tuplets.is_empty());
        assert!(row.sig_attached);
        assert_eq!(
            row.graph.tuplet_registration(key)["registered"],
            original["retained_tuplets"][0]["in_actual_SIG"]
        );
        assert_eq!(
            row.graph.retained_tuplet_body(key)["abnormal"],
            original["retained_tuplets"][0]["abnormal"]
        );
        let mut live = 0;
        for (position, (relation, body)) in relations.iter().enumerate() {
            let (held, removed) = row
                .graph
                .retained_explicit_tuplet_relation(*relation)
                .unwrap();
            assert_eq!(held, body);
            assert_eq!(removed, position <= index);
            live += usize::from(!removed);
        }
        assert_eq!(live, 2 - index);
        assert_eq!(
            live,
            original["retained_tuplets"][0]["chord_relations"]
                .as_array()
                .unwrap()
                .len()
        );
        let frame = raw_callback_frame(&music, &row.graph, key).unwrap();
        assert_eq!(frame["chords"], initial_frame["chords"]);
        assert_eq!(frame["expected"], initial_frame["expected"]);
        let (held, removed) = row.graph.retained_explicit_tuplet_relation(*eid).unwrap();
        assert!(removed);
        assert_eq!(held, edge);
    }
    for (eid, body) in &relations {
        let (held, removed) = graph.retained_explicit_tuplet_relation(*eid).unwrap();
        assert!(removed);
        assert_eq!(held, body);
    }

    // This declared late-failure control keeps initialized actual owners. The
    // independent original image observations above have NULL pre-LINKS clocks.
    // The frozen survivor fails after contextualization but before any weak
    // deletion, so this does not cover rollback after earlier container removal.
    let mut failed = music.owner.graph.clone();
    for c in &music.chords {
        if !failed.retained_inter(c.key).unwrap().1 {
            failed.count_owned_augmentation(c.key).unwrap();
        }
    }
    let seed = music
        .chords
        .iter()
        .find(|c| c.key == links[0])
        .unwrap()
        .clone();
    let exact = Fraction::new(7, 64).unwrap();
    let vi =
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
    let mut seed_body = failed.retained_inter(seed.key).unwrap().0.clone();
    seed_body["manual"] = json!(true);
    seed_body["time_offset"] = json!(crate::music_time::format_fraction(exact));
    failed.vertex(seed.key, seed_body, false).unwrap();
    music.measures[seed.measure].timing.expected = Some(exact);
    music.measures[seed.measure].timing.slots.push(slot.clone());
    let clocked = raw_callback_frame(&music, &failed, key).unwrap();
    let (failed_links, mut failed_plan) = link_and_remove(&music, &mut failed).unwrap();
    for callback in failed_links["tuplets"][0]["additions"].as_array().unwrap() {
        assert_eq!(callback["after"]["chords"], clocked["chords"]);
        assert_eq!(callback["after"]["expected"], clocked["expected"]);
    }
    let mut frozen = failed.retained_inter(key).unwrap().0.clone();
    frozen["frozen"] = json!(true);
    failed.vertex(key, frozen, false).unwrap();
    let before_failure = failed.snapshot();
    let before_plan = failed_plan.clone();
    assert_eq!(
        contextualize_and_purge(&music, &mut failed, &mut failed_plan).unwrap_err(),
        "music_tuplet_surviving_linked_rhythm_pending"
    );
    assert_eq!(failed.snapshot(), before_failure);
    assert_eq!(failed_plan, before_plan);
    assert_eq!(
        failed.actual_chord_time_offset(seed.key).unwrap(),
        Some(exact)
    );
    assert!(failed.chord_has_voice(seed.key, &music.voices[vi]));
    assert!(Arc::ptr_eq(
        failed.chord_slot_identity(seed.key).unwrap(),
        &slot.identity
    ));
    assert_eq!(music.measures[seed.measure].timing.expected, Some(exact));
    assert_eq!(music.chords.as_ptr(), chord_storage);
    assert!(
        music
            .measures
            .iter()
            .zip(&geometry)
            .all(|(m, g)| Arc::ptr_eq(m.geometry.as_ref().unwrap(), g))
    );
    assert!(music.measures[mi].stack_tuplets.contains(&key));
    std::fs::write("/tmp/real-linked-tuplets-native-observed.json",serde_json::to_vec_pretty(&json!({"ornaments":ornaments,"lookup":lookup_row,"linked":linked,"context":context,"cached_chord_residuals":confidence_residuals,"contextual_residual":contextual_residual,"same_relation_bodies_after_remove":true,"actual_chord_and_stack_owners_retained":true,"late_survivor_failure_exact_7_64_voice_slot_expected_rollback":true,"positive_surviving_rhythm_complete":false})).unwrap()).unwrap();
}
