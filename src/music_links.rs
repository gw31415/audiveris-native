// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original beam scheduling preflight and owned links/cleaner continuation,
//! plus the bounded retained-owner LINKS/cleaner continuation.
use crate::{
    music_measures::OwnedMusic,
    shared_sig::{Key, Phase},
    stem_image_owned::OwnedItems,
};
use serde_json::json;

/// Preflight the actual retained beam catalogs before original VLinker scheduling.
/// Empty builders and cross-system orphan beams must survive until linkStems /
/// Finalizer / SigReducer; the former horizontal-envelope shortcut is obsolete.
pub(crate) fn prepare_beam_linking(mut o: OwnedItems) -> Result<OwnedItems, String> {
    let initial = o.graph.snapshot();
    let nodes = initial["inters"]
        .as_array()
        .ok_or("music_beam_SIG_required")?;
    if nodes.iter().any(|n| n["kind"] == "StemInter") {
        return Err("music_preexisting_stem_pending".into());
    }
    if o.upstream_report["line_completion"]["global_slope"] != 0.0 {
        return Err("music_beam_zero_skew_required".into());
    }
    let generations =
        &o.prepared.report["head_chunks_plan"]["head_parts_plan"]["beam_generation_plan"];
    let mut scheduled = Vec::new();
    for n in nodes.iter().filter(|n| {
        matches!(
            n["kind"].as_str(),
            Some("BeamInter" | "BeamHookInter" | "SmallBeamInter")
        )
    }) {
        if n["kind"] == "SmallBeamInter" {
            return Err("music_small_beam_pending".into());
        }
        let key: Key = serde_json::from_value(n["origin"].clone())
            .map_err(|_| "music_beam_origin_required")?;
        let sys = generations["systems"]
            .as_array()
            .ok_or("music_beam_generation_required")?
            .iter()
            .find(|s| s["system"] == key.system)
            .ok_or("music_beam_system_required")?;
        sys["beam_catalog"]
            .as_array()
            .ok_or("music_beam_catalog_required")?
            .iter()
            .find(|b| b["beam"]["id"] == key.local)
            .ok_or("music_beam_catalog_identity_required")?;
        scheduled.push(n["id"].clone());
    }
    o.prepared.report["owned_beam_linking"] = json!({"scheduled_beam_ids":scheduled,"scope":"original actual VLinker side/intermediate scheduling including empty builders; orphan removal deferred to ordered Finalizer and endpoint reduction","stems_complete":false});
    Ok(o)
}

/// SigReducer.contextualizeAndPurge on unsupported isolated symbols has no
/// support contribution: contextual grade equals intrinsic grade. Containment
/// is not a support relation. Weak rest removal also empties its rest chord.
pub(crate) fn continue_links(mut m: OwnedMusic) -> Result<OwnedMusic, String> {
    let mut graph = m.owner.graph.clone();
    let alters = crate::music_alters::purge_unlinked(&mut graph)?;
    let flags = crate::music_flags::contextualize(&mut graph)?;
    let pauses = crate::music_pauses::contextualize_and_purge(&mut graph, &m.chords)?;
    let reduction = crate::music_symbol_reduction::reduce(&mut graph)?;
    let removed_beam_head_supports = crate::music_beams::clean_head_supports(&mut graph)?;
    let containers =
        crate::music_containers::refresh(&graph, &m.measures, &mut m.chords, &mut m.voices)?;
    let empty_chords = containers.removed_chords;
    let dot_counting = crate::music_dots::contextualize_and_count(&mut graph, &m.chords)?;
    m.owner.graph = graph;
    m.owner.prepared.report["shared_sig"] = m.owner.graph.snapshot();
    m.owner.prepared.report["limited_links"] = json!({"removed_weak_symbol_ids":reduction["removed_weak_symbol_ids"],"symbol_conflict_reduction":reduction,"removed_empty_chord_ids":empty_chords,"scope":"Rest/TimeWhole physical overlaps with incoming zero-target augmentation support, contextualization/weak purge/exclusion reduction; actual member/container cleanup; admitted single-voice path has no lyrics/slurs/tremolos; temporary BeamHead relations are removed by the connected BeamHeadCleaner","flag_support_and_weak_purge":flags,"full_LINKS_complete":false,"heads_complete":false,"stems_complete":false,"epilog_complete":false});
    if !removed_beam_head_supports.is_empty() {
        m.owner.prepared.report["limited_links"]["BeamHeadCleaner_removed_relation_ids"] =
            json!(removed_beam_head_supports);
    }
    m.owner.prepared.report["limited_links"]["containers"] = containers.projection;
    m.owner.prepared.report["limited_links"]["dot_counting"] = dot_counting;
    m.owner.prepared.report["limited_links"]["pause_weak_purge"] = pauses;
    m.owner.prepared.report["limited_links"]["alter_weak_purge"] = alters;
    Ok(m)
}

/// Post-reduction single-voice admission/cache checks and beam endpoints.
/// Original automatic checkHeadStems already runs after each system links,
/// before Finalizer.boostBeamSides. General epilog/cross-system parity is pending.
pub(crate) fn finalize_stems(mut o: OwnedItems) -> Result<OwnedItems, String> {
    if o.prepared.report["ordered_foundation_consistency"]["check_order_and_per_check_purge_connected"]
        != true
    {
        return Err("music_ordered_foundation_consistency_required".into());
    }
    let consistency = o.prepared.report["ordered_foundation_consistency"].clone();
    let removed_stems = o.prepared.report["removed_cached_stems"].clone();
    let beam_checks = json!({"removed_beam_ids":consistency["removed_beam_ids"],"scope":"original hook/beam checks in six-check order, each followed by contextualization/owned weak purge; full reduction pending"});
    let snap = o.graph.snapshot();
    let nodes = snap["inters"]
        .as_array()
        .ok_or("music_finalizer_SIG_required")?;
    let mut checked = Vec::new();
    for head in nodes.iter().filter(|n| n["kind"] == "HeadInter") {
        let key: Key = serde_json::from_value(head["origin"].clone())
            .map_err(|_| "music_finalizer_head_origin_required")?;
        let edges = snap["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["kind"] == "HeadStemRelation" && r["source"] == head["id"])
            .collect::<Vec<_>>();
        if edges.len() != 1 {
            return Err("music_finalizer_one_stem_per_head_required".into());
        }
        let stem = nodes
            .iter()
            .find(|n| n["id"] == edges[0]["target"])
            .ok_or("music_finalizer_stem_required")?;
        let sk: Key = serde_json::from_value(stem["origin"].clone())
            .map_err(|_| "music_finalizer_stem_origin_required")?;
        if sk.phase != Phase::Stem
            || o.caches
                .get(&sk.system)
                .ok_or("music_finalizer_cache_required")?
                .registered_stem(&o.graph, sk.local)?
                .snapshot()?["id"]
                != stem["id"]
        {
            return Err("music_finalizer_canonical_cache_required".into());
        }
        let check = crate::stem_cleaner::clean_head(&mut o.graph, key, &[sk])?;
        checked.push(
            json!({"head_id":head["id"],"removed_relation_ids":check["removed_relation_ids"]}),
        );
    }
    o.graph.contextualize_head_stems()?;
    o.prepared.report["shared_sig"] = o.graph.snapshot();
    o.prepared.report["limited_stem_finalizer"] = json!({"head_checks":checked,"canonical_stem_cache_checked":true,"no_live_beam_finalizer":!nodes.iter().any(|n|matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))),"contextualize_after_epilog":true,"full_STEMS_epilog_complete":false});
    o.prepared.report["limited_stem_finalizer"]["original_stem_consistency"] = json!({"ordered_foundation_consistency":consistency,"removed_cached_stems":removed_stems,"scope":"ordinary original six-check consistency/per-check weak purge and post-consistency exclusion/epoch order; general overlap/chord/frozen/late reduction remains pending"});
    if nodes
        .iter()
        .any(|n| matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter")))
    {
        o.prepared.report["limited_stem_finalizer"]["beam_checks"] = beam_checks;
    }
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weak_rest_removes_actual_container_without_replacing_stem_cache_objects() {
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-half-rest/half-rest-y240.png"
        ))
        .unwrap();
        let o = crate::stem_image_owned::prepare_items(&image).unwrap();
        let before = o.graph.snapshot();
        let o = prepare_beam_linking(o).unwrap();
        assert_eq!(o.graph.snapshot(), before);
        assert_eq!(
            o.prepared.report["owned_beam_linking"]["scheduled_beam_ids"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let o = crate::stem_image_head_linking::continue_process(o).unwrap();
        let o = finalize_stems(crate::stem_foundation_consistency::continue_reduction(o).unwrap())
            .unwrap();
        let m = crate::music_measures::continue_measures(o).unwrap();
        let m = crate::music_symbols::continue_symbols(m).unwrap();
        let before = m.owner.graph.snapshot();
        let cached = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "StemInter")
            .map(|n| {
                let k: Key = serde_json::from_value(n["origin"].clone()).unwrap();
                let s = m.owner.caches[&k.system]
                    .registered_stem(&m.owner.graph, k.local)
                    .unwrap();
                (k, s.clone(), s.snapshot().unwrap())
            })
            .collect::<Vec<_>>();
        assert_eq!(m.chords.len(), 5);
        let m = continue_links(m).unwrap();
        assert_eq!(m.chords.len(), 4);
        assert_eq!(m.layout()["rest_chords"].as_array().unwrap().len(), 0);
        let links = &m.owner.prepared.report["limited_links"];
        assert_eq!(
            links["removed_weak_symbol_ids"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            links["removed_empty_chord_ids"].as_array().unwrap().len(),
            1
        );
        let after = m.owner.graph.snapshot();
        let removed = links["removed_weak_symbol_ids"][0].clone();
        let containers = &links["containers"];
        assert!(
            containers["staff_notes"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| !s["note_ids"].as_array().unwrap().contains(&removed))
        );
        assert!(
            containers["measure_chords"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| s["rest_chord_ids"].as_array().unwrap().is_empty())
        );
        for c in &m.chords {
            assert!(
                c.members
                    .iter()
                    .all(|n| after["inters"].as_array().unwrap().contains(n))
            );
        }

        assert!(
            after["tombstones"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["id"] == removed)
        );
        assert!(
            after["relations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|n| n["source"] != removed && n["target"] != removed)
        );
        for (k, s, state) in cached {
            assert!(std::sync::Arc::ptr_eq(
                &s,
                &m.owner.caches[&k.system]
                    .registered_stem(&m.owner.graph, k.local)
                    .unwrap()
            ));
            assert_eq!(s.snapshot().unwrap(), state);
        }
    }
}
