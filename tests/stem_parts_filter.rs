// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_parts_filter_controls.rs"]
mod controls;
use audiveris_native::{
    beam_dispatch,
    stem_parts_alignment::{Parameters as Alignment, Part},
    stem_parts_filter::{self, Kind, Linker, Parameters},
};
use serde_json::{Value, json};
fn glyph(y: i32) -> audiveris_staff_removal::GlyphPixels {
    beam_dispatch::glyph_from_json(
        &json!({"bounds":[20,y,1,3],"orientation":"VERTICAL","runs":[[0,0,3]]}),
    )
    .unwrap()
}
#[test]
fn original_4152_real_head_beam_filter_controls_match_null_stumps_clipping_aliases_and_items() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-parts-filter/controls.json"
    ))
    .unwrap();
    let n = controls::observe(&f);
    assert_eq!(controls::compare(&f, &n), 0.);
    assert!(f["cases"] == n["cases"]);
    let c = f["cases"].as_array().unwrap();
    assert_eq!(c.len(), 4152);
    assert_eq!(f["builders_observed"], 612);
    assert_eq!(
        c.iter().filter(|x| x["last_head_y"].is_number()).count(),
        312
    );
    assert_eq!(
        c.iter().filter(|x| x["start_stump_id"].is_null()).count(),
        1974
    );
    assert_eq!(c.iter().filter(|x| x["y_range"][2] == 0).count(), 1680);
}
#[test]
fn filter_inclusive_head_stop_y_only_clipping_empty_ranges_and_invalid_identity_preserve_inputs() {
    let a = glyph(30);
    let b = glyph(32);
    let parts = [Part { id: 1, glyph: &a }, Part { id: 2, glyph: &b }];
    let head = Linker {
        id: 1,
        kind: Kind::HeadCorner,
        stump: None,
        reference: [20., 33.],
        beam_height: None,
    };
    let p = Parameters {
        alignment: Alignment::new(1, 1.65, 44., 0.).unwrap(),
        y_range: [20, 20, 0, 40],
        start_is_beam: true,
        head_box: None,
    };
    let before = beam_dispatch::glyph_json(&a);
    let n = stem_parts_filter::filter(&parts, None, &[head], &p).unwrap();
    assert_eq!(n.retained_part_order, vec![1]);
    assert_eq!(n.last_head_y, Some(33.));
    assert_eq!(n.items.last().unwrap().contrib, 3);
    let mut fractional = head;
    fractional.reference[1] = 33.1;
    assert_eq!(
        stem_parts_filter::filter(&parts, None, &[fractional], &p)
            .unwrap()
            .retained_part_order,
        vec![1, 2]
    );
    let mut native = p.clone();
    native.start_is_beam = false;
    native.head_box = Some([1000, 33, 0, 1]);
    let n = stem_parts_filter::filter(&parts, None, &[head], &native).unwrap();
    assert_eq!(n.retained_part_order, vec![1, 2]);
    assert_eq!(n.items.iter().filter(|i| i.kind == "GlyphItem").count(), 1); // y overlap excludes b regardless x/width; touching a stays
    native.y_range[3] = 0;
    let n = stem_parts_filter::filter(&parts, None, &[], &native).unwrap();
    assert!(n.items.is_empty());
    assert_eq!(n.retained_part_order, vec![1, 2]);
    let mut invalid = p.clone();
    invalid.alignment.slope = 0.01;
    assert!(stem_parts_filter::filter(&parts, None, &[head], &invalid).is_err());
    invalid = p.clone();
    invalid.y_range[3] = -1;
    assert!(stem_parts_filter::filter(&parts, None, &[], &invalid).is_err());
    let mut bad = head;
    bad.reference[0] = f64::NAN;
    assert!(stem_parts_filter::filter(&parts, None, &[bad], &p).is_err());
    let mut conflicting = head;
    conflicting.reference[1] = 34.;
    assert!(stem_parts_filter::filter(&parts, None, &[head, conflicting], &p).is_err());
    assert!(
        stem_parts_filter::filter(&[parts[0], Part { id: 1, glyph: &b }], None, &[], &p).is_err()
    );
    assert!(stem_parts_filter::filter(&parts, None, &[head; 32], &p).is_err());
    let too_many = vec![parts[0]; audiveris_native::stem_glyph::MAX_PARTS + 1];
    assert!(stem_parts_filter::filter(&too_many, None, &[], &p).is_err());
    assert_eq!(beam_dispatch::glyph_json(&a), before);
    let n = stem_parts_filter::filter(&[], None, &[], &p).unwrap();
    assert!(
        n.items.is_empty()
            && n.retained_part_order.is_empty()
            && n.retained_linker_order.is_empty()
    );
}
