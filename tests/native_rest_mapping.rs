// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, beam_candidates::SystemCandidates, beam_dispatch::Registry, beam_impacts::Impacts,
    beam_merge::MergeCandidate, beam_structure::BeamItem, native_rest_mapping::Mapping,
    sections::Orientation,
};
use serde_json::{Value, json};
fn grid() -> Value {
    json!({"systems":[{"id":1,"area":{"x_range":[0,128],"north_neighbor_last_line":null,"south_neighbor_first_line":null}}],"staffs":[{"id":1,"system_id":1,"specific_interline":4,"left":0,"right":127,"lines":(0..5).map(|n|json!({"points":[[0,10+4*n],[128,10+4*n]],"thickness":1.0})).collect::<Vec<_>>()}],"vlag":[{"id":1,"first_pos":20,"runs":[[10,10]]}],"hlag":[{"id":1,"first_pos":10,"runs":[[20,10]]}]})
}
fn candidate(y: f64) -> SystemCandidates {
    let image = GrayImage::new(128, 64, vec![0; 128 * 64]).unwrap();
    let mut state = SystemCandidates::empty(1);
    state
        .register_merge(
            &image,
            &mut Registry::default(),
            &MergeCandidate {
                item: BeamItem {
                    median: [20., y, 100., y],
                    height: 4.,
                },
                is_small: false,
                impacts: Impacts::new([1.; 6]).unwrap(),
            },
        )
        .unwrap();
    state
}
#[test]
fn native_curve_pitch_and_lag_order_are_retained() {
    let m = Mapping::from_grid(&grid(), 128, 64, 1).unwrap();
    assert_eq!(m.rows[20], [10, 26]);
    assert_eq!(m.middle[20], 18);
    assert_eq!(m.sections[0].orientation, Orientation::Vertical);
    assert_eq!(m.sections[1].orientation, Orientation::Horizontal);
    let state = candidate(18.);
    let position = m.position(state.active(1).unwrap()).unwrap().unwrap();
    assert_eq!(position.endpoint_pitches, [0., 0.]);
    assert_eq!(position.mid_line_thickness, 1.);
    assert_eq!(m.geometry(1).interline, 4);
}
#[test]
fn unsupported_staff_and_populated_ledger_paths_are_explicit() {
    let mut g = grid();
    let other = g["staffs"][0].clone();
    g["staffs"].as_array_mut().unwrap().push(other);
    assert!(Mapping::from_grid(&g, 128, 64, 1).is_err());
    let m = Mapping::from_grid(&grid(), 128, 64, 1).unwrap();
    assert!(
        m.position(candidate(5.).active(1).unwrap())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        m.position(candidate(40.).active(1).unwrap())
            .unwrap()
            .unwrap()
            .endpoint_pitches,
        [11., 11.]
    );
    let mut populated = grid();
    populated["staffs"][0]["ledger_lines"] = json!([{"index": 1}]);
    assert_eq!(
        Mapping::from_grid(&populated, 128, 64, 1).err().unwrap(),
        "unsupported_rest_non_linear_ledger_mapping"
    );
}

#[test]
fn original_empty_ledger_pitch_controls_match_native_mapping() {
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/official-empty-ledger-pitch.json")).unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    for case in fixture["cases"].as_array().unwrap() {
        let mut g = grid();
        g["staffs"][0]["lines"] = fixture["lines"].clone();
        g["staffs"][0]["specific_interline"] = fixture["specific_interline"].clone();
        g["staffs"][0]["ledger_lines"] = json!([]);
        let m = Mapping::from_grid(&g, 128, 256, 1).unwrap();
        let pitch = m
            .pitch_at(case["x"].as_f64().unwrap(), case["y"].as_f64().unwrap())
            .unwrap();
        let original = case["pitch"].as_f64().unwrap();
        assert!(
            (pitch - original).abs() <= 1e-12,
            "{case}: {pitch} != {original}"
        );
    }
    let m = Mapping::from_grid(&grid(), 128, 64, 1).unwrap();
    assert!(m.pitch_at(f64::NAN, 18.).is_err());
}

#[test]
fn original_populated_ledger_pitch_controls_match_native_mapping() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../fixtures/official-populated-ledger-pitch.json"
    ))
    .unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    for configuration in fixture["configurations"].as_array().unwrap() {
        let mut g = grid();
        g["staffs"][0]["lines"] = fixture["lines"].clone();
        g["staffs"][0]["specific_interline"] = fixture["specific_interline"].clone();
        g["staffs"][0]["ledger_lines"] = configuration["ledgers"].clone();
        let m = Mapping::from_grid(&g, 128, 256, 1).unwrap();
        for case in configuration["cases"].as_array().unwrap() {
            let actual = m
                .pitch_at(case["x"].as_f64().unwrap(), case["y"].as_f64().unwrap())
                .unwrap();
            let expected = case["pitch"].as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 1e-12,
                "{case}: {actual} != {expected}"
            );
        }
    }
}

#[test]
fn invalid_ledger_identity_shape_and_order_are_rejected() {
    let mut g = grid();
    let ledger = json!({"index":1,"points":[[0,30],[128,30]]});
    g["staffs"][0]["ledger_lines"] = json!([ledger, ledger]);
    assert_eq!(
        Mapping::from_grid(&g, 128, 64, 1).err().unwrap(),
        "duplicate_rest_ledger_index"
    );
    g["staffs"][0]["ledger_lines"] = json!([{"index":0,"points":[[0,30],[128,30]]}]);
    assert!(Mapping::from_grid(&g, 128, 64, 1).is_err());
    g["staffs"][0]["ledger_lines"] = json!([{"index":1,"points":[[0,20],[128,20]]}]);
    let mapping = Mapping::from_grid(&g, 128, 64, 1).unwrap();
    assert_eq!(
        mapping.pitch_at(20., 40.).err().unwrap(),
        "unsupported_rest_ledger_order"
    );
}

#[test]
fn original_multistaff_areas_closest_and_above_controls_match() {
    use audiveris_native::native_rest_mapping::SystemMapping;
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/official-multistaff-area.json")).unwrap();
    assert_eq!(fixture["native_outputs_used"], false);
    let mapping = SystemMapping::from_grid(&fixture["grid"], 160, 320, 1).unwrap();
    for c in fixture["cases"].as_array().unwrap() {
        let x = c["x"].as_f64().unwrap();
        let y = c["y"].as_f64().unwrap();
        let id = |v: Option<usize>| v.map(|i| mapping.staffs[i].id).unwrap_or(0);
        assert_eq!(
            id(mapping.closest_staff(x, y).unwrap()),
            c["closest"].as_u64().unwrap(),
            "closest: {c}"
        );
        assert_eq!(
            id(mapping.staff_at_or_above(x, y).unwrap()),
            c["above"].as_u64().unwrap(),
            "above: {c}"
        );
    }
    assert!(mapping.staff_at_or_above(f64::NAN, 50.).is_err());
    let mut g = fixture["grid"].clone();
    g["staffs"][1]["right"] = json!(19);
    assert!(SystemMapping::from_grid(&g, 160, 320, 1).is_err());
    g = fixture["grid"].clone();
    let other = g["systems"][0].clone();
    g["systems"].as_array_mut().unwrap().push(other);
    assert_eq!(
        SystemMapping::from_grid(&g, 160, 320, 1).err().unwrap(),
        "duplicate_rest_mapping_system"
    );
}

#[test]
fn original_scanned_score_multisystem_area_selection_and_bounds_match() {
    use audiveris_native::native_rest_mapping::SystemMapping;
    let f: Value =
        serde_json::from_str(include_str!("../fixtures/official-multisystem-area.json")).unwrap();
    assert_eq!(f["native_outputs_used"], false);
    for sys in f["systems"].as_array().unwrap() {
        let sid = sys["id"].as_u64().unwrap();
        let m = SystemMapping::from_grid(
            &f["input"]["grid"],
            f["input"]["width"].as_u64().unwrap() as usize,
            f["input"]["height"].as_u64().unwrap() as usize,
            sid,
        )
        .unwrap();
        for case in sys["cases"].as_array().unwrap() {
            let x = case["x"].as_f64().unwrap();
            let y = case["y"].as_f64().unwrap();
            let id = |i: Option<usize>| i.map(|n| m.staffs[n].id).unwrap_or(0);
            assert_eq!(
                id(m.closest_staff(x, y).unwrap()),
                case["closest"].as_u64().unwrap(),
                "closest: {case}"
            );
            assert_eq!(
                id(m.staff_at_or_above(x, y).unwrap()),
                case["above"].as_u64().unwrap(),
                "above: {case}"
            );
        }
        assert_eq!(
            m.sections.iter().map(|s| s.id.0).collect::<Vec<_>>(),
            serde_json::from_value::<Vec<u32>>(sys["section_ids"].clone()).unwrap()
        );
    }
}
#[test]
fn original_supplied_clipped_system_areas_match() {
    use audiveris_native::native_rest_mapping::SystemMapping;
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-clipped-system-area.json"
    ))
    .unwrap();
    assert_eq!(f["native_outputs_used"], false);
    for sys in f["systems"].as_array().unwrap() {
        let sid = sys["id"].as_u64().unwrap();
        let m = SystemMapping::from_grid(
            &f["input"]["grid"],
            f["input"]["width"].as_u64().unwrap() as usize,
            f["input"]["height"].as_u64().unwrap() as usize,
            sid,
        )
        .unwrap();
        for case in sys["cases"].as_array().unwrap() {
            let x = case["x"].as_f64().unwrap();
            let y = case["y"].as_f64().unwrap();
            let id = |i: Option<usize>| i.map(|n| m.staffs[n].id).unwrap_or(0);
            assert_eq!(
                id(m.closest_staff(x, y).unwrap()),
                case["closest"].as_u64().unwrap(),
                "closest: {case}"
            );
            assert_eq!(
                id(m.staff_at_or_above(x, y).unwrap()),
                case["above"].as_u64().unwrap(),
                "above: {case}"
            );
        }
        assert_eq!(
            m.sections.iter().map(|s| s.id.0).collect::<Vec<_>>(),
            serde_json::from_value::<Vec<u32>>(sys["section_ids"].clone()).unwrap()
        );
    }
}

#[test]
fn clipped_curved_areas_and_invalid_ends_are_explicitly_rejected() {
    use audiveris_native::native_rest_mapping::SystemMapping;
    let f: Value =
        serde_json::from_str(include_str!("../fixtures/official-multistaff-area.json")).unwrap();
    let mut g = f["grid"].clone();
    g["systems"][0]["area"]["x_range"] = json!([20, 140]);
    assert_eq!(
        SystemMapping::from_grid(&g, 160, 320, 1).err().unwrap(),
        "unsupported_curved_system_area_clip"
    );
    for ends in [json!([140, 20]), json!([0, 161]), json!([20, 20])] {
        g["systems"][0]["area"]["x_range"] = ends;
        assert!(SystemMapping::from_grid(&g, 160, 320, 1).is_err());
    }
}
