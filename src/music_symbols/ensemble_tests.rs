// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(crate) fn assert_real_ensemble_visits(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-cleaner/image-observed.json"
    ))
    .unwrap();
    let graph = &music.owner.graph;
    let before = graph.snapshot();
    let visits = graph.symbol_cleaner_visits().unwrap();
    assert!(visits.iter().all(|(_, weak)| !weak));
    assert!(
        source["optionals"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["glyphs"].as_array().unwrap().is_empty())
    );
    let mut weak_child = 0;
    let mut heads = 0;
    let mut grades = Vec::new();
    for system in source["systems"].as_array().unwrap() {
        for expected in system["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "HeadInter")
        {
            let matching = visits
                .iter()
                .filter_map(|(k, weak)| {
                    let (body, _) = graph.retained_inter(*k).unwrap();
                    (Some(k.system) == system["system"].as_u64()
                        && body["kind"] == "HeadInter"
                        && body["bounds"] == expected["bounds"])
                        .then_some((*k, body, *weak))
                })
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1);
            let (key, body, weak) = matching[0];
            let grade = body["contextual_grade"]
                .as_f64()
                .or(body["grade"].as_f64())
                .unwrap();
            grades.push(
                serde_json::json!({"system":key.system,"bounds":body["bounds"],
                "native_best":grade,"original_best":expected["best_grade"],
                "absolute_difference":(grade-expected["best_grade"].as_f64().unwrap()).abs()}),
            );
            assert_eq!(grade >= 0.6, expected["can_hide"].as_bool().unwrap());
            assert!(!weak);
            let parent = music
                .chords
                .iter()
                .find(|c| graph.has_relation(c.key, key, "Containment"))
                .unwrap();
            let (chord, _) = graph.retained_inter(parent.key).unwrap();
            assert!(chord["grade"].as_f64().unwrap() >= 0.4);
            assert_eq!(expected["ensemble"], "HeadChordInter");
            if grade < 0.6 {
                weak_child += 1;
            }
            heads += 1;
        }
    }
    assert_eq!(heads, 94);
    assert_eq!(weak_child, 1);
    for chord in &music.chords {
        let member_keys = visits
            .iter()
            .filter(|(key, _)| graph.has_relation(chord.key, *key, "Containment"))
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        let native = member_keys
            .iter()
            .map(|k| graph.retained_inter(*k).unwrap().0)
            .collect::<Vec<_>>();
        assert_eq!(
            native,
            crate::head_chord_geometry::ordered(&native).unwrap()
        );
    }
    assert_eq!(graph.snapshot(), before);
    std::fs::write(
        "/tmp/real-symbol-ensemble-head-grades.json",
        serde_json::to_vec_pretty(&grades).unwrap(),
    )
    .unwrap();
}
