// SPDX-License-Identifier: AGPL-3.0-or-later
//! AbstractPauseInter rightmost preceding chord and original weak-symbol purge.
use crate::{
    music_measures::{Chord, ChordKind, Measure},
    shared_sig::{Graph, Key},
};
use serde_json::{Value, json};

pub(crate) fn lookup(
    b: [i64; 4],
    system: u64,
    staff: u64,
    il: usize,
    measures: &[Measure],
    chords: &[Chord],
) -> Result<Option<(Key, Value)>, String> {
    let x = b[0] + b[2] / 2;
    let mut selected = None;
    for m in measures.iter().filter(|m| m.system == system) {
        let abscissae = if let Some(g) = &m.geometry {
            let Some(held) = g.part.staffs.iter().find(|s| s.id == staff) else {
                continue;
            };
            g.abscissae(held)?
        } else {
            if m.staff != staff {
                continue;
            }
            [m.left, m.right]
        };
        if abscissae[0] < x && x <= abscissae[1] {
            selected = Some(abscissae[0]);
            break;
        }
    }
    let Some(left) = selected else {
        return Ok(None);
    };
    let max = b[0] + b[2] - 1;
    let mut heads = chords
        .iter()
        .filter(|c| {
            c.kind == ChordKind::Head
                && c.key.system == system
                && c.members.iter().any(|n| n["staff_id"] == staff)
        })
        .collect::<Vec<_>>();
    heads.sort_by_key(|c| c.bounds[0]);
    let Some(chord) = heads.into_iter().rfind(|c| {
        let cx = c.bounds[0] + c.bounds[2] / 2;
        cx >= left && cx <= max
    }) else {
        return Ok(None);
    };
    let cx = chord.bounds[0] + chord.bounds[2] / 2;
    let dx = (max - cx) as f64 / il as f64;
    let grade = crate::music_dots::gap_grade(dx, 0., 100., 4., 2., 1.);
    Ok((grade>=0.1).then(||(chord.key,json!({"kind":"ChordPauseRelation","grade":grade,"dx":dx,"dy":0.,"source_ratio":1.,"target_ratio":1.+3.*grade,"profile":1,"manual":false}))))
}
pub(crate) fn contextualize_and_purge(
    graph: &mut Graph,
    chords: &[Chord],
) -> Result<Value, String> {
    let snap = graph.snapshot();
    let nodes = snap["inters"].as_array().unwrap();
    let mut staged = graph.clone();
    let mut removed = Vec::new();
    for pause in nodes
        .iter()
        .filter(|n| matches!(n["kind"].as_str(), Some("CaesuraInter" | "BreathMarkInter")))
    {
        let relations = snap["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "ChordPauseRelation" && e["target"] == pause["id"])
            .collect::<Vec<_>>();
        if relations.len() != 1 {
            return Err("music_pause_unique_chord_support_required".into());
        }
        let r = relations[0];
        let chord = chords
            .iter()
            .find(|c| r["source"] == c.id)
            .ok_or("music_pause_actual_chord_required")?;
        // Support reads the confidence cached at membership invalidation.
        // Later Head contextualization must not recompute this Chord grade.
        let partner = nodes
            .iter()
            .find(|n| n["id"] == chord.id)
            .and_then(|n| n["grade"].as_f64())
            .ok_or("music_pause_actual_cached_Chord_grade_required")?;
        let grade = pause["grade"]
            .as_f64()
            .ok_or("music_pause_grade_required")?;
        let sum = 3.
            * r["grade"]
                .as_f64()
                .ok_or("music_pause_relation_grade_required")?
            * partner;
        let contextual = ((1. + sum) * grade) / (1. + sum * grade);
        let key: Key = serde_json::from_value(pause["origin"].clone())
            .map_err(|_| "music_pause_origin_required")?;
        if contextual < 0.5 && pause["frozen"] != true {
            staged.remove_music_inter(key)?;
            removed.push(json!({"id":pause["id"],"shape":pause["shape"],"glyph_id":pause["glyph_id"],"bounds":pause["bounds"],"grade":grade,"relation_grade":r["grade"],"dx":r["dx"],"contextual_grade":contextual,"partner_bounds":chord.bounds}));
        } else {
            let mut n = pause.clone();
            n["contextual_grade"] = json!(contextual);
            staged.vertex(key, n, false)?;
        }
    }
    *graph = staged;
    Ok(json!({"removed_weak_pauses":removed,"positive_pause_notation_export_complete":false}))
}

#[cfg(test)]
mod image_tests;
