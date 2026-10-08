// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(crate) fn assert_real_brace_glyphs(owner: &mut crate::stem_image_owned::OwnedItems) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-grid-overlap/observed.json"
    ))
    .unwrap();
    let graph = owner.graph.snapshot();
    let held = owner.grid_glyphs.clone();
    let storage = owner.grid_glyphs.as_ptr();
    owner.upstream_report["grid_ir"]["glyphs"] = json!([]);
    let mut checked = 0;
    for expected in source["actual"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "BraceInter")
    {
        let keys = owner
            .graph
            .live_inter_keys(expected["system"].as_u64().unwrap(), "BraceInter");
        let matching = keys
            .iter()
            .filter(|key| {
                owner.graph.retained_inter(**key).unwrap().0["bounds"] == expected["bounds"]
            })
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        let glyph = owner.grid_physical_glyph(*matching[0]).unwrap();
        assert_eq!(beam_dispatch::glyph_json(glyph), expected["glyph"]);
        assert!(held.iter().any(|old| std::sync::Arc::ptr_eq(old, glyph)));
        let mut canvas = GrayImage::new(
            glyph.left as usize + glyph.width + 1,
            glyph.top as usize + glyph.height + 1,
            vec![
                0;
                (glyph.left as usize + glyph.width + 1) * (glyph.top as usize + glyph.height + 1)
            ],
        )
        .unwrap();
        glyph_pixels_erase(&mut canvas, glyph);
        assert_eq!(
            canvas.pixels.iter().filter(|p| **p == 255).count(),
            glyph.runs.iter().map(|r| r.length).sum::<usize>()
        );
        checked += 1;
    }
    assert_eq!(checked, 2);
    assert_eq!(owner.grid_glyphs.as_ptr(), storage);
    assert_eq!(owner.graph.snapshot(), graph);
}
