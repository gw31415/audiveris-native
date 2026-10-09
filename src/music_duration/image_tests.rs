// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{music_measures::OwnedMusic, music_time::format_fraction};
use serde_json::json;

pub(crate) fn assert_real_duration_reads(mut music: OwnedMusic, preparation: Value) {
    let untouched = music.owner.graph.clone();
    let flags_before = music.owner.graph.snapshot();
    let mut competition = preparation;
    let selection = crate::music_flags::reduce_exclusions(&mut music.owner.graph).unwrap();
    competition
        .as_object_mut()
        .unwrap()
        .extend(selection.as_object().unwrap().clone());
    assert_eq!(
        competition["removed_conflicting_flags"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let constraints = untouched.assert_flag_competition_identity(music.chords[0].key, &competition);
    let flags_after = music.owner.graph.snapshot();
    let stages: Value = serde_json::from_str(include_str!(
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
    let source_flags = |name: &str| {
        stages["stages"].as_array().unwrap().iter().find(|s|s["stage"] == name).unwrap()["systems"].as_array().unwrap().iter().flat_map(|sys|sys["flags"].as_array().unwrap().iter().map(move |f|json!({"system":sys["system"],"shape":f["shape"],"bounds":f["bounds"]}))).collect::<Vec<_>>()
    };
    let sorted = |mut rows: Vec<Value>| {
        rows.sort_by_key(Value::to_string);
        rows
    };
    assert_eq!(
        sorted(flags(&flags_before)),
        sorted(source_flags("WEAK_PURGED"))
    );
    assert_eq!(
        sorted(flags(&flags_after)),
        sorted(source_flags("FLAG_EXCLUSIONS_REDUCED"))
    );
    assert_eq!(sorted(flags(&flags_after)), sorted(source_flags("LINKS")));
    let mut source_pairs = Vec::new();
    for sys in stages["stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["stage"] == "FLAG_OVERLAPS")
        .unwrap()["systems"]
        .as_array()
        .unwrap()
    {
        for e in sys["relations"].as_array().unwrap().iter().filter(|e| {
            e["kind"] == "Exclusion"
                && e["source_kind"] == "FlagInter"
                && e["target_kind"] == "FlagInter"
        }) {
            source_pairs.push(json!({"system":sys["system"],"bounds":sorted(vec![e["source_bounds"].clone(),e["target_bounds"].clone()])}));
        }
    }
    let native_pairs = competition["inserted_flag_overlap_pairs"].as_array().unwrap().iter().map(|pair| {
        let nodes=flags_before["inters"].as_array().unwrap();
        let a=nodes.iter().find(|n|n["id"] == pair[0]).unwrap();let b=nodes.iter().find(|n|n["id"] == pair[1]).unwrap();
        json!({"system":a["origin"]["system"],"bounds":sorted(vec![a["bounds"].clone(),b["bounds"].clone()])})
    }).collect::<Vec<_>>();
    assert_eq!(sorted(native_pairs), sorted(source_pairs));
    for before in flags_before["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "HeadChordInter")
    {
        assert_eq!(
            flags_after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == before["id"])
                .unwrap(),
            before
        );
    }
    crate::music_dots::reduce_consistencies(&mut music.owner.graph, &music.chords).unwrap();
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-chord-duration/image-observed.json"
    ))
    .unwrap();
    let before = music.owner.graph.snapshot();
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for system in source["systems"].as_array().unwrap() {
        for original in system["chords"].as_array().unwrap() {
            let chord = music
                .chords
                .iter()
                .find(|c| {
                    c.key.system == system["system"].as_u64().unwrap()
                        && json!(c.bounds) == original["bounds"]
                })
                .unwrap();
            let measure = &music.measures[chord.measure];
            music
                .owner
                .graph
                .warm_retained_chord_beams(chord.key)
                .unwrap();
            let snapshot = music.owner.graph.snapshot();
            let node = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == chord.id)
                .unwrap();
            let result = live_exact(&snapshot, chord.id);
            let mut row = json!({"system":system["system"],"bounds":chord.bounds,
                "part":measure.part,"measure_index":measure.index,"staff":node["staff_id"],
                "beam_count":crate::music_voices::chord_beams(&snapshot,chord.id).unwrap().len(),
                "dots_number":node["dots_number"].as_u64().unwrap_or(0)});
            match result {
                Ok(duration) => {
                    row["duration"] = json!(duration.intrinsic.map(format_fraction));
                    row["flags_number"] = json!(
                        duration.beams_or_flags - row["beam_count"].as_u64().unwrap() as usize
                    );
                }
                Err(error) => row["error"] = json!(error),
            }
            actual.push(row);
            expected.push(
                json!({"system":system["system"],"bounds":original["bounds"],
                "part":original["part"],"measure_index":original["measure_index"],
                "staff":original["staff"],"beam_count":original["beam_count"],
                "flags_number":original["flags_number"],"dots_number":original["dots_number"],
                "duration":original["duration"]}),
            );
        }
    }
    std::fs::write(
        "/tmp/real-chord-duration-native-observed.json",
        serde_json::to_vec_pretty(
            &json!({"actual":actual,"expected":expected,"flag_competition":competition,
                "constraints":constraints,"original_weak_purged_flag_membership_matches":true,
                "original_overlap_pairs_and_reduced_LINKS_flag_membership_match":true,
                "actual_Chord_bodies_and_clocks_retained":true}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(actual.len(), 67);
    assert_eq!(actual, expected);
    let after = music.owner.graph.snapshot();
    assert_eq!(before["inters"], after["inters"]);
    assert_eq!(before["relations"], after["relations"]);
    assert_eq!(before["tombstones"], after["tombstones"]);
    crate::music_voices::image_tests::assert_real_voice_families(&music);
    crate::music_rookie_mapping::image_tests::assert_real_voice_distances(&music);
    crate::music_measure_slots::image_tests::assert_real_measure_slots(&music);
    let frame = crate::music_xml::stack_rhythm::image_tests::assert_real_measure_rhythm(&music);
    music.owner.graph = frame.graph;
    music.measures.clone_from(&frame.measures);
    music.voices = frame.voices;
    crate::music_voices::shared_stack::image_tests::assert_real_shared_slots(&mut music);
}
