// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual AlterInter factory pitch, nearest Head/mirror links and bounded weak purge.
use crate::shared_sig::{Graph, Key, Phase};
use serde_json::{Value, json};
pub(crate) fn create(
    owner: &crate::stem_image_owned::OwnedItems,
    glyph: &audiveris_staff_removal::GlyphPixels,
    shape: &str,
    grade: f64,
    staff: &std::sync::Arc<crate::grid_ir::StaffGeometry>,
    glyph_id: usize,
) -> Result<Value, String> {
    if !matches!(
        shape,
        "FLAT" | "NATURAL" | "SHARP" | "DOUBLE_FLAT" | "DOUBLE_SHARP"
    ) || !grade.is_finite()
        || !(0.0..=1.0).contains(&grade)
        || !owner
            .staffs
            .iter()
            .any(|s| std::sync::Arc::ptr_eq(s, staff))
    {
        return Err("music_alter_actual_Staff_and_shape_required".into());
    }
    let centroid = crate::beam_dispatch::centroid(glyph)?.map(|v| v as f64);
    let mass = owner.ledger_memberships.pitch_at(staff, centroid)?;
    let measured = if matches!(shape, "FLAT" | "DOUBLE_FLAT") {
        if owner.catalog.family != "Bravura" {
            return Err("music_alter_font_area_pitch_offset_pending".into());
        }
        let point = [
            glyph.left as f64 + glyph.width as f64 / 2.,
            glyph.top as f64 + glyph.height as f64 / 2.,
        ];
        0.5 * (mass + 0.65 + owner.ledger_memberships.pitch_at(staff, point)? + 1.0559375)
    } else {
        mass
    };
    let geometry = crate::beam_dispatch::glyph_json(glyph);
    Ok(
        json!({"kind":"AlterInter","shape":shape,"grade":grade,"bounds":geometry["bounds"],"glyph":geometry,"glyph_id":glyph_id,"staff_id":staff.id,"pitch":measured.round_ties_even(),"measured_pitch":measured,"manual":false,"abnormal":false,"contextual_grade":null}),
    )
}
pub(crate) fn set_links(
    graph: &mut Graph,
    alter: Key,
    interline: usize,
    profile: u32,
) -> Result<(), String> {
    if interline == 0 || profile > 1 {
        return Err("music_alter_supported_scale_profile_required".into());
    }
    let (node, removed) = graph.retained_inter(alter)?;
    if removed || node["kind"] != "AlterInter" || alter.phase != Phase::Symbol {
        return Err("music_alter_live_owner_required".into());
    }
    let bounds: [i64; 4] = serde_json::from_value(node["bounds"].clone())
        .map_err(|_| "music_alter_bounds_required")?;
    let point = [
        bounds[0] + bounds[2],
        bounds[1]
            + if matches!(node["shape"].as_str(), Some("FLAT" | "DOUBLE_FLAT")) {
                3 * bounds[3] / 4
            } else {
                bounds[3] / 2
            },
    ];
    let (x_max, y_max) = if profile == 0 { (2., 0.45) } else { (3., 0.6) };
    let dx = (x_max * interline as f64).round_ties_even() as i64;
    let dy = (y_max * interline as f64).round_ties_even() as i64;
    let area = [point[0], point[1] - dy, dx, 2 * dy];
    let mut heads = graph
        .live_inter_keys(alter.system, "HeadInter")
        .into_iter()
        .map(|key| {
            let (head, _) = graph.retained_inter(key)?;
            let bounds: [i64; 4] = serde_json::from_value(head["bounds"].clone())
                .map_err(|_| "music_alter_head_bounds_required")?;
            Ok((key, bounds))
        })
        .collect::<Result<Vec<_>, String>>()?;
    heads.sort_by_key(|(_, bounds)| bounds[0]);
    let mut best = None;
    let mut best_y = i64::MAX;
    for (head, b) in heads {
        if area[0] >= b[0] + b[2]
            || b[0] >= area[0] + area[2]
            || area[1] >= b[1] + b[3]
            || b[1] >= area[1] + area[3]
        {
            continue;
        }
        let y = (b[1] + b[3] / 2 - point[1]).abs();
        let dx = (b[0] - point[0]).abs() as f64 / interline as f64;
        let dy = y as f64 / interline as f64;
        let grade = crate::music_dots::gap_grade(dx, dy, x_max, y_max, 1., 4.);
        if grade >= 0.1 && y < best_y {
            best_y = y;
            best = Some((
                head,
                json!({"kind":"AlterHeadRelation","dx":dx,"dy":dy,"grade":grade,"source_ratio":1.+3.*grade,"target_ratio":1.,"manual":false}),
            ));
        }
    }
    if let Some((head, relation)) = best {
        let mut links = vec![(alter, head, relation.clone())];
        let snapshot = graph.snapshot();
        let id = graph.id(head).unwrap();
        if let Some(mirror) = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"] == "MirrorRelation" && (r["source"] == id || r["target"] == id))
        {
            let other = if mirror["source"] == id {
                &mirror["target"]
            } else {
                &mirror["source"]
            };
            let actual = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == *other && n["kind"] == "HeadInter")
                .ok_or("music_alter_actual_mirror_head_required")?;
            let key: Key = serde_json::from_value(actual["origin"].clone())
                .map_err(|_| "music_alter_actual_mirror_origin_required")?;
            links.push((alter, key, relation));
        }
        graph.relations(alter.system, Phase::Symbol, links)?;
    }
    Ok(())
}
fn contextual_grade(snapshot: &Value, alter: &Value) -> Result<f64, String> {
    let grade = alter["grade"]
        .as_f64()
        .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
        .ok_or("music_alter_grade_required")?;
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("music_alter_SIG_required")?;
    let relations = snapshot["relations"]
        .as_array()
        .ok_or("music_alter_relations_required")?;
    let mut partners = Vec::new();
    let mut contributions = std::collections::BTreeMap::new();
    for relation in relations
        .iter()
        .filter(|e| e["source"] == alter["id"] || e["target"] == alter["id"])
    {
        if relation["kind"] == "Exclusion" {
            continue;
        }
        if relation["kind"] != "AlterHeadRelation"
            || relation["source"] != alter["id"]
            || relation["target_ratio"] != 1.0
        {
            return Err("music_alter_typed_context_support_required".into());
        }
        let head = nodes
            .iter()
            .find(|n| {
                n["id"] == relation["target"]
                    && n["kind"] == "HeadInter"
                    && n["origin"]["system"] == alter["origin"]["system"]
            })
            .ok_or("music_alter_actual_context_Head_required")?;
        let partner = head["id"]
            .as_u64()
            .ok_or("music_alter_context_partner_id_required")?;
        let intrinsic = head["grade"]
            .as_f64()
            .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
            .ok_or("music_alter_context_partner_grade_required")?;
        let support = relation["grade"]
            .as_f64()
            .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
            .ok_or("music_alter_context_support_grade_required")?;
        let ratio = relation["source_ratio"]
            .as_f64()
            .filter(|r| r.is_finite() && (*r - (1. + 3. * support)).abs() < 1e-12)
            .ok_or("music_alter_context_support_ratio_required")?;
        if ratio > 1. {
            partners.push((partner, intrinsic));
            contributions.insert(partner, intrinsic * (ratio - 1.));
        }
    }
    // Keep the existing partition storage bound; an unadmitted large support
    // graph must fail before changing any contextual grades or deleting owners.
    if partners.len() > 16 {
        return Err("music_alter_context_partner_limit_16".into());
    }
    partners.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut conflicts = vec![Vec::new(); partners.len()];
    for (i, (partner, _)) in partners.iter().enumerate() {
        for edge in relations.iter().filter(|e| e["kind"] == "Exclusion") {
            let other = if edge["source"] == *partner {
                edge["target"].as_u64()
            } else if edge["target"] == *partner {
                edge["source"].as_u64()
            } else {
                None
            };
            if let Some(j) =
                other.and_then(|other| partners.iter().position(|(id, _)| *id == other))
                && j > i
                && !conflicts[i].contains(&j)
            {
                conflicts[i].push(j);
            }
        }
    }
    let mut best = 0.0_f64;
    for partition in crate::sig_context::compatible_partitions(&conflicts) {
        let sum = partition
            .into_iter()
            .map(|i| contributions[&partners[i].0])
            .sum::<f64>();
        best = best.max(((1. + sum) * grade) / (1. + sum * grade));
    }
    Ok(best)
}
pub(crate) fn contextualize_and_purge(graph: &mut Graph) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let mut staged = graph.clone();
    let mut weak = Vec::new();
    let mut contextualized = Vec::new();
    for alter in snapshot["inters"]
        .as_array()
        .ok_or("music_alter_SIG_required")?
        .iter()
        .filter(|n| n["kind"] == "AlterInter" && n["origin"]["phase"] == "SYMBOL")
    {
        let contextual = contextual_grade(&snapshot, alter)?;
        let key: Key = serde_json::from_value(alter["origin"].clone())
            .map_err(|_| "music_alter_origin_required")?;
        if key.phase != Phase::Symbol {
            return Err("music_alter_symbol_origin_required".into());
        }
        let mut data = alter.clone();
        data["contextual_grade"] = json!(contextual);
        staged.vertex(key, data, false)?;
        contextualized.push(json!({"id":alter["id"],"contextual_grade":contextual}));
        if contextual < 0.5 && alter["frozen"] != true {
            weak.push(key);
        }
    }
    let mut removed = Vec::new();
    for key in weak {
        let (alter, _) = staged.retained_inter(key)?;
        removed.push(json!({"id":staged.id(key),"shape":alter["shape"],"bounds":alter["bounds"],"glyph_id":alter["glyph_id"],"grade":alter["grade"],"contextual_grade":alter["contextual_grade"]}));
        staged.remove_music_inter(key)?;
    }
    *graph = staged;
    Ok(
        json!({"contextualized_alters":contextualized,"removed_weak_alters":removed,"positive_accidental_head_semantics_complete":false}),
    )
}

#[cfg(test)]
mod image_tests;
#[cfg(test)]
pub(crate) use image_tests::{assert_actual_image_factory, assert_supported_context};
