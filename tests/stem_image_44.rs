// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_image_head_expansion_controls.rs"]
mod controls;
use audiveris_native::{GrayImage, stem_image_head_expansion, stem_image_owned};
use serde_json::Value;
#[test]
fn automatic_44_image_catalog_parts_grades_cache_match_original_without_sig_replacement() {
    let image = GrayImage::decode(include_bytes!(
        "../fixtures/official-stem-image-44/quarter-melody.png"
    ))
    .unwrap();
    let owned = stem_image_owned::prepare_items(&image).unwrap();
    assert_eq!(
        owned.upstream_report["head_registration"]["template_catalog"]["stored_point_size"],
        44
    );
    let before = format!("{:?}", owned.graph);
    let owned = stem_image_head_expansion::continue_expansion(owned).unwrap();
    let oracle: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-image-44/controls.json"
    ))
    .unwrap();
    controls::compare(&oracle, &owned.prepared.report);
    assert_eq!(owned.prepared.report["corner_count"], 172);
    assert_eq!(owned.prepared.report["accepted_creations"], 16);
    assert_eq!(owned.prepared.report["systems"][0]["cache_size"], 8);
    assert_eq!(format!("{:?}", owned.graph), before);
}
