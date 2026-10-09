// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
impl Graph {
    pub(crate) fn assert_actual_image_dot_stacks(music: &crate::music_measures::OwnedMusic) {
        let original: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-dot-shared-stacks/image-observed.json"
        ))
        .unwrap();
        let owner = &music.owner;
        let before = owner.graph.snapshot();
        let mut graph = owner.graph.clone();
        graph
            .bind_owned_dot_measures(
                &music.measures,
                &owner.staffs,
                &owner.staff_areas,
                &owner.parts,
            )
            .unwrap();
        let bound = graph.dot_ownership.as_ref().unwrap();
        assert_eq!(bound.measures.len(), 8);
        for m in &bound.measures {
            let g = m.geometry.as_ref().unwrap();
            assert!(
                music
                    .measures
                    .iter()
                    .any(|m| m.geometry.as_ref().is_some_and(|held| Arc::ptr_eq(held, g)))
            );
        }
        let state = |g: &Graph, id: usize| {
            let v = &g.vertices[id - 1];
            json!({"cached_part":v.cached_part,"members":g.dot_ownership.as_ref().unwrap().measures.iter()
                .filter(|m| m.dots.contains(&id)).map(|m|json!({"stack_index":m.index,"part":m.part})).collect::<Vec<_>>()})
        };
        let mut queries = 0;
        let mut controls = 0;
        for sys in original["systems"].as_array().unwrap() {
            let system = sys["system"].as_u64().unwrap();
            for query in sys["queries"].as_array().unwrap() {
                let point = if query["point"].is_null() {
                    None
                } else {
                    Some(serde_json::from_value(query["point"].clone()).unwrap())
                };
                let selected = graph.stack_at_dot_point(system, point).unwrap();
                assert_eq!(
                    json!(
                        selected.map(|i| graph.dot_ownership.as_ref().unwrap().measures[i].index)
                    ),
                    query["stack_index"],
                    "system{system} {point:?}"
                );
                queries += 1;
            }
            for case in sys["controls"].as_array().unwrap() {
                let point: [i64; 2] = serde_json::from_value(case["point"].clone()).unwrap();
                let key = Key {
                    system,
                    phase: Phase::Symbol,
                    local: 100_000 + controls,
                };
                let data = json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","grade":0.8,"manual":false,"staff_id":case["staff"],"bounds":[point[0]-1,point[1]-1,2,2],"abnormal":false});
                // Constructor has no SIG or membership; added performs the lookup.
                assert!(case["plain"]["cached_part"].is_null());
                assert!(case["plain"]["members"].as_array().unwrap().is_empty());
                let id = graph.vertex(key, data, false).unwrap();
                assert_eq!(state(&graph, id), case["added"]);
                graph.populate_dot_measures().unwrap();
                assert_eq!(state(&graph, id), case["populated"]);
                graph.remove_music_inter(key).unwrap();
                assert_eq!(state(&graph, id), case["removed"]);
                controls += 1;
            }
        }
        assert_eq!((queries, controls), (52, 24));
        for vertex in before["inters"].as_array().unwrap() {
            let after = graph.snapshot();
            assert_eq!(
                after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == vertex["id"])
                    .unwrap(),
                vertex
            );
        }
        assert_eq!(graph.snapshot()["relations"], before["relations"]);
        let bound = graph.snapshot();
        assert!(
            graph
                .bind_owned_dot_measures(
                    &music.measures,
                    &owner.staffs,
                    &owner.staff_areas[..owner.staff_areas.len() - 1],
                    &owner.parts
                )
                .is_err()
        );
        assert_eq!(graph.snapshot(), bound);
        let mut bad = music.measures.clone();
        bad.last_mut().unwrap().geometry = None;
        assert!(
            graph
                .bind_owned_dot_measures(&bad, &owner.staffs, &owner.staff_areas, &owner.parts)
                .is_err()
        );
        assert_eq!(graph.snapshot(), bound);
        assert_eq!(owner.graph.snapshot(), before);
    }
}
