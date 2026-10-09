// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
impl Graph {
    pub(crate) fn assert_flag_competition_identity(
        &self,
        chord: Key,
        competition: &Value,
    ) -> Value {
        let source: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-flag-competition-manual/image-observed.json"
        ))
        .unwrap();
        let stage = |name: &str| {
            source["stages"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["stage"] == name)
                .unwrap()["declared_relations"]
                .as_array()
                .unwrap()
        };
        let declared = stage("MANUAL_DECLARED");
        let retired = stage("FLAG_EXCLUSIONS_REDUCED");
        assert_eq!(declared.len(), 2);
        assert_eq!(retired.len(), 2);
        for (a, b) in declared.iter().zip(retired) {
            assert_eq!(a["relation_identity_label"], b["relation_identity_label"]);
            assert_eq!(a["manual"], true);
            assert_eq!(a["present"], true);
            assert_eq!(b["manual"], true);
            assert_eq!(b["present"], false);
            assert_ne!(b["source_removed"], b["target_removed"]);
        }
        let mut manual = self.clone();
        let before = self.snapshot();
        for pair in competition["inserted_flag_overlap_pairs"]
            .as_array()
            .unwrap()
        {
            let mut keys = Vec::new();
            for id in pair.as_array().unwrap() {
                let node = before["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == *id)
                    .unwrap();
                let key: Key = serde_json::from_value(node["origin"].clone()).unwrap();
                let mut body = node.clone();
                body["manual"] = json!(true);
                manual.vertex(key, body, false).unwrap();
                keys.push(key);
            }
            manual
                .relations(
                    keys[0].system,
                    Phase::Symbol,
                    vec![(
                        keys[0],
                        keys[1],
                        json!({"kind":"Exclusion","cause":"OVERLAP","manual":true}),
                    )],
                )
                .unwrap();
        }
        let relations = manual
            .edges
            .iter()
            .filter(|e| !e.removed && e.key.kind == "Exclusion" && e.data["manual"] == true)
            .map(|e| (e.data["id"].as_u64().unwrap() as usize, e.data.clone()))
            .collect::<Vec<_>>();
        assert_eq!(relations.len(), 2);
        let mut chord_body = manual.retained_inter(chord).unwrap().0.clone();
        chord_body["manual"] = json!(true);
        chord_body["time_offset"] = json!({"num":7,"den":64});
        chord_body["expected_duration"] = json!({"num":3,"den":4});
        manual.vertex(chord, chord_body, false).unwrap();
        let checkpoint = manual.snapshot();
        let mut invalid = manual.clone();
        let last = competition["removed_conflicting_flags"]
            .as_array()
            .unwrap()
            .last()
            .unwrap();
        let node = checkpoint["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == *last)
            .unwrap();
        let key: Key = serde_json::from_value(node["origin"].clone()).unwrap();
        let mut frozen = node.clone();
        frozen["frozen"] = json!(true);
        invalid.vertex(key, frozen, false).unwrap();
        let failed = invalid.snapshot();
        assert_eq!(
            crate::music_flags::reduce_overlaps(&mut invalid).unwrap_err(),
            "flag_frozen_overlap_pending"
        );
        assert_eq!(invalid.snapshot(), failed);
        let result = crate::music_flags::reduce_overlaps(&mut manual).unwrap();
        assert_eq!(
            result["removed_conflicting_flags"],
            competition["removed_conflicting_flags"]
        );
        for (id, data) in relations {
            assert!(manual.edges[id - 1].removed);
            assert_eq!(manual.edges[id - 1].data, data);
        }
        for id in result["removed_conflicting_flags"].as_array().unwrap() {
            let node = checkpoint["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == *id)
                .unwrap();
            let key: Key = serde_json::from_value(node["origin"].clone()).unwrap();
            let (body, removed) = manual.retained_inter(key).unwrap();
            assert!(removed);
            assert_eq!(body["manual"], true);
        }
        let after = manual.snapshot();
        for node in checkpoint["inters"]
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
                    .find(|n| n["id"] == node["id"])
                    .unwrap(),
                node
            );
        }
        json!({"declared_manual_flag_and_relation_identity_retained":true,"frozen_overlap_preserves_full_graph":true,"actual_Chord_manual_7_64_clock_expected_3_4_preserved":true})
    }
}
