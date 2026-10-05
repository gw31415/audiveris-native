// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded automatic weak-candidate pass after actual image head/stem linking.
//! Original overlap/chord exclusions, subsequent checks/reduction and epilog
//! are pending. A weak stem refuses before mutation until cache removal is wired.
use crate::stem_image_owned::OwnedItems;
use serde_json::json;

/// Consume the retained owner; no serialized Graph, IDs or selected grades input.
pub fn continue_purge(mut owned: OwnedItems) -> Result<OwnedItems, String> {
    if owned.prepared.report["schema"] != "audiveris-native.stem-image-head-linking.v1"
        || owned.prepared.report["shared_sig"] != owned.graph.snapshot()
    {
        return Err("actual_linked_image_SIG_owner_required".into());
    }
    let removed = owned.graph.purge_weak_foundations()?;
    let linking = std::mem::take(&mut owned.prepared.report);
    owned.prepared.report = json!({
        "schema":"audiveris-native.stem-foundation-weak-purge.v1",
        "corner_count":linking["corner_count"],
        "linked_head_stems":linking,
        "removed_native_vertex_ids":removed,
        "shared_sig":owned.graph.snapshot(),
        "actual_live_Graph_updated":true,
        "cached_stem_objects_retained":true,
        "comparison_scope":"Original SIGraph.deleteWeakInters threshold on native image SIG; quarter retained head/stem/HeadStem subset matches XML-loaded first purge, overlap/chord exclusions not yet reproduced",
        "unsupported":["weak StemInter cache removal callback","overlap/chord exclusions and full SigReducer","finalizeStems/cleaner/epilog","pitch/duration/rests/rhythm/export"],
        "heads_complete":false,"stems_complete":false,"musicxml_complete":false,
        "epilog_complete":false,"automatic_raster_pipeline_complete":false
    });
    Ok(owned)
}
