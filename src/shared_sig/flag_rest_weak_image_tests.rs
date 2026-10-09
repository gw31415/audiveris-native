// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
impl Graph {
    pub(crate) fn assert_real_flag_rest_weak(music: &crate::music_measures::OwnedMusic) {
        let automatic: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-flag-rest-weak/image-observed.json"
        ))
        .unwrap();
        let manual: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-flag-rest-manual/image-observed.json"
        ))
        .unwrap();
        let mut records = Vec::new();
        for (mode, source) in [(false, &automatic), (true, &manual)] {
            let mut graph = music.owner.graph.clone();
            crate::music_alters::contextualize_and_purge(&mut graph).unwrap();
            crate::music_flags::contextualize(&mut graph).unwrap();
            let cases = source["systems"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|s| s["before_weak_purge"].as_array().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(cases.len(), 2);
            let snap = graph.snapshot();
            let mut pairs = Vec::new();
            for c in &cases {
                let flag = snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "FlagInter" && n["bounds"] == c["flag"]["bounds"])
                    .unwrap();
                let rest = snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "RestInter" && n["bounds"] == c["rest"]["bounds"])
                    .unwrap();
                let fk: Key = serde_json::from_value(flag["origin"].clone()).unwrap();
                let rk: Key = serde_json::from_value(rest["origin"].clone()).unwrap();
                assert_eq!(flag["glyph_id"], rest["glyph_id"]);
                assert_eq!(flag["glyph"], rest["glyph"]);
                assert_eq!(c["same_glyph_object"], true);
                assert!(
                    (flag["contextual_grade"].as_f64().unwrap()
                        - c["flag"]["contextual_grade"].as_f64().unwrap())
                    .abs()
                        < 1e-12
                );
                let e = snap["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| {
                        e["kind"] == "Exclusion"
                            && e["source"] == flag["id"]
                            && e["target"] == rest["id"]
                    })
                    .unwrap();
                let eid = e["id"].as_u64().unwrap() as usize;
                if mode {
                    let mut data = rest.clone();
                    data["manual"] = json!(true);
                    graph.vertex(rk, data, false).unwrap();
                    let mut edge = e.clone();
                    edge["manual"] = json!(true);
                    graph
                        .relations(fk.system, Phase::Symbol, vec![(fk, rk, edge)])
                        .unwrap();
                }
                pairs.push((fk, rk, eid, c));
            }
            let before = graph.snapshot();
            let relations = pairs
                .iter()
                .map(|(_, _, id, _)| graph.edges[*id - 1].data.clone())
                .collect::<Vec<_>>();
            let mut invalid = graph.clone();
            let (fk, rk, _, _) = pairs[1];
            let mut rest = invalid.retained_inter(rk).unwrap().0.clone();
            rest["frozen"] = json!(true);
            invalid.vertex(rk, rest, false).unwrap();
            let chord = &music.chords[0];
            let mut body = invalid.retained_inter(chord.key).unwrap().0.clone();
            body["manual"] = json!(true);
            body["time_offset"] = json!({"num":7,"den":64});
            body["expected_duration"] = json!({"num":3,"den":4});
            invalid.vertex(chord.key, body, false).unwrap();
            let bad_before = invalid.snapshot();
            assert_eq!(
                crate::music_symbol_reduction::reduce(&mut invalid).unwrap_err(),
                "music_symbol_cross_kind_exclusion_pending"
            );
            assert_eq!(invalid.snapshot(), bad_before);
            assert_eq!(invalid.id(fk), graph.id(fk));
            let reduction = crate::music_symbol_reduction::reduce(&mut graph).unwrap();
            let after = graph.snapshot();
            for ((fk, rk, id, c), original_relation) in pairs.iter().zip(&relations) {
                let (flag, removed) = graph.retained_inter(*fk).unwrap();
                assert!(!removed);
                let (rest, removed) = graph.retained_inter(*rk).unwrap();
                assert!(removed);
                assert_eq!(rest["manual"] == true, mode);
                assert!(
                    (rest["contextual_grade"].as_f64().unwrap()
                        - c["rest"]["contextual_grade"].as_f64().unwrap())
                    .abs()
                        < 1e-12
                );
                assert!(graph.edges[*id - 1].removed);
                assert_eq!(&graph.edges[*id - 1].data, original_relation);
                assert_eq!(graph.edges[*id - 1].data["manual"] == true, mode);
                let old = before["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == flag["id"])
                    .unwrap();
                assert_eq!(flag, old);
            }
            for old in before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["kind"] == "HeadChordInter")
            {
                assert_eq!(
                    after["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == old["id"])
                        .unwrap(),
                    old
                );
            }
            records.push(json!({"declared_manual":mode,"actual_competing_pairs":2,"weak_rest_contextualized_before_removal":true,"Flag_bodies_and_actual_HeadChord_clocks_unchanged":true,"removed_manual_relation_ids_and_bodies_retained":true,"late_frozen_failure_preserves_full_graph":true,"reduction":reduction}));
        }
        std::fs::write("/tmp/real-flag-rest-weak-native-observed.json",serde_json::to_vec_pretty(&json!({"cases":records,"source_whole_SIG_weak_purge_not_full_LINKS":true,"real_image_quality_accepted":false})).unwrap()).unwrap();
    }
}
