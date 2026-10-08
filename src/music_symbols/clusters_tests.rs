// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
fn fingerprint(g: &GlyphPixels) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&beam_dispatch::glyph_json(g)).unwrap())
    )
}
pub(super) fn assert_actual_image_clusters(music: &OwnedMusic, cleaned: &GrayImage) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-clusters/image-observed.json"
    ))
    .unwrap();
    let glyphs = extract_glyphs(
        BinaryPage::new(cleaned.width, cleaned.height, &cleaned.pixels).unwrap(),
        Orientation::Vertical,
        (0, 0),
        Limits::default(),
    )
    .unwrap();
    let mut count = 0;
    let mut trials = 0;
    let mut physical = 0;
    let mut largest = 0;
    for group in geometry::dispatch(&music.owner, &glyphs).unwrap() {
        let expected = source["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| Some(group.system.id) == s["system"].as_u64())
            .unwrap();
        let fine_boxes =
            actual_fine_boxes(&music.owner.graph, group.system.id, music.owner.interline).unwrap();
        assert_eq!(json!(fine_boxes.len()), expected["fine_boxes"]);
        let parts = group.glyphs.into_iter().cloned().collect::<Vec<_>>();
        let eligible = parts
            .iter()
            .filter(|g| clusters::eligible(g, music.owner.interline, &fine_boxes))
            .map(fingerprint)
            .collect::<Vec<_>>();
        assert_eq!(
            json!(eligible),
            expected["physical_glyph_sha256"],
            "System {} physical glyphs",
            group.system.id
        );
        physical += eligible.len();
        let actual = clusters::components(&parts, music.owner.interline, &fine_boxes).unwrap();
        let golden = expected["clusters"].as_array().unwrap();
        assert_eq!(actual.len(), golden.len());
        for (i, (cluster, expected)) in actual.iter().zip(golden).enumerate() {
            assert_eq!(
                json!(cluster.all_part_count),
                expected["all_part_count"],
                "System {} cluster {i}",
                group.system.id
            );
            largest = largest.max(cluster.all_part_count);
            let physical = cluster.parts.iter().map(fingerprint).collect::<Vec<_>>();
            let original = expected["selected_parts_sha256"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            assert_eq!(
                physical.iter().collect::<BTreeSet<_>>(),
                original.iter().collect::<BTreeSet<_>>(),
                "System {} cluster {i} selected parts",
                group.system.id
            );
            let order = original
                .iter()
                .map(|id| physical.iter().position(|p| p == id).unwrap())
                .collect::<Vec<_>>();
            let original_order = cluster.in_observed_part_order(&order);
            let actual_trials = original_order.trials(music.owner.interline, group.system.left);
            let rows=actual_trials.iter().map(|members| {
                let mask=members.iter().fold(0,|m,&i|m|1<<i);
                let g=crate::clef_candidates::compound(&original_order.parts,mask);
                json!({"parts":members,"glyph_sha256":fingerprint(&g),"bounds":beam_dispatch::glyph_json(&g)["bounds"],"weight":crate::clef_candidates::weight(&g)})
            }).collect::<Vec<_>>();
            assert_eq!(rows.len(), expected["trials"].as_array().unwrap().len());
            for (t, (actual, golden)) in rows
                .iter()
                .zip(expected["trials"].as_array().unwrap())
                .enumerate()
            {
                assert_eq!(
                    actual, golden,
                    "System {} cluster {i} original-order trial {t}",
                    group.system.id
                );
            }
            let native_trials = cluster.trials(music.owner.interline, group.system.left);
            let mut native_candidates = native_trials
                .iter()
                .map(|members| {
                    let mask = members.iter().fold(0, |m, &i| m | 1 << i);
                    fingerprint(&crate::clef_candidates::compound(&cluster.parts, mask))
                })
                .collect::<Vec<_>>();
            let mut original_candidates = rows
                .iter()
                .map(|r| r["glyph_sha256"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            native_candidates.sort();
            original_candidates.sort();
            assert_eq!(
                native_candidates, original_candidates,
                "System {} cluster {i} candidate multiset",
                group.system.id
            );
            trials += rows.len();
            count += 1;
        }
    }
    assert_eq!((physical, count, trials, largest), (543, 176, 788, 23));
}
