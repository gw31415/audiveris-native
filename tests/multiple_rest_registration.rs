// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    Run,
    beam_dispatch::Registry,
    multiple_rest_detection::{RestPlan, SerifPlan},
    multiple_rest_registration::RestGraph,
    sections::{Orientation, Section, SectionId},
    serif_peaks::Peak,
};
fn plan(id: usize) -> RestPlan {
    RestPlan {
        beam_id: id,
        staff_id: 7,
        median: [100., 153., 200., 153.],
        height: 8.,
        grade: 0.72,
        serifs: std::array::from_fn(|n| {
            let x = if n == 0 { 100 } else { 198 };
            SerifPlan {
                left: n == 0,
                peak: Peak {
                    start: x,
                    stop: x + 3,
                    top: 135,
                    bottom: 171,
                    impacts: [0.9; 6],
                    grade: 0.7,
                },
                median: [x as f64 + 2., 134.5, x as f64 + 2., 172.5],
                width: 4.,
            }
        }),
    }
}
fn sections() -> Vec<Section> {
    [100, 198]
        .into_iter()
        .enumerate()
        .map(|(id, x)| Section {
            id: SectionId(id as u32 + 1),
            orientation: Orientation::Vertical,
            first_pos: x,
            weight: 4 * 37,
            runs: (0..4)
                .map(|_| Run {
                    start: 135,
                    length: 37,
                })
                .collect(),
        })
        .collect()
}
#[test]
fn ordered_vertices_relations_and_canonical_glyph_identity() {
    let mut graph = RestGraph::default();
    let mut registry = Registry::default();
    graph
        .register(&[plan(1), plan(2)], &sections(), 16, &mut registry)
        .unwrap();
    assert_eq!(graph.rests.iter().map(|r| r.id).collect::<Vec<_>>(), [1, 4]);
    assert_eq!(
        graph
            .serifs
            .iter()
            .map(|s| (s.id, s.glyph_id))
            .collect::<Vec<_>>(),
        [(2, 1), (3, 2), (5, 1), (6, 2)]
    );
    assert_eq!(
        graph
            .relations
            .iter()
            .map(|r| (r.rest_id, r.serif_id, r.left))
            .collect::<Vec<_>>(),
        [(1, 2, true), (1, 3, false), (4, 5, true), (4, 6, false)]
    );
    assert_eq!(registry.len(), 2);
    assert!(registry.get(1).unwrap().groups.is_empty());
    assert!(
        graph
            .register(&[plan(1)], &sections(), 16, &mut registry)
            .is_err()
    );
    assert_eq!(graph.rests.len(), 2);
}
#[test]
fn absent_filament_rejects_before_graph_or_glyph_mutation() {
    let mut graph = RestGraph::default();
    let mut registry = Registry::default();
    assert!(graph.register(&[plan(1)], &[], 16, &mut registry).is_err());
    assert!(
        graph.rests.is_empty()
            && graph.serifs.is_empty()
            && graph.relations.is_empty()
            && registry.is_empty()
    );
}

#[test]
fn final_cleanup_updates_real_candidates_groups_and_exclusions() {
    use audiveris_native::{
        GrayImage, beam_candidates::SystemCandidates, beam_groups, beam_impacts::Impacts,
        beam_merge::MergeCandidate, beam_structure::BeamItem,
    };
    let image = GrayImage::new(300, 200, vec![0; 300 * 200]).unwrap();
    let mut registry = Registry::default();
    let mut state = SystemCandidates::empty(1);
    for y in [153., 163.] {
        state
            .register_merge(
                &image,
                &mut registry,
                &MergeCandidate {
                    item: BeamItem {
                        median: [100., y, 200., y],
                        height: 8.,
                    },
                    is_small: false,
                    impacts: Impacts {
                        values: [1.; 6],
                        grade: 0.8,
                        non_finite: false,
                    },
                },
            )
            .unwrap();
    }
    state.add_exclusion(1, 2).unwrap();
    let mut groups = beam_groups::populate_system(&state, 16).unwrap();
    let group_id = groups.groups[0].id;
    let mut graph = RestGraph::default();
    graph
        .register(&[plan(1)], &sections(), 16, &mut registry)
        .unwrap();
    let glyph_count = registry.len();
    assert!(state.active(1).is_some());
    graph.finish_cleanup(&mut state, &mut groups).unwrap();
    assert!(state.active(1).is_none());
    assert!(state.active(2).is_some());
    assert_eq!(state.inters().len(), 2);
    assert!(state.exclusions().is_empty());
    assert_eq!(groups.groups[0].members, [2]);
    assert_eq!(groups.groups[0].id, group_id);
    assert_eq!(groups.groups[0].bounds, state.active(2).unwrap().bounds);
    assert!(groups.supports.is_empty());
    assert_eq!(registry.len(), glyph_count);
}
