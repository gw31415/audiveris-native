// SPDX-License-Identifier: AGPL-3.0-or-later
// Install as shared_sig/clef/filler/scalar_tests.rs with a cfg(test) module hook.
use super::*;
use crate::{beam_dispatch::Registry, curves::Curve, sheet_skew::SheetSkew};

struct ScalarFiller {
    graph: Graph,
    registry: Registry,
    layout: Layout,
    staff: Arc<StaffGeometry>,
    clef: Key,
    keys: [Key; 5],
}

fn scalar_filler() -> ScalarFiller {
    let points = (0..5)
        .map(|line| {
            vec![
                [10., 100. + line as f64 * 11.],
                [500., 100. + line as f64 * 11.],
            ]
        })
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
    let geometry = json!({
        "systems":[{"id":1,"coordinates":[10,100,491,144,44],"staff_ids":[1]}],
        "staffs":[{"id":1,"system_id":1,"left":10,"right":500,
            "lines":points.iter().map(|p|json!({"points":p})).collect::<Vec<_>>()}]
    });
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
                json!({"kind":"BarlineInter","staff_id":1,"bounds":[x,100,1,45],
                "shape":"THIN_BARLINE","grade":0.8,"manual":true,"frozen":true}),
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
    let mut registry = Registry::default();
    let glyph = json!({"bounds":[99,119,3,3],"orientation":"VERTICAL","runs":[[1,0,3]]});
    let gid = registry
        .register(&crate::beam_dispatch::glyph_from_json(&glyph).unwrap())
        .unwrap();
    let clef = Key {
        system: 1,
        phase: Phase::MusicHeader,
        local: 1,
    };
    graph
        .vertex(
            clef,
            json!({"kind":"ClefInter","shape":"G_CLEF","clef_kind":"TREBLE",
        "staff_id":1,"bounds":[99,119,3,3],"glyph":glyph,"glyph_id":gid,
        "frozen":true,"manual":true}),
            false,
        )
        .unwrap();
    graph.bind_clef_identity(clef, staff.clone(), true).unwrap();
    let keys = std::array::from_fn(|i| Key {
        system: 1,
        phase: Phase::MusicKey,
        local: i + 1,
    });
    // The fourth point has no Staff area. The last two admitted Keys intentionally
    // arrive in descending x order to distinguish LinkedHashSet order from sorting.
    for (key, [x, y]) in
        keys.iter()
            .zip([[10, 120], [250, 120], [499, 120], [100, 210], [251, 120]])
    {
        graph
            .vertex(
                *key,
                json!({"kind":"KeyInter","shape":"KEY_SHARP_1","fifths":1,
            "staff_id":1,"bounds":[x,y,1,1],"glyph":null,"frozen":true,"manual":false}),
                false,
            )
            .unwrap();
        graph.bind_filler_key_identity(*key, staff.clone()).unwrap();
    }
    ScalarFiller {
        graph,
        registry,
        layout,
        staff,
        clef,
        keys,
    }
}

fn scalar_memberships(graph: &Graph) -> Vec<(Vec<Key>, Vec<Key>)> {
    graph
        .clef_ownership
        .as_ref()
        .unwrap()
        .measures
        .iter()
        .map(|m| (m.members.clone(), m.keys.clone()))
        .collect()
}

fn assert_scalar_owners(graph: &Graph, scalar: &ScalarFiller) {
    let owners = graph.clef_ownership.as_ref().unwrap();
    assert!(Arc::ptr_eq(&owners.staffs[0], &scalar.staff));
    assert!(Arc::ptr_eq(&owners.parts[0], &scalar.layout.parts[0].part));
    for (held, actual) in owners.measures.iter().zip(&scalar.layout.parts[0].measures) {
        assert!(Arc::ptr_eq(&held.measure, actual));
        assert!(Arc::ptr_eq(&held.measure.stack, &actual.stack));
    }
    assert_eq!(owners.measures.len(), scalar.layout.parts[0].measures.len());
    assert!(Arc::ptr_eq(
        &graph.clef_staff(scalar.clef).unwrap(),
        &scalar.staff
    ));
}

#[test]
fn filler_uses_left_open_right_closed_bounds_and_preserves_key_insertion_order() {
    let mut s = scalar_filler();
    let first = s.layout.parts[0].measures[0].clone();
    let second = s.layout.parts[0].measures[1].clone();
    assert_eq!(first.abscissae(&s.staff).unwrap(), [10, 250]);
    assert_eq!(second.abscissae(&s.staff).unwrap(), [250, 499]);
    assert_eq!(
        scalar_memberships(&s.graph),
        vec![(vec![], vec![]), (vec![], vec![])]
    );
    assert!(s.graph.require_header_filler_boundary(1).is_err());
    let before_graph = s.graph.snapshot();
    let before_glyphs = s.registry.snapshot();
    let runs = s.registry.get(1).unwrap().glyph.runs.as_ptr();
    let first_pass = s.graph.fill_owned_clefs_and_keys(&s.layout, 1).unwrap();
    assert_eq!(
        first_pass["filling_keys"],
        json!([
            s.clef, s.keys[0], s.keys[1], s.keys[2], s.keys[3], s.keys[4]
        ])
    );
    assert_eq!(s.graph.owned_measure_clefs(&first).unwrap(), [s.clef]);
    assert!(s.graph.owned_measure_clefs(&second).unwrap().is_empty());
    assert_eq!(s.graph.owned_measure_keys(&first).unwrap(), [s.keys[1]]);
    assert_eq!(
        s.graph.owned_measure_keys(&second).unwrap(),
        [s.keys[2], s.keys[4]]
    );
    assert_eq!(
        first_pass["additions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["inter"].clone())
            .collect::<Vec<_>>(),
        vec![
            json!(s.clef),
            json!(s.keys[1]),
            json!(s.keys[2]),
            json!(s.keys[4])
        ]
    );
    s.graph.require_header_filler_boundary(1).unwrap();
    let members = scalar_memberships(&s.graph);
    s.graph.fill_owned_clefs_and_keys(&s.layout, 1).unwrap();
    assert_eq!(scalar_memberships(&s.graph), members);
    assert_eq!(s.graph.snapshot(), before_graph);
    assert_eq!(s.registry.snapshot(), before_glyphs);
    assert_eq!(s.registry.get(1).unwrap().glyph.runs.as_ptr(), runs);
    assert_scalar_owners(&s.graph, &s);
}

#[test]
fn foreign_equal_id_layout_or_missing_key_owner_preserves_existing_memberships() {
    let mut s = scalar_filler();
    s.graph.fill_owned_clefs_and_keys(&s.layout, 1).unwrap();
    let foreign = scalar_filler();
    assert_eq!(foreign.staff.id, s.staff.id);
    assert!(!Arc::ptr_eq(&foreign.staff, &s.staff));
    let before_graph = s.graph.snapshot();
    let before_members = scalar_memberships(&s.graph);
    let before_glyphs = s.registry.snapshot();
    let runs = s.registry.get(1).unwrap().glyph.runs.as_ptr();
    assert_eq!(
        s.graph
            .fill_owned_clefs_and_keys(&foreign.layout, 1)
            .unwrap_err(),
        "music_filler_same_actual_Stack_order_required"
    );
    assert_eq!(s.graph.snapshot(), before_graph);
    assert_eq!(scalar_memberships(&s.graph), before_members);
    s.graph.require_header_filler_boundary(1).unwrap();
    assert_scalar_owners(&s.graph, &s);

    let mut missing = s.graph.clone();
    missing
        .clef_ownership
        .as_mut()
        .unwrap()
        .key_signatures
        .remove(&s.keys[4])
        .unwrap();
    let before_failure_graph = missing.snapshot();
    let before_failure_members = scalar_memberships(&missing);
    let before_filled = missing
        .clef_ownership
        .as_ref()
        .unwrap()
        .filled_systems
        .clone();
    // Missing ownership is preflight refusal; no late staged-mutation claim.
    assert_eq!(
        missing.fill_owned_clefs_and_keys(&s.layout, 1).unwrap_err(),
        "music_filler_bound_Key_required"
    );
    assert_eq!(missing.snapshot(), before_failure_graph);
    assert_eq!(scalar_memberships(&missing), before_failure_members);
    assert_eq!(
        missing.clef_ownership.as_ref().unwrap().filled_systems,
        before_filled
    );
    assert!(
        !missing
            .clef_ownership
            .as_ref()
            .unwrap()
            .key_signatures
            .contains_key(&s.keys[4])
    );
    assert_scalar_owners(&missing, &s);
    assert_eq!(s.registry.snapshot(), before_glyphs);
    assert_eq!(s.registry.get(1).unwrap().glyph.runs.as_ptr(), runs);
}
