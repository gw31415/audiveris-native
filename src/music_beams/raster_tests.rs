// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
#[test]
fn original_declared_sloping_contours_and_normalization_boundary_pixels_match() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-beam-sloped-raster/observed.json"
    ))
    .unwrap();
    for expected in source["cases"].as_array().unwrap() {
        assert_eq!(
            json!(area_rows(expected, 11).unwrap()),
            expected["white_rows"],
            "{}",
            expected["median"]
        );
    }
}
pub(crate) fn assert_real_rasters(owner: &crate::stem_image_owned::OwnedItems) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-areas/image-observed.json"
    ))
    .unwrap();
    let before = owner.graph.snapshot();
    let mut count = 0;
    for system in source["systems"].as_array().unwrap() {
        for expected in system["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter")))
        {
            let keys = owner.graph.live_inter_keys(
                system["system"].as_u64().unwrap(),
                expected["kind"].as_str().unwrap(),
            );
            let matches = keys
                .iter()
                .filter_map(|k| {
                    let (body, _) = owner.graph.retained_inter(*k).unwrap();
                    (body["bounds"] == expected["bounds"]).then_some(body)
                })
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            let body = matches[0];
            for i in 0..4 {
                assert!(
                    (body["median"][i].as_f64().unwrap() - expected["median"][i].as_f64().unwrap())
                        .abs()
                        < 1e-8
                );
            }
            assert!(
                (body["height"].as_f64().unwrap() - expected["height"].as_f64().unwrap()).abs()
                    < 1e-8
            );
            assert_eq!(
                json!(area_rows(body, owner.interline).unwrap()),
                expected["white_rows"]
            );
            count += 1;
        }
    }
    assert_eq!(count, 18);
    assert_eq!(owner.graph.snapshot(), before);
}
