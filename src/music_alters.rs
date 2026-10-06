// SPDX-License-Identifier: AGPL-3.0-or-later
//! AlterInter empty head lookup and original unsupported weak-symbol purge.
use crate::shared_sig::{Graph, Key, Phase};
use serde_json::{Value, json};
pub(crate) fn require_empty_links(
    shape: &str,
    b: [i64; 4],
    system: u64,
    il: usize,
    snapshot: &Value,
) -> Result<(), String> {
    let x = b[0] + b[2];
    let y = b[1]
        + if matches!(shape, "FLAT" | "DOUBLE_FLAT") {
            3 * b[3] / 4
        } else {
            b[3] / 2
        };
    let max_dx = (3. * il as f64).round_ties_even() as i64;
    let max_dy = (0.6 * il as f64).round_ties_even() as i64;
    let lu = [x, y - max_dy, max_dx, 2 * max_dy];
    for head in snapshot["inters"]
        .as_array()
        .ok_or("music_alter_SIG_required")?
        .iter()
        .filter(|n| n["kind"] == "HeadInter" && n["origin"]["system"] == system)
    {
        let h: [i64; 4] = serde_json::from_value(head["bounds"].clone())
            .map_err(|_| "music_alter_head_bounds_required")?;
        if lu[0] >= h[0] + h[2]
            || h[0] >= lu[0] + lu[2]
            || lu[1] >= h[1] + h[3]
            || h[1] >= lu[1] + lu[3]
        {
            continue;
        }
        // AlterHeadRelation.setOutGaps takes absolute x, including overlap.
        let dx = (h[0] - x).abs() as f64 / il as f64;
        let dy = (h[1] + h[3] / 2 - y).abs() as f64 / il as f64;
        if crate::music_dots::gap_grade(dx, dy, 3., 0.6, 1., 4.) >= 0.1 {
            return Err("music_alter_positive_head_link_pending".into());
        }
    }
    Ok(())
}
pub(crate) fn purge_unlinked(graph: &mut Graph) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let mut staged = graph.clone();
    let mut removed = Vec::new();
    for alter in snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "AlterInter" && n["origin"]["phase"] == "SYMBOL")
    {
        if snapshot["relations"].as_array().unwrap().iter().any(|e| {
            (e["source"] == alter["id"] || e["target"] == alter["id"]) && e["kind"] != "Exclusion"
        }) {
            return Err("music_alter_supported_context_pending".into());
        }
        let grade = alter["grade"]
            .as_f64()
            .ok_or("music_alter_grade_required")?;
        let key: Key = serde_json::from_value(alter["origin"].clone())
            .map_err(|_| "music_alter_origin_required")?;
        if key.phase != Phase::Symbol {
            return Err("music_alter_symbol_origin_required".into());
        }
        if grade < 0.5 && alter["frozen"] != true {
            staged.remove_music_inter(key)?;
            removed.push(json!({"id":alter["id"],"shape":alter["shape"],"bounds":alter["bounds"],"glyph_id":alter["glyph_id"],"grade":grade,"contextual_grade":grade}));
        } else {
            let mut n = alter.clone();
            n["contextual_grade"] = json!(grade);
            staged.vertex(key, n, false)?;
        }
    }
    *graph = staged;
    Ok(
        json!({"removed_weak_unlinked_alters":removed,"positive_accidental_head_semantics_complete":false}),
    )
}
