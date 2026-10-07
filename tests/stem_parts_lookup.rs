// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_parts_lookup_controls.rs"]
mod controls;
use audiveris_native::{
    stem_parts_lookup::{self, Parameters},
    vertical_sticks::SeedSection,
};
use serde_json::{Value, json};
fn parameters() -> Parameters {
    Parameters {
        direction: 1,
        slope: 0.,
        max_stem_thickness: 5,
        max_line_section_dx: 3.3,
        last_head_y: None,
        theoretical_line: [20., 0., 20., 100.],
    }
}
#[test]
fn original_2343_actual_and_ulp_controls_match_centroids_section_order_and_raw_glyphs() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-parts-lookup/controls.json"
    ))
    .unwrap();
    let n = controls::observe(&f);
    assert_eq!(n["cases"].as_array().unwrap().len(), 2343);
    assert_eq!(f["vertical_sections"].as_array().unwrap().len(), 1390);
    assert_eq!(f["horizontal_sections"].as_array().unwrap().len(), 1645);
    assert_eq!(
        n["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["raw_sticks"].as_array().unwrap().len())
            .sum::<usize>(),
        4459
    );
}
#[test]
fn empty_invalid_skew_identity_and_centroid_overflow_refuse_without_mutation() {
    let section = SeedSection {
        id: 1,
        vertical: true,
        first_pos: 20,
        runs: vec![[10, 5]],
    };
    let before = json!(section);
    let area = [[19., 0.], [25., 0.], [25., 100.], [19., 100.]];
    let p = parameters();
    let empty = stem_parts_lookup::collect(&[], &[], &area, None, &p).unwrap();
    assert!(empty.raw_glyphs.is_empty());
    assert!(
        stem_parts_lookup::collect(std::slice::from_ref(&section), &[], &[], None, &p)
            .unwrap()
            .vertical_ids
            .is_empty()
    );
    assert_eq!(
        stem_parts_lookup::section_centroid(&section).unwrap(),
        [20., 12.5]
    );
    let mut stop = p.clone();
    stop.last_head_y = Some(12.5);
    assert!(
        stem_parts_lookup::collect(std::slice::from_ref(&section), &[], &area, None, &stop)
            .unwrap()
            .vertical_ids
            .is_empty()
    );
    stop.last_head_y = Some(12.5f64.next_up());
    assert_eq!(
        stem_parts_lookup::collect(std::slice::from_ref(&section), &[], &area, None, &stop)
            .unwrap()
            .vertical_ids,
        vec![1]
    );
    let mut bad = p.clone();
    bad.slope = 0.01;
    assert!(
        stem_parts_lookup::collect(std::slice::from_ref(&section), &[], &area, None, &bad).is_err()
    );
    bad = p.clone();
    bad.theoretical_line = [20., 0., 20., 0.];
    assert!(stem_parts_lookup::collect(&[], &[], &area, None, &bad).is_err());
    assert!(
        stem_parts_lookup::collect(&[section.clone(), section.clone()], &[], &area, None, &p)
            .is_err()
    );
    assert!(
        stem_parts_lookup::collect(&[], std::slice::from_ref(&section), &area, None, &p).is_err()
    );
    let mut large = section.clone();
    large.first_pos = 3_000_000;
    large.runs = vec![[0, 400]];
    assert_eq!(
        stem_parts_lookup::section_centroid(&large).unwrap_err(),
        "unsupported_stem_lookup_centroid_int_overflow"
    );
    assert_eq!(json!(section), before);
}

#[test]
fn raw_chunk_glyph_does_not_require_invertible_rough_endpoints() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-chunk-glyph/observed.json"
    ))
    .unwrap();
    let sections: Vec<SeedSection> =
        serde_json::from_value(f["vertical_sections"].clone()).unwrap();
    let p = Parameters {
        theoretical_line: [530., 230., 530., 265.],
        ..parameters()
    };
    let area = [[526., 230.], [534., 230.], [534., 265.], [526., 265.]];
    let selected = stem_parts_lookup::collect(&sections, &[], &area, None, &p).unwrap();
    assert_eq!(selected.vertical_ids, vec![1]);
    assert_eq!(
        selected.raw_glyphs,
        vec![f["candidates"][0]["glyph"].clone()]
    );
    assert!(
        audiveris_native::vertical_sticks::retrieve(&sections, &[], 5, 1)
            .unwrap_err()
            .starts_with("unsupported_vertical_endpoint_geometry")
    );
}
