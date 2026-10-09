// SPDX-License-Identifier: AGPL-3.0-or-later
//! Declared scalar lifecycle controls; saved-book raw Part was not observed.
use super::*;
use crate::{curves::Curve, sheet_skew::SheetSkew};
use serde_json::json;

fn owners() -> (Graph, Layout, Arc<PartGeometry>) {
    let points = (0..5)
        .map(|i| vec![[10., 100. + i as f64 * 11.], [500., 100. + i as f64 * 11.]])
        .collect::<Vec<_>>();
    let staff = Arc::new(StaffGeometry {
        id: 1,
        system: 1,
        specific_interline: 11,
        abscissae: [10, 500],
        lines: points
            .iter()
            .map(|p| Arc::new(Curve::interpolate(p.clone()).unwrap()))
            .collect(),
    });
    let part = Arc::new(PartGeometry {
        id: 1,
        system: 1,
        staffs: vec![staff.clone()],
        bounds: [10, 100, 491, 45],
    });
    let geometry = json!({"systems":[{"id":1,"coordinates":[10,100,491,144,44],"staff_ids":[1]}],
        "staffs":[{"id":1,"system_id":1,"left":10,"right":500,
            "lines":points.iter().map(|p|json!({"points":p})).collect::<Vec<_>>()}]});
    let areas = crate::system_area_geometry::build(&geometry, 512, 200, 11)
        .unwrap()
        .staff_areas
        .into_iter()
        .map(Arc::new)
        .collect::<Vec<_>>();
    let mut graph = Graph::default();
    for (i, x) in [10, 250, 499].into_iter().enumerate() {
        graph
            .vertex(
                Key {
                    system: 1,
                    phase: Phase::Grid,
                    local: i + 1,
                },
                json!({"kind":"BarlineInter",
            "staff_id":1,"bounds":[x,100,1,45],"shape":"THIN_BARLINE",
            "grade":0.8,"manual":true,"frozen":true}),
                false,
            )
            .unwrap();
    }
    let layout = crate::music_measure_layout::build(
        &mut graph,
        std::slice::from_ref(&staff),
        std::slice::from_ref(&part),
        SheetSkew::new(0.).unwrap(),
        11,
    )
    .unwrap();
    graph
        .bind_clef_layout(
            &layout,
            std::slice::from_ref(&staff),
            &areas,
            std::slice::from_ref(&part),
        )
        .unwrap();
    (graph, layout, part)
}
fn linked_flag(graph: &mut Graph, x: i64, first_staff: Option<u64>) -> (Key, Key, Key, usize) {
    linked_flag_at(graph, 1, x, first_staff)
}
fn linked_flag_at(
    graph: &mut Graph,
    local: usize,
    x: i64,
    first_staff: Option<u64>,
) -> (Key, Key, Key, usize) {
    let stem = Key {
        system: 1,
        phase: Phase::Stem,
        local,
    };
    graph
        .vertex(
            stem,
            json!({"kind":"StemInter","bounds":[x,100,1,40],"grade":0.8}),
            false,
        )
        .unwrap();
    let mut heads = Vec::new();
    for (i, staff) in [first_staff, Some(1)].into_iter().enumerate() {
        let head = Key {
            system: 1,
            phase: Phase::Head,
            local: 2 * (local - 1) + i + 1,
        };
        graph
            .vertex(
                head,
                json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK",
            "staff_id":staff,"bounds":[x,110+i as i64*11,6,4],"grade":0.8}),
                false,
            )
            .unwrap();
        graph
            .relations(
                1,
                Phase::Symbol,
                vec![(
                    head,
                    stem,
                    json!({"kind":"HeadStemRelation",
            "grade":1.,"consistency":1.,"source_ratio":5.,"target_ratio":11.}),
                )],
            )
            .unwrap();
        heads.push(head);
    }
    let flag = Key {
        system: 1,
        phase: Phase::Symbol,
        local,
    };
    graph
        .vertex(
            flag,
            json!({"kind":"FlagInter","shape":"FLAG_1_DOWN",
        "bounds":[x,110,11,25],"staff_id":null,"part_id":null,"value":1,
        "grade":0.29,"frozen":false,"manual":true}),
            false,
        )
        .unwrap();
    graph.fresh_flag_added(flag).unwrap();
    assert_eq!(graph.retained_inter(flag).unwrap().0["abnormal"], true);
    assert!(graph.owned_flag_cached_part(flag).unwrap().is_none());
    assert!(
        graph
            .apply_owned_flag_stem(
                flag,
                stem,
                json!({"kind":"FlagStemRelation",
        "grade":0.543,"source_ratio":2.629,"target_ratio":2.629,"manual":true})
            )
            .unwrap()
    );
    assert_eq!(graph.retained_inter(flag).unwrap().0["abnormal"], false);
    assert!(graph.owned_flag_cached_part(flag).unwrap().is_none());
    let edge = graph
        .incident_edges(graph.id(flag).unwrap())
        .into_iter()
        .find(|eid| graph.edges[*eid - 1].key.kind == "FlagStemRelation")
        .unwrap();
    (flag, stem, heads[0], edge)
}

#[test]
fn flag_added_link_then_remove_materializes_part_only_at_actual_remove_lookup() {
    let (mut graph, layout, part) = owners();
    let (flag, stem, head, edge) = linked_flag(&mut graph, 100, Some(1));
    for measure in &layout.parts[0].measures {
        assert!(graph.owned_measure_flags(measure).unwrap().is_empty());
    }
    let original_edge = graph.edges[edge - 1].data.clone();
    assert!(
        !graph
            .apply_owned_flag_stem(
                flag,
                stem,
                json!({"kind":"FlagStemRelation",
        "grade":0.1,"manual":false})
            )
            .unwrap()
    );
    assert_eq!(graph.edges[edge - 1].data, original_edge);
    let fid = graph.id(flag).unwrap();
    let hid = graph.id(head).unwrap();
    assert_eq!(graph.vertices[hid - 1].cached_part, None);
    let mut events = Vec::new();
    graph
        .remove_owned_flag(flag, |g, event| {
            assert!(g.retained_inter(flag)?.0["staff_id"].is_null());
            assert!(Arc::ptr_eq(
                &g.owned_flag_cached_part(flag)?.unwrap(),
                &part
            ));
            assert_eq!(g.vertices[hid - 1].cached_part, Some(part.id));
            assert_eq!(g.retained_inter(flag)?.0["abnormal"], false);
            let v = &g.vertices[fid - 1];
            match event.stage {
                FlagRemovalStage::ContainerLookupComplete => assert!(!v.removing && !v.removed),
                FlagRemovalStage::EdgeRemoved => {
                    assert!(v.removing && !v.removed);
                    assert_eq!(event.edge, Some(edge));
                    assert_eq!(g.edges[edge - 1].data, original_edge);
                }
                FlagRemovalStage::VertexRemoved => assert!(!v.removing && v.removed),
            }
            for measure in &layout.parts[0].measures {
                assert!(g.owned_measure_flags(measure)?.is_empty());
            }
            events.push(event.stage);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        events,
        vec![
            FlagRemovalStage::ContainerLookupComplete,
            FlagRemovalStage::EdgeRemoved,
            FlagRemovalStage::VertexRemoved
        ]
    );
}

#[test]
fn flag_part_returns_first_actual_head_null_without_skipping_to_the_second() {
    let (mut graph, _, _) = owners();
    let (flag, _, _, _) = linked_flag(&mut graph, 100, None);
    assert!(graph.owned_flag_part(flag).unwrap().is_none());
    assert!(graph.owned_flag_cached_part(flag).unwrap().is_none());
    assert!(graph.retained_inter(flag).unwrap().0["staff_id"].is_null());
}

#[test]
fn late_flag_callback_failure_restores_both_flag_and_head_part_caches() {
    let (mut graph, _, _) = owners();
    let (flag, _, head, _) = linked_flag(&mut graph, 100, Some(1));
    let before = format!("{graph:?}");
    let error = graph
        .remove_owned_flag(flag, |g, event| {
            if event.stage == FlagRemovalStage::VertexRemoved {
                assert!(g.owned_flag_cached_part(flag)?.is_some());
                assert!(g.vertices[g.id(head).unwrap() - 1].cached_part.is_some());
                return Err("declared_late_flag_failure".into());
            }
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error, "declared_late_flag_failure");
    assert_eq!(format!("{graph:?}"), before);
}

#[test]
fn absent_actual_stack_does_not_eagerly_read_part_during_flag_removal() {
    let (mut graph, _, _) = owners();
    let (flag, _, head, _) = linked_flag(&mut graph, -30, Some(1));
    graph
        .remove_owned_flag(flag, |g, _| {
            assert!(g.owned_flag_cached_part(flag)?.is_none());
            assert_eq!(g.vertices[g.id(head).unwrap() - 1].cached_part, None);
            Ok(())
        })
        .unwrap();
}

fn unrelated_clock(
    graph: &mut Graph,
    layout: &Layout,
) -> (crate::music_measures::Chord, crate::music_measures::Measure) {
    use crate::{
        music_measures::{Chord, ChordKind, Measure},
        music_time::Fraction,
    };
    let geometry = layout.parts[0].measures[0].clone();
    let staff = geometry.part.staffs[0].clone();
    let head = Key {
        system: 1,
        phase: Phase::Head,
        local: 90,
    };
    graph
        .vertex(
            head,
            json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK",
        "staff_id":staff.id,"bounds":[180,120,6,4],"grade":0.8,"manual":true}),
            false,
        )
        .unwrap();
    let key = Key {
        system: 1,
        phase: Phase::Chord,
        local: 90,
    };
    let id = graph
        .vertex(
            key,
            json!({"kind":"HeadChordInter","bounds":[180,120,6,4],
        "staff_id":staff.id,"part_id":geometry.part.id,"measure_index":geometry.index,
        "time_offset":"7/64","manual":true,"frozen":false}),
            false,
        )
        .unwrap();
    graph
        .relations(
            1,
            Phase::Chord,
            vec![(key, head, json!({"kind":"Containment","manual":true}))],
        )
        .unwrap();
    let chord = Chord {
        kind: ChordKind::Head,
        key,
        id,
        stem: None,
        members: vec![graph.retained_inter(head).unwrap().0.clone()],
        measure: 0,
        bounds: [180, 120, 6, 4],
    };
    let [left, right] = geometry.abscissae(&staff).unwrap();
    let mut measure = Measure {
        system: 1,
        part: geometry.part.id,
        staff: staff.id,
        index: geometry.index,
        left,
        right,
        geometry: Some(geometry),
        timing: Default::default(),
        tuplets: vec![],
        stack_tuplets: vec![],
    };
    measure.timing.expected = Some(Fraction::new(3, 4).unwrap());
    assert_eq!(graph.chord_end_time(key).unwrap(), Some("23/64".into()));
    (chord, measure)
}

#[test]
fn weak_caller_reports_actual_lifecycle_and_retains_unrelated_exact_chord_clock() {
    let (mut graph, layout, part) = owners();
    let (flag, _, head, edge) = linked_flag(&mut graph, 100, Some(1));
    let (chord, measure) = unrelated_clock(&mut graph, &layout);
    let original_chord = graph.retained_inter(chord.key).unwrap().0.clone();
    let original_relation = graph.edges[edge - 1].data.clone();
    let registered = graph
        .clef_ownership
        .as_ref()
        .unwrap()
        .flag_added_keys
        .clone();
    let mut observed = 0;
    let result = crate::music_flags::contextualize_observing(&mut graph, |g, event| {
        assert_eq!(event.key, flag);
        assert!(g.retained_inter(flag)?.0["staff_id"].is_null());
        assert_eq!(g.retained_inter(flag)?.0["manual"], true);
        assert_eq!(g.retained_inter(chord.key)?.0, &original_chord);
        assert_eq!(g.chord_end_time(chord.key)?, Some("23/64".into()));
        assert!(Arc::ptr_eq(
            &g.owned_flag_cached_part(flag)?.unwrap(),
            &part
        ));
        assert_eq!(
            g.vertices[g.id(head).unwrap() - 1].cached_part,
            Some(part.id)
        );
        assert_eq!(g.edges[edge - 1].data, original_relation);
        observed += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(observed, 3);
    assert_eq!(result["removed"], json!([graph.id(flag).unwrap()]));
    let events = result["lifecycle"].as_array().unwrap();
    assert_eq!(
        events
            .iter()
            .map(|e| e["stage"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["ContainerLookupComplete", "EdgeRemoved", "VertexRemoved"]
    );
    assert_eq!(events[1]["relation_id"], edge);
    assert!(
        events
            .iter()
            .all(|e| e["key"] == json!(flag) && e["inter_id"] == graph.id(flag).unwrap())
    );
    assert_eq!(
        graph.clef_ownership.as_ref().unwrap().flag_added_keys,
        registered
    );
    assert_eq!(graph.retained_inter(chord.key).unwrap().0, &original_chord);
    assert_eq!(
        graph.chord_end_time(chord.key).unwrap(),
        Some("23/64".into())
    );
    assert_eq!(
        measure.timing.expected,
        Some(crate::music_time::Fraction::new(3, 4).unwrap())
    );
    assert!(Arc::ptr_eq(
        measure.geometry.as_ref().unwrap(),
        &layout.parts[0].measures[0]
    ));
    assert!(Arc::ptr_eq(&measure.geometry.as_ref().unwrap().part, &part));
    assert_eq!(chord.members[0]["manual"], true);
    // This family pass has no external Chord/Measure/Voice mutation parameter.
    // The unrelated Chord getter is stable; a removed flag's own stem may
    // legitimately change that stem's uncached duration.
    let empty = crate::music_flags::contextualize(&mut graph).unwrap();
    assert_eq!(empty, json!({"flags":0,"removed":[],"lifecycle":[]}));
}

#[test]
fn second_flag_callback_failure_restores_earlier_removal_and_all_context_part_caches() {
    let (mut graph, layout, part) = owners();
    let (first, _, head1, edge1) = linked_flag_at(&mut graph, 1, 100, Some(1));
    let (second, _, head2, edge2) = linked_flag_at(&mut graph, 2, 300, Some(1));
    let (chord, measure) = unrelated_clock(&mut graph, &layout);
    let registered = graph
        .clef_ownership
        .as_ref()
        .unwrap()
        .flag_added_keys
        .clone();
    let before = format!("{graph:?}");
    let context_before = graph
        .vertices
        .iter()
        .map(|v| v.data["contextual_grade"].clone())
        .collect::<Vec<_>>();
    let mut late = false;
    let err = crate::music_flags::contextualize_observing(&mut graph, |g, event| {
        assert_eq!(g.chord_end_time(chord.key)?, Some("23/64".into()));
        if event.key == second && event.stage == FlagRemovalStage::VertexRemoved {
            assert!(g.retained_inter(first)?.1);
            assert!(g.retained_inter(second)?.1);
            assert!(g.edges[edge1 - 1].removed && g.edges[edge2 - 1].removed);
            for (flag, head) in [(first, head1), (second, head2)] {
                assert!(Arc::ptr_eq(
                    &g.owned_flag_cached_part(flag)?.unwrap(),
                    &part
                ));
                assert_eq!(
                    g.vertices[g.id(head).unwrap() - 1].cached_part,
                    Some(part.id)
                );
                assert!(g.retained_inter(flag)?.0["staff_id"].is_null());
            }
            assert_ne!(
                g.vertices
                    .iter()
                    .map(|v| v.data["contextual_grade"].clone())
                    .collect::<Vec<_>>(),
                context_before
            );
            late = true;
            return Err("declared_second_flag_vertex_failure".into());
        }
        Ok(())
    })
    .unwrap_err();
    assert!(late);
    assert_eq!(err, "declared_second_flag_vertex_failure");
    assert_eq!(format!("{graph:?}"), before);
    assert_eq!(
        graph.clef_ownership.as_ref().unwrap().flag_added_keys,
        registered
    );
    for (flag, head) in [(first, head1), (second, head2)] {
        assert!(!graph.retained_inter(flag).unwrap().1);
        assert!(graph.owned_flag_cached_part(flag).unwrap().is_none());
        assert_eq!(
            graph.vertices[graph.id(head).unwrap() - 1].cached_part,
            None
        );
    }
    assert_eq!(
        graph
            .vertices
            .iter()
            .map(|v| v.data["contextual_grade"].clone())
            .collect::<Vec<_>>(),
        context_before
    );
    assert_eq!(
        graph.chord_end_time(chord.key).unwrap(),
        Some("23/64".into())
    );
    assert_eq!(
        measure.timing.expected,
        Some(crate::music_time::Fraction::new(3, 4).unwrap())
    );
}

#[test]
fn exclusion_caller_keeps_incoming_before_outgoing_callback_order_and_manual_edge_identity() {
    let (mut graph, _, part) = owners();
    let (winner, _, _, _) = linked_flag_at(&mut graph, 1, 100, Some(1));
    let (loser, _, head, stem_edge) = linked_flag_at(&mut graph, 2, 110, Some(1));
    let (other, _, _, _) = linked_flag_at(&mut graph, 3, 120, Some(1));
    for (key, grade) in [(winner, 0.8), (other, 0.7)] {
        let mut body = graph.retained_inter(key).unwrap().0.clone();
        body["grade"] = json!(grade);
        graph.vertex(key, body, false).unwrap();
    }
    graph
        .relations(
            1,
            Phase::Symbol,
            vec![
                (
                    loser,
                    other,
                    json!({"kind":"Exclusion","cause":"OVERLAP","manual":true}),
                ),
                (
                    winner,
                    loser,
                    json!({"kind":"Exclusion","cause":"OVERLAP","manual":true}),
                ),
            ],
        )
        .unwrap();
    let lid = graph.id(loser).unwrap();
    let wid = graph.id(winner).unwrap();
    let oid = graph.id(other).unwrap();
    let outgoing = graph
        .edges
        .iter()
        .position(|e| e.key.source == lid && e.key.target == oid && e.key.kind == "Exclusion")
        .unwrap()
        + 1;
    let incoming = graph
        .edges
        .iter()
        .position(|e| e.key.source == wid && e.key.target == lid && e.key.kind == "Exclusion")
        .unwrap()
        + 1;
    assert!(outgoing < incoming);
    let bodies = [incoming, stem_edge, outgoing].map(|id| (id, graph.edges[id - 1].data.clone()));
    let result = crate::music_flags::reduce_exclusions(&mut graph).unwrap();
    assert_eq!(result["removed_conflicting_flags"], json!([lid]));
    let lifecycle = result["lifecycle"].as_array().unwrap();
    assert_eq!(
        lifecycle.first().unwrap()["stage"],
        "ContainerLookupComplete"
    );
    assert_eq!(lifecycle.last().unwrap()["stage"], "VertexRemoved");
    let removed_edges = lifecycle
        .iter()
        .filter(|e| e["stage"] == "EdgeRemoved")
        .map(|e| e["relation_id"].as_u64().unwrap() as usize)
        .collect::<Vec<_>>();
    assert_eq!(removed_edges, vec![incoming, stem_edge, outgoing]);
    for (id, body) in bodies {
        assert!(graph.edges[id - 1].removed);
        assert_eq!(graph.edges[id - 1].data, body);
        assert_eq!(body["manual"], true);
    }
    assert_eq!(graph.retained_inter(loser).unwrap().0["abnormal"], false);
    assert!(Arc::ptr_eq(
        &graph.owned_flag_cached_part(loser).unwrap().unwrap(),
        &part
    ));
    assert_eq!(
        graph.vertices[graph.id(head).unwrap() - 1].cached_part,
        Some(part.id)
    );
    for key in [winner, other] {
        assert!(!graph.retained_inter(key).unwrap().1);
        assert!(graph.retained_inter(key).unwrap().0["staff_id"].is_null());
        assert!(graph.owned_flag_cached_part(key).unwrap().is_none());
    }
    let mut replay_events = 0;
    graph
        .remove_owned_flag(loser, |_, _| {
            replay_events += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(replay_events, 0);
}

#[test]
fn registered_factory_flag_cannot_bypass_typed_removal() {
    let (mut graph, _, _) = owners();
    let (flag, _, head, _) = linked_flag(&mut graph, 100, Some(1));
    let before = format!("{graph:?}");
    assert!(graph.requires_owned_flag_removal(flag));
    assert_eq!(
        graph.remove_music_inter(flag).unwrap_err(),
        "music_removal_actual_Flag_owners_required"
    );
    assert_eq!(format!("{graph:?}"), before);
    assert!(graph.owned_flag_cached_part(flag).unwrap().is_none());
    assert_eq!(
        graph.vertices[graph.id(head).unwrap() - 1].cached_part,
        None
    );
    graph.remove_owned_flag(flag, |_, _| Ok(())).unwrap();
    // Idempotence precedes the bypass guard, as with a retired actual Rest.
    graph.remove_music_inter(flag).unwrap();
    assert!(graph.requires_owned_flag_removal(flag));
}
