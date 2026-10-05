// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_chunk_head_filter_controls.rs"]
mod controls;
use audiveris_native::{
    beam_dispatch::{self, Registry},
    stem_chunk_head_filter,
};
use serde_json::json;
#[test]
fn original_actual_head_chunks_vip_remaining_14_15_16_and_zero_overlap_match() {
    let original = serde_json::from_str(include_str!(
        "../fixtures/official-stem-chunk-head-filter/controls.json"
    ))
    .unwrap();
    let result = controls::observe(&original);
    assert_eq!(result["groups"].as_array().unwrap().len(), 600);
    assert_eq!(
        result["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| g["cases"].as_array().unwrap().len())
            .sum::<usize>(),
        6600
    );
}
#[test]
fn physical_mask_vip_and_nonpositive_overlap_differ_from_bbox_without_mutation() {
    let head = beam_dispatch::glyph_from_json(
        &json!({"bounds":[10,10,2,5],"orientation":"VERTICAL","runs":[[0,0,1],[0,4,1],[1,1,1]]}),
    )
    .unwrap();
    let chunk = beam_dispatch::glyph_from_json(
        &json!({"bounds":[10,10,2,5],"orientation":"VERTICAL","runs":[[0,1,3],[1,0,1],[1,2,3]]}),
    )
    .unwrap();
    let zero = beam_dispatch::glyph_from_json(
        &json!({"bounds":[10,15,1,1],"orientation":"VERTICAL","runs":[[0,0,1]]}),
    )
    .unwrap();
    let last = beam_dispatch::glyph_from_json(
        &json!({"bounds":[10,14,1,1],"orientation":"VERTICAL","runs":[[0,0,1]]}),
    )
    .unwrap();
    let mut r = Registry::default();
    let id = r.register(&chunk).unwrap();
    let z = r.register(&zero).unwrap();
    let last_id = r.register(&last).unwrap();
    let before = format!("{r:?}");
    let nonvip = stem_chunk_head_filter::filter(&r, &[id, id, z], &head, false).unwrap();
    assert_eq!(nonvip.remaining_weights, vec![Some(7), Some(7), None]);
    assert_eq!(nonvip.retained_ids, vec![id, id, z]);
    let vip = stem_chunk_head_filter::filter(&r, &[id, id, z], &head, true).unwrap();
    assert_eq!(vip.retained_ids, vec![z]);
    let boundary = stem_chunk_head_filter::filter(&r, &[last_id, z], &head, true).unwrap();
    assert_eq!(boundary.remaining_weights, vec![Some(0), None]);
    assert_eq!(boundary.retained_ids, vec![z]);
    assert!(stem_chunk_head_filter::filter(&r, &[999], &head, false).is_err());
    assert!(stem_chunk_head_filter::filter(&r, &[id; 4097], &head, false).is_err());
    let mut bad = head.clone();
    bad.runs[0].length = 0;
    assert!(stem_chunk_head_filter::filter(&r, &[], &bad, false).is_err());
    assert!(
        stem_chunk_head_filter::filter(&r, &[], &head, true)
            .unwrap()
            .retained_ids
            .is_empty()
    );
    assert_eq!(format!("{r:?}"), before);
}
