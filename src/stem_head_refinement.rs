// SPDX-License-Identifier: AGPL-3.0-or-later
//! ReductionStep default refineStemHeadEnd keeps physical/cache identity.
use crate::{shared_sig::Key, stem_image_owned::OwnedItems};
use serde_json::{Value, json};
pub(crate) fn apply(owner: &mut OwnedItems) -> Result<Value, String> {
    let mut snapshot = owner.graph.snapshot();
    snapshot["interline"] = json!(owner.interline);
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("stem_refinement_live_SIG_required")?;
    let edges = snapshot["relations"].as_array().unwrap();
    let bank = owner.catalog.bank()?;
    let mut next = owner.graph.clone();
    let mut objects = Vec::new();
    let mut updates = Vec::new();
    for stem in nodes.iter().filter(|n| n["kind"] == "StemInter") {
        let key: Key = serde_json::from_value(stem["origin"].clone())
            .map_err(|_| "stem_refinement_origin_required")?;
        let mut heads = edges
            .iter()
            .filter(|e| e["kind"] == "HeadStemRelation" && e["target"] == stem["id"])
            .map(|e| {
                nodes
                    .iter()
                    .find(|n| n["id"] == e["source"] && n["kind"] == "HeadInter")
                    .map(|h| (h, e))
                    .ok_or("stem_refinement_actual_Head_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if heads.is_empty() {
            continue;
        }
        let mut centers = std::collections::BTreeMap::new();
        for (head, _) in &heads {
            let b: [i64; 4] = serde_json::from_value(head["bounds"].clone())
                .map_err(|_| "stem_refinement_Head_bounds_required")?;
            centers.insert(
                head["id"]
                    .as_u64()
                    .ok_or("stem_refinement_Head_identity_required")?,
                b[1] + b[3] / 2,
            );
        }
        heads.sort_by_key(|(h, _)| centers[&h["id"].as_u64().unwrap()]);
        let dir = crate::stem_head_foundation_plan::direction(&snapshot, stem);
        let (head, relation) = if dir > 0 {
            heads[0]
        } else {
            *heads.last().unwrap()
        };
        let side = match relation["head_side"].as_str() {
            Some("LEFT") => crate::head_stem_relation::HeadSide::Left,
            Some("RIGHT") => crate::head_stem_relation::HeadSide::Right,
            _ => return Err("stem_refinement_actual_Head_side_required".into()),
        };
        let vertical = if dir > 0 {
            crate::head_stem_relation::VerticalSide::Bottom
        } else {
            crate::head_stem_relation::VerticalSide::Top
        };
        // Only the anchor ordinate is read; learned horizontal offsets cannot change it.
        let anchor = crate::head_stem_relation::stem_reference_point(
            &bank,
            owner.catalog.stored_point_size,
            head["shape"]
                .as_str()
                .ok_or("stem_refinement_Head_shape_required")?,
            serde_json::from_value(head["bounds"].clone())
                .map_err(|_| "stem_refinement_Head_bounds_required")?,
            side,
            vertical,
            [None, None],
        )?;
        let bounds: [i64; 4] = serde_json::from_value(stem["bounds"].clone())
            .map_err(|_| "stem_refinement_Stem_bounds_required")?;
        let mut median: [f64; 4] = serde_json::from_value(stem["median"].clone())
            .map_err(|_| "stem_refinement_actual_median_required")?;
        let line = if bounds[3] >= owner.interline as i64 {
            median
        } else {
            owner.sheet_skew.vertical([
                (bounds[0] + bounds[2] / 2) as f64,
                (bounds[1] + bounds[3] / 2) as f64,
            ])
        };
        if line[3] == line[1] {
            return Err("stem_refinement_reliable_vertical_required".into());
        }
        let x = line[0] + (anchor[1] - line[1]) * (line[2] - line[0]) / (line[3] - line[1]);
        if dir > 0 {
            median[0] = x;
            median[1] = anchor[1];
        } else {
            median[2] = x;
            median[3] = anchor[1];
        }
        objects.push(
            owner
                .caches
                .get(&key.system)
                .ok_or("stem_refinement_actual_cache_required")?
                .registered_stem(&owner.graph, key.local)?,
        );
        let updated = next.refine_stem_median(key, median)?;
        updates.push((key, updated));
    }
    let mut locks = objects
        .iter()
        .map(|s| s.removal_guard())
        .collect::<Result<Vec<_>, _>>()?;
    for (lock, (key, _)) in locks.iter().zip(&updates) {
        if lock["in_sig"] != true
            || lock["removed"] == true
            || lock["id"] != owner.graph.id(*key).unwrap()
        {
            return Err("stem_refinement_active_cache_identity_required".into());
        }
    }
    for (lock, (_, updated)) in locks.iter_mut().zip(&updates) {
        lock["median"] = updated["median"].clone();
        lock["bounds"] = updated["bounds"].clone();
    }
    owner.graph = next;
    Ok(
        json!({"refined_stem_ids":updates.iter().map(|(k,_)|owner.graph.id(*k)).collect::<Vec<_>>(),"physical_glyphs_preserved":true,"tail_refinement_default":false}),
    )
}
