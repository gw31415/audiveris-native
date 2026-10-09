// SPDX-License-Identifier: AGPL-3.0-or-later
//! Existing AnLaura owner only; this helper never starts recognition.
use super::*;
use crate::music_measures::OwnedMusic;

fn describe(graph: &Graph, key: Key) -> Value {
    let (body, removed) = graph.retained_inter(key).unwrap();
    assert!(!removed);
    let (staff, part) = graph.filler_owner(key).unwrap();
    json!({"kind":body["kind"],"bounds":body["bounds"],"staff":staff.id,"part":part.id})
}
pub(super) fn observe_peer_memberships(graph: &Graph, layout: &Layout, system: u64) -> Value {
    json!(
        layout
            .parts
            .iter()
            .filter(|part| part.part.system == system)
            .flat_map(|part| {
                part.measures.iter().map(move |measure| {
                    let owners = graph.clef_ownership.as_ref().unwrap();
                    let row = owners
                        .measures
                        .iter()
                        .find(|row| Arc::ptr_eq(&row.measure, measure))
                        .unwrap();
                    assert!(Arc::ptr_eq(&row.measure.part, &part.part));
                    assert!(
                        owners
                            .parts
                            .iter()
                            .any(|held| Arc::ptr_eq(held, &part.part))
                    );
                    assert!(owners.stacks.iter().any(|held| Arc::ptr_eq(
                        &held.stack,
                        &measure.stack
                    )
                        && held.measures.iter().any(|peer| Arc::ptr_eq(peer, measure))));
                    json!({"system":system,"part":part.part.id,"measure":measure.index,
                "stack":measure.stack.index,
                "clefs":row.members.iter().map(|key|describe(graph,*key)).collect::<Vec<_>>(),
                "keys":row.keys.iter().map(|key|describe(graph,*key)).collect::<Vec<_>>()})
                })
            })
            .collect::<Vec<_>>()
    )
}
fn memberships(graph: &Graph, music: &OwnedMusic) -> Value {
    json!(music.measure_layout.parts.iter().flat_map(|part| {
        part.measures.iter().map(move |measure| {
            let row = graph.clef_ownership.as_ref().unwrap().measures.iter()
                .find(|row|Arc::ptr_eq(&row.measure,measure)).unwrap();
            assert!(Arc::ptr_eq(&row.measure.part, &part.part));
            assert!(music.measures.iter().any(|held| held.geometry.as_ref()
                .is_some_and(|g| Arc::ptr_eq(g,measure))));
            assert!(music.measure_layout.stacks.iter().any(|stack|
                Arc::ptr_eq(&stack.stack,&measure.stack)
                && stack.measures.iter().any(|held| Arc::ptr_eq(held,measure))));
            json!({"system":part.part.system,"part":part.part.id,"measure":measure.index,
                "stack":measure.stack.index,
                "clefs":graph.owned_measure_clefs(measure).unwrap().iter().map(|k|describe(graph,*k)).collect::<Vec<_>>(),
                "keys":graph.owned_measure_keys(measure).unwrap().iter().map(|k|describe(graph,*k)).collect::<Vec<_>>()})
        })
    }).collect::<Vec<_>>())
}
fn chord_caches(graph: &Graph) -> Value {
    json!(
        graph
            .vertices
            .iter()
            .filter_map(|v| v
                .chord_cache
                .as_ref()
                .map(|cache| json!({"key":v.key,"cache":cache.snapshot()})))
            .collect::<Vec<_>>()
    )
}
pub(super) fn assert_real_owned_filler(music: &OwnedMusic) -> Value {
    let mut graph = music.owner.graph.clone();
    let before = memberships(&graph, music);
    assert!(
        before
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["clefs"].as_array().unwrap().is_empty()
                && row["keys"].as_array().unwrap().is_empty())
    );
    let graph_before = graph.snapshot();
    let caches_before = chord_caches(&graph);
    let mut stages = Vec::new();
    let mut actual_systems = Vec::new();
    for system in &music.owner.systems {
        assert!(graph.require_header_filler_boundary(system.id).is_err());
        let keys = graph
            .vertices
            .iter()
            .filter(|v| {
                !v.removed
                    && v.key.system == system.id
                    && matches!(v.data["kind"].as_str(), Some("ClefInter" | "KeyInter"))
            })
            .map(|v| v.key)
            .collect::<Vec<_>>();
        assert!(
            keys.iter()
                .all(|key| graph.native_header_source(*key).is_ok())
        );
        let selected = music
            .measure_layout
            .stacks
            .iter()
            .filter(|stack| stack.stack.system == system.id)
            .map(|stack| {
                json!({"stack":stack.stack.index,"kept":keys.iter().filter(|key|
                graph.filter_filler_inter(&music.measure_layout,stack,**key).unwrap().is_some())
                .map(|key|describe(&graph,*key)).collect::<Vec<_>>()})
            })
            .collect::<Vec<_>>();
        actual_systems.push(json!({"system":system.id,"filling":keys.iter()
            .map(|key|describe(&graph,*key)).collect::<Vec<_>>(),"stacks":selected}));
        stages.push(
            graph
                .fill_owned_clefs_and_keys(&music.measure_layout, system.id)
                .unwrap(),
        );
        graph.require_header_filler_boundary(system.id).unwrap();
    }
    let after = memberships(&graph, music);
    assert_eq!(graph.snapshot(), graph_before);
    assert_eq!(chord_caches(&graph), caches_before);
    for system in &music.owner.systems {
        graph
            .fill_owned_clefs_and_keys(&music.measure_layout, system.id)
            .unwrap();
    }
    assert_eq!(memberships(&graph, music), after);
    assert_eq!(graph.snapshot(), graph_before);
    assert_eq!(chord_caches(&graph), caches_before);
    let observation = json!({"before":before,"systems":actual_systems,"stages":stages,"after":after,
        "second_call_same_membership":true,"graph_data_and_cached_Chords_unchanged":true,
        "actual_Measure_Part_Stack_Arcs_retained":true,"source_fresh_first_add_callbacks_observed":false,
        "source_Time_and_global_SIG_order_claimed":false,"nonheader_Clef_admission":false});
    std::fs::write(
        "/tmp/real-header-filler-native-control.json",
        serde_json::to_vec_pretty(&observation).unwrap(),
    )
    .unwrap();
    let original: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/official-measure-filler-headers/an-laura-observed.json"
    ))
    .unwrap();
    assert_eq!(observation["systems"], original["systems"]);
    assert_eq!(after, original["final_memberships"]);
    assert_eq!(music.owner.graph.snapshot(), graph_before);
    observation
}
pub(super) fn assert_completed_header_filler(music: &OwnedMusic, control: &Value) {
    for system in &music.owner.systems {
        music
            .owner
            .graph
            .require_header_filler_boundary(system.id)
            .unwrap();
    }
    let actual = memberships(&music.owner.graph, music);
    assert_eq!(actual, control["after"]);
    std::fs::write(
        "/tmp/real-header-filler-native-LINKS.json",
        serde_json::to_vec_pretty(&json!({"actual_memberships":actual,
            "actual_Measure_Part_Stack_Arcs_retained":true,"quality_accepted":false}))
        .unwrap(),
    )
    .unwrap();
}
