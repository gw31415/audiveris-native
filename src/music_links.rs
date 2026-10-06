// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original empty-head removal or delegation to connected positive beam builders,
//! plus the bounded retained-owner LINKS/cleaner continuation.
use crate::{
    music_measures::OwnedMusic,
    shared_sig::{Key, Phase},
    stem_image_owned::OwnedItems,
};
use serde_json::json;

/// Original VLinker.link returns false when StemBuilder.getCLinkers is empty.
/// A deliberately larger horizontal envelope than every original lookup area
/// proves that condition without implementing positive beam-origin builders.
/// Head bounds as well as all corner reference points must be outside it.
pub(crate) fn remove_unreachable_beams(mut o: OwnedItems) -> Result<OwnedItems, String> {
    let initial = o.graph.snapshot();
    let nodes = initial["inters"]
        .as_array()
        .ok_or("music_beam_SIG_required")?;
    if nodes.iter().any(|n| n["kind"] == "StemInter") {
        return Err("music_preexisting_stem_pending".into());
    }
    let generations =
        &o.prepared.report["head_chunks_plan"]["head_parts_plan"]["beam_generation_plan"];
    let contexts = &generations["beam_target_plan"]["head_linker_plan"]["source_plan"]["stump_plan"]
        ["seed_plan"]["head_plan"]["systems"];
    let image = crate::GrayImage::from_pgm(&o.upstream_output_pixels)?;
    let mut removals = Vec::new();
    let mut evidence = Vec::new();
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
        let cat = sys["beam_catalog"]
            .as_array()
            .ok_or("music_beam_catalog_required")?
            .iter()
            .find(|b| b["beam"]["id"] == key.local)
            .ok_or("music_beam_catalog_identity_required")?;
        // Positive standard beams now go through the actual VLinker builder,
        // profile4 cache/SIG scheduler; hooks use the system link profile
        // and their original one-side endpoint check.
        let physical = ["stumps", "admitted_seeds"]
            .iter()
            .any(|field| cat[*field].as_array().is_some_and(|v| !v.is_empty()));
        if physical {
            evidence.push(json!({"native_beam_id":n["id"],"scheduled_positive_beam":true}));
            continue;
        }
        let context = contexts
            .as_array()
            .ok_or("music_beam_context_required")?
            .iter()
            .find(|s| s["system"] == key.system)
            .ok_or("music_beam_context_system_required")?;
        let b: [f64; 4] = serde_json::from_value(n["median"].clone())
            .map_err(|_| "music_beam_median_required")?;
        if b[1] != b[3] || o.upstream_report["line_completion"]["global_slope"] != 0.0 {
            return Err("music_beam_zero_skew_required".into());
        }
        // Original halfBeamLuDx=.3 interline and slopeMargin=.015.
        // Endpoints lie inside the image. Ignoring closer beam limits and y
        // filtering only enlarges this envelope, including boundary contacts.
        let margin = 0.3 * o.interline as f64 + 0.015 * image.height as f64;
        let mut left = b[0].min(b[2]);
        let mut right = b[0].max(b[2]);
        for linker in cat["final_blinkers"]
            .as_array()
            .ok_or("music_final_blinkers_required")?
        {
            let x = linker["reference"][0]
                .as_f64()
                .filter(|x| x.is_finite())
                .ok_or("music_beam_linker_reference_required")?;
            left = left.min(x);
            right = right.max(x);
        }
        let left = left - margin;
        let right = right + margin;
        if nodes
            .iter()
            .filter(|h| h["kind"] == "HeadInter" && h["origin"]["system"] == key.system)
            .any(|h| {
                let x = h["bounds"][0].as_f64().unwrap_or(f64::NAN);
                let w = h["bounds"][2].as_f64().unwrap_or(f64::NAN);
                !x.is_finite() || !w.is_finite() || (x <= right && x + w >= left)
            })
            || context["image_context"]["corners"]
                .as_array()
                .ok_or("music_beam_corners_required")?
                .iter()
                .any(|c| {
                    c["reference"][0]
                        .as_f64()
                        .is_none_or(|x| x >= left && x <= right)
                })
        {
            evidence.push(json!({"native_beam_id":n["id"],"scheduled_positive_beam":true}));
            continue;
        }
        if initial["relations"].as_array().unwrap().iter().any(|r| {
            (r["source"] == n["id"] || r["target"] == n["id"])
                && !matches!(
                    r["kind"].as_str(),
                    Some("Exclusion" | "Containment" | "BeamBeamRelation")
                )
        }) {
            return Err("music_beam_existing_relation_pending".into());
        }
        removals.push(key);
        evidence.push(json!({"id":n["id"],"origin":key,"bounds":n["bounds"],"horizontal_lookup_superset":[left,right],"reachable_heads":0,"original_check":if n["kind"]=="BeamHookInter"{"checkHooksHaveStem"}else{"checkBeamsHaveBothStems"}}));
    }
    let removed_ids = removals
        .iter()
        .map(|k| o.graph.id(*k).unwrap())
        .collect::<Vec<_>>();
    let mut groups = Vec::new();
    for n in nodes.iter().filter(|n| n["kind"] == "BeamGroupInter") {
        if n["members"]
            .as_array()
            .ok_or("music_beam_group_members_required")?
            .iter()
            .any(|id| {
                id.as_u64()
                    .is_none_or(|id| !removed_ids.contains(&(id as usize)))
            })
        {
            if n["members"]
                .as_array()
                .unwrap()
                .iter()
                .any(|id| removed_ids.contains(&(id.as_u64().unwrap() as usize)))
            {
                return Err("music_mixed_removed_beam_group_pending".into());
            }
            continue;
        }
        groups.push(
            serde_json::from_value::<Key>(n["origin"].clone())
                .map_err(|_| "music_beam_group_origin_required")?,
        );
    }
    for key in removals {
        o.graph.remove_music_inter(key)?;
    }
    for key in groups {
        o.graph.remove_music_inter(key)?;
    }
    o.prepared.report["unreachable_beam_reduction"] = json!({"scope":"zero skew/no stump or seed/no possibly reachable head; original empty VLinker and orphan checks only","removed":evidence,"positive_beam_origin_linking_complete":false});
    Ok(o)
}

/// SigReducer.contextualizeAndPurge on unsupported isolated symbols has no
/// support contribution: contextual grade equals intrinsic grade. Containment
/// is not a support relation. Weak rest removal also empties its rest chord.
pub(crate) fn continue_links(mut m: OwnedMusic) -> Result<OwnedMusic, String> {
    let mut graph = m.owner.graph.clone();
    let flags = crate::music_flags::contextualize(&mut graph)?;
    let reduction = crate::music_symbol_reduction::reduce(&mut graph)?;
    let removed_beam_head_supports = crate::music_beams::clean_head_supports(&mut graph)?;
    let removed = reduction["removed_weak_symbol_ids"]
        .as_array()
        .unwrap()
        .iter()
        .chain(reduction["removed_conflict_symbol_ids"].as_array().unwrap())
        .map(|n| n.as_u64().unwrap())
        .collect::<Vec<_>>();
    let mut empty_chords = Vec::new();
    for c in &mut m.chords {
        c.members
            .retain(|n| !removed.contains(&n["id"].as_u64().unwrap()));
        if c.members.is_empty() {
            graph.remove_music_inter(c.key)?;
            empty_chords.push(c.id);
        }
    }
    m.chords.retain(|c| !c.members.is_empty());
    m.owner.graph = graph;
    m.owner.prepared.report["shared_sig"] = m.owner.graph.snapshot();
    m.owner.prepared.report["limited_links"] = json!({"removed_weak_symbol_ids":reduction["removed_weak_symbol_ids"],"symbol_conflict_reduction":reduction,"removed_empty_chord_ids":empty_chords,"scope":"unsupported-support-free Rest/TimeWhole physical overlaps, contextualization/weak purge/exclusion reduction; actual member/container cleanup; admitted single-voice path has no lyrics/slurs/tremolos; temporary BeamHead relations are removed by the connected BeamHeadCleaner","flag_support_and_weak_purge":flags,"full_LINKS_complete":false,"heads_complete":false,"stems_complete":false,"epilog_complete":false});
    if !removed_beam_head_supports.is_empty() {
        m.owner.prepared.report["limited_links"]["BeamHeadCleaner_removed_relation_ids"] =
            json!(removed_beam_head_supports);
    }
    Ok(m)
}

/// StemsRetriever.checkHeadStems/HeadStemsCleaner and StemsStep.doEpilog
/// on the admitted one-stem-per-head subset, plus owned single-beam endpoint
/// checks. Shared-glyph cross-system beams remain admission refusals.
pub(crate) fn finalize_stems(mut o: OwnedItems) -> Result<OwnedItems, String> {
    o = crate::music_ledger_reduction::check(o)?;
    let beam_checks = crate::music_beams::check_ends(&mut o.graph)?;
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
        let o = remove_unreachable_beams(o).unwrap();
        assert_eq!(
            o.prepared.report["unreachable_beam_reduction"]["removed"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let o = crate::stem_image_head_linking::continue_linking(o).unwrap();
        let o = crate::stem_foundation_purge::continue_purge(o).unwrap();
        let o =
            finalize_stems(crate::stem_head_foundations::continue_reduction(o).unwrap()).unwrap();
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
