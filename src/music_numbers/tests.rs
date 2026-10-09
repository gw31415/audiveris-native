// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::GrayImage;

#[test]
fn real_image_temporary_numbers_preserve_headers_and_remove_same_objects() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-numbers/image-observed.json"
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
    let music = crate::music_symbols::continue_symbols(music).unwrap();
    let initial = music.owner.graph.snapshot();
    let keys = music
        .owner
        .systems
        .iter()
        .flat_map(|s| music.owner.graph.live_inter_keys(s.id, "NumberInter"))
        .collect::<Vec<_>>();
    let mut observations = Vec::new();
    let mut expected = Vec::new();
    for system in source["SYMBOLS"].as_array().unwrap() {
        for n in system["numbers"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "NumberInter")
        {
            expected.push((system["system"].clone(), n.clone()));
        }
    }
    assert_eq!(keys.len(), expected.len());
    for key in &keys {
        let n = music.owner.graph.retained_inter(*key).unwrap().0;
        let original = &expected
            .iter()
            .find(|(system, e)| {
                *system == key.system && e["shape"] == n["shape"] && e["bounds"] == n["bounds"]
            })
            .unwrap()
            .1;
        for (native, field) in [
            ("staff_id", "staff"),
            ("grade", "grade"),
            ("manual", "manual"),
            ("abnormal", "abnormal"),
            ("frozen", "frozen"),
        ] {
            if field == "grade" {
                assert!(
                    (n[native].as_f64().unwrap() - original[field].as_f64().unwrap()).abs() < 1e-10
                );
            } else {
                assert_eq!(n[native], original[field]);
            }
        }
        let glyph = &music
            .owner
            .prepared
            .registry
            .get(n["glyph_id"].as_u64().unwrap() as usize)
            .unwrap()
            .glyph;
        let centroid = beam_dispatch::centroid(glyph).unwrap();
        assert_eq!(json!(centroid), original["centroid"]);
        let staff = music
            .owner
            .staffs
            .iter()
            .find(|s| s.system == key.system && n["staff_id"] == s.id)
            .unwrap();
        let pitch = music
            .owner
            .ledger_memberships
            .pitch_at(staff, centroid.map(|v| v as f64))
            .unwrap();
        assert!((pitch - original["centroid_pitch"].as_f64().unwrap()).abs() < 1e-10);
        let found = search(&music, &music.owner.graph, *key).unwrap();
        assert_eq!(found["search_links"], original["search_links"]);
        assert_eq!(
            found["center_inside_staff"],
            original["center_inside_staff"]
        );
        assert!(
            (found["center_pitch"].as_f64().unwrap() - original["center_pitch"].as_f64().unwrap())
                .abs()
                < 1e-10
        );
        observations
            .push(json!({"inter":n,"centroid":centroid,"centroid_pitch":pitch,"lookup":found}));
    }
    let mut staged = music.owner.graph.clone();
    let mut frozen = staged.retained_inter(keys[1]).unwrap().0.clone();
    frozen["frozen"] = json!(true);
    frozen["manual"] = json!(true);
    staged.vertex(keys[1], frozen, false).unwrap();
    let before_failure = staged.snapshot();
    assert_eq!(
        link_and_remove(&music, &mut staged).unwrap_err(),
        "music_removal_frozen_inter"
    );
    assert_eq!(staged.snapshot(), before_failure);
    let chord_storage = music.chords.as_ptr();
    let mut graph = music.owner.graph.clone();
    let removed = link_and_remove(&music, &mut graph).unwrap();
    for key in &keys {
        let (body, is_removed) = graph.retained_inter(*key).unwrap();
        assert!(is_removed);
        assert_eq!(body, music.owner.graph.retained_inter(*key).unwrap().0);
        assert_eq!(graph.id(*key), music.owner.graph.id(*key));
    }
    let after = graph.snapshot();
    for n in initial["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] != "NumberInter")
    {
        assert_eq!(
            Some(n),
            after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["id"] == n["id"])
        );
    }
    for e in initial["relations"].as_array().unwrap() {
        if !keys
            .iter()
            .any(|k| e["source"] == graph.id(*k).unwrap() || e["target"] == graph.id(*k).unwrap())
        {
            assert_eq!(
                Some(e),
                after["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|a| a["id"] == e["id"])
            );
        }
    }
    assert_eq!(music.chords.as_ptr(), chord_storage);
    assert_eq!(music.owner.graph.snapshot(), initial);
    let time_five_trials = music.owner.prepared.report["classifier_trials"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| {
            t["evaluations"]
                .as_array()
                .is_some_and(|e| e.iter().any(|e| e["shape"] == "TIME_FIVE"))
        })
        .cloned()
        .collect::<Vec<_>>();
    assert!(!time_five_trials.is_empty());
    for trial in &time_five_trials {
        assert!(!keys.iter().any(|key| {
            let n = music.owner.graph.retained_inter(*key).unwrap().0;
            n["glyph_id"] == trial["glyph_id"] && n["shape"] == "TIME_FIVE"
        }));
    }
    std::fs::write("/tmp/real-numbers-native-observed.json", serde_json::to_vec_pretty(&json!({"numbers":observations,"removed":removed,"late_frozen_failure_full_graph_rollback":true,"non_number_vertices_and_unrelated_relations_unchanged":true,"actual_chord_storage_unchanged":true,"time_five_factory_null_trials":time_five_trials})).unwrap()).unwrap();
}
