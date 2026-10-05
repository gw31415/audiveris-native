// SPDX-License-Identifier: AGPL-3.0-or-later
//! Automatic image head consistency/competition subset on the retained owner.
//! This is not complete SigReducer, STEMS, final notes, rhythm or MusicXML.
use crate::{
    shared_sig::{Key, Phase},
    stem_head_foundation_plan::{self, Event},
    stem_image_owned::OwnedItems,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
/// No source IDs, selected grades, serialized Graph import or original outputs.
pub fn continue_reduction(mut owned: OwnedItems) -> Result<OwnedItems, String> {
    if owned.prepared.report["schema"] != "audiveris-native.stem-foundation-weak-purge.v1"
        || owned.prepared.report["shared_sig"] != owned.graph.snapshot()
    {
        return Err("actual_weak_purged_image_owner_required".into());
    }
    let initial = owned.graph.snapshot();
    let plan = stem_head_foundation_plan::prepare(&initial)?;
    let ids = plan
        .events
        .iter()
        .filter_map(|e| {
            if let Event::RemoveVertex(id) = e {
                Some(*id)
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    let mut removals = Vec::new();
    for n in initial["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "StemInter" && ids.contains(&n["id"].as_u64().unwrap()))
    {
        let key: Key = serde_json::from_value(n["origin"].clone())
            .map_err(|_| "invalid_foundation_stem_origin")?;
        if key.phase != Phase::Stem {
            return Err("invalid_foundation_stem_phase".into());
        }
        let cache = owned
            .caches
            .get(&key.system)
            .ok_or("missing_foundation_retriever")?;
        let stem = cache.registered_stem(&owned.graph, key.local)?;
        if stem.snapshot()?["id"] != n["id"] {
            return Err("stale_foundation_cache_graph_id".into());
        }
        removals.push(stem);
    }
    // Hold every affected canonical object's lock before any mutation. The
    // Graph is staged by clone; failure leaves Graph and all cache states intact.
    let mut guards = removals
        .iter()
        .map(|s| s.removal_guard())
        .collect::<Result<Vec<_>, _>>()?;
    if guards
        .iter()
        .any(|g| g["removed"] == true || g["in_sig"] != true)
    {
        return Err("inactive_foundation_removal_guard".into());
    }
    let mut graph = owned.graph.clone();
    graph.apply_head_foundation_plan(&plan)?;
    for state in &mut guards {
        state["removed"] = json!(true);
    }
    owned.graph = graph;
    drop(guards);
    let weak_purge = std::mem::take(&mut owned.prepared.report);
    let removed_stem_states = removals
        .iter()
        .map(|s| s.snapshot())
        .collect::<Result<Vec<Value>, _>>()?;
    owned.prepared.report = json!({"schema":"audiveris-native.stem-head-foundations.v1","corner_count":weak_purge["corner_count"],"weak_purge":weak_purge,"comparison_stage_counts":plan.stages,"shared_sig":owned.graph.snapshot(),"removed_cached_stems":removed_stem_states,"actual_live_Graph_and_cache_updated":true,"comparison_scope":"Original XML-loaded quarter retained heads/stems/HeadStem fields at ending/side checks and exclusion reduction; original overlap/chord/late consistency and full SIG exclusion parity remain unported","unsupported":["overlap/chord/late consistency and complete SigReducer","beam/ledger/frozen/percussion/stemless reduction scope","finalizeStems/cleaner/epilog","pitch/duration/rests/rhythm/export","nonzero skew/real scans/special staff/full24shape pages/32sources bound"],"heads_complete":false,"stems_complete":false,"musicxml_complete":false,"epilog_complete":false,"automatic_raster_pipeline_complete":false});
    owned.prepared.report["header_diatonic_pitch"] = crate::head_musical_pitch::header_candidates(
        &owned.graph.snapshot(),
        &owned.upstream_report,
    );
    Ok(owned)
}
