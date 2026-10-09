// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
fn focus(node: &Value) -> bool {
    matches!(
        node["kind"].as_str(),
        Some(
            "HeadInter"
                | "StemInter"
                | "BeamInter"
                | "BeamHookInter"
                | "SmallBeamInter"
                | "FlagInter"
        )
    )
}
fn normalize_edges(snapshot: &Value) -> Vec<Value> {
    let nodes = snapshot["inters"].as_array().unwrap();
    snapshot["relations"].as_array().unwrap().iter().filter_map(|edge|{
  if !matches!(edge["kind"].as_str(),Some("Exclusion"|"NoExclusion")){return None;}
  let a=nodes.iter().find(|n|n["id"]==edge["source"]).unwrap();let b=nodes.iter().find(|n|n["id"]==edge["target"]).unwrap();
  (focus(a)!=focus(b)).then(||json!({"kind":edge["kind"],"source_kind":a["kind"],"source_bounds":a["bounds"],"target_kind":b["kind"],"target_bounds":b["bounds"],"manual":edge["manual"]==true}))
 }).collect()
}
impl Graph {
    pub(crate) fn assert_real_context_projection(music: crate::music_measures::OwnedMusic) {
        let source: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-context-projection/image-observed.json"
        ))
        .unwrap();
        let expected = source["systems"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| {
                assert_eq!(s["before"], s["without_external_non_support_edges"]);
                assert_eq!(s["before"], s["restored"]);
                assert_eq!(s["same_relation_objects_restored"], true);
                s["external_non_support_edges"].as_array().unwrap().clone()
            })
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), 2);
        let music = crate::music_symbols::continue_symbols(music).unwrap();
        let mut graph = music.owner.graph.clone();
        let before = graph.snapshot();
        let external = normalize_edges(&before);
        let matched = external
            .iter()
            .filter(|e| e["target_kind"] == "RestInter")
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(matched, expected);
        let unaccepted_time_factory_edges = external
            .iter()
            .filter(|e| e["target_kind"] == "TimeWholeInter")
            .cloned()
            .collect::<Vec<_>>();
        graph.contextualize_head_stem_flags().unwrap();
        let after = graph.snapshot();
        assert_eq!(before["relations"], after["relations"]);
        let actual = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| (n["id"].as_u64().unwrap(), n))
            .collect::<BTreeMap<_, _>>();
        for old in before["inters"].as_array().unwrap() {
            let mut old = old.clone();
            let mut new = actual[&old["id"].as_u64().unwrap()].clone();
            old.as_object_mut().unwrap().remove("contextual_grade");
            new.as_object_mut().unwrap().remove("contextual_grade");
            assert_eq!(old, new);
        }
        let ids = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| focus(n))
            .map(|n| n["id"].as_u64().unwrap() as usize)
            .collect::<BTreeSet<_>>();
        let external_ids = graph
            .edges
            .iter()
            .filter(|e| {
                !e.removed
                    && matches!(e.key.kind.as_str(), "Exclusion" | "NoExclusion")
                    && ids.contains(&e.key.source) != ids.contains(&e.key.target)
            })
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let mut without = graph.clone();
        for id in &external_ids {
            without.edges[*id - 1].removed = true;
        }
        without.contextualize_head_stem_flags().unwrap();
        let projected = without.snapshot();
        let context = |snapshot: &Value| {
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| focus(n))
                .map(|n| (n["id"].as_u64().unwrap(), n["contextual_grade"].clone()))
                .collect::<BTreeMap<_, _>>()
        };
        assert_eq!(context(&after), context(&projected));
        let mut invalid = graph.clone();
        let edge = &mut invalid.edges[external_ids[0] - 1];
        edge.key.kind = "UnsupportedNonzeroSupport".into();
        edge.data["kind"] = json!("UnsupportedNonzeroSupport");
        edge.data["source_ratio"] = json!(2.0);
        edge.data["target_ratio"] = json!(2.0);
        let before = invalid.snapshot();
        assert_eq!(
            invalid.contextualize_head_stem_flags().unwrap_err(),
            "unsupported_head_stem_flag_external_relation"
        );
        assert_eq!(invalid.snapshot(), before);
        std::fs::write("/tmp/real-context-projection-native-observed.json",serde_json::to_vec_pretty(&json!({"external_non_support_edges":external,"matched_original_Flag_Rest_edges":matched,"unaccepted_time_factory_edges":unaccepted_time_factory_edges,"focus_context_count":context(&after).len(),"context_identical_without_external_non_support_edges":true,"actual_relations_retained":true,"unsupported_external_support_rollback":true})).unwrap()).unwrap();
    }
}
