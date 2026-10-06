// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded automatic weak-candidate pass after actual image head/stem linking.
//! Original overlap/chord exclusions, subsequent checks/reduction and epilog
//! are pending. Weak stems update the actual retained canonical cache objects.
use crate::stem_image_owned::OwnedItems;
use serde_json::json;

/// Reusable original weak-purge boundary after native contextualization. Only
/// retained owners can commit staged SIG removals and canonical cache callbacks.
pub(crate) fn purge_owned(
    owned: &mut OwnedItems,
) -> Result<(Vec<usize>, Vec<serde_json::Value>), String> {
    let (graph, removed) = owned.graph.prepare_weak_foundations()?;
    let ids = removed.iter().map(|id| *id as u64).collect();
    let stems = crate::stem_head_foundations::commit_reduction_graph(owned, graph, &ids)?;
    Ok((removed, stems))
}

/// Consume the retained owner; no serialized Graph, IDs or selected grades input.
pub fn continue_purge(mut owned: OwnedItems) -> Result<OwnedItems, String> {
    if owned.prepared.report["schema"] != "audiveris-native.stem-image-head-linking.v1"
        || owned.prepared.report["shared_sig"] != owned.graph.snapshot()
    {
        return Err("actual_linked_image_SIG_owner_required".into());
    }
    let (removed, removed_stems) = purge_owned(&mut owned)?;
    let linking = std::mem::take(&mut owned.prepared.report);
    owned.prepared.report = json!({
        "schema":"audiveris-native.stem-foundation-weak-purge.v1",
        "corner_count":linking["corner_count"],
        "linked_head_stems":linking,
        "removed_native_vertex_ids":removed,
        "removed_cached_stems":removed_stems,
        "shared_sig":owned.graph.snapshot(),
        "actual_live_Graph_updated":true,
        "cached_stem_objects_retained":true,
        "actual_live_Graph_and_cache_updated":true,
        "comparison_scope":"Original SIGraph.deleteWeakInters threshold/exemptions on native image SIG; weak stems mark the same cached object removed; declared weak stem conditions and retained same-image outputs are distinct, overlap/chord exclusions and full per-check weak purge not yet reproduced",
        "unsupported":["overlap/chord exclusions and full SigReducer/per-check weak purge","full cleaner/epilog","full pitch/duration/rests/rhythm/export"],
        "heads_complete":false,"stems_complete":false,"musicxml_complete":false,
        "epilog_complete":false,"automatic_raster_pipeline_complete":false
    });
    Ok(owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GrayImage, beam_dispatch, stem_checks, stem_glyph};
    use serde_json::Value;

    #[test]
    fn original_weak_created_stem_removes_same_cache_atomically_without_glyph_loss() {
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-cross-system/beam-two-systems.png"
        ))
        .unwrap();
        let mut owned = crate::stem_image_head_linking::continue_process(
            crate::stem_image_owned::prepare_items(&image).unwrap(),
        )
        .unwrap();
        // Existing original createStem raster condition, not a claim that this
        // MusicXML image itself creates the extra weak stem. Grade is computed
        // by the native translated checker, never patched into the live SIG.
        let fixture: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-builder-create/controls.json"
        ))
        .unwrap();
        let case = &fixture["cases"][48];
        let parts = case["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| beam_dispatch::glyph_from_json(v).unwrap())
            .collect::<Vec<_>>();
        let glyph = stem_glyph::assemble(&parts).unwrap();
        let width = fixture["image_width"].as_u64().unwrap() as usize;
        let height = fixture["image_height"].as_u64().unwrap() as usize;
        let mut raster = GrayImage {
            width,
            height,
            pixels: vec![255; width * height],
        };
        for run in &glyph.runs {
            for y in run.start..run.start + run.length {
                raster.pixels
                    [(glyph.top as usize + y) * width + glyph.left as usize + run.position] = 0;
            }
        }
        let params = stem_checks::Parameters::new(
            fixture["interline"].as_u64().unwrap() as usize,
            fixture["max_stem"].as_u64().unwrap() as usize,
            fixture["global_slope"].as_f64().unwrap(),
            case["profile"].as_u64().unwrap() as u8,
        )
        .unwrap();
        let cache = owned.caches.get_mut(&1).unwrap();
        let cached = cache
            .create_selected(&mut owned.prepared.registry, &parts, &raster, &params)
            .unwrap()
            .stem
            .unwrap();
        let id = cache
            .register(&owned.prepared.registry, &mut owned.graph, &cached)
            .unwrap();
        owned.graph.contextualize_head_stem_flags().unwrap();
        let before = owned.graph.snapshot();
        let node = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap();
        let grade = case["first"]["grade"].as_f64().unwrap();
        assert!((node["contextual_grade"].as_f64().unwrap() - grade).abs() < 1e-12);
        assert!(grade < 0.5);
        let state = cached.snapshot().unwrap();
        let registry = owned.prepared.registry.snapshot();
        let (staged, removed) = owned.graph.prepare_weak_foundations().unwrap();
        assert!(removed.contains(&id));
        let ids = removed.iter().map(|id| *id as u64).collect();
        let caches = std::mem::take(&mut owned.caches);
        assert!(
            crate::stem_head_foundations::commit_reduction_graph(&mut owned, staged, &ids).is_err()
        );
        assert_eq!(owned.graph.snapshot(), before);
        assert_eq!(cached.snapshot().unwrap(), state);
        owned.caches = caches;
        owned.prepared.report["shared_sig"] = before;
        let mut owned = continue_purge(owned).unwrap();
        assert_eq!(cached.snapshot().unwrap()["removed"], true);
        assert_eq!(cached.snapshot().unwrap()["glyph"], state["glyph"]);
        assert_eq!(owned.prepared.registry.snapshot(), registry);
        assert!(
            owned.prepared.report["removed_cached_stems"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["id"] == id)
        );
        assert!(
            owned.graph.snapshot()["tombstones"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["id"] == id)
        );
        assert!(
            owned
                .caches
                .get_mut(&1)
                .unwrap()
                .register(&owned.prepared.registry, &mut owned.graph, &cached)
                .is_err()
        );
    }
}
