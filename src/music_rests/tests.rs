// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::cell::RefCell;

thread_local! {
    static FRAMES: RefCell<Option<Vec<(Value, Graph)>>> = const { RefCell::new(None) };
}
pub(super) fn record(system: u64, before: Value, after: Value, actual: &Graph) {
    FRAMES.with(|frames| {
        if let Some(frames) = &mut *frames.borrow_mut() {
            frames.push((
                json!({"system":system,"before":before,"after":after}),
                actual.clone(),
            ));
        }
    });
}
fn origin(body: &Value) -> Key {
    serde_json::from_value(body["origin"].clone()).unwrap()
}
fn nodes(snapshot: &Value, system: u64, kind: &str) -> Vec<Value> {
    snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["origin"]["system"] == system && n["kind"] == kind)
        .cloned()
        .collect()
}
fn source_frame<'a>(source: &'a Value, system: u64, stage: &str) -> &'a Value {
    source["frames"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["system"] == system && f["stage"] == stage)
        .unwrap()
}

#[test]
fn real_rest_factory_keeps_lower_staff_owners_before_allocating_chords() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-rests/image-observed.json"
    ))
    .unwrap();
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/real-rest-page/der-winter-gray.png"
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
    let old_chords = music
        .chords
        .iter()
        .map(|c| (c.key, c.id, c.measure))
        .collect::<Vec<_>>();
    FRAMES.with(|frames| *frames.borrow_mut() = Some(Vec::new()));
    let mut music = crate::music_symbols::build_symbols(music).unwrap();
    let captured = FRAMES.with(|frames| frames.borrow_mut().take().unwrap());
    let (frames, actual_factory_graphs): (Vec<_>, Vec<_>) = captured.into_iter().unzip();
    assert_eq!(frames.len(), 2);
    let mut observed = Vec::new();
    for frame in &frames {
        let system = frame["system"].as_u64().unwrap();
        let before = &frame["before"];
        let after = &frame["after"];
        let rests = nodes(before, system, "RestInter");
        let expected = source_frame(&source, system, "after_buildSymbols_before_rest_chords");
        let linked = source_frame(&source, system, "after_buildRestChords_before_lateChecks");
        assert_eq!(rests.len(), expected["rests"].as_array().unwrap().len());
        assert!(nodes(before, system, "RestChordInter").is_empty());
        assert_eq!(nodes(after, system, "RestInter"), rests);
        assert_eq!(nodes(after, system, "RestChordInter").len(), rests.len());
        for rest in &rests {
            let original = expected["rests"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["bounds"] == rest["bounds"] && r["shape"] == rest["shape"])
                .unwrap();
            assert_eq!(rest["staff_id"], original["staff"]);
            assert_eq!(rest["manual"], original["manual"]);
            assert!(
                (rest["grade"].as_f64().unwrap() - original["grade"].as_f64().unwrap()).abs()
                    < 1e-10
            );
            assert!(
                (rest["pitch"].as_f64().unwrap() - original["pitch"].as_f64().unwrap()).abs()
                    < 1e-10
            );
            assert!(original["chord_id"].is_null());
            let glyph = &music
                .owner
                .prepared
                .registry
                .get(rest["glyph_id"].as_u64().unwrap() as usize)
                .unwrap()
                .glyph;
            assert_eq!(
                json!(beam_dispatch::centroid(glyph).unwrap().map(|v| v as f64)),
                original["centroid"]
            );
            let held_staff = music
                .owner
                .staffs
                .iter()
                .find(|s| s.system == system && rest["staff_id"] == s.id)
                .unwrap();
            let chord = music
                .chords
                .iter()
                .find(|c| c.kind == ChordKind::Rest && c.members[0]["id"] == rest["id"])
                .unwrap();
            let measure = &music.measures[chord.measure];
            let layout = measure.geometry.as_ref().unwrap();
            assert!(
                layout
                    .part
                    .staffs
                    .iter()
                    .any(|s| Arc::ptr_eq(s, held_staff))
            );
            assert!(Arc::ptr_eq(layout, &measures[chord.measure]));
            let original_rest = linked["rests"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["object"] == original["object"])
                .unwrap();
            let original_chord = linked["rest_chords"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["object"] == original_rest["chord_object"])
                .unwrap();
            let actual = after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == chord.id)
                .unwrap();
            assert_eq!(actual["staff_id"], original_chord["staff"]);
            assert_eq!(actual["bounds"], original_chord["bounds"]);
            assert_eq!(actual["part_id"], original_chord["part"]);
            assert_eq!(
                measure.index,
                original_chord["measure"]["measure_index"].as_u64().unwrap() as usize
            );
            assert!(
                (actual["grade"].as_f64().unwrap() - original_chord["grade"].as_f64().unwrap())
                    .abs()
                    < 1e-10
            );
            for field in ["voice", "slot_id", "time_offset"] {
                assert!(actual[field].is_null());
            }
            let relation = after["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| {
                    r["kind"] == "Containment"
                        && r["source"] == chord.id
                        && r["target"] == rest["id"]
                })
                .unwrap();
            let final_graph = music.owner.graph.snapshot();
            assert!(
                final_graph["relations"]
                    .as_array()
                    .unwrap()
                    .contains(relation)
            );
            assert_eq!(
                music.owner.graph.id(origin(rest)),
                rest["id"].as_u64().map(|v| v as usize)
            );
            observed.push(json!({"system":system,"rest":rest,"chord":actual,"relation":relation,"actual_staff_and_measure_retained":true,
                "factory_chord_null":true,"measure_index":measure.index}));
        }
    }
    assert_eq!(observed.len(), 12);
    assert_eq!(
        observed
            .iter()
            .filter(|o| matches!(o["rest"]["staff_id"].as_u64(), Some(3 | 6)))
            .count(),
        3
    );
    for (key, id, measure) in old_chords {
        assert!(
            music
                .chords
                .iter()
                .any(|c| c.key == key && c.id == id && c.measure == measure)
        );
    }
    // A later invalid Rest must not publish the first newly allocated Chord,
    // lose an existing exact clock, or replace retained manual status.
    let first = &frames[0];
    // This is the retained Graph clone from the actual factory boundary,
    // preserving its objects and relations before any RestChord was added.
    let mut graph = actual_factory_graphs.into_iter().next().unwrap();
    let system = first["system"].as_u64().unwrap();
    let rests = nodes(&graph.snapshot(), system, "RestInter");
    let mut manual = rests[0].clone();
    manual["manual"] = json!(true);
    graph.vertex(origin(&manual), manual, false).unwrap();
    let mut invalid = rests.last().unwrap().clone();
    invalid["bounds"] = Value::Null;
    graph.vertex(origin(&invalid), invalid, false).unwrap();
    let head = music
        .chords
        .iter()
        .find(|c| c.kind == ChordKind::Head)
        .unwrap();
    let (head_key, measure_index) = (head.key, head.measure);
    let exact = crate::music_time::Fraction::new(7, 64).unwrap();
    let expected = crate::music_time::Fraction::new(3, 4).unwrap();
    let mut head_body = graph.retained_inter(head_key).unwrap().0.clone();
    head_body["manual"] = json!(true);
    head_body["time_offset"] = json!(crate::music_time::format_fraction(exact));
    graph.vertex(head_key, head_body, false).unwrap();
    music.measures[measure_index].timing.expected = Some(expected);
    assert_eq!(
        graph.actual_chord_time_offset(head_key).unwrap(),
        Some(exact)
    );
    let duration =
        crate::music_time::Fraction::parse(&graph.chord_duration(head_key).unwrap().unwrap())
            .unwrap();
    let expected_end = exact.plus(duration).unwrap().to_string();
    assert_eq!(
        graph.chord_end_time(head_key).unwrap(),
        Some(expected_end.clone())
    );
    let before = graph.snapshot();
    let chord_storage = music.chords.as_ptr();
    let chord_state = music.chords.iter().map(|c| json!({"key":c.key,"id":c.id,"members":c.members,"measure":c.measure,"bounds":c.bounds})).collect::<Vec<_>>();
    let measure_expected = music
        .measures
        .iter()
        .map(|m| m.timing.expected)
        .collect::<Vec<_>>();
    let voice_storage = music.voices.as_ptr();
    assert!(music.voices.is_empty());
    assert_eq!(
        add_rest_chords(&mut graph, &music, system).err().unwrap(),
        "music_rest_bounds_required"
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(
        graph.actual_chord_time_offset(head_key).unwrap(),
        Some(exact)
    );
    assert_eq!(graph.chord_end_time(head_key).unwrap(), Some(expected_end));
    assert_eq!(music.chords.as_ptr(), chord_storage);
    assert_eq!(music.chords.iter().map(|c| json!({"key":c.key,"id":c.id,"members":c.members,"measure":c.measure,"bounds":c.bounds})).collect::<Vec<_>>(), chord_state);
    assert_eq!(
        music
            .measures
            .iter()
            .map(|m| m.timing.expected)
            .collect::<Vec<_>>(),
        measure_expected
    );
    assert_eq!(
        music.measures[measure_index].timing.expected,
        Some(expected)
    );
    assert!(
        music
            .measures
            .iter()
            .zip(&measures)
            .all(|(m, held)| Arc::ptr_eq(m.geometry.as_ref().unwrap(), held))
    );
    assert!(music.voices.is_empty());
    assert_eq!(music.voices.as_ptr(), voice_storage);
    let output = std::env::var_os("AUDIVERIS_REAL_RESTS_OBSERVATIONS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/tmp/real-rest-owner-native-observed.json".into());
    std::fs::write(output, serde_json::to_vec_pretty(&json!({"ordinary_image_symbols_rest_count":12,
        "lower_Staff_3_and_6_rest_count":3,"source_factory_then_chord_boundaries_match":true,
        "original_only_no_final_export_quality_claim":true,"late_failure_graph_manual_and_exact_clock_rollback":true,"actual_Measure_expected_retained":true,"actual_Chord_vector_and_Measure_Arcs_retained":true,"Voice_count":0,"observed":observed})).unwrap()).unwrap();
}
