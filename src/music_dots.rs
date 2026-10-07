// SPDX-License-Identifier: AGPL-3.0-or-later
//! DotFactory late note augmentation, owned AbstractNote/Chord dot getters.
use crate::{
    music_measures::{Chord, ChordKind, Measure},
    music_symbols::flat_staff_y,
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};

fn rect(n: &Value) -> Result<[i64; 4], String> {
    serde_json::from_value(n["bounds"].clone()).map_err(|_| "music_dot_bounds_required".into())
}
fn center(b: [i64; 4]) -> [i64; 2] {
    [b[0] + b[2] / 2, b[1] + b[3] / 2]
}
// GradeImpacts zeroes the product even for a zero-weight exhausted impact.
pub(crate) fn gap_grade(
    dx: f64,
    dy: f64,
    x_max: f64,
    y_max: f64,
    x_weight: f64,
    y_weight: f64,
) -> f64 {
    let x = (1. - dx / x_max).clamp(0., 1.);
    let y = (1. - dy / y_max).clamp(0., 1.);
    if x == 0. || y == 0. {
        0.
    } else {
        (x.powf(x_weight) * y.powf(y_weight)).powf(1. / (x_weight + y_weight))
    }
}
fn intersects(a: [i64; 4], b: [i64; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
fn key(n: &Value) -> Result<Key, String> {
    serde_json::from_value(n["origin"].clone())
        .map_err(|_| "music_dot_owned_origin_required".into())
}
pub(crate) fn distant_from_line(
    b: [i64; 4],
    staff: &Value,
    il: usize,
    snapshot: &Value,
) -> Result<bool, String> {
    let y = b[1] as f64 + b[3] as f64 / 2.;
    let pitch = 2. * (y - flat_staff_y(staff, 2)?) / il as f64;
    let nearest = 2. * (pitch / 2.).round_ties_even();
    let distance = if pitch.abs() <= 5. {
        (pitch - nearest).abs() / 2.
    } else {
        let index = (nearest as i64 - nearest.signum() as i64 * 4) / 2;
        let x = b[0] as f64 + b[2] as f64 / 2.;
        let il = il as f64;
        snapshot["inters"]
            .as_array()
            .ok_or("music_dot_SIG_required")?
            .iter()
            .filter(|n| {
                n["kind"] == "LedgerInter" && n["staff_id"] == staff["id"] && n["index"] == index
            })
            .filter_map(|n| {
                let r: [f64; 4] = serde_json::from_value(n["bounds"].clone()).ok()?;
                (r[0] < x + il && x - il < r[0] + r[2] && r[1] < y + il && y - il < r[1] + r[3])
                    .then_some((y - (r[1] + r[3] / 2.)).abs() / il)
            })
            .min_by(f64::total_cmp)
            .unwrap_or(f64::INFINITY)
    };
    Ok(distance >= 0.3)
}

pub(crate) fn first_dot<'a>(snapshot: &'a Value, note: &Value) -> Option<&'a Value> {
    let edge = snapshot["relations"]
        .as_array()?
        .iter()
        .find(|e| e["kind"] == "AugmentationRelation" && e["target"] == note["id"])?;
    snapshot["inters"]
        .as_array()?
        .iter()
        .find(|n| n["id"] == edge["source"] && n["kind"] == "AugmentationDotInter")
}
fn note_count(snapshot: &Value, note: &Value) -> usize {
    first_dot(snapshot, note).map_or(0, |dot| {
        if snapshot["relations"].as_array().unwrap().iter().any(|e| {
            e["kind"] == "DoubleDotRelation"
                && (e["source"] == dot["id"] || e["target"] == dot["id"])
        }) {
            2
        } else {
            1
        }
    })
}

fn relation(dx: f64, dy: f64) -> Option<Value> {
    // AugmentationRelation uses the original zero/one impact weights.
    let grade = gap_grade(dx, dy, 2., 1.2, 0., 1.);
    (grade >= 0.1).then(|| json!({"kind":"AugmentationRelation","grade":grade,"dx":dx,"dy":dy,"source_ratio":1.+3.*grade,"target_ratio":1.,"profile":1}))
}
fn head_link(
    snapshot: &Value,
    b: [i64; 4],
    il: usize,
    chords: &[&Chord],
) -> Result<Option<(Key, Value)>, String> {
    let p = center(b);
    // getNotesLuBox uses the new dot's own profile0; gap grading uses system1.
    let max_dx = 2 * il as i64;
    let max_dy = (0.8 * il as f64).round_ties_even() as i64;
    let lu = [p[0] - max_dx, p[1] - max_dy, max_dx, 2 * max_dy];
    let min_dx = (0.1 * il as f64).round_ties_even() as i64;
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut links = Vec::new();
    for chord in chords
        .iter()
        .filter(|c| c.kind == ChordKind::Head && intersects(c.bounds, lu))
    {
        let mut notes = chord
            .members
            .iter()
            .map(|n| {
                nodes
                    .iter()
                    .find(|v| v["id"] == n["id"])
                    .ok_or("music_dot_live_head_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        notes.sort_by_key(|n| center(rect(n).unwrap())[1]);
        for head in notes {
            let hb = rect(head)?;
            let cy = center(hb)[1];
            if cy < lu[1] || cy >= lu[1] + lu[3] || first_dot(snapshot, head).is_some() {
                continue;
            }
            let mut dx = p[0] - (hb[0] + hb[2] - 1);
            if dx < min_dx {
                continue;
            }
            if snapshot["relations"].as_array().unwrap().iter().any(|e| {
                e["kind"] == "MirrorRelation"
                    && (e["source"] == head["id"] || e["target"] == head["id"])
            }) {
                return Err("music_dot_shared_head_links_pending".into());
            }
            if snapshot["relations"].as_array().unwrap().iter().any(|e| {
                e["kind"] == "HeadStemRelation"
                    && e["source"] == head["id"]
                    && e["head_side"] == "RIGHT"
            }) {
                let right = chord
                    .members
                    .iter()
                    .filter(|n| {
                        snapshot["relations"].as_array().unwrap().iter().any(|e| {
                            e["kind"] == "HeadStemRelation"
                                && e["source"] == n["id"]
                                && e["head_side"] == "LEFT"
                        })
                    })
                    .map(rect)
                    .collect::<Result<Vec<_>, _>>()?;
                if let (Some(left), Some(end)) = (
                    right.iter().map(|r| r[0]).min(),
                    right.iter().map(|r| r[0] + r[2]).max(),
                ) {
                    dx = (dx - (end - left)).max(1);
                }
            }
            if dx > 0
                && let Some(r) =
                    relation(dx as f64 / il as f64, (cy - p[1]).abs() as f64 / il as f64)
            {
                links.push((
                    head["pitch"]
                        .as_f64()
                        .ok_or("music_dot_head_pitch_required")?
                        .round_ties_even() as i64,
                    key(head)?,
                    r,
                ));
            }
        }
    }
    let best = links
        .iter()
        .position(|(pitch, _, _)| pitch % 2 != 0)
        .unwrap_or(0);
    Ok(links.into_iter().nth(best).map(|(_, k, r)| (k, r)))
}

fn second_dot<'a>(snapshot: &'a Value, first: &Value) -> Option<&'a Value> {
    let edge = snapshot["relations"].as_array()?.iter().find(|e| {
        e["kind"] == "DoubleDotRelation" && e["target"] == first["id"] && e["source"] != first["id"]
    })?;
    snapshot["inters"]
        .as_array()?
        .iter()
        .find(|n| n["id"] == edge["source"] && n["kind"] == "AugmentationDotInter")
}
fn double_link(b: [i64; 4], il: usize, firsts: &[&Value]) -> Result<Option<(Key, Value)>, String> {
    let p = center(b);
    let max_dx = (1.25 * il as f64).round_ties_even() as i64;
    let max_dy = (0.25 * il as f64).round_ties_even() as i64;
    let lu = [p[0] - max_dx, p[1] - max_dy, max_dx, 2 * max_dy];
    let min_dx = (0.1 * il as f64).round_ties_even() as i64;
    let mut best: Option<(Key, Value)> = None;
    for first in firsts {
        let d = rect(first)?;
        if center(d) == p || !intersects(d, lu) {
            continue;
        }
        let dx = p[0] - (d[0] + d[2] - 1);
        let dy = (p[1] - center(d)[1]).abs();
        if dx < min_dx {
            continue;
        }
        let dx = dx as f64 / il as f64;
        let dy = dy as f64 / il as f64;
        let grade = gap_grade(dx, dy, 2., 0.4, 1., 3.);
        if grade >= 0.1
            && best
                .as_ref()
                .is_none_or(|(_, r)| grade > r["grade"].as_f64().unwrap())
        {
            best = Some((
                key(first)?,
                json!({"kind":"DoubleDotRelation","grade":grade,"dx":dx,"dy":dy,"source_ratio":1.+5.*grade,"target_ratio":1.,"profile":1}),
            ));
        }
    }
    Ok(best)
}
pub(crate) fn late_checks(
    graph: &mut Graph,
    registry: &crate::beam_dispatch::Registry,
    trials: &[Value],
    staffs: &[Value],
    measures: &[Measure],
    chords: &[Chord],
    il: usize,
) -> Result<Value, String> {
    let mut staged = graph.clone();
    let mut candidates = trials
        .iter()
        .filter_map(|t| {
            t["evaluations"]
                .as_array()?
                .iter()
                .find(|e| e["shape"] == "DOT_set")
                .map(|e| (t, e))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(t, _)| t["bounds"][0].as_i64());
    let mut created = Vec::new();
    let mut eligible = std::collections::BTreeSet::new();
    for &(trial, evaluation) in &candidates {
        let b = rect(trial)?;
        let p = center(b);
        let staff = staffs
            .iter()
            .map(|s| Ok(((p[1] as f64 - flat_staff_y(s, 2)?).abs(), s)))
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .ok_or("music_dot_staff_required")?
            .1;
        let snapshot = staged.snapshot();
        if !distant_from_line(b, staff, il, &snapshot)? {
            continue;
        }
        eligible.insert(
            trial["glyph_id"]
                .as_u64()
                .ok_or("music_dot_registered_glyph_required")?,
        );
        let system = staff["system_id"]
            .as_u64()
            .ok_or("music_dot_system_required")?;
        let Some(mi) = measures.iter().position(|m| {
            m.system == system
                && Some(m.staff) == staff["id"].as_u64()
                && m.left < p[0]
                && p[0] <= m.right
        }) else {
            return Err("music_dot_stack_required".into());
        };
        let measure = &measures[mi];
        // Conservative admission for not-yet-connected repeat/articulation
        // interpretations. These guards retain ambiguous dots for later ports.
        if (p[0] - measure.left)
            .abs()
            .min((p[0] - measure.right).abs())
            <= (2.2 * il as f64).round_ties_even() as i64
        {
            return Err("music_dot_repeat_checks_pending".into());
        }
        let mut nearby = chords
            .iter()
            .filter(|c| c.measure == mi)
            .collect::<Vec<_>>();
        nearby.sort_by_key(|c| c.bounds[0]);
        for c in nearby.iter().filter(|c| c.kind == ChordKind::Head) {
            let cb = c.bounds;
            let lu_dx = il as i64;
            if intersects(
                cb,
                [
                    p[0] - lu_dx,
                    flat_staff_y(staff, 0)? as i64,
                    2 * lu_dx + 1,
                    (4 * il + 1) as i64,
                ],
            ) && !intersects(cb, b)
                && (p[1] < cb[1] || p[1] >= cb[1] + cb[3])
            {
                return Err("music_dot_articulation_checks_pending".into());
            }
        }
        // Positive rest augmentation is deferred until supported rest conflict
        // reduction is connected; never reinterpret it as an undotted rest.
        let lu = [
            p[0] - 2 * il as i64,
            p[1] - (0.8 * il as f64).round_ties_even() as i64,
            2 * il as i64,
            2 * (0.8 * il as f64).round_ties_even() as i64,
        ];
        for c in nearby
            .iter()
            .filter(|c| c.kind == ChordKind::Rest && intersects(c.bounds, lu))
        {
            let rb = rect(&c.members[0])?;
            let dx = p[0] - (rb[0] + rb[2] - 1);
            let dy = ((p[1] - center(rb)[1]).abs() as f64 - 0.45 * rb[3] as f64).max(0.);
            if dx >= (0.1 * il as f64).round_ties_even() as i64
                && relation(dx as f64 / il as f64, dy / il as f64).is_some()
            {
                return Err("music_dot_supported_rest_reduction_pending".into());
            }
        }
        if let Some((head, rel)) = head_link(&snapshot, b, il, &nearby)? {
            let local = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|n| key(n).ok())
                .filter(|k| k.system == system && k.phase == Phase::Symbol)
                .map(|k| k.local)
                .max()
                .unwrap_or(0)
                + 1;
            let dk = Key {
                system,
                phase: Phase::Symbol,
                local,
            };
            let grade = 0.8
                * evaluation["score"]
                    .as_f64()
                    .ok_or("music_dot_classifier_grade_required")?;
            let gid = trial["glyph_id"]
                .as_u64()
                .ok_or("music_dot_registered_glyph_required")? as usize;
            let glyph = crate::beam_dispatch::glyph_json(&registry.get(gid)?.glyph);
            let id = staged.vertex(dk,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":b,"glyph":glyph,"glyph_id":gid,"staff_id":staff["id"],"grade":grade,"abnormal":false}),false)?;
            staged.relations(system, Phase::Symbol, vec![(dk, head, rel)])?;
            created.push(id);
        }
    }
    // Phase2 searches only the firsts collected at the end of phase1.
    let snapshot = staged.snapshot();
    let mut firsts = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "AugmentationDotInter")
        .collect::<Vec<_>>();
    firsts.sort_by_key(|n| n["bounds"][0].as_i64());
    let mut seconds = Vec::new();
    for (trial, evaluation) in candidates {
        let gid = trial["glyph_id"]
            .as_u64()
            .ok_or("music_dot_registered_glyph_required")?;
        if !eligible.contains(&gid) {
            continue;
        }
        let b = rect(trial)?;
        let p = center(b);
        let staff = staffs
            .iter()
            .map(|s| Ok(((p[1] as f64 - flat_staff_y(s, 2)?).abs(), s)))
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .ok_or("music_dot_staff_required")?
            .1;
        let system = staff["system_id"]
            .as_u64()
            .ok_or("music_dot_system_required")?;
        let Some(mi) = measures.iter().position(|m| {
            m.system == system
                && Some(m.staff) == staff["id"].as_u64()
                && m.left < p[0]
                && p[0] <= m.right
        }) else {
            continue;
        };
        let same_stack = firsts
            .iter()
            .filter_map(|n| {
                let k = key(n).ok()?;
                let c = center(rect(n).ok()?);
                (k.system == system && measures[mi].left < c[0] && c[0] <= measures[mi].right)
                    .then_some(*n)
            })
            .collect::<Vec<_>>();
        if let Some((first, rel)) = double_link(b, il, &same_stack)? {
            if firsts.iter().any(|n| n["glyph_id"] == gid) {
                return Err("music_dot_first_second_glyph_competition_pending".into());
            }
            let current = staged.snapshot();
            let first_id = staged.id(first).ok_or("music_dot_owned_first_required")?;
            if current["relations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "DoubleDotRelation" && e["target"] == first_id)
            {
                return Err("music_dot_competing_seconds_pending".into());
            }
            let local = current["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|n| key(n).ok())
                .filter(|k| k.system == system && k.phase == Phase::Symbol)
                .map(|k| k.local)
                .max()
                .unwrap_or(0)
                + 1;
            let dk = Key {
                system,
                phase: Phase::Symbol,
                local,
            };
            let glyph = crate::beam_dispatch::glyph_json(&registry.get(gid as usize)?.glyph);
            let grade = 0.8
                * evaluation["score"]
                    .as_f64()
                    .ok_or("music_dot_classifier_grade_required")?;
            // Original phase2 inherits part/Voice through DoubleDot, not staff.
            let id=staged.vertex(dk,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":b,"glyph":glyph,"glyph_id":gid,"grade":grade,"abnormal":false}),false)?;
            staged.relations(system, Phase::Symbol, vec![(dk, first, rel)])?;
            seconds.push(id);
        }
    }
    *graph = staged;
    Ok(
        json!({"created_first_dot_ids":created,"created_second_dot_ids":seconds,"full_dot_factory_complete":false}),
    )
}

pub(crate) fn contextualize_and_count(
    graph: &mut Graph,
    chords: &[Chord],
) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let mut staged = graph.clone();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut removed = Vec::new();
    let mut discarded = Vec::new();
    for dot in nodes.iter().filter(|n| n["kind"] == "AugmentationDotInter") {
        let rels = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                matches!(
                    e["kind"].as_str(),
                    Some("AugmentationRelation" | "DoubleDotRelation")
                ) && e["source"] == dot["id"]
            })
            .collect::<Vec<_>>();
        if rels.len() != 1 {
            return Err("music_dot_shared_or_competing_support_pending".into());
        }
        let rel = rels[0];
        let note = nodes
            .iter()
            .find(|n| n["id"] == rel["target"])
            .ok_or("music_dot_live_note_required")?;
        let grade = dot["grade"].as_f64().ok_or("music_dot_grade_required")?;
        let coefficient = if rel["kind"] == "DoubleDotRelation" {
            5.
        } else {
            3.
        };
        let sum = coefficient
            * rel["grade"]
                .as_f64()
                .ok_or("music_dot_relation_grade_required")?
            * note["grade"]
                .as_f64()
                .ok_or("music_dot_note_grade_required")?;
        let contextual = ((1. + sum) * grade) / (1. + sum * grade);
        if contextual < 0.5 && dot["frozen"] != true {
            staged.remove_music_inter(key(dot)?)?;
            removed.push(dot["id"].clone());
        } else {
            let mut n = dot.clone();
            n["contextual_grade"] = json!(contextual);
            staged.vertex(key(dot)?, n, false)?;
        }
    }
    for chord in chords {
        let snap = staged.snapshot();
        let mut counts = Vec::new();
        for note in &chord.members {
            counts.push(note_count(&snap, note));
        }
        if counts.is_empty() {
            continue;
        }
        let dots =
            (counts.iter().sum::<usize>() as f64 / counts.len() as f64).round_ties_even() as usize;
        if dots == 0 && counts.iter().any(|&c| c != 0) {
            for note in &chord.members {
                if let Some(dot) = first_dot(&snap, note)
                    && dot["manual"] != true
                {
                    staged.remove_music_inter(key(dot)?)?;
                    removed.push(dot["id"].clone());
                    discarded.push(json!({"id":dot["id"],"kind":dot["kind"],"shape":dot["shape"],"glyph_id":dot["glyph_id"],"bounds":dot["bounds"],"chord_id":chord.id,"note_id":note["id"],"member_dot_counts":counts,"rounded_chord_count":dots,"reason":"AbstractChord.countDots_inconsistent_nonmanual_zero"}));
                }
            }
        }
        let mut n = snap["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == chord.id)
            .ok_or("music_dot_live_chord_required")?
            .clone();
        if dots != 0 || !n["dots_number"].is_null() {
            n["dots_number"] = json!(dots);
            staged.vertex(chord.key, n, false)?;
        }
    }
    *graph = staged;
    Ok(
        json!({"removed_weak_or_inconsistent_dot_ids":removed,"discarded_inconsistent_dots":discarded,"scope":"ordinary unshared first/second-dot supports and AbstractChord.countDots; repeat/staccato/rest/mirror and competing dot interpretations pending"}),
    )
}
pub(crate) fn bounds_with_dots(
    snapshot: &Value,
    chord: &Chord,
    mut b: [i64; 4],
) -> Result<[i64; 4], String> {
    for note in &chord.members {
        if let Some(first) = first_dot(snapshot, note) {
            for dot in std::iter::once(first).chain(second_dot(snapshot, first)) {
                let d = rect(dot)?;
                let x = b[0].min(d[0]);
                let y = b[1].min(d[1]);
                b = [
                    x,
                    y,
                    (b[0] + b[2]).max(d[0] + d[2]) - x,
                    (b[1] + b[3]).max(d[1] + d[3]) - y,
                ];
            }
        }
    }
    Ok(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inconsistent_chord_dot_removal_admits_only_actual_automatic_cleanup() {
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/inconsistent-augmentation.png"
        ))
        .unwrap();
        let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
        let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
        let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
        let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
        let owner = crate::music_links::finalize_stems(owner).unwrap();
        let m = crate::music_measures::continue_measures(owner).unwrap();
        let m = crate::music_symbols::continue_symbols(m).unwrap();
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-augmentation/inconsistent-count-controls.json"
        ))
        .unwrap();
        let snapshot = m.owner.graph.snapshot();
        let dot = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "AugmentationDotInter")
            .unwrap();
        for c in original["controls"].as_array().unwrap() {
            let mut graph = m.owner.graph.clone();
            let mut n = dot.clone();
            n["manual"] = c["manual"].clone();
            graph.vertex(key(dot).unwrap(), n, false).unwrap();
            let counted = contextualize_and_count(&mut graph, &m.chords).unwrap();
            let after = graph.snapshot();
            assert_eq!(
                after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|n| n["id"] != dot["id"]),
                c["dot_removed"]
            );
            if c["dot_removed"] == true {
                let receipt = &counted["discarded_inconsistent_dots"][0];
                assert_eq!(receipt["bounds"], c["glyph_bounds"]);
                assert_eq!(receipt["member_dot_counts"], c["before_member_dot_counts"]);
                assert_eq!(receipt["rounded_chord_count"], c["rounded_chord_dots"]);
                let chord = after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == receipt["chord_id"])
                    .unwrap();
                assert_eq!(chord["bounds"], c["chord_bounds"]);
                assert!(
                    after["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|e| e["source"] != dot["id"] && e["target"] != dot["id"])
                );
            } else {
                assert!(
                    counted["discarded_inconsistent_dots"]
                        .as_array()
                        .unwrap()
                        .is_empty()
                );
            }
        }
        let mut frozen = m.owner.graph.clone();
        let mut n = dot.clone();
        n["frozen"] = json!(true);
        frozen.vertex(key(dot).unwrap(), n, false).unwrap();
        let before = frozen.snapshot();
        assert_eq!(
            contextualize_and_count(&mut frozen, &m.chords).unwrap_err(),
            "music_removal_frozen_inter"
        );
        assert_eq!(frozen.snapshot(), before);
        let export = crate::music_xml::continue_export(m).unwrap();
        let report = &export.owner.owner.prepared.report;
        let expected: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-augmentation/inconsistent-observed.json"
        ))
        .unwrap();
        assert_eq!(report["events"].as_array().unwrap().len(), 12);
        for (a, e) in report["events"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["events"].as_array().unwrap())
        {
            for k in [
                "measure_number",
                "step",
                "octave",
                "alter",
                "voice",
                "time_offset",
                "duration_quarters",
                "type",
                "stem_direction",
                "chord",
                "dots",
            ] {
                assert_eq!(a[k], e[k], "{k}");
            }
        }
        assert!(!export.xml.contains("<dot/>"));
        assert_eq!(export.xml.matches("<chord/>").count(), 4);
        assert_eq!(
            report["symbols"]["limited_links"]["dot_counting"]["discarded_inconsistent_dots"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(report["musicxml_complete"], false);
    }
    #[test]
    fn exhausted_zero_weight_x_impact_preserves_original_relation_rejection() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-augmentation/gap-controls.json"
        ))
        .unwrap();
        for c in original["cases"].as_array().unwrap() {
            let actual = relation(c["dx"].as_f64().unwrap(), c["dy"].as_f64().unwrap());
            assert_eq!(actual.is_some(), c["accepted"].as_bool().unwrap());
            if let Some(r) = actual {
                assert_eq!(r["grade"], c["grade"]);
            }
        }
    }
}
