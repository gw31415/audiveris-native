// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::sync::Arc;
fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/official-checked-rejections/observed.json"
    ))
    .unwrap()
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-10, "{a} vs {b}");
}
fn assert_evaluations(actual: &Value, expected: &Value) {
    let actual = actual.as_array().unwrap();
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert_eq!(a["shape"], e["shape"]);
        close(a["score"].as_f64().unwrap(), e["grade"].as_f64().unwrap());
    }
}
fn trial_at<'a>(trials: &'a [Value], expected: &Value) -> &'a Value {
    let rows = trials
        .iter()
        .filter(|t| t["system_id"] == expected["system"] && t["bounds"] == expected["bounds"])
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        1,
        "unique declared geometry: {}",
        expected["bounds"]
    );
    rows[0]
}
fn actual_check(music: &OwnedMusic, trial: &Value, shape: &str) -> Value {
    let owner = &music.owner;
    let glyph = &owner
        .prepared
        .registry
        .get(trial["glyph_id"].as_u64().unwrap() as usize)
        .unwrap()
        .glyph;
    let point = [
        glyph.left as f64 + glyph.width as f64 / 2.,
        glyph.top as f64 + glyph.height as f64 / 2.,
    ];
    let system = owner
        .systems
        .iter()
        .find(|s| trial["system_id"] == s.id)
        .unwrap();
    let staff = geometry::closest(owner, system, point).unwrap().unwrap();
    assert!(owner.staffs.iter().any(|s| Arc::ptr_eq(s, &staff)));
    let pitch = owner.ledger_memberships.pitch_at(&staff, point).unwrap();
    let failure =
        checked_rejections::rejection(shape, glyph, &staff, owner.interline, Some(pitch)).unwrap();
    let mut row = json!({"system":system.id,"bounds":trial["bounds"],"shape":shape,"center2D":point,
        "closest_staff":staff.id,"pitch":pitch,"failure":failure});
    if shape == "ACCENT" {
        row["native_actual_rectangle_gap"] =
            json!(checked_rejections::staff_gap(glyph, &staff).unwrap());
        row["max_gap"] = json!((owner.interline as f64 * 8.).round_ties_even());
    }
    row
}
/// Extends the existing real-image tests; fixtures only assert the resulting evaluations.
pub(crate) fn assert_real_checked_rejections(music: &OwnedMusic, page: &str) {
    let all = source();
    let source = &all[page];
    assert!(source.is_object());
    let before = music.owner.graph.snapshot();
    let trials = music.owner.prepared.report["classifier_trials"]
        .as_array()
        .unwrap();
    let mut rejected = Vec::new();
    for expected in source["rejected"].as_array().unwrap() {
        let trial = trial_at(trials, expected);
        let mut actual = actual_check(music, trial, expected["shape"].as_str().unwrap());
        assert_eq!(actual["center2D"], expected["center2D"]);
        assert_eq!(actual["failure"], expected["failure"]);
        if let Some(pitch) = expected["pitch"].as_f64() {
            assert!((actual["pitch"].as_f64().unwrap() - pitch).abs() < 1e-7);
            assert_eq!(actual["closest_staff"], expected["closest_staff"]);
        }
        assert_evaluations(&trial["evaluations"], &expected["actual_checked_return"]);
        actual["actual_checked_return"] = trial["evaluations"].clone();
        rejected.push(actual);
    }
    let labels = source["labels"].as_array().unwrap();
    let mut remaining=trials.iter().flat_map(|t|t["evaluations"].as_array().into_iter().flatten()
        .filter(|e|labels.contains(&e["shape"]))
        .map(move |e|json!({"system":t["system_id"],"bounds":t["bounds"],"shape":e["shape"],"score":e["score"]}))).collect::<Vec<_>>();
    let mut protected = Vec::new();
    for expected in source["factory_arrivals"].as_array().unwrap() {
        let matches = trials
            .iter()
            .filter(|t| {
                t["system_id"] == expected["system"]
                    && t["bounds"] == expected["bounds"]
                    && t["evaluations"].as_array().into_iter().flatten().any(|e| {
                        e["shape"] == expected["shape"]
                            && (e["score"].as_f64().unwrap()
                                - expected["evaluation_grade"].as_f64().unwrap())
                            .abs()
                                < 1e-10
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "retain each original factory arrival: {expected}"
        );
        assert_evaluations(
            &matches[0]["evaluations"],
            &expected["actual_checked_return"],
        );
        let index = remaining
            .iter()
            .position(|e| {
                e["system"] == expected["system"]
                    && e["bounds"] == expected["bounds"]
                    && e["shape"] == expected["shape"]
                    && (e["score"].as_f64().unwrap()
                        - expected["evaluation_grade"].as_f64().unwrap())
                    .abs()
                        < 1e-10
            })
            .unwrap();
        protected.push(remaining.remove(index));
    }
    for expected in source["unmatched_native"].as_array().unwrap() {
        let index = remaining
            .iter()
            .position(|e| {
                e["system"] == expected["system"]
                    && e["bounds"] == expected["bounds"]
                    && e["shape"] == expected["shape"]
                    && (e["score"].as_f64().unwrap() - expected["score"].as_f64().unwrap()).abs()
                        < 1e-10
            })
            .unwrap();
        remaining.remove(index);
    }
    assert!(
        remaining.is_empty(),
        "unexpected surviving family evaluations: {remaining:?}"
    );
    for expected in source["accepted_limit_controls"].as_array().unwrap() {
        assert_eq!(expected["declared_limit"], 2);
        assert_eq!(expected["raw_evaluation"]["failure"], Value::Null);
        let trial = trial_at(trials, expected);
        assert_evaluations(&trial["evaluations"], &expected["actual_checked_return"]);
        assert_eq!(
            actual_check(music, trial, "FERMATA_BELOW")["failure"],
            Value::Null
        );
    }
    let mut accent = Value::Null;
    if !source["accent"].is_null() {
        let expected = &source["accent"];
        let trial = trial_at(trials, expected);
        accent = actual_check(music, trial, "ACCENT");
        assert_eq!(accent["failure"], "StaffGap");
        close(
            accent["max_gap"].as_f64().unwrap(),
            expected["max_gap"].as_f64().unwrap(),
        );
        assert!(
            accent["native_actual_rectangle_gap"].as_f64().unwrap()
                > accent["max_gap"].as_f64().unwrap()
        );
        assert_eq!(accent["center2D"], expected["center2D"]);
        assert_eq!(accent["closest_staff"], expected["closest_staff"]);
        assert!(
            (accent["pitch"].as_f64().unwrap() - expected["pitch"].as_f64().unwrap()).abs() < 1e-7
        );
        assert_evaluations(&trial["evaluations"], &expected["original_checked"]);
        accent["original_exact_gap_and_failure_string_observed"] = json!(false);
        accent["actual_checked_return"] = trial["evaluations"].clone();
    }
    let counts = match page {
        "an_laura" => (5, 15, 0, 0),
        "der_winter" => (10, 77, 4, 2),
        _ => panic!("unknown test page"),
    };
    assert_eq!(
        (
            rejected.len(),
            protected.len(),
            source["accepted_limit_controls"].as_array().unwrap().len(),
            source["unmatched_native"].as_array().unwrap().len()
        ),
        counts
    );
    assert_eq!(music.owner.graph.snapshot(), before);
    let output = match page {
        "an_laura" => "/tmp/real-an-laura-native-checked-rejections.json",
        _ => "/tmp/real-der-winter-native-checked-rejections.json",
    };
    std::fs::write(output,serde_json::to_vec_pretty(&json!({"scope":"Actual native evaluation results with retained Staff/ledger geometry; source data is test-only.",
        "page":page,"rejected":rejected,"protected_factory_arrivals":protected,"accepted_limit_controls":source["accepted_limit_controls"],
        "unmatched_CURVES_candidates_preserved":source["unmatched_native"],"accent":accent,
        "graph_unchanged_during_assertion":true,"factory_lifecycles_admitted_by_this_unit":false,"whole_page_checked_or_quality_parity_claimed":false})).unwrap()).unwrap();
}
fn staff(slope: f64) -> crate::grid_ir::StaffGeometry {
    crate::grid_ir::StaffGeometry {
        id: 1,
        system: 1,
        specific_interline: 10,
        abscissae: [0, 100],
        lines: (0..5)
            .map(|i| {
                Arc::new(
                    crate::curves::Curve::interpolate(vec![
                        [0., 100. + 10. * i as f64],
                        [100., 100. + 10. * i as f64 + 100. * slope],
                    ])
                    .unwrap(),
                )
            })
            .collect(),
    }
}
fn glyph(top: i32, height: usize, width: usize) -> GlyphPixels {
    GlyphPixels {
        left: 10,
        top,
        width,
        height,
        orientation: Orientation::Vertical,
        runs: Vec::new(),
    }
}
#[test]
fn clef_height_rejects_null_and_boundary_without_admitting_other_families() {
    let staff = staff(0.);
    let glyph = glyph(90, 20, 8);
    for shape in ["G_CLEF", "F_CLEF", "F_CLEF_SMALL", "C_CLEF"] {
        for p in [None, Some(-4.), Some(4.), Some(-4.000001), Some(4.000001)] {
            assert_eq!(
                checked_rejections::rejection(shape, &glyph, &staff, 11, p).unwrap(),
                Some("NotWithinStaffHeight")
            );
        }
        for p in [-3.999999, 0., 3.999999] {
            assert_eq!(
                checked_rejections::rejection(shape, &glyph, &staff, 11, Some(p)).unwrap(),
                None
            );
        }
    }
    for shape in ["QUINDICESIMA", "OTTAVA", "FERMATA_BELOW", "BOW_DOWN"] {
        assert_eq!(
            checked_rejections::rejection(shape, &glyph, &staff, 11, None).unwrap(),
            None
        );
    }
}
#[test]
fn markers_require_above_pitch_and_positive_fractional_center_gap() {
    let flat = staff(0.);
    let clear = glyph(89, 10, 8);
    let touch = glyph(90, 10, 8);
    for shape in ["CODA", "DA_CAPO", "DAL_SEGNO"] {
        for p in [None, Some(-4.), Some(4.)] {
            assert_eq!(
                checked_rejections::rejection(shape, &clear, &flat, 11, p).unwrap(),
                Some("AboveStaff")
            );
        }
        assert_eq!(
            checked_rejections::rejection(shape, &touch, &flat, 11, Some(-5.)).unwrap(),
            Some("AboveStaff")
        );
        assert_eq!(
            checked_rejections::rejection(shape, &clear, &flat, 11, Some(-5.2)).unwrap(),
            None
        );
        assert_eq!(
            checked_rejections::rejection(shape, &glyph(90, 11, 1), &staff(0.1), 11, Some(-5.11))
                .unwrap(),
            None
        );
    }
}
#[test]
fn measure_repeat_distinguishes_null_pitch_and_inclusive_unit_boundary() {
    let staff = staff(0.);
    let glyph = glyph(90, 20, 8);
    assert_eq!(
        checked_rejections::rejection("REPEAT_TWO_BARS", &glyph, &staff, 11, None).unwrap(),
        Some("MeasureRepeats:tablature")
    );
    for p in [-1., 0., 1.] {
        assert_eq!(
            checked_rejections::rejection("REPEAT_TWO_BARS", &glyph, &staff, 11, Some(p)).unwrap(),
            None
        );
    }
    for p in [-1.000001, 1.000001] {
        assert_eq!(
            checked_rejections::rejection("REPEAT_TWO_BARS", &glyph, &staff, 11, Some(p)).unwrap(),
            Some("MeasureRepeats:pitch")
        );
    }
}
#[test]
fn accent_gap_uses_main_scale_and_includes_contact_and_limit() {
    let staff = staff(0.);
    for (top, passes) in [
        (228, true),
        (229, false),
        (11, true),
        (10, false),
        (140, true),
    ] {
        let glyph = glyph(top, 1, 1);
        assert_eq!(
            checked("ACCENT", &glyph, &staff, 11, &[], 99.)
                .unwrap()
                .is_some(),
            passes,
            "glyph top {top}"
        );
    }
}
