// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::sync::Arc;

#[test]
fn empty_real_header_preserves_staff_part_measure_and_head_chords() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-null-header/image-observed.json"
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
    let before = owner.graph.snapshot();
    let staffs = owner.staffs.clone();
    let parts = owner.parts.clone();
    let headers = owner.upstream_report["headers"].clone();
    let stems = before["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "StemInter")
        .map(|n| {
            let key: Key = serde_json::from_value(n["origin"].clone()).unwrap();
            let stem = owner.caches[&key.system]
                .registered_stem(&owner.graph, key.local)
                .unwrap();
            (key, stem)
        })
        .collect::<Vec<_>>();
    let music = continue_measures(owner).unwrap();
    let after = music.owner.graph.snapshot();
    assert_eq!(music.owner.upstream_report["headers"], headers);
    for (before, after) in staffs.iter().zip(&music.owner.staffs) {
        assert!(Arc::ptr_eq(before, after));
    }
    for (before, after) in parts.iter().zip(&music.owner.parts) {
        assert!(Arc::ptr_eq(before, after));
    }
    for (key, stem) in stems {
        assert!(Arc::ptr_eq(
            &stem,
            &music.owner.caches[&key.system]
                .registered_stem(&music.owner.graph, key.local)
                .unwrap()
        ));
    }
    for edge in before["relations"].as_array().unwrap() {
        assert_eq!(
            after["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == edge["id"]),
            Some(edge)
        );
    }
    for head in before["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "HeadInter")
    {
        let retained = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == head["id"])
            .unwrap();
        for field in ["origin", "bounds", "shape", "pitch", "staff_id", "manual"] {
            assert_eq!(retained[field], head[field], "{field}");
        }
    }

    let mut observed = Vec::new();
    let mut null_staffs = 0;
    for system in source["systems"].as_array().unwrap() {
        let sid = system["system"].as_u64().unwrap();
        for expected in system["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["clef"].is_null())
        {
            null_staffs += 1;
            assert_eq!(expected["same_HEADERS_Staff_Part_Header_objects"], true);
            assert_eq!(expected["header_inter_count"], 0);
            let staff_id = expected["staff"].as_u64().unwrap();
            let staff = staffs
                .iter()
                .find(|s| s.system == sid && s.id == staff_id)
                .unwrap();
            let part = parts
                .iter()
                .find(|p| p.system == sid && p.id == expected["part"])
                .unwrap();
            assert!(part.staffs.iter().any(|s| Arc::ptr_eq(s, staff)));
            let header = headers["staffs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|h| h["system_id"] == sid && h["staff_id"] == staff_id)
                .unwrap();
            for field in ["start", "stop", "clef", "key", "time"] {
                assert_eq!(header[field], expected[field], "{field}");
            }
            assert!(!after["inters"].as_array().unwrap().iter().any(|n| {
                n["origin"]["system"] == sid
                    && n["staff_id"] == staff_id
                    && matches!(
                        n["kind"].as_str(),
                        Some("ClefInter" | "KeyInter" | "TimePairInter" | "TimeWholeInter")
                    )
            }));
            let mut actual = Vec::new();
            for chord in &music.chords {
                let members = chord
                    .members
                    .iter()
                    .filter(|h| h["staff_id"] == staff_id && chord.key.system == sid)
                    .map(|h| {
                        json!({"bounds":h["bounds"],"shape":h["shape"],
                            "pitch":h["pitch"].as_f64().unwrap().round_ties_even() as i64,
                            "manual":h["manual"].as_bool().unwrap_or(false)})
                    })
                    .collect::<Vec<_>>();
                if members.is_empty() {
                    continue;
                }
                let measure = &music.measures[chord.measure];
                let geometry = measure.geometry.as_ref().unwrap();
                assert!(Arc::ptr_eq(&geometry.part, part));
                assert!(music.measure_layout.parts.iter().any(|layout| {
                    Arc::ptr_eq(&layout.part, part)
                        && layout.measures.iter().any(|m| Arc::ptr_eq(m, geometry))
                }));
                assert!(measure.timing.expected.is_none());
                assert!(measure.timing.excess.is_none());
                assert!(measure.timing.slots.is_empty());
                let (body, removed) = music.owner.graph.retained_inter(chord.key).unwrap();
                assert!(!removed);
                actual.push(json!({"bounds":body["bounds"],"measure_index":measure.index,
                    "stack_index":geometry.stack.index,"manual":body["manual"].as_bool().unwrap_or(false),
                    "members":members}));
            }
            let mut wanted = expected["chords"].as_array().unwrap().clone();
            for chord in &mut wanted {
                for head in chord["members"].as_array_mut().unwrap() {
                    assert_eq!(head["same_actual_owners"], true);
                    assert!(head["clef_before"].is_null());
                    head.as_object_mut().unwrap().remove("same_actual_owners");
                    head.as_object_mut().unwrap().remove("clef_before");
                }
            }
            actual.sort_by_key(Value::to_string);
            wanted.sort_by_key(Value::to_string);
            observed.push(json!({"system":sid,"staff":staff_id,"header":header,"chords":actual}));
            std::fs::write(
                "/tmp/real-null-header-native-observed.json",
                serde_json::to_vec_pretty(&observed).unwrap(),
            )
            .unwrap();
            assert_eq!(
                actual, wanted,
                "actual empty-header Staff chords and owners"
            );
        }
    }
    assert_eq!(null_staffs, 1);
}
