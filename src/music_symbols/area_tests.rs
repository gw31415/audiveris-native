// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(crate) fn assert_real_vertical_areas(owner: &crate::stem_image_owned::OwnedItems) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-areas/image-observed.json"
    ))
    .unwrap();
    let before = owner.graph.snapshot();
    let mut checked = 0;
    for system in source["systems"].as_array().unwrap() {
        for expected in system["inters"].as_array().unwrap().iter().filter(|n| {
            matches!(
                n["kind"].as_str(),
                Some("BarlineInter" | "BarConnectorInter")
            )
        }) {
            let keys = owner.graph.live_inter_keys(
                system["system"].as_u64().unwrap(),
                expected["kind"].as_str().unwrap(),
            );
            let matches = keys
                .iter()
                .filter_map(|key| {
                    let (body, _) = owner.graph.retained_inter(*key).unwrap();
                    (Some(key.system) == system["system"].as_u64()
                        && body["kind"] == expected["kind"]
                        && body["bounds"] == expected["bounds"])
                        .then_some((key, body))
                })
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            let (_, body) = matches[0];
            assert_eq!(body["median"], expected["median"]);
            assert_eq!(body["width"], expected["width"]);
            assert_eq!(
                json!(vertical_area_rows(body, owner.interline).unwrap()),
                expected["white_rows"]
            );
            if body["kind"] == "BarConnectorInter" {
                assert!(body["glyph"].is_null());
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 30);
    assert_eq!(owner.graph.snapshot(), before);
}
