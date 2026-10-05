// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_parts_alignment_controls.rs"]
mod controls;
use audiveris_native::{
    beam_dispatch,
    stem_parts_alignment::{self, Parameters, Part},
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
fn rect(x: i32, y: i32, height: usize) -> GlyphPixels {
    beam_dispatch::glyph_from_json(
        &json!({"bounds":[x,y,1,height],"orientation":"VERTICAL","runs":[[0,0,height]]}),
    )
    .unwrap()
}
fn part(id: usize, glyph: &GlyphPixels) -> Part<'_> {
    Part { id, glyph }
}
#[test]
fn original_fifteen_hundred_part_alignment_checks_match_caller_order_aliases_and_actual_stumps() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-parts-alignment/controls.json"
    ))
    .unwrap();
    let n = controls::observe(&f);
    assert_eq!(n["cases"], f["cases"]);
    let cases = n["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 1500);
    assert_eq!(
        cases
            .iter()
            .map(|c| c["removed_order"].as_array().unwrap().len())
            .sum::<usize>(),
        2810
    );
    for direction in [-1, 1] {
        assert!(
            cases
                .iter()
                .any(|c| c["direction"] == direction && c["mode"] == "duplicates")
        );
        assert!(
            cases
                .iter()
                .any(|c| c["direction"] == direction && c["mode"] == "with-stump")
        );
    }
}
#[test]
fn alignment_inclusive_thresholds_ties_identity_and_invalid_budgets_preserve_inputs() {
    // Null stump and custom thresholds are native boundary checks, distinct
    // from the independently observed nonnull-stump Laura controls.
    let p = Parameters::new(1, 1., 4., 0.).unwrap();
    let a = rect(20, 20, 3);
    let dx_equal = rect(21, 24, 3);
    let dx_over = rect(22, 24, 3);
    let dy_over = rect(22, 25, 3);
    let run = |b: &GlyphPixels| {
        stem_parts_alignment::filter(&[part(1, &a), part(2, b)], None, &p).unwrap()
    };
    assert_eq!(run(&dx_equal).retained_order, vec![1, 2]);
    assert_eq!(run(&dx_over).removed_order, vec![2]); // equal heights remove second
    assert_eq!(run(&dy_over).retained_order, vec![1, 2]);
    let fractional = beam_dispatch::glyph_from_json(
        &json!({"bounds":[21,20,2,3],"orientation":"HORIZONTAL","runs":[[0,0,2],[1,0,2],[2,0,2]]}),
    )
    .unwrap();
    let fractional_parameters = Parameters::new(1, 1.6, 4., 0.).unwrap();
    assert_eq!(
        stem_parts_alignment::filter(
            &[part(1, &a), part(2, &fractional)],
            None,
            &fractional_parameters
        )
        .unwrap()
        .retained_order,
        vec![1, 2]
    );
    let reverse = Parameters::new(-1, 1., 4., 0.).unwrap();
    assert_eq!(
        stem_parts_alignment::filter(&[part(1, &a), part(2, &dx_over)], None, &reverse)
            .unwrap()
            .removed_order,
        vec![1]
    );
    let duplicates = [part(1, &a), part(2, &dx_over), part(2, &dx_over)];
    assert_eq!(
        stem_parts_alignment::filter(&duplicates, None, &p)
            .unwrap()
            .retained_order,
        vec![1]
    );
    assert!(stem_parts_alignment::filter(&[part(1, &a), part(1, &dx_over)], None, &p).is_err());
    assert!(stem_parts_alignment::filter(&[part(0, &a)], None, &p).is_err());
    assert!(Parameters::new(0, 1., 4., 0.).is_err());
    assert!(Parameters::new(1, f64::NAN, 4., 0.).is_err());
    assert!(Parameters::new(1, 1., 4., 0.01).is_err());
    assert!(Parameters::new(1, 1., 4., f64::NAN).is_err());
    let empty = stem_parts_alignment::filter(&[], Some(part(1, &a)), &p).unwrap();
    assert!(empty.retained_order.is_empty() && empty.removed_order.is_empty());
    let too_many = vec![part(1, &a); audiveris_native::stem_glyph::MAX_PARTS + 1];
    assert!(stem_parts_alignment::filter(&too_many, None, &p).is_err());
    let huge = rect(20, 20, audiveris_native::stem_glyph::MAX_PART_PIXELS + 1);
    assert!(stem_parts_alignment::filter(&[part(1, &huge)], None, &p).is_err());
    let half = rect(
        20,
        20,
        audiveris_native::stem_glyph::MAX_PART_PIXELS / 2 + 1,
    );
    assert!(stem_parts_alignment::filter(&[part(1, &half), part(2, &half)], None, &p).is_err());
    // Logical aliases count once toward the validated unique pixel budget.
    assert!(stem_parts_alignment::filter(&[part(1, &half), part(1, &half)], None, &p).is_ok());
    assert_eq!(
        beam_dispatch::glyph_json(&a)["bounds"],
        json!([20, 20, 1, 3])
    );
}
