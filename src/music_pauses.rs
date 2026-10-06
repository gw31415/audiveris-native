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
    let x = (b[0] as f64 + b[2] as f64 / 2.).round_ties_even() as i64;
    let Some(mi) = measures
        .iter()
        .position(|m| m.system == system && m.staff == staff && m.left < x && x <= m.right)
    else {
        return Ok(None);
    };
    let max = b[0] + b[2];
    let mut heads = chords
        .iter()
        .filter(|c| c.kind == ChordKind::Head && c.measure == mi)
        .collect::<Vec<_>>();
    heads.sort_by_key(|c| c.bounds[0]);
    let Some(chord) = heads.into_iter().rfind(|c| {
        let cx = (c.bounds[0] as f64 + c.bounds[2] as f64 / 2.).round_ties_even() as i64;
        cx >= measures[mi].left && cx <= max
    }) else {
        return Ok(None);
    };
    let cx = (chord.bounds[0] as f64 + chord.bounds[2] as f64 / 2.).round_ties_even() as i64;
    let dx = (max - cx) as f64 / il as f64;
    let grade = (2. * (1. - dx / 100.) + 1.) / 3.;
    Ok((grade>=0.1).then(||(chord.key,json!({"kind":"ChordPauseRelation","grade":grade,"dx":dx,"dy":0.,"source_ratio":1.,"target_ratio":1.+3.*grade,"profile":1}))))
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
        // AbstractChord.invalidateCache uses the mean member contextual grade.
        let grades = chord
            .members
            .iter()
            .map(|n| {
                let live = nodes
                    .iter()
                    .find(|v| v["id"] == n["id"])
                    .ok_or("music_pause_live_note_required")?;
                live["contextual_grade"]
                    .as_f64()
                    .or_else(|| live["grade"].as_f64())
                    .ok_or("music_pause_note_grade_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if grades.is_empty() {
            return Err("music_pause_chord_members_required".into());
        }
        let partner = grades.iter().sum::<f64>() / grades.len() as f64;
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
            removed.push(json!({"id":pause["id"],"shape":pause["shape"],"glyph_id":pause["glyph_id"],"bounds":pause["bounds"],"grade":grade,"contextual_grade":contextual,"partner_bounds":chord.bounds}));
        } else {
            let mut n = pause.clone();
            n["contextual_grade"] = json!(contextual);
            staged.vertex(key, n, false)?;
        }
    }
    *graph = staged;
    Ok(json!({"removed_weak_pauses":removed,"positive_pause_notation_export_complete":false}))
}
