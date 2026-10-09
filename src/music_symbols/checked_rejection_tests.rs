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
fn glyph_ink(glyph: &GlyphPixels) -> BTreeSet<(i32, i32)> {
    assert_eq!(glyph.orientation, Orientation::Vertical);
    glyph
        .runs
        .iter()
        .flat_map(|run| {
            (run.start..run.start + run.length)
                .map(move |y| (glyph.left + run.position as i32, glyph.top + y as i32))
        })
        .collect()
}
fn assert_historical_candidates_erased_by_actual_slurs(
    music: &OwnedMusic,
    trials: &[Value],
    historical: &[Value],
) -> Vec<Value> {
    // These masks are old native candidate observations, not original glyphs
    // or recognition inputs. Partial overlap does not mean whole-box erasure.
    let controls: Value = serde_json::from_str(r#"[{"system":2,"bounds":[565,1047,37,13],"weight":77,"parts":[{"bounds":[574,1055,28,5],"orientation":"VERTICAL","runs":[[0,1,1],[1,1,1],[2,1,2],[3,1,2],[4,1,3],[5,1,3],[6,1,3],[7,1,3],[8,2,3],[9,2,3],[10,2,3],[11,2,3],[12,2,3],[13,2,3],[14,2,3],[15,2,3],[16,2,3],[17,2,3],[18,2,3],[19,2,2],[20,2,2],[21,1,3],[22,1,2],[23,1,2],[24,0,2],[25,0,2],[26,0,1],[27,0,1]]},{"bounds":[565,1047,5,5],"orientation":"VERTICAL","runs":[[0,0,2],[1,1,2],[2,2,2],[3,3,2],[4,4,1]]}]},{"system":2,"bounds":[642,1194,36,13],"weight":81,"parts":[{"bounds":[651,1203,27,4],"orientation":"VERTICAL","runs":[[0,0,1],[1,0,1],[2,0,2],[3,0,2],[4,0,3],[5,0,3],[6,0,3],[7,0,3],[8,1,3],[9,1,3],[10,1,3],[11,1,3],[12,1,3],[13,1,3],[14,1,3],[15,1,3],[16,1,3],[17,1,3],[18,1,3],[19,1,3],[20,1,2],[21,0,3],[22,0,3],[23,0,2],[24,0,2],[25,0,1],[26,0,1]]},{"bounds":[642,1194,7,6],"orientation":"VERTICAL","runs":[[0,0,1],[1,0,3],[2,2,2],[3,3,2],[4,4,2],[5,4,2],[6,5,1]]}]}]"#).unwrap();
    let controls = controls.as_array().unwrap();
    assert_eq!(historical.len(), controls.len());
    let registry = &music.owner.prepared.registry;
    let classified_ink = (1..=registry.len())
        .map(|id| registry.get(id).unwrap())
        .filter(|entry| entry.groups.contains(&beam_dispatch::Group::Symbol))
        .flat_map(|entry| glyph_ink(&entry.glyph))
        .collect::<BTreeSet<_>>();
    let state = music
        .slurs
        .as_ref()
        .expect("accepted actual Slur owners before SYMBOLS");
    let mut used_slurs = BTreeSet::new();
    let mut observed = Vec::new();
    for (expected, control) in historical.iter().zip(controls) {
        assert_eq!(expected["system"], control["system"]);
        assert_eq!(expected["bounds"], control["bounds"]);
        assert_eq!(expected["shape"], "FERMATA_BELOW");
        assert_eq!(expected["created_inter_ids"], json!([]));
        assert!(
            trials
                .iter()
                .all(|trial| trial["system_id"] != expected["system"]
                    || trial["bounds"] != expected["bounds"]),
            "old candidate must be absent after actual Slur cleanup: {expected}"
        );
        let parts = control["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|part| glyph_ink(&beam_dispatch::glyph_from_json(part).unwrap()))
            .collect::<Vec<_>>();
        let old_ink = parts.iter().flatten().copied().collect::<BTreeSet<_>>();
        assert_eq!(old_ink.len() as u64, control["weight"].as_u64().unwrap());
        for &(x, y) in &old_ink {
            assert!(x >= 0 && y >= 0);
            assert_eq!(
                music.owner.raw_binary.pixels
                    [y as usize * music.owner.raw_binary.width + x as usize],
                0
            );
        }
        let matches = state
            .slurs
            .iter()
            .filter_map(|slur| {
                if expected["system"] != slur.system.id {
                    return None;
                }
                let glyph = &registry.get(slur.glyph_id).unwrap().glyph;
                let ink = glyph_ink(glyph);
                let overlap = old_ink.intersection(&ink).copied().collect::<BTreeSet<_>>();
                (!overlap.is_empty()).then_some((slur, glyph, ink, overlap))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "unique actual Slur consumes old component ink"
        );
        let (slur, glyph, ink, overlap) = &matches[0];
        assert!(used_slurs.insert(slur.key));
        let (node, removed) = music.owner.graph.retained_inter(slur.key).unwrap();
        assert!(!removed);
        assert_eq!(node["kind"], "SlurInter");
        assert_eq!(node["manual"], false);
        assert_eq!(node["is_tie"], false);
        assert_eq!(node["abnormal"], false);
        assert_eq!(node["glyph_id"], slur.glyph_id);
        assert_eq!(node["glyph"], beam_dispatch::glyph_json(glyph));
        assert!(
            music
                .owner
                .systems
                .iter()
                .any(|s| Arc::ptr_eq(s, &slur.system))
        );
        assert_eq!(
            state
                .parts
                .iter()
                .filter(|part| Arc::ptr_eq(&part.part, &slur.part)
                    && part.slurs.iter().any(|s| Arc::ptr_eq(s, slur)))
                .count(),
            1
        );
        for (id, relation) in slur.relation_ids.iter().zip(&slur.relations) {
            assert!(Arc::ptr_eq(
                music.owner.graph.fresh_slur_relation_owner(*id).unwrap(),
                relation
            ));
        }
        let component_overlap = parts
            .iter()
            .map(|part| part.intersection(ink).count())
            .collect::<Vec<_>>();
        assert!(component_overlap.iter().all(|&count| count > 0));
        assert!(
            ink.is_disjoint(&classified_ink),
            "accepted Slur ink must be absent from actual post-cleanup Symbol glyphs"
        );
        observed.push(json!({"historical_candidate":expected,"historical_native_weight":old_ink.len(),
            "actual_slur":slur.key,"actual_Slur_owner":Arc::as_ptr(slur) as usize,
            "actual_glyph_id":slur.glyph_id,"actual_relation_ids":slur.relation_ids,
            "historical_component_overlap_pixels":component_overlap,"erased_old_pixels":overlap.len(),
            "old_pixels_outside_this_Slur":old_ink.len()-overlap.len(),
            "old_candidate_absent":true,"accepted_Slur_ink_absent_from_post_cleanup_Symbol_glyphs":true}));
    }
    observed
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
    let historical_unmatched = source["unmatched_native"].as_array().unwrap();
    let removed_by_slurs = if page == "der_winter" {
        assert_historical_candidates_erased_by_actual_slurs(music, trials, historical_unmatched)
    } else {
        assert!(historical_unmatched.is_empty());
        Vec::new()
    };
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
        "historical_unmatched_CURVES_candidates":source["unmatched_native"],"unmatched_CURVES_candidates_removed_by_actual_Slur_cleanup":removed_by_slurs,"accent":accent,
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
