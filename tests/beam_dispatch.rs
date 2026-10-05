// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::beam_dispatch::{self, Geometry, Group, Registry, SystemSpots};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde_json::{Value, json};
fn glyph(left: i32, top: i32) -> GlyphPixels {
    GlyphPixels {
        left,
        top,
        width: 2,
        height: 2,
        orientation: Orientation::Vertical,
        runs: vec![
            Run {
                position: 0,
                start: 0,
                length: 2,
            },
            Run {
                position: 1,
                start: 0,
                length: 2,
            },
        ],
    }
}
fn geometry_input() -> Value {
    json!({"staffs":[{"id":1,"system_id":1,"lines":(0..5).map(|i|json!({"points":[[10,10+5*i],[90,10+5*i]]})).collect::<Vec<_>>()},{"id":2,"system_id":2,"lines":(0..5).map(|i|json!({"points":[[10,70+5*i],[90,70+5*i]]})).collect::<Vec<_>>()}],"systems":[{"id":1,"staff_ids":[1],"coordinates":[10,10,81,30,0],"area":{"x_range":[0,100],"vertical_margin":4.0,"north_neighbor_last_line":null,"south_neighbor_first_line":2}},{"id":2,"staff_ids":[2],"coordinates":[10,70,81,90,0],"area":{"x_range":[0,100],"vertical_margin":4.0,"north_neighbor_last_line":1,"south_neighbor_first_line":null}}]})
}
#[test]
fn registry_preserves_existing_canonical_object_and_groups() {
    let mut registry = Registry::default();
    let first = registry.register(&glyph(20, 20)).unwrap();
    registry.add_group(first, Group::BlackHeadSpot).unwrap();
    let same = registry.register(&glyph(20, 20)).unwrap();
    assert_eq!(first, same);
    assert_eq!(registry.len(), 1);
    assert!(
        registry
            .get(same)
            .unwrap()
            .groups
            .contains(&Group::BlackHeadSpot)
    );
    let mut other = glyph(20, 20);
    other.orientation = Orientation::Horizontal;
    assert_ne!(registry.register(&other).unwrap(), first);
    let mut other = glyph(20, 20);
    other.width = 3;
    assert_ne!(registry.register(&other).unwrap(), first);
    assert_ne!(registry.register(&glyph(21, 20)).unwrap(), first);
    assert_ne!(registry.register(&glyph(20, 21)).unwrap(), first);
    let mut other = glyph(20, 20);
    other.runs[0].length = 1;
    assert_ne!(registry.register(&other).unwrap(), first);
}
#[test]
fn centroid_is_half_even_with_signed_origins() {
    assert_eq!(beam_dispatch::centroid(&glyph(20, 20)).unwrap(), [20, 20]);
    assert_eq!(beam_dispatch::centroid(&glyph(21, 21)).unwrap(), [22, 22]);
    assert_eq!(beam_dispatch::centroid(&glyph(-2, -2)).unwrap(), [-2, -2]);
    assert_eq!(beam_dispatch::centroid(&glyph(-1, -1)).unwrap(), [0, 0]);
}
#[test]
fn dispatch_includes_right_limit_and_overlapping_systems() {
    let geometry = Geometry::from_grid(100, 100, &geometry_input(), 5).unwrap();
    assert_eq!(geometry.eligible([10, 10]).unwrap(), vec![1]);
    assert_eq!(geometry.eligible([91, 10]).unwrap(), vec![1]);
    assert!(geometry.eligible([92, 10]).unwrap().is_empty());
    assert!(geometry.eligible([9, 10]).unwrap().is_empty());
    assert_eq!(geometry.eligible([50, 33]).unwrap(), vec![1]);
    assert_eq!(geometry.eligible([50, 34]).unwrap(), vec![1, 2]);
    assert_eq!(geometry.eligible([50, 65]).unwrap(), vec![1, 2]);
    assert_eq!(geometry.eligible([50, 66]).unwrap(), vec![2]);
    assert_eq!(geometry.eligible([50, 99]).unwrap(), vec![2]);
    assert!(geometry.eligible([50, 100]).unwrap().is_empty());
    let mut registry = Registry::default();
    let spots = vec![
        glyph(20, 20),
        glyph(20, 50),
        glyph(20, 80),
        glyph(1, 1),
        glyph(20, 50),
    ];
    let out = beam_dispatch::dispatch(&spots, &geometry, &mut registry).unwrap();
    assert_eq!(registry.len(), 3);
    assert_eq!(out[0].glyph_ids, vec![1, 2]);
    assert_eq!(out[1].glyph_ids, vec![2, 3]);
}
#[test]
fn epilog_removes_only_exclusively_beam_grouped_free_glyphs() {
    let mut registry = Registry::default();
    let a = registry.register(&glyph(20, 20)).unwrap();
    let b = registry.register(&glyph(20, 30)).unwrap();
    let c = registry.register(&glyph(20, 40)).unwrap();
    registry.add_group(a, Group::BeamSpot).unwrap();
    registry.add_group(b, Group::BeamSpot).unwrap();
    registry.add_group(b, Group::BlackHeadSpot).unwrap();
    registry.add_group(c, Group::VerticalSeed).unwrap();
    let mut s = SystemSpots {
        system_id: 1,
        glyph_ids: vec![a, b, c],
    };
    beam_dispatch::remove_exclusive_beam_spots(&mut s, &registry).unwrap();
    assert_eq!(s.glyph_ids, vec![b, c]);
    assert!(registry.get(a).unwrap().groups.contains(&Group::BeamSpot));
}
#[test]
fn invalid_registry_and_geometry_inputs_fail_explicitly() {
    let mut registry = Registry::default();
    let mut g = glyph(0, 0);
    g.runs[0].length = 0;
    assert!(registry.register(&g).is_err());
    let mut g = glyph(0, 0);
    g.runs.swap(0, 1);
    assert!(registry.register(&g).is_err());
    let mut g = glyph(0, 0);
    g.runs[0].start = usize::MAX;
    assert!(registry.register(&g).is_err());
    let mut g = glyph(0, 0);
    g.width = usize::MAX;
    assert!(registry.register(&g).is_err());
    assert!(registry.get(0).is_err());
    assert!(registry.add_group(1, Group::BeamSpot).is_err());
    let mut grid = geometry_input();
    grid["systems"][0]["area"]["south_neighbor_first_line"] = json!(1);
    assert!(Geometry::from_grid(100, 100, &grid, 5).is_err());
    let mut grid = geometry_input();
    grid["systems"][0]["staff_ids"] = json!([1, 1]);
    assert!(Geometry::from_grid(100, 100, &grid, 5).is_err());
    let mut grid = geometry_input();
    grid["systems"][0]["area"]["vertical_margin"] = json!(0);
    assert!(Geometry::from_grid(100, 100, &grid, 5).is_err());
    let mut grid = geometry_input();
    grid["systems"][0]["coordinates"][2] = json!(i64::MIN);
    assert!(Geometry::from_grid(100, 100, &grid, 5).is_err());
}
#[test]
fn initial_free_glyph_order_survives_duplicate_dispatch_and_parser_roundtrip() {
    let geometry = Geometry::from_grid(100, 100, &geometry_input(), 5).unwrap();
    let mut registry = Registry::default();
    let original = glyph(20, 20);
    let seed = registry.register(&original).unwrap();
    registry.add_group(seed, Group::VerticalSeed).unwrap();
    let mut systems = vec![
        SystemSpots {
            system_id: 1,
            glyph_ids: vec![seed],
        },
        SystemSpots {
            system_id: 2,
            glyph_ids: vec![],
        },
    ];
    beam_dispatch::dispatch_into(
        &[glyph(30, 20), original.clone()],
        &geometry,
        &mut registry,
        &mut systems,
    )
    .unwrap();
    assert_eq!(systems[0].glyph_ids, vec![seed, 2]);
    assert_eq!(registry.get(seed).unwrap().groups.len(), 2);
    let snapshot = beam_dispatch::grouped_snapshot(&systems, &registry, Group::BeamSpot).unwrap();
    assert_eq!(
        snapshot[0]["grouped_spots"][0]["glyph"],
        beam_dispatch::glyph_json(&original)
    );
    assert_eq!(
        beam_dispatch::glyph_from_json(&beam_dispatch::glyph_json(&original)).unwrap(),
        original
    );
    let mut invalid = beam_dispatch::glyph_json(&original);
    invalid["runs"][0][0] = json!(u64::MAX);
    assert!(beam_dispatch::glyph_from_json(&invalid).is_err());
    beam_dispatch::remove_exclusive_beam_spots(&mut systems[0], &registry).unwrap();
    assert_eq!(systems[0].glyph_ids, vec![seed]);
}

#[test]
fn sheet_epilog_preserves_shared_identity_groups_and_survivor_order() {
    let mut registry = Registry::default();
    let a = registry.register(&glyph(20, 20)).unwrap();
    let b = registry.register(&glyph(20, 30)).unwrap();
    let c = registry.register(&glyph(20, 40)).unwrap();
    let d = registry.register(&glyph(20, 50)).unwrap();
    registry.add_group(a, Group::BeamSpot).unwrap();
    registry.add_group(b, Group::BeamSpot).unwrap();
    registry.add_group(b, Group::VerticalSeed).unwrap();
    registry.add_group(c, Group::BlackHeadSpot).unwrap();
    let before = registry.snapshot();
    let mut systems = vec![
        SystemSpots {
            system_id: 1,
            glyph_ids: vec![a, b, c, d],
        },
        SystemSpots {
            system_id: 2,
            glyph_ids: vec![d, a, b],
        },
    ];
    let removed = beam_dispatch::finish_beams_epilog(&mut systems, &registry).unwrap();
    assert_eq!(removed, vec![(1, vec![a]), (2, vec![a])]);
    assert_eq!(systems[0].glyph_ids, vec![b, c, d]);
    assert_eq!(systems[1].glyph_ids, vec![d, b]);
    assert_eq!(registry.snapshot(), before);
    assert_eq!(
        beam_dispatch::finish_beams_epilog(&mut systems, &registry).unwrap(),
        vec![(1, vec![]), (2, vec![])]
    );
}
#[test]
fn invalid_sheet_epilog_reference_rejects_before_any_owner_mutates() {
    let mut registry = Registry::default();
    let id = registry.register(&glyph(20, 20)).unwrap();
    registry.add_group(id, Group::BeamSpot).unwrap();
    let mut systems = vec![
        SystemSpots {
            system_id: 1,
            glyph_ids: vec![id],
        },
        SystemSpots {
            system_id: 2,
            glyph_ids: vec![99],
        },
    ];
    let before: Vec<_> = systems
        .iter()
        .map(|s| (s.system_id, s.glyph_ids.clone()))
        .collect();
    assert!(beam_dispatch::finish_beams_epilog(&mut systems, &registry).is_err());
    assert_eq!(
        systems
            .iter()
            .map(|s| (s.system_id, s.glyph_ids.clone()))
            .collect::<Vec<_>>(),
        before
    );
}

#[test]
fn supplied_clipped_ends_bound_beam_sides_without_changing_staff_limits() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-clipped-system-area.json"
    ))
    .unwrap();
    let g = Geometry::from_grid(1489, 2104, &f["input"]["grid"], 11).unwrap();
    for sys in [1, 2] {
        let y = if sys == 1 { 500. } else { 1100. };
        assert!(!g.is_beam_side_in_system(sys, [249.75, y], 4.).unwrap());
        assert!(g.is_beam_side_in_system(sys, [250., y], 4.).unwrap());
        assert!(g.is_beam_side_in_system(sys, [1399.75, y], 4.).unwrap());
        assert!(!g.is_beam_side_in_system(sys, [1400., y], 4.).unwrap());
    }
}
