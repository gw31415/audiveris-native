// SPDX-License-Identifier: AGPL-3.0-or-later
//! OrnamentInter factory lookup against the retained actual HeadChords.
use crate::{
    music_measures::{ChordKind, OwnedMusic},
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};

pub(crate) fn is_shape(shape: &str) -> bool {
    matches!(
        shape,
        "TR" | "TURN" | "TURN_INVERTED" | "TURN_UP" | "TURN_SLASH" | "MORDENT" | "MORDENT_INVERTED"
    )
}

fn intersects(a: [i64; 4], b: [i64; 4]) -> bool {
    a[2] > 0
        && a[3] > 0
        && b[2] > 0
        && b[3] > 0
        && a[0] < b[0] + b[2]
        && b[0] < a[0] + a[2]
        && a[1] < b[1] + b[3]
        && b[1] < a[1] + a[3]
}

fn lookup(
    graph: &Graph,
    music: &OwnedMusic,
    system: u64,
    bounds: [i64; 4],
) -> Result<Option<(Key, Value)>, String> {
    let mut chords = music
        .chords
        .iter()
        .filter(|c| c.key.system == system && c.kind == ChordKind::Head)
        .collect::<Vec<_>>();
    if chords.is_empty() {
        return Ok(None);
    }
    chords.sort_by_key(|c| c.bounds[0]);
    let profile = music
        .owner
        .systems
        .iter()
        .find(|s| s.id == system)
        .ok_or("music_ornament_actual_System_required")?
        .profile;
    let (xmax, ymax) = match profile {
        0 => (0.75, 3.),
        1 => (1.2, 4.),
        _ => return Err("music_ornament_profile_pending".into()),
    };
    let interline = music.owner.interline as f64;
    let dx = (xmax * interline).round_ties_even() as i64;
    let dy = (ymax * interline).round_ties_even() as i64;
    let center = [bounds[0] + bounds[2] / 2, bounds[1] + bounds[3] / 2];
    // Rectangle(Point) starts with zero extent. The articulation lookup's
    // inclusive +1 and Staff restrictions would change this factory's links.
    let area = [center[0] - dx, center[1] - dy, 2 * dx, 2 * dy];
    let mut best = None;
    for chord in chords {
        let (body, removed) = graph.retained_inter(chord.key)?;
        if removed
            || graph.id(chord.key) != Some(chord.id)
            || body["kind"] != "HeadChordInter"
            || body["bounds"] != json!(chord.bounds)
        {
            return Err("music_ornament_actual_HeadChord_required".into());
        }
        let cb = chord.bounds;
        if !intersects(cb, area) || intersects(cb, bounds) {
            continue;
        }
        let chord_center = [cb[0] + cb[2] / 2, cb[1] + cb[3] / 2];
        let y_ref = if center[1] > chord_center[1] {
            cb[1] + cb[3]
        } else {
            cb[1]
        };
        let x_gap = (chord_center[0] - center[0]).abs() as f64 / interline;
        let y_pixels = (y_ref - center[1]).abs();
        let y_gap = y_pixels as f64 / interline;
        let grade = crate::music_dots::gap_grade(x_gap, y_gap, xmax, ymax, 2., 1.);
        // Source selection minimizes vertical distance, retaining the first
        // abscissa-sorted Chord on ties; grade is only an admission threshold.
        if grade >= 0.1 && best.as_ref().is_none_or(|(_, _, gap)| y_pixels < *gap) {
            best = Some((
                chord.key,
                json!({"kind":"ChordOrnamentRelation","grade":grade,"dx":x_gap,"dy":y_gap,
                    "source_ratio":1.,"target_ratio":1.+0.5*grade,"manual":false}),
                y_pixels,
            ));
        }
    }
    Ok(best.map(|(key, relation, _)| (key, relation)))
}

pub(crate) fn create(
    graph: &Graph,
    music: &OwnedMusic,
    system: u64,
    glyph_id: usize,
    glyph: Value,
    shape: &str,
    grade: f64,
) -> Result<Option<(Key, Value, Value)>, String> {
    if !is_shape(shape) {
        return Err("music_ornament_shape_required".into());
    }
    if !grade.is_finite() || !(0.0..=1.0).contains(&grade) {
        return Err("music_ornament_grade_required".into());
    }
    let bounds: [i64; 4] = serde_json::from_value(glyph["bounds"].clone())
        .map_err(|_| "music_ornament_bounds_required")?;
    let link = lookup(graph, music, system, bounds)?;
    #[cfg(test)]
    tests::record(graph, music, system, bounds, shape, grade, &link);
    Ok(link.map(|(chord, relation)| {
        (
            chord,
            json!({"kind":"OrnamentInter","shape":shape,"bounds":bounds,
            "glyph_id":glyph_id,"glyph":glyph,"grade":grade,"manual":false,"frozen":false,
            "abnormal":false,"staff_id":null,"contextual_grade":null}),
            relation,
        )
    }))
}

pub(crate) fn create_added(
    graph: &mut Graph,
    music: &OwnedMusic,
    system: u64,
    glyph_id: usize,
    glyph: Value,
    shape: &str,
    grade: f64,
) -> Result<Option<Key>, String> {
    let Some((chord, body, relation)) =
        create(graph, music, system, glyph_id, glyph, shape, grade)?
    else {
        return Ok(None);
    };
    let mut next = graph.clone();
    let key = next.next_symbol_key(system)?;
    next.vertex(key, body, false)?;
    // Neither OrnamentInter.added nor ChordOrnamentRelation.added adds an
    // abnormal callback. Raw Staff remains null until its lazy getter is used.
    next.relations(system, Phase::Symbol, vec![(chord, key, relation)])?;
    *graph = next;
    Ok(Some(key))
}

pub(crate) fn contextualize_and_purge(
    graph: &mut Graph,
    music: &OwnedMusic,
) -> Result<Value, String> {
    let mut next = graph.clone();
    let mut contexts = Vec::new();
    let mut removed = Vec::new();
    for system in &music.owner.systems {
        let keys = next.live_inter_keys(system.id, "OrnamentInter");
        let snapshot = next.snapshot();
        let mut weak = Vec::new();
        for key in keys {
            let (ornament, is_removed) = next.retained_inter(key)?;
            if is_removed {
                return Err("music_ornament_live_owner_required".into());
            }
            let mut body = ornament.clone();
            let id = next.id(key).ok_or("music_ornament_live_owner_required")?;
            if snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|relation| {
                    (relation["source"] == id || relation["target"] == id)
                        && (!matches!(
                            relation["kind"].as_str(),
                            Some("ChordOrnamentRelation" | "Exclusion" | "NoExclusion")
                        ) || (relation["kind"] == "ChordOrnamentRelation"
                            && relation["target"] != id))
                })
            {
                return Err("music_ornament_incident_support_pending".into());
            }
            let supports = snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "ChordOrnamentRelation" && e["target"] == id)
                .collect::<Vec<_>>();
            if supports.len() > 1 {
                return Err("music_ornament_single_Chord_source_required".into());
            }
            let mut sum = 0.;
            let mut sources = Vec::new();
            for relation in supports {
                let relation_grade = relation["grade"]
                    .as_f64()
                    .filter(|v| (0.0..=1.0).contains(v))
                    .ok_or("music_ornament_relation_grade_required")?;
                let ratio = relation["target_ratio"]
                    .as_f64()
                    .filter(|v| v.is_finite() && (*v - (1. + 0.5 * relation_grade)).abs() < 1e-12)
                    .ok_or("music_ornament_support_ratio_required")?;
                if relation["source_ratio"] != 1.0 {
                    return Err("music_ornament_source_ratio_required".into());
                }
                let chord = music
                    .chords
                    .iter()
                    .find(|c| {
                        c.kind == ChordKind::Head
                            && c.key.system == system.id
                            && relation["source"] == c.id
                    })
                    .ok_or("music_ornament_actual_HeadChord_required")?;
                let (partner, is_removed) = next.retained_inter(chord.key)?;
                if is_removed || next.id(chord.key) != Some(chord.id) {
                    return Err("music_ornament_actual_HeadChord_required".into());
                }
                // Support reads the existing cached Chord grade. Today's
                // contextual Head grades would change this source calculation.
                let partner_grade = partner["grade"]
                    .as_f64()
                    .filter(|v| (0.0..=1.0).contains(v))
                    .ok_or("music_ornament_cached_Chord_grade_required")?;
                sum += (ratio - 1.) * partner_grade;
                sources.push(json!({"relation_id":relation["id"],"relation_grade":relation["grade"],
                    "partner_id":chord.id,"partner_grade":partner_grade,"partner_bounds":chord.bounds}));
            }
            let grade = body["grade"]
                .as_f64()
                .filter(|v| (0.0..=1.0).contains(v))
                .ok_or("music_ornament_grade_required")?;
            let contextual = ((1. + sum) * grade) / (1. + sum * grade);
            body["contextual_grade"] = json!(contextual);
            next.vertex(key, body.clone(), false)?;
            contexts.push(
                json!({"id":id,"system":system.id,"bounds":body["bounds"],"grade":grade,
                "contextual_grade":contextual,"sources":sources}),
            );
            if contextual < 0.5 && body["frozen"] != true {
                weak.push(key);
            }
        }
        // SIGraph contextualizes every Inter before deleteWeakInters starts.
        // A removal observer must see the other ornaments' computed grades.
        #[cfg(test)]
        tests::record_purge("contextualized", system.id, &next);
        for key in weak {
            let id = next.id(key).ok_or("music_ornament_live_owner_required")?;
            next.remove_music_inter(key)?;
            removed.push(id);
            #[cfg(test)]
            tests::record_purge("removed", system.id, &next);
        }
    }
    if music
        .owner
        .systems
        .iter()
        .any(|system| !next.live_inter_keys(system.id, "OrnamentInter").is_empty())
    {
        return Err("music_ornament_survivor_export_pending".into());
    }
    *graph = next;
    Ok(json!({"contextualized":contexts,"removed_weak_ornament_ids":removed}))
}

#[cfg(test)]
pub(crate) use tests::assert_der_winter_owned_lifecycle;

#[cfg(test)]
mod tests;
