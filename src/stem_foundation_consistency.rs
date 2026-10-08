// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original AdapterForFoundations check order on the actual image owner.
//! Ordinary overlap/chord prolog/late is connected; full SIG contracts are pending.
use crate::{stem_head_foundation_plan as plan, stem_image_owned::OwnedItems};
use serde_json::{Value, json};

fn apply(owned: &mut OwnedItems, prepared: plan::Plan) -> Result<(usize, Vec<Value>), String> {
    let modified = prepared.stages["modified"]
        .as_u64()
        .or_else(|| prepared.stages["modified_stems"].as_u64())
        .ok_or("missing_foundation_modification_count")? as usize;
    let removed = crate::stem_head_foundations::apply_plan(owned, &prepared)?;
    Ok((modified, removed))
}

fn contextualize_and_purge(owned: &mut OwnedItems) -> Result<(Vec<usize>, Vec<Value>), String> {
    owned.graph.contextualize_head_stems()?;
    crate::stem_foundation_purge::purge_owned(owned)
}

/// Consume native ownership, without caller-selected candidates or graph import.
pub(crate) fn continue_reduction(mut owned: OwnedItems) -> Result<OwnedItems, String> {
    if !matches!(
        owned.prepared.report["schema"].as_str(),
        Some(
            "audiveris-native.stem-foundation-weak-purge.v1"
                | "audiveris-native.stem-image-head-linking.v1"
        )
    ) || owned.prepared.report["shared_sig"] != owned.graph.snapshot()
    {
        return Err("actual_weak_purged_image_owner_required".into());
    }
    let weak_purge = std::mem::take(&mut owned.prepared.report);
    // Original detectOverlaps precedes contextualization and adapter prolog.
    let overlap = crate::stem_foundation_overlap::discover(&mut owned)?;
    owned.graph.contextualize_head_stems()?;
    // Original adapter.prolog happens before the epoch's first weak purge.
    let chord_prolog = crate::stem_chord_consistency::analyze(&mut owned.graph)?;
    let mut epochs = Vec::new();
    let mut removed_cached_stems = Vec::new();
    let mut removed_beams = Vec::new();
    let mut removed_ledgers = Vec::new();
    let mut finished = false;
    for epoch in 0..64 {
        let (initial_purge, stems) = contextualize_and_purge(&mut owned)?;
        removed_cached_stems.extend(stems);
        let mut deleted = initial_purge.clone();
        let mut passes = Vec::new();
        let mut consistent = false;
        for iteration in 0..64 {
            let mut modified = 0;
            let mut checks = Vec::new();
            // Each check uses current identities/geometry/grades. Original weak
            // purge follows every check, before the next check sees the graph.
            for name in [
                "checkStemEndingHeads",
                "checkHeads",
                "checkHooksHaveStem",
                "checkBeamsHaveBothStems",
                "checkLedgers",
                "checkStems",
            ] {
                let count = match name {
                    "checkStemEndingHeads" => {
                        let p = plan::prepare_ending_heads(&owned.graph.snapshot())?;
                        let (n, stems) = apply(&mut owned, p)?;
                        removed_cached_stems.extend(stems);
                        n
                    }
                    "checkHeads" => {
                        let p = plan::prepare_heads(&owned.graph.snapshot())?;
                        let (n, stems) = apply(&mut owned, p)?;
                        removed_cached_stems.extend(stems);
                        n
                    }
                    "checkHooksHaveStem" => {
                        let ids = crate::music_beams::check_hooks(&mut owned.graph)?;
                        let n = ids.len();
                        removed_beams.extend(ids);
                        n
                    }
                    "checkBeamsHaveBothStems" => {
                        let ids = crate::music_beams::check_beams(&mut owned.graph)?;
                        let n = ids.len();
                        removed_beams.extend(ids);
                        n
                    }
                    "checkLedgers" => {
                        owned = crate::music_ledger_reduction::check(owned)?;
                        let ids = owned.prepared.report["orphan_ledger_consistency"]["removed"]
                            .as_array()
                            .ok_or("missing_foundation_ledger_removals")?
                            .clone();
                        let n = ids.len();
                        removed_ledgers.extend(ids);
                        n
                    }
                    "checkStems" => {
                        let p = plan::prepare_stem_checks(&owned.graph.snapshot())?;
                        let (n, stems) = apply(&mut owned, p)?;
                        removed_cached_stems.extend(stems);
                        n
                    }
                    _ => unreachable!(),
                };
                modified += count;
                let (purged, stems) = contextualize_and_purge(&mut owned)?;
                removed_cached_stems.extend(stems);
                deleted.extend(purged.iter().copied());
                checks.push(json!({"check":name,"modified":count,"purged_vertex_ids":purged}));
            }
            passes.push(json!({"iteration":iteration+1,"modified":modified,"checks":checks}));
            if modified == 0 {
                consistent = true;
                break;
            }
        }
        if !consistent {
            return Err("music_foundation_consistency_iteration_budget".into());
        }
        // Original exclusion reduction is after consistency, not between head
        // checks and beam/stem checks. This reuses the validated support subset.
        let p = plan::prepare_exclusions(&owned.graph.snapshot())?;
        let reduced = p
            .events
            .iter()
            .filter_map(|e| {
                if let plan::Event::RemoveVertex(id) = e {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let (_, stems) = apply(&mut owned, p)?;
        removed_cached_stems.extend(stems);
        // Original late consistency: analyzeChords, reduceExclusions, then
        // contextualizeAndPurge. Its modification return is zero; deletions
        // determine the next epoch rather than a separate structural loop.
        let chord_late = crate::stem_chord_consistency::analyze(&mut owned.graph)?;
        let late_plan = plan::prepare_exclusions(&owned.graph.snapshot())?;
        let late_reduced = late_plan
            .events
            .iter()
            .filter_map(|e| {
                if let plan::Event::RemoveVertex(id) = e {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let (_, stems) = apply(&mut owned, late_plan)?;
        removed_cached_stems.extend(stems);
        let (late_purge, stems) = contextualize_and_purge(&mut owned)?;
        removed_cached_stems.extend(stems);
        deleted.extend(late_purge.iter().copied());
        let changed = !deleted.is_empty() || !reduced.is_empty() || !late_reduced.is_empty();
        epochs.push(json!({"epoch":epoch+1,"initial_purge":initial_purge,
            "passes":passes,"deleted_weak_vertex_ids":deleted,"reduced_vertex_ids":reduced,
            "chord_late":chord_late,"late_reduced_vertex_ids":late_reduced,"late_purge":late_purge}));
        if !changed {
            finished = true;
            break;
        }
    }
    if !finished {
        return Err("music_foundation_reduction_epoch_budget".into());
    }
    owned.prepared.report["orphan_ledger_consistency"]["removed"] = json!(removed_ledgers);
    let ledger_checks = owned.prepared.report["orphan_ledger_consistency"].clone();
    owned.prepared.report = json!({
        "schema":"audiveris-native.stem-head-foundations.v1",
        "corner_count":weak_purge["corner_count"],"weak_purge":weak_purge,
        "shared_sig":owned.graph.snapshot(),"removed_cached_stems":removed_cached_stems,
        "orphan_ledger_consistency":ledger_checks,
        "ordered_foundation_consistency":{"epochs":epochs,"removed_beam_ids":removed_beams,"chord_prolog":chord_prolog,"overlap_discovery":overlap,
            "check_order_and_per_check_purge_connected":true,
            "original_general_overlap_discovery_connected":false,
            "ordinary_overlap_discovery_connected":true,
            "ordinary_analyzeChords_prolog_late_connected":true,
            "scope":"original six AdapterForFoundations checks/contextualizeAndPurge and post-consistency exclusion/epoch order, retained ordinary head/beam/ledger support subset; full reduceFoundations pending"},
        "actual_live_Graph_and_cache_updated":true,
        "heads_complete":false,"stems_complete":false,"musicxml_complete":false,
        "epilog_complete":false,"automatic_raster_pipeline_complete":false
    });
    if owned.prepared.report["weak_purge"]["schema"]
        == "audiveris-native.stem-image-head-linking.v1"
    {
        let linked = owned
            .prepared
            .report
            .as_object_mut()
            .unwrap()
            .remove("weak_purge")
            .unwrap();
        owned.prepared.report["linked_head_stems"] = linked;
    }
    owned.prepared.report["header_diatonic_pitch"] = crate::head_musical_pitch::header_candidates(
        &owned.graph.snapshot(),
        &owned.upstream_report,
    );
    let refinement = crate::stem_head_refinement::apply(&mut owned)?;
    owned.prepared.report["stem_head_end_refinement"] = refinement;
    owned.prepared.report["shared_sig"] = owned.graph.snapshot();
    Ok(owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GrayImage, beam_dispatch,
        shared_sig::{Key, Phase},
        stem_checks, stem_glyph,
    };
    #[test]
    fn original_six_check_cascade_updates_actual_cache_at_each_purge_boundary() {
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-cross-system/beam-two-systems.png"
        ))
        .unwrap();
        let o = crate::stem_image_head_linking::continue_process(
            crate::stem_image_owned::prepare_items(&image).unwrap(),
        )
        .unwrap();
        let mut owned =
            continue_reduction(crate::stem_foundation_purge::continue_purge(o).unwrap()).unwrap();
        // Append one declared original cascade to an already reduced, actual
        // image owner. No claim that the image itself produces this condition.
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-foundation-order/controls.json"
        ))
        .unwrap();
        let creations: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-builder-create/controls.json"
        ))
        .unwrap();
        let c = &creations["cases"][oracle["original_creation_case"].as_u64().unwrap() as usize];
        let parts = c["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| beam_dispatch::glyph_from_json(g).unwrap())
            .collect::<Vec<_>>();
        let glyph = stem_glyph::assemble(&parts).unwrap();
        let width = creations["image_width"].as_u64().unwrap() as usize;
        let height = creations["image_height"].as_u64().unwrap() as usize;
        let mut raster = GrayImage {
            width,
            height,
            pixels: vec![255; width * height],
        };
        for r in &glyph.runs {
            for y in r.start..r.start + r.length {
                raster.pixels
                    [(glyph.top as usize + y) * width + glyph.left as usize + r.position] = 0;
            }
        }
        let parameters =
            stem_checks::Parameters::new(11, 5, 0., c["profile"].as_u64().unwrap() as u8).unwrap();
        let cache = owned.caches.get_mut(&1).unwrap();
        let stem = cache
            .create_selected(&mut owned.prepared.registry, &parts, &raster, &parameters)
            .unwrap()
            .stem
            .unwrap();
        assert!(
            (stem.snapshot().unwrap()["grade"].as_f64().unwrap()
                - c["first"]["grade"].as_f64().unwrap())
            .abs()
                < 1e-12
        );
        let sid = cache
            .register(&owned.prepared.registry, &mut owned.graph, &stem)
            .unwrap();
        let sk = Key {
            system: 1,
            phase: Phase::Stem,
            local: stem.glyph_id(),
        };
        let hk = Key {
            system: 1,
            phase: Phase::Head,
            local: 1_000_000,
        };
        let bk = Key {
            system: 1,
            phase: Phase::Beam,
            local: 1_000_000,
        };
        let hid = owned
            .graph
            .vertex(hk, oracle["head"].clone(), false)
            .unwrap();
        // The original cascade probe constructs this glyphless hook at height
        // 6. Its historical snapshot omitted height, which overlap now needs.
        let mut hook = oracle["hook"].clone();
        hook["height"] = json!(6.);
        let bid = owned.graph.vertex(bk, hook, false).unwrap();
        let mut hr = oracle["head_relation"].clone();
        hr["local_relation_id"] = json!(1_000_000);
        owned
            .graph
            .relations(1, Phase::Stem, vec![(hk, sk, hr)])
            .unwrap();
        owned
            .graph
            .relations(
                1,
                Phase::Beam,
                vec![(bk, sk, oracle["beam_relation"].clone())],
            )
            .unwrap();
        owned.graph.contextualize_head_stems().unwrap();
        let initial = owned.graph.snapshot();
        for (&id, expected) in [hid, sid, bid]
            .iter()
            .zip(oracle["initial"].as_array().unwrap())
        {
            let n = initial["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == id)
                .unwrap();
            assert!(
                (n["contextual_grade"].as_f64().unwrap()
                    - expected["contextual_grade"].as_f64().unwrap())
                .abs()
                    < 1e-12
            );
        }
        let registry = owned.prepared.registry.snapshot();
        let reference_nodes = initial["inters"].clone();
        owned.prepared.report = json!({"schema":"audiveris-native.stem-foundation-weak-purge.v1",
            "shared_sig":initial,"declared_oracle_append":true});
        let owned = continue_reduction(owned).unwrap();
        let after = owned.graph.snapshot();
        let passes = owned.prepared.report["ordered_foundation_consistency"]["epochs"][0]["passes"]
            .as_array()
            .unwrap();
        assert_eq!(passes.len(), oracle["passes"].as_array().unwrap().len());
        for (pass, expected) in passes.iter().zip(oracle["passes"].as_array().unwrap()) {
            assert_eq!(pass["modified"], expected["modified"]);
            for (check, e) in pass["checks"]
                .as_array()
                .unwrap()
                .iter()
                .zip(expected["checks"].as_array().unwrap())
            {
                assert_eq!(check["check"], e["check"]);
                assert_eq!(check["modified"], e["modified"]);
                let kinds = check["purged_vertex_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| {
                        reference_nodes
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|n| n["id"] == *id)
                            .unwrap()["kind"]
                            .clone()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(json!(kinds), e["purged_kinds"]);
            }
        }
        for (&id, expected) in [hid, sid, bid]
            .iter()
            .zip(oracle["final"].as_array().unwrap())
        {
            let n = after["tombstones"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == id)
                .unwrap();
            assert_eq!(n["id"], id);
            assert_eq!(expected["removed"], true);
            assert!(
                !after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|n| n["id"] == id)
            );
        }
        assert_eq!(stem.snapshot().unwrap()["removed"], true);
        assert_eq!(owned.prepared.registry.snapshot(), registry);
        assert!(
            owned.caches[&1]
                .registered_stem(&owned.graph, stem.glyph_id())
                .is_err()
        );
    }
}
