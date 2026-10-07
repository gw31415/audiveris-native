// SPDX-License-Identifier: AGPL-3.0-or-later
//! DotFactory late note/rest augmentation, owned AbstractNote/Chord dot getters.
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
pub(crate) fn note_count(snapshot: &Value, note: &Value) -> usize {
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
    staged.bind_dot_measures(measures)?;
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
        let mut links = Vec::new();
        if let Some(link) = head_link(&snapshot, b, il, &nearby)? {
            links.push(link);
        }
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
                && let Some(rel) = relation(dx as f64 / il as f64, dy / il as f64)
            {
                links.push((key(&c.members[0])?, rel));
            }
        }
        // Original applies all links. Multiple competing/shared supports still
        // require their original exclusion lifecycle before image admission.
        if links.len() > 1 {
            return Err("music_dot_shared_or_competing_support_pending".into());
        }
        if let Some((note, rel)) = links.pop() {
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
            let data = json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":b,"glyph":glyph,"glyph_id":gid,"grade":grade,"abnormal":true});
            let id = staged.vertex(dk, data, false)?;
            staged.edit_note_owner(
                dk,
                crate::shared_sig::NoteOwnerEdit::Staff(Some(
                    staff["id"]
                        .as_u64()
                        .ok_or("music_dot_owned_staff_required")?,
                )),
            )?;
            staged.relations(system, Phase::Symbol, vec![(dk, note, rel)])?;
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

pub(crate) fn check_targets(graph: &mut Graph) -> Result<Value, String> {
    let mut staged = graph.clone();
    let initial = graph.snapshot();
    let mut removed = Vec::new();
    for dot in initial["inters"]
        .as_array()
        .ok_or("music_dot_SIG_required")?
        .iter()
        .filter(|n| n["kind"] == "AugmentationDotInter")
    {
        let snapshot = staged.snapshot();
        let edges = snapshot["relations"].as_array().unwrap();
        if edges.iter().any(|e| {
            e["kind"] == "AugmentationRelation"
                && (e["source"] == dot["id"] || e["target"] == dot["id"])
        }) {
            continue;
        }
        let x = center(rect(dot)?)[0];
        let mut left = false;
        for e in edges.iter().filter(|e| {
            e["kind"] == "DoubleDotRelation"
                && (e["source"] == dot["id"] || e["target"] == dot["id"])
        }) {
            let other = if e["source"] == dot["id"] {
                &e["target"]
            } else {
                &e["source"]
            };
            let opposite = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| &n["id"] == other && n["kind"] == "AugmentationDotInter")
                .ok_or("music_dot_live_double_target_required")?;
            left |= center(rect(opposite)?)[0] < x;
        }
        if !left {
            staged.remove_music_inter(key(dot)?)?;
            removed.push(dot["id"].clone());
        }
    }
    *graph = staged;
    Ok(
        json!({"removed_orphan_dot_ids":removed,"scope":"original SigReducer.checkAugmentationDots; full LINK consistency sequence pending"}),
    )
}

fn reduce_head_competition(graph: &mut Graph) -> Result<Vec<Value>, String> {
    let initial = graph.snapshot();
    let nodes = initial["inters"]
        .as_array()
        .ok_or("music_dot_SIG_required")?;
    let edges = initial["relations"].as_array().unwrap();
    let mut checked = std::collections::BTreeSet::new();
    let mut removed = Vec::new();
    for chord in nodes.iter().filter(|n| n["kind"] == "HeadChordInter") {
        let mut members = edges
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["source"] == chord["id"])
            .map(|e| {
                let head = nodes
                    .iter()
                    .find(|n| n["id"] == e["target"] && n["kind"] == "HeadInter")
                    .ok_or("music_dot_owned_head_member_required")?;
                Ok((center(rect(head)?)[1], head))
            })
            .collect::<Result<Vec<_>, String>>()?;
        members.sort_by_key(|(y, _)| *y);
        for (_, head) in members {
            checked.insert(
                head["id"]
                    .as_u64()
                    .ok_or("music_dot_head_identity_required")?,
            );
            let snapshot = graph.snapshot();
            let relations = snapshot["relations"].as_array().unwrap();
            let rels = relations
                .iter()
                .filter(|e| e["kind"] == "AugmentationRelation" && e["target"] == head["id"])
                .collect::<Vec<_>>();
            if rels.len() <= 1 {
                continue;
            }
            let mut dots = rels
                .iter()
                .map(|e| {
                    let dot = snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == e["source"] && n["kind"] == "AugmentationDotInter")
                        .ok_or("music_dot_live_source_required")?;
                    Ok((center(rect(dot)?)[1], dot))
                })
                .collect::<Result<Vec<_>, String>>()?;
            dots.sort_by_key(|(y, _)| *y);
            // The original tries only the two extremes; another head on a
            // space can reserve either dot even when its relation is weaker.
            let mut selected = None;
            for (_, dot) in [dots.first().unwrap(), dots.last().unwrap()] {
                let mut taken = false;
                for rel in relations
                    .iter()
                    .filter(|e| e["kind"] == "AugmentationRelation" && e["source"] == dot["id"])
                {
                    let note = snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == rel["target"])
                        .ok_or("music_dot_live_supported_note_required")?;
                    match note["kind"].as_str() {
                        Some("HeadInter") => {
                            let pitch = note["pitch"]
                                .as_f64()
                                .filter(|p| p.is_finite())
                                .ok_or("music_dot_supported_head_pitch_required")?;
                            if (pitch.round_ties_even() as i32) % 2 != 0 {
                                taken = true;
                                break;
                            }
                        }
                        Some("RestInter") => (),
                        _ => return Err("music_dot_typed_supported_note_required".into()),
                    }
                }
                if !taken {
                    selected = Some(dot["id"].clone());
                    break;
                }
            }
            for rel in rels {
                if selected.as_ref() != Some(&rel["source"]) {
                    let id = rel["id"]
                        .as_u64()
                        .ok_or("music_dot_relation_identity_required")?
                        as usize;
                    graph.remove_augmentation_edge(id)?;
                    removed.push(json!(id));
                }
            }
        }
    }
    for head in nodes.iter().filter(|n| n["kind"] == "HeadInter") {
        if !checked.contains(
            &head["id"]
                .as_u64()
                .ok_or("music_dot_head_identity_required")?,
        ) && edges
            .iter()
            .filter(|e| e["kind"] == "AugmentationRelation" && e["target"] == head["id"])
            .count()
            > 1
        {
            return Err("music_dot_owned_head_chord_required".into());
        }
    }
    Ok(removed)
}

pub(crate) fn reduce_supported_competition(graph: &mut Graph) -> Result<Value, String> {
    let mut staged = graph.clone();
    let initial = staged.snapshot();
    let nodes = initial["inters"]
        .as_array()
        .ok_or("music_dot_SIG_required")?;
    let mut removed = reduce_head_competition(&mut staged)?;
    for (class, kind) in [
        ("RestInter", "AugmentationRelation"),
        ("AugmentationDotInter", "DoubleDotRelation"),
    ] {
        for node in nodes.iter().filter(|n| n["kind"] == class) {
            let snapshot = staged.snapshot();
            let edges = snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| {
                    e["kind"] == kind && (e["source"] == node["id"] || e["target"] == node["id"])
                })
                .collect::<Vec<_>>();
            if edges.len() <= 1 {
                continue;
            }
            let mut best_grade = 0.;
            let mut best = None;
            for e in &edges {
                let grade = e["grade"]
                    .as_f64()
                    .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
                    .ok_or("music_dot_relation_grade_required")?;
                if grade > best_grade {
                    best_grade = grade;
                    best = Some(e["id"].clone());
                }
            }
            for e in edges {
                if best.as_ref() != Some(&e["id"]) {
                    let id = e["id"]
                        .as_u64()
                        .ok_or("music_dot_relation_identity_required")?
                        as usize;
                    staged.remove_augmentation_edge(id)?;
                    removed.push(json!(id));
                }
            }
        }
    }
    *graph = staged;
    Ok(
        json!({"removed_competing_relation_ids":removed,"scope":"original checkAugmentedHeads upper/lower/all-supported-pitches then checkAugmentedRests/checkAugmentedDots positive grade/first tie; general LINK consistency pending"}),
    )
}

pub(crate) fn reduce_consistencies(graph: &mut Graph, chords: &[Chord]) -> Result<Value, String> {
    let mut staged = graph.clone();
    let mut targets = Vec::new();
    let mut relations = Vec::new();
    let mut purged = Vec::new();
    let budget = staged.snapshot()["inters"].as_array().unwrap().len()
        + staged.snapshot()["relations"].as_array().unwrap().len()
        + 1;
    for _ in 0..budget {
        let target = check_targets(&mut staged)?;
        let competition = reduce_supported_competition(&mut staged)?;
        let counted = contextualize_and_count(&mut staged, chords)?;
        let t = target["removed_orphan_dot_ids"].as_array().unwrap();
        let r = competition["removed_competing_relation_ids"]
            .as_array()
            .unwrap();
        let p = counted["removed_weak_or_inconsistent_dot_ids"]
            .as_array()
            .unwrap();
        targets.extend(t.iter().cloned());
        relations.extend(r.iter().cloned());
        purged.extend(p.iter().cloned());
        if t.is_empty() && r.is_empty() && p.is_empty() {
            let deletions = staged.take_dot_removals();
            let discarded = deletions
                .iter()
                .filter(|n| n["reason"] == "AbstractChord.countDots_inconsistent_nonmanual_zero")
                .cloned()
                .collect::<Vec<_>>();
            for n in &discarded {
                if !purged.contains(&n["id"]) {
                    purged.push(n["id"].clone());
                }
            }
            *graph = staged;
            return Ok(
                json!({"targets":{"removed_orphan_dot_ids":targets},"competition":{"removed_competing_relation_ids":relations},"counting":{"removed_weak_or_inconsistent_dot_ids":purged,"discarded_inconsistent_dots":discarded,"deleted_dot_inters":deletions},"scope":"bounded connected dot checks/context/count repeat; full LINK adapter/order remains partial"}),
            );
        }
    }
    Err("music_dot_consistency_convergence_required".into())
}

pub(crate) fn contextualize_and_count(
    graph: &mut Graph,
    chords: &[Chord],
) -> Result<Value, String> {
    let start_removals = graph.pending_dot_removals().len();
    let snapshot = graph.snapshot();
    let mut staged = graph.clone();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut removed = Vec::new();
    let mut weak = Vec::new();
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
        if rels.len() > 1 {
            return Err("music_dot_shared_or_competing_support_pending".into());
        }
        let grade = dot["grade"].as_f64().ok_or("music_dot_grade_required")?;
        let contextual = if let Some(rel) = rels.first() {
            let note = snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == rel["target"])
                .ok_or("music_dot_live_note_required")?;
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
            ((1. + sum) * grade) / (1. + sum * grade)
        } else {
            grade
        };
        let mut n = dot.clone();
        n["contextual_grade"] = json!(contextual);
        staged.vertex(key(dot)?, n, false)?;
        if contextual < 0.5 && dot["frozen"] != true {
            weak.push(key(dot)?);
        }
    }
    for key in weak {
        staged.remove_music_inter(key)?;
    }
    for chord in chords {
        staged.count_owned_augmentation(chord.key)?;
    }
    let deletions = staged.pending_dot_removals()[start_removals..].to_vec();
    let discarded = deletions
        .iter()
        .filter(|n| n["reason"] == "AbstractChord.countDots_inconsistent_nonmanual_zero")
        .cloned()
        .collect::<Vec<_>>();
    for n in &deletions {
        if !removed.contains(&n["id"]) {
            removed.push(n["id"].clone());
        }
    }
    *graph = staged;
    Ok(
        json!({"removed_weak_or_inconsistent_dot_ids":removed,"discarded_inconsistent_dots":discarded,"deleted_dot_inters":deletions,"scope":"ordinary unshared first/second-dot supports and AbstractChord.countDots; repeat/staccato/mirror and competing dot interpretations pending"}),
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
    fn original_rest_and_double_competition_preserves_edge_order_and_owner_callbacks() {
        let f: Value = serde_json::from_str(include_str!(
            "../fixtures/official-dot-competition/observed.json"
        ))
        .unwrap();
        for c in f["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let rk = Key {
                system: 1,
                phase: Phase::Symbol,
                local: 1,
            };
            let ck = Key {
                phase: Phase::Chord,
                ..rk
            };
            let rid = graph.vertex(rk,json!({"kind":"RestInter","shape":"HALF_REST","bounds":c["rest"]["bounds"],"grade":c["rest"]["grade"],"staff_id":1}),false).unwrap();
            let cid = graph.vertex(ck,json!({"kind":"RestChordInter","bounds":c["chord"]["bounds"],"part_id":1,"staff_id":1,"measure_index":1,"duration":null,"voice":null,"dots_number":c["before"]["chord_dots"]}),false).unwrap();
            graph
                .relations(
                    1,
                    Phase::Chord,
                    vec![(ck, rk, json!({"kind":"Containment"}))],
                )
                .unwrap();
            let mut dots = Vec::new();
            for (i, bounds) in c["dot_bounds"].as_array().unwrap().iter().enumerate() {
                let dk = Key { local: i + 2, ..rk };
                graph.vertex(dk,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":bounds,"grade":0.8,"staff_id":1,"abnormal":true}),false).unwrap();
                dots.push(dk);
            }
            let doubled = c["kind"] == "double";
            let mut added = Vec::new();
            if doubled {
                added.push((dots[0],rk,json!({"kind":"AugmentationRelation","grade":0.7,"source_ratio":3.1,"target_ratio":1.})));
            }
            for (i, grade) in c["grades"].as_array().unwrap().iter().enumerate() {
                added.push((dots[i + usize::from(doubled)],if doubled {dots[0]} else {rk},json!({"kind":if doubled {"DoubleDotRelation"} else {"AugmentationRelation"},"grade":grade,"source_ratio":1.+ if doubled {5.} else {3.}*grade.as_f64().unwrap(),"target_ratio":1.})));
            }
            graph.relations(1, Phase::Symbol, added).unwrap();
            let before = graph.snapshot();
            let edge_ids = before["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] != "Containment")
                .map(|e| e["id"].clone())
                .collect::<Vec<_>>();
            let observe = |g: &Graph, expected: &Value| {
                let snapshot = g.snapshot();
                for (k, e) in dots.iter().zip(expected["dots"].as_array().unwrap()) {
                    let id = g.id(*k).unwrap();
                    let node = snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .chain(snapshot["tombstones"].as_array().unwrap())
                        .find(|n| n["id"] == id)
                        .unwrap();
                    assert_eq!(
                        snapshot["tombstones"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|n| n["id"] == id),
                        e["removed"]
                    );
                    if e["removed"] != true {
                        assert_eq!(node["abnormal"], e["abnormal"]);
                    }
                }
                for (id, live) in edge_ids
                    .iter()
                    .zip(expected["edges_live"].as_array().unwrap())
                {
                    assert_eq!(
                        snapshot["relations"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|e| &e["id"] == id),
                        *live
                    );
                }
                let chord = snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == cid)
                    .unwrap();
                assert_eq!(chord["dots_number"], expected["chord_dots"]);
                assert!(
                    snapshot["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|n| n["id"] == rid)
                );
            };
            let mut repeated = graph.clone();
            let chord = Chord {
                kind: ChordKind::Rest,
                key: ck,
                id: cid,
                stem: None,
                members: vec![
                    before["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == rid)
                        .unwrap()
                        .clone(),
                ],
                measure: 0,
                bounds: serde_json::from_value(c["chord"]["bounds"].clone()).unwrap(),
            };
            let report = reduce_consistencies(&mut repeated, &[chord]).unwrap();
            assert_eq!(
                report["competition"]["removed_competing_relation_ids"]
                    .as_array()
                    .unwrap()
                    .len() as u64,
                c["competition_modifications"].as_u64().unwrap()
            );
            observe(&repeated, &c["after_next_target_check_and_count"]);
            if c["grades"] == json!([0.3, 0.8]) && !doubled {
                let mut frozen = graph.clone();
                let mut dot = before["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == graph.id(dots[0]).unwrap())
                    .unwrap()
                    .clone();
                dot["frozen"] = json!(true);
                frozen.vertex(dots[0], dot, false).unwrap();
                let before = frozen.snapshot();
                assert_eq!(
                    reduce_supported_competition(&mut frozen).unwrap_err(),
                    "music_dot_frozen_competition_pending"
                );
                assert_eq!(frozen.snapshot(), before);
            }
            observe(&graph, &c["before"]);
            assert!(
                check_targets(&mut graph).unwrap()["removed_orphan_dot_ids"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
            let result = reduce_supported_competition(&mut graph).unwrap();
            assert_eq!(
                result["removed_competing_relation_ids"]
                    .as_array()
                    .unwrap()
                    .len() as u64,
                c["competition_modifications"].as_u64().unwrap()
            );
            observe(&graph, &c["after_competition_callback"]);
            for id in result["removed_competing_relation_ids"].as_array().unwrap() {
                let before = graph.snapshot();
                graph
                    .remove_augmentation_edge(id.as_u64().unwrap() as usize)
                    .unwrap();
                assert_eq!(graph.snapshot(), before);
            }
            assert_eq!(
                check_targets(&mut graph).unwrap()["removed_orphan_dot_ids"]
                    .as_array()
                    .unwrap()
                    .len() as u64,
                c["later_target_check_modifications"].as_u64().unwrap()
            );
            observe(&graph, &c["after_next_target_check_and_count"]);
        }
    }
    #[test]
    fn original_head_competition_uses_extremes_and_all_supported_integer_pitches() {
        let f: Value = serde_json::from_str(include_str!(
            "../fixtures/official-head-dot-competition/observed.json"
        ))
        .unwrap();
        for c in f["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let mut heads = Vec::new();
            let mut chord_ids = Vec::new();
            for (i, n) in c["heads"].as_array().unwrap().iter().enumerate() {
                let hk = Key {
                    system: 1,
                    phase: Phase::Head,
                    local: i + 1,
                };
                let ck = Key {
                    phase: Phase::Chord,
                    ..hk
                };
                graph.vertex(hk, json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK","bounds":n["bounds"],"pitch":n["pitch"],"staff_id":1,"grade":0.8}), false).unwrap();
                let cid = graph.vertex(ck, json!({"kind":"HeadChordInter","bounds":if i==0 {&c["chord_bounds"]} else {&n["bounds"]},"part_id":1,"staff_id":1,"measure_index":1,"duration":null,"voice":null,"dots_number":c["before"]["chord_dots"]}), false).unwrap();
                graph
                    .relations(
                        1,
                        Phase::Chord,
                        vec![(ck, hk, json!({"kind":"Containment"}))],
                    )
                    .unwrap();
                heads.push(hk);
                chord_ids.push(cid);
            }
            let mut dots = Vec::new();
            let mut relations = Vec::new();
            for (i, b) in c["dot_bounds"].as_array().unwrap().iter().enumerate() {
                let dk = Key {
                    system: 1,
                    phase: Phase::Symbol,
                    local: i + 1,
                };
                graph.vertex(dk, json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":b,"staff_id":1,"grade":0.8,"abnormal":true}), false).unwrap();
                dots.push(dk);
                relations.push((dk, heads[0], json!({"kind":"AugmentationRelation","grade":c["grades"][i],"source_ratio":1.,"target_ratio":1.})));
            }
            for e in c["additional_supports"].as_array().unwrap() {
                relations.push((dots[e[0].as_u64().unwrap() as usize],heads[e[1].as_u64().unwrap() as usize],json!({"kind":"AugmentationRelation","grade":0.7,"source_ratio":1.,"target_ratio":1.})));
            }
            graph.relations(1, Phase::Symbol, relations).unwrap();
            let before = graph.snapshot();
            let ids = before["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "AugmentationRelation")
                .map(|e| e["id"].clone())
                .collect::<Vec<_>>();
            let report = reduce_supported_competition(&mut graph).unwrap();
            assert_eq!(
                report["removed_competing_relation_ids"]
                    .as_array()
                    .unwrap()
                    .len() as u64,
                c["modifications"].as_u64().unwrap(),
                "{}",
                c["name"]
            );
            let after = graph.snapshot();
            for (id, expected) in ids.iter().zip(c["after"]["edges_live"].as_array().unwrap()) {
                assert_eq!(
                    after["relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|e| &e["id"] == id),
                    *expected,
                    "{}",
                    c["name"]
                );
            }
            for (dk, expected) in dots.iter().zip(c["after"]["dots"].as_array().unwrap()) {
                let id = graph.id(*dk).unwrap();
                let node = after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id)
                    .unwrap();
                assert_eq!(node["abnormal"], expected["abnormal"], "{}", c["name"]);
            }
            let chord = after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == chord_ids[0])
                .unwrap();
            assert_eq!(
                chord["dots_number"], c["after"]["chord_dots"],
                "{}",
                c["name"]
            );
            assert!(
                reduce_supported_competition(&mut graph).unwrap()["removed_competing_relation_ids"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
    }
    fn double_dot_symbolic_music() -> crate::music_measures::OwnedMusic {
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
        let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
        let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
        let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
        let owner = crate::music_links::finalize_stems(owner).unwrap();
        let m = crate::music_measures::continue_measures(owner).unwrap();
        crate::music_symbols::continue_symbols(m).unwrap()
    }
    #[test]
    fn actual_double_dot_chain_detachment_recounts_owner_and_preserves_glyphs() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-dot-chain-detach/observed.json"
        ))
        .unwrap();
        let cold = &original["cases"][0];
        let mut m = double_dot_symbolic_music();
        contextualize_and_count(&mut m.owner.graph, &m.chords).unwrap();
        let snapshot = m.owner.graph.snapshot();
        let by_bounds = |class, bounds: &Value| {
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == class && &n["bounds"] == bounds)
                .unwrap()
        };
        let first = by_bounds("AugmentationDotInter", &cold["first_bounds"]);
        let second = by_bounds("AugmentationDotInter", &cold["second_bounds"]);
        let head = by_bounds("HeadInter", &cold["head_bounds"]);
        let chord = by_bounds("HeadChordInter", &cold["chord_bounds"]);
        let edge = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| {
                e["kind"] == "AugmentationRelation"
                    && e["source"] == first["id"]
                    && e["target"] == head["id"]
            })
            .unwrap();
        assert_eq!(chord["dots_number"], cold["before"]["chord_dots"]);
        let physical = [first, second].map(|d| {
            let id = d["glyph_id"].as_u64().unwrap() as usize;
            (
                id,
                crate::beam_dispatch::glyph_json(&m.owner.prepared.registry.get(id).unwrap().glyph),
            )
        });
        let mut timed = m.owner.graph.clone();
        let mut data = chord.clone();
        data["voice"] = json!(1);
        timed.vertex(key(chord).unwrap(), data, false).unwrap();
        let before = timed.snapshot();
        assert_eq!(
            timed
                .remove_augmentation_edge(edge["id"].as_u64().unwrap() as usize)
                .unwrap_err(),
            "music_dot_rhythm_reset_required"
        );
        assert_eq!(timed.snapshot(), before);
        m.owner
            .graph
            .remove_augmentation_edge(edge["id"].as_u64().unwrap() as usize)
            .unwrap();
        let detached = m.owner.graph.snapshot();
        let live = detached["inters"].as_array().unwrap();
        for (node, flag) in [(first, "first_abnormal"), (second, "second_abnormal")] {
            assert_eq!(
                live.iter().find(|n| n["id"] == node["id"]).unwrap()["abnormal"],
                cold["after_original_augmentation_edge_remove_callback"][flag]
            );
        }
        assert_eq!(
            live.iter().find(|n| n["id"] == chord["id"]).unwrap()["dots_number"],
            cold["after_original_augmentation_edge_remove_callback"]["chord_dots"]
        );
        assert_eq!(
            check_targets(&mut m.owner.graph).unwrap()["removed_orphan_dot_ids"]
                .as_array()
                .unwrap()
                .len() as u64,
            cold["original_target_check_modifications"]
                .as_u64()
                .unwrap()
        );
        assert!(
            check_targets(&mut m.owner.graph).unwrap()["removed_orphan_dot_ids"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let after = m.owner.graph.snapshot();
        let live = after["inters"].as_array().unwrap();
        assert!(
            live.iter()
                .all(|n| n["id"] != first["id"] && n["id"] != second["id"])
        );
        assert!(live.iter().any(|n| n["id"] == head["id"]));
        assert!(live.iter().any(|n| n["id"] == chord["id"]));
        assert_eq!(
            live.iter().find(|n| n["id"] == chord["id"]).unwrap()["dots_number"],
            cold["after_original_target_check"]["chord_dots"]
        );
        for (id, glyph) in physical {
            assert_eq!(
                crate::beam_dispatch::glyph_json(&m.owner.prepared.registry.get(id).unwrap().glyph),
                glyph
            );
        }
        assert!(crate::music_xml::continue_export(m).is_err());
    }
    #[test]
    fn retained_editing_task_preserves_relation_identity_owners_and_publication_order() {
        use crate::shared_sig::{DefaultEditingTask, EditingLink, EditingOperation, NoteOwnerEdit};
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-editing-task/observed.json"
        ))
        .unwrap();
        let mut m = double_dot_symbolic_music();
        contextualize_and_count(&mut m.owner.graph, &m.chords).unwrap();
        m.owner.graph.populate_dot_measures().unwrap();
        let initial = m.owner.graph.snapshot();
        let find = |kind: &str, bounds: &Value| {
            initial["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == kind && &n["bounds"] == bounds)
                .unwrap()
                .clone()
        };
        let notes: Vec<_> = original["notes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| find("HeadInter", &n["bounds"]))
            .collect();
        let chords: Vec<_> = original["owner_bounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| find("HeadChordInter", b))
            .collect();
        let dots: Vec<_> = original["before"]["dot_fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| find("AugmentationDotInter", &n["bounds"]))
            .collect();
        let note_ids: Vec<_> = notes
            .iter()
            .map(|n| n["id"].as_u64().unwrap() as usize)
            .collect();
        let owner_ids: Vec<_> = chords
            .iter()
            .map(|n| n["id"].as_u64().unwrap() as usize)
            .collect();
        let dot_ids: Vec<_> = dots
            .iter()
            .map(|n| n["id"].as_u64().unwrap() as usize)
            .collect();
        // The oracle starts from a saved cold owner model. Install that model
        // explicitly; this is not evidence of automatic correction discovery.
        for (i, d) in dots.iter().enumerate() {
            let k = key(d).unwrap();
            m.owner
                .graph
                .edit_note_owner(k, NoteOwnerEdit::Part(None))
                .unwrap();
            let staff = original["before"]["ownership"][i]["staff_id"].as_u64();
            m.owner
                .graph
                .edit_note_owner(k, NoteOwnerEdit::Staff(staff))
                .unwrap();
            let mut data = d.clone();
            data["staff_id"] = json!(staff);
            data["grade"] = original["before"]["dot_fields"][i]["grade"].clone();
            data["contextual_grade"] =
                original["before"]["dot_fields"][i]["contextual_grade"].clone();
            data["attachments"] = if i == 0 {
                json!({"lookup":[1,2,3,4]})
            } else {
                json!({})
            };
            m.owner.graph.vertex(k, data, false).unwrap();
        }
        let physical: Vec<_> = dots
            .iter()
            .map(|d| {
                let id = d["glyph_id"].as_u64().unwrap() as usize;
                (
                    id,
                    crate::beam_dispatch::glyph_json(
                        &m.owner.prepared.registry.get(id).unwrap().glyph,
                    ),
                )
            })
            .collect();
        let edge_id = |kind: &str, source: usize, target: usize| {
            initial["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["kind"] == kind && e["source"] == source && e["target"] == target)
                .unwrap()["id"]
                .as_u64()
                .unwrap() as usize
        };
        let old = edge_id("AugmentationRelation", dot_ids[0], note_ids[0]);
        let double = edge_id("DoubleDotRelation", dot_ids[1], dot_ids[0]);
        let relation = json!({"kind":"AugmentationRelation","grade":0.7});
        let mut latest: [i64; 4] = serde_json::from_value(dots[0]["bounds"].clone()).unwrap();
        latest[0] += 3;
        let mut task = DefaultEditingTask::new(
            &m.owner.graph,
            key(&dots[0]).unwrap(),
            latest,
            vec![
                EditingLink::new(key(&notes[0]).unwrap(), true, relation.clone()),
                EditingLink::new(key(&notes[1]).unwrap(), true, relation),
            ],
            vec![
                EditingLink::from_registered(&m.owner.graph, key(&dots[0]).unwrap(), old).unwrap(),
            ],
        )
        .unwrap();
        let state = |snapshot: &Value, edges: &[usize], registered: bool| {
            let node = |id| {
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id)
                    .unwrap()
            };
            let ownership = &snapshot["augmentation_dot_ownership"];
            json!({"owners":{
                "note_dot_counts":note_ids.iter().map(|id|note_count(snapshot,node(*id))).collect::<Vec<_>>(),
                "chord_dot_counts":owner_ids.iter().map(|id|node(*id)["dots_number"].clone()).collect::<Vec<_>>(),
                "dots":dot_ids.iter().map(|id|{let n=node(*id);json!({"removed":n["lifecycle_removed"]==true,"registered":n["registered"]!=false,"abnormal":n["abnormal"]})}).collect::<Vec<_>>(),
                "edges_live":edges.iter().map(|id|snapshot["relations"].as_array().unwrap().iter().any(|r|r["id"]==*id)).collect::<Vec<_>>()},
                "ownership":dot_ids.iter().map(|id|{let n=node(*id);json!({"cached_part_id":ownership["part_cache"].as_array().unwrap().iter().find(|c|c["id"]==*id).unwrap()["part_id"],"staff_id":n["staff_id"],"measure_member":ownership["measures"].as_array().unwrap().iter().any(|a|a["augmentation_dot_ids"].as_array().unwrap().contains(&json!(id))),"registered":n["registered"]!=false,"removed":n["lifecycle_removed"]==true})}).collect::<Vec<_>>(),
                "dot_fields":dot_ids.iter().map(|id|{let n=node(*id);json!({"bounds":n["bounds"],"grade":n["grade"],"contextual_grade":n["contextual_grade"],"has_glyph":!n["glyph_id"].is_null() || !n["glyph"].is_null(),"attachments":n["attachments"].as_object().map_or(0,|a|a.len())})}).collect::<Vec<_>>(),
                "original_glyph_registered":registered})
        };
        assert_eq!(
            state(&m.owner.graph.snapshot(), &[old, usize::MAX, double], true),
            original["before"]
        );
        m.owner.graph.enable_dot_callback_trace();
        let mut added = 0;
        for (i, operation) in [
            EditingOperation::Do,
            EditingOperation::Undo,
            EditingOperation::Redo,
        ]
        .into_iter()
        .enumerate()
        {
            let publication = task.perform(&mut m.owner.graph, operation).unwrap();
            assert_eq!(publication["controller_impact_complete"], false);
            if i == 0 {
                added = m.owner.graph.dot_callback_trace()[0]["id"]
                    .as_u64()
                    .unwrap() as usize;
            }
            assert_eq!(
                task.retained_links(),
                original["steps"][i]["retained_links"].as_u64().unwrap() as usize
            );
            let registered = physical.iter().all(|(id, glyph)| {
                crate::beam_dispatch::glyph_json(&m.owner.prepared.registry.get(*id).unwrap().glyph)
                    == *glyph
            });
            assert_eq!(
                state(&m.owner.graph.snapshot(), &[old, added, double], registered),
                original["steps"][i]["state"]
            );
        }
        let trace:Vec<_>=m.owner.graph.dot_callback_trace().iter().map(|event| {
            let is_edge=event["event"].as_str().unwrap().starts_with("edge");
            let ids=if is_edge {vec![old,added,double]} else {dot_ids.clone()};
            let index=ids.iter().position(|id|event["id"]==*id).unwrap();
            let mut n=json!({"event":event["event"],"stage":event["stage"],"entity":format!("{}{}",if is_edge {"edge"} else {"dot"},index),"state":state(&event["state"],&[old,added,double],true)});
            if is_edge {n["depth"]=event["depth"].clone();} n
        }).collect();
        assert_eq!(json!(trace), original["trace"]);
    }
    #[test]
    fn editing_sequence_reverses_actual_tasks_before_notifying_rhythm_impacts() {
        use crate::shared_sig::{DefaultEditingTask, EditingOperation, EditingSequence};
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-editing-task/impact-observed.json"
        ))
        .unwrap();
        let mut m = double_dot_symbolic_music();
        let snapshot = m.owner.graph.snapshot();
        let dots: Vec<_> = original["task_bounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "AugmentationDotInter" && &n["bounds"] == b)
                    .unwrap()
                    .clone()
            })
            .collect();
        let ids: Vec<_> = dots
            .iter()
            .map(|n| n["id"].as_u64().unwrap() as usize)
            .collect();
        let tasks = dots
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let mut bounds: [i64; 4] = serde_json::from_value(n["bounds"].clone()).unwrap();
                bounds[0] += 3 + i as i64;
                DefaultEditingTask::new(&m.owner.graph, key(n).unwrap(), bounds, vec![], vec![])
                    .unwrap()
            })
            .collect();
        let mut sequence = EditingSequence::new(tasks);
        let before = m.owner.graph.snapshot();
        assert_eq!(
            sequence
                .perform(&mut m.owner.graph, EditingOperation::Do, "not-a-step")
                .unwrap_err(),
            "editing_latest_step_required"
        );
        assert_eq!(m.owner.graph.snapshot(), before);
        for (i, op) in [
            EditingOperation::Do,
            EditingOperation::Undo,
            EditingOperation::Redo,
        ]
        .into_iter()
        .enumerate()
        {
            let effect = sequence.perform(&mut m.owner.graph, op, "HEADS").unwrap();
            let snap = m.owner.graph.snapshot();
            let c = &original["epilog_below_first"][i];
            let actual: Vec<_> = snap["edit_publications"].as_array().unwrap()[i * 2..i * 2 + 2]
                .iter()
                .map(|p| {
                    format!(
                        "dot{}",
                        ids.iter().position(|id| p["inter_id"] == *id).unwrap()
                    )
                })
                .collect();
            assert_eq!(json!(actual), c["task_publications"]);
            let bounds: Vec<_> = ids
                .iter()
                .map(|id| {
                    snap["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == *id)
                        .unwrap()["bounds"]
                        .clone()
                })
                .collect();
            assert_eq!(json!(bounds), c["dot_bounds"]);
            assert_eq!(
                effect["first_impacted_step"],
                original["first_impacted_step"]
            );
            assert_eq!(effect["mark_sheet_modified"], c["modified"]);
            assert_eq!(effect["planned_steps"], c["impact_calls"]);
            assert_eq!(effect["reprocessed_steps"], json!([]));
            assert_eq!(effect["controller_impact_complete"], false);
            assert_eq!(snap["edit_impacts"].as_array().unwrap().len(), i + 1);
        }
        let effect = sequence
            .perform(&mut m.owner.graph, EditingOperation::Redo, "PAGE")
            .unwrap();
        assert_eq!(effect["planned_steps"], original["planned_through_page"]);
        assert_eq!(effect["reprocessed_steps"], json!([]));
        assert_eq!(effect["controller_impact_complete"], false);
    }
    #[test]
    fn edited_current_centers_select_owned_stacks_without_rebuilding_measure_membership() {
        use crate::shared_sig::{
            DefaultEditingTask, EditingOperation, EditingSequence, NoteOwnerEdit,
        };
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-editing-task/stack-observed.json"
        ))
        .unwrap();
        let mut m = double_dot_symbolic_music();
        m.owner.graph.populate_dot_measures().unwrap();
        let snapshot = m.owner.graph.snapshot();
        let dots: Vec<_> = original["original_bounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "AugmentationDotInter" && &n["bounds"] == b)
                    .unwrap()
                    .clone()
            })
            .collect();
        let ids: Vec<_> = dots
            .iter()
            .map(|n| n["id"].as_u64().unwrap() as usize)
            .collect();
        for (i, d) in dots.iter().enumerate() {
            m.owner
                .graph
                .edit_note_owner(key(d).unwrap(), NoteOwnerEdit::Part(None))
                .unwrap();
            m.owner
                .graph
                .edit_note_owner(
                    key(d).unwrap(),
                    NoteOwnerEdit::Staff(if i == 0 { Some(1) } else { None }),
                )
                .unwrap();
        }
        let mut sequence = EditingSequence::new(
            dots.iter()
                .enumerate()
                .map(|(i, d)| {
                    DefaultEditingTask::new(
                        &m.owner.graph,
                        key(d).unwrap(),
                        serde_json::from_value(original["latest_bounds"][i].clone()).unwrap(),
                        vec![],
                        vec![],
                    )
                    .unwrap()
                })
                .collect(),
        );
        for (i, op) in [
            EditingOperation::Do,
            EditingOperation::Undo,
            EditingOperation::Redo,
        ]
        .into_iter()
        .enumerate()
        {
            let notification = sequence.perform(&mut m.owner.graph, op, "PAGE").unwrap();
            assert_eq!(
                notification["rhythm_stack_targets"],
                original["steps"][i]["stack_targets"]
            );
            assert_eq!(notification["reprocessed_steps"], json!([]));
            assert_eq!(notification["controller_impact_complete"], false);
            let s = m.owner.graph.snapshot();
            let owner = &s["augmentation_dot_ownership"];
            let nodes: Vec<_> = ids
                .iter()
                .map(|id| {
                    s["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == *id)
                        .unwrap()
                })
                .collect();
            assert_eq!(
                json!(
                    nodes
                        .iter()
                        .map(|n| n["bounds"].clone())
                        .collect::<Vec<_>>()
                ),
                original["steps"][i]["dot_bounds"]
            );
            let membership = owner["measures"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["system_id"] == 1 && a["measure_index"] == 1)
                .unwrap();
            let ownership:Vec<_>=ids.iter().zip(nodes).map(|(id,n)|json!({"cached_part_id":owner["part_cache"].as_array().unwrap().iter().find(|c|c["id"]==*id).unwrap()["part_id"],"staff_id":n["staff_id"],"measure_member":membership["augmentation_dot_ids"].as_array().unwrap().contains(&json!(id)),"registered":true,"removed":false})).collect();
            assert_eq!(json!(ownership), original["steps"][i]["ownership"]);
        }
    }
    fn normalized_dot_callback_trace(
        graph: &Graph,
        notes: &[usize],
        owners: &[usize],
        dots: &[usize],
        edges: &[usize],
    ) -> Vec<Value> {
        graph.dot_callback_trace().iter().map(|event| {
            let snap=&event["state"];
            let node=|id|snap["inters"].as_array().unwrap().iter().find(|n|n["id"]==id).unwrap();
            let state=json!({
                "note_dot_counts":notes.iter().map(|id|note_count(snap,node(*id))).collect::<Vec<_>>(),
                "chord_dot_counts":owners.iter().map(|id|node(*id)["dots_number"].clone()).collect::<Vec<_>>(),
                "dots":dots.iter().map(|id|{let n=node(*id);json!({"removed":n["lifecycle_removed"],"registered":n["registered"],"abnormal":n["abnormal"]})}).collect::<Vec<_>>(),
                "edges_live":edges.iter().map(|id|snap["relations"].as_array().unwrap().iter().any(|r|r["id"]==*id)).collect::<Vec<_>>()
            });
            let (prefix,ids)=if event["event"].as_str().unwrap().starts_with("edge") {("edge",edges)}else{("dot",dots)};
            let index=ids.iter().position(|id|event["id"]==*id).unwrap();
            json!({"event":event["event"],"stage":event["stage"],"depth":event["depth"],"entity":format!("{prefix}{index}"),"state":state})
        }).collect()
    }
    #[test]
    fn original_callbacks_preserve_mixed_owner_intermediate_state_and_reentry_order() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-dot-callback-order/observed.json"
        ))
        .unwrap();
        for c in original["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let mut notes = Vec::new();
            let mut note_ids = Vec::new();
            let mut owners = Vec::new();
            let mut owner_ids = Vec::new();
            let mut dots = Vec::new();
            let mut dot_ids = Vec::new();
            for (i, n) in c["notes"].as_array().unwrap().iter().enumerate() {
                let k = Key {
                    system: 1,
                    phase: if n["kind"] == "RestInter" {
                        Phase::Symbol
                    } else {
                        Phase::Head
                    },
                    local: i + 1,
                };
                let id=graph.vertex(k,json!({"kind":n["kind"],"shape":n["shape"],"bounds":n["bounds"],"grade":0.8,"staff_id":1}),false).unwrap();
                notes.push(k);
                note_ids.push(id);
            }
            for (i, b) in c["owner_bounds"].as_array().unwrap().iter().enumerate() {
                let k = Key {
                    system: 1,
                    phase: Phase::Chord,
                    local: i + 1,
                };
                let id=graph.vertex(k,json!({"kind":if i==0 {"HeadChordInter"}else{"RestChordInter"},"bounds":b,"part_id":1,"staff_id":1,"measure_index":1,"duration":null,"voice":null,"dots_number":c["before"]["chord_dot_counts"][i]}),false).unwrap();
                owners.push(k);
                owner_ids.push(id);
            }
            for (i, n) in notes.iter().enumerate() {
                graph
                    .relations(
                        1,
                        Phase::Chord,
                        vec![(
                            owners[usize::from(i == 2)],
                            *n,
                            json!({"kind":"Containment"}),
                        )],
                    )
                    .unwrap();
            }
            for (i, b) in c["dot_bounds"].as_array().unwrap().iter().enumerate() {
                let k = Key {
                    system: 1,
                    phase: Phase::Symbol,
                    local: i + 4,
                };
                dot_ids.push(graph.vertex(k,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":b,"grade":0.8,"staff_id":1,"glyph_id":i+1,"manual":i==1 && c["remaining_manual"]==true,"abnormal":true}),false).unwrap());
                dots.push(k);
            }
            let rels = c["edge_targets"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    (
                        dots[usize::from(i == 2)],
                        notes[n.as_u64().unwrap() as usize],
                        json!({"kind":"AugmentationRelation","grade":0.7}),
                    )
                })
                .collect();
            graph.relations(1, Phase::Symbol, rels).unwrap();
            let edge_ids = graph.snapshot()["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "AugmentationRelation")
                .map(|e| e["id"].as_u64().unwrap() as usize)
                .collect::<Vec<_>>();
            graph.enable_dot_callback_trace();
            graph.remove_music_inter(dots[0]).unwrap();
            assert_eq!(
                json!(normalized_dot_callback_trace(
                    &graph, &note_ids, &owner_ids, &dot_ids, &edge_ids
                )),
                c["trace"],
                "rest first={},manual={}",
                c["rest_edge_first"],
                c["remaining_manual"]
            );
            let removed = c["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["event"] == "vertexRemoved" && e["stage"] == "enter")
                .map(|e| dot_ids[e["entity"].as_str().unwrap()[3..].parse::<usize>().unwrap()])
                .collect::<Vec<_>>();
            assert_eq!(
                graph
                    .pending_dot_removals()
                    .iter()
                    .map(|n| n["id"].as_u64().unwrap() as usize)
                    .collect::<Vec<_>>(),
                removed
            );
            let receipts = reduce_consistencies(&mut graph, &[]).unwrap();
            let counted = receipts["counting"]["discarded_inconsistent_dots"]
                .as_array()
                .unwrap();
            assert_eq!(counted.len(), usize::from(c["remaining_manual"] != true));
            if let Some(receipt) = counted.first() {
                assert_eq!(receipt["id"], dot_ids[1]);
                assert_eq!(receipt["chord_id"], owner_ids[0]);
                assert_eq!(receipt["note_id"], note_ids[1]);
                assert_eq!(receipt["bounds"], c["dot_bounds"][1]);
                assert_eq!(receipt["member_dot_counts"], json!([0, 1]));
            }
            assert_eq!(
                receipts["counting"]["deleted_dot_inters"]
                    .as_array()
                    .unwrap()
                    .len(),
                removed.len()
            );
            assert_eq!(
                reduce_consistencies(&mut graph, &[]).unwrap()["counting"]["deleted_dot_inters"],
                json!([])
            );
        }
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-dot-callback-order/actual-chain.json"
        ))
        .unwrap();
        let mut m = double_dot_symbolic_music();
        contextualize_and_count(&mut m.owner.graph, &m.chords).unwrap();
        let snap = m.owner.graph.snapshot();
        let by_bounds = |kind, b: &Value| {
            snap["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["kind"] == kind && &n["bounds"] == b)
                .unwrap()
        };
        let head = by_bounds("HeadInter", &original["notes"][0]["bounds"]);
        let chord = by_bounds("HeadChordInter", &original["owner_bounds"][0]);
        let dots = original["dot_bounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| by_bounds("AugmentationDotInter", b))
            .collect::<Vec<_>>();
        let ids = dots
            .iter()
            .map(|d| d["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let edges = [
            ("AugmentationRelation", ids[0]),
            ("DoubleDotRelation", ids[1]),
        ]
        .map(|(kind, source)| {
            snap["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["kind"] == kind && e["source"] == source)
                .unwrap()["id"]
                .as_u64()
                .unwrap() as usize
        });
        m.owner.graph.enable_dot_callback_trace();
        m.owner
            .graph
            .remove_music_inter(key(dots[0]).unwrap())
            .unwrap();
        assert_eq!(
            json!(normalized_dot_callback_trace(
                &m.owner.graph,
                &[head["id"].as_u64().unwrap() as usize],
                &[chord["id"].as_u64().unwrap() as usize],
                &ids,
                &edges
            )),
            original["trace"]
        );
    }
    #[test]
    fn weak_dot_callback_records_strong_cascade_without_snapshot_resurrection() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-dot-weak-cascade/observed.json"
        ))
        .unwrap();
        let mut graph = Graph::default();
        let ck = Key {
            system: 1,
            phase: Phase::Chord,
            local: 1,
        };
        let cid = graph.vertex(ck, json!({"kind":"HeadChordInter","bounds":original["chord_bounds"],"part_id":1,"staff_id":1,"duration":null,"voice":null,"dots_number":1}), false).unwrap();
        let mut notes = Vec::new();
        let mut dots = Vec::new();
        let mut note_ids = Vec::new();
        let mut dot_ids = Vec::new();
        for (i, n) in original["notes"].as_array().unwrap().iter().enumerate() {
            let nk = Key {
                system: 1,
                phase: Phase::Head,
                local: i + 1,
            };
            note_ids.push(graph.vertex(nk, json!({"kind":n["kind"],"shape":n["shape"],"bounds":n["bounds"],"grade":n["grade"],"staff_id":1}), false).unwrap());
            notes.push(nk);
            let dk = Key {
                system: 1,
                phase: Phase::Symbol,
                local: i + 1,
            };
            dot_ids.push(graph.vertex(dk, json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":original["dot_bounds"][i],"grade":original["dot_grades"][i],"glyph_id":i+1,"staff_id":1,"abnormal":true}), false).unwrap());
            dots.push(dk);
            graph
                .relations(
                    1,
                    Phase::Chord,
                    vec![(ck, nk, json!({"kind":"Containment"}))],
                )
                .unwrap();
            graph
                .relations(
                    1,
                    Phase::Symbol,
                    vec![(dk, nk, json!({"kind":"AugmentationRelation","grade":0.7}))],
                )
                .unwrap();
        }
        let before = graph.snapshot();
        let edge_ids = before["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "AugmentationRelation")
            .map(|e| e["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let chord = Chord {
            kind: ChordKind::Head,
            key: ck,
            id: cid,
            stem: None,
            members: before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["kind"] == "HeadInter")
                .cloned()
                .collect(),
            measure: 0,
            bounds: serde_json::from_value(original["chord_bounds"].clone()).unwrap(),
        };
        let mut frozen = graph.clone();
        let mut strong = before["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == dot_ids[1])
            .unwrap()
            .clone();
        strong["frozen"] = json!(true);
        frozen.vertex(dots[1], strong, false).unwrap();
        let frozen_before = frozen.snapshot();
        assert!(contextualize_and_count(&mut frozen, std::slice::from_ref(&chord)).is_err());
        assert_eq!(frozen.snapshot(), frozen_before);
        assert!(frozen.pending_dot_removals().is_empty());
        graph.enable_dot_callback_trace();
        let counted = contextualize_and_count(&mut graph, std::slice::from_ref(&chord)).unwrap();
        assert_eq!(
            json!(normalized_dot_callback_trace(
                &graph,
                &note_ids,
                &[cid],
                &dot_ids,
                &edge_ids
            )),
            original["trace"]
        );
        let deletions = counted["deleted_dot_inters"].as_array().unwrap();
        assert_eq!(
            deletions
                .iter()
                .map(|n| n["id"].clone())
                .collect::<Vec<_>>(),
            vec![json!(dot_ids[1]), json!(dot_ids[0])]
        );
        for (receipt, index) in deletions.iter().zip([1, 0]) {
            assert_eq!(receipt["bounds"], original["dot_bounds"][index]);
            assert_eq!(receipt["glyph_id"], index + 1);
            assert_eq!(receipt["grade"], original["dot_grades"][index]);
            assert!(
                (receipt["contextual_grade"].as_f64().unwrap()
                    - original["dot_contextual_grades"][index].as_f64().unwrap())
                .abs()
                    < 1e-12
            );
        }
        assert_eq!(deletions[0]["chord_id"], cid);
        assert_eq!(deletions[0]["note_id"], note_ids[1]);
        assert_eq!(deletions[0]["member_dot_counts"], json!([0, 1]));
        assert_eq!(
            deletions[0]["reason"],
            "AbstractChord.countDots_inconsistent_nonmanual_zero"
        );
        assert_eq!(deletions[1]["reason"], "Inter.remove");
        assert!(
            graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .all(|n| n["kind"] != "AugmentationDotInter")
        );
        let linked = reduce_consistencies(&mut graph, std::slice::from_ref(&chord)).unwrap();
        assert_eq!(
            linked["counting"]["deleted_dot_inters"],
            counted["deleted_dot_inters"]
        );
        assert_eq!(
            linked["counting"]["discarded_inconsistent_dots"],
            json!([deletions[0]])
        );
        assert_eq!(
            reduce_consistencies(&mut graph, &[chord]).unwrap()["counting"]["deleted_dot_inters"],
            json!([])
        );
    }
    #[test]
    fn dot_vertex_callback_recounts_chain_and_cascades_only_automatic_chord_dots() {
        let f: Value = serde_json::from_str(include_str!(
            "../fixtures/official-dot-vertex-lifecycle/observed.json"
        ))
        .unwrap();
        let mut m = double_dot_symbolic_music();
        contextualize_and_count(&mut m.owner.graph, &m.chords).unwrap();
        let c = &f["cases"][0];
        let snapshot = m.owner.graph.snapshot();
        let dots = c["dot_bounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["kind"] == "AugmentationDotInter" && &n["bounds"] == b)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let physical = dots
            .iter()
            .map(|d| {
                let gid = d["glyph_id"].as_u64().unwrap() as usize;
                (
                    gid,
                    crate::beam_dispatch::glyph_json(
                        &m.owner.prepared.registry.get(gid).unwrap().glyph,
                    ),
                )
            })
            .collect::<Vec<_>>();
        let observe = |graph: &Graph, keys: &[Key], chord: usize, expected: &Value| {
            let snap = graph.snapshot();
            for (k, e) in keys.iter().zip(expected["dots"].as_array().unwrap()) {
                let id = graph.id(*k).unwrap();
                let live = snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id);
                assert_eq!(live.is_none(), e["removed"]);
                if let Some(n) = live {
                    assert_eq!(n["abnormal"], e["abnormal"]);
                }
            }
            assert_eq!(
                snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == chord)
                    .unwrap()["dots_number"],
                expected["chord_dots"]
            );
        };
        let chord = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == c["chord_bounds"])
            .unwrap();
        let cid = chord["id"].as_u64().unwrap() as usize;
        let keys = dots.iter().map(|d| key(d).unwrap()).collect::<Vec<_>>();
        m.owner.graph.remove_music_inter(keys[0]).unwrap();
        observe(
            &m.owner.graph,
            &keys,
            cid,
            &c["after_first_dot_vertex_remove"],
        );
        let removed = m.owner.graph.snapshot();
        m.owner.graph.remove_music_inter(keys[0]).unwrap();
        assert_eq!(m.owner.graph.snapshot(), removed);
        assert_eq!(
            check_targets(&mut m.owner.graph).unwrap()["removed_orphan_dot_ids"]
                .as_array()
                .unwrap()
                .len() as u64,
            c["target_modifications"].as_u64().unwrap()
        );
        observe(&m.owner.graph, &keys, cid, &c["after_target_check"]);
        for (gid, ink) in physical {
            assert_eq!(
                crate::beam_dispatch::glyph_json(
                    &m.owner.prepared.registry.get(gid).unwrap().glyph
                ),
                ink
            );
        }
        for c in f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["kind"] == "declared_multi")
        {
            let mut graph = Graph::default();
            let ck = Key {
                system: 1,
                phase: Phase::Chord,
                local: 1,
            };
            let cid=graph.vertex(ck,json!({"kind":"HeadChordInter","bounds":c["chord_bounds"],"part_id":1,"staff_id":1,"measure_index":1,"duration":null,"voice":null,"dots_number":c["before"]["chord_dots"]}),false).unwrap();
            let mut dots = Vec::new();
            for (i, b) in c["head_bounds"].as_array().unwrap().iter().enumerate() {
                let hk = Key {
                    phase: Phase::Head,
                    local: i + 1,
                    ..ck
                };
                let dk = Key {
                    phase: Phase::Symbol,
                    ..hk
                };
                graph.vertex(hk,json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK","bounds":b,"grade":0.8,"staff_id":1}),false).unwrap();
                graph.vertex(dk,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":c["dot_bounds"][i],"grade":0.8,"staff_id":1,"manual":i==1 && c["remaining_dot_manual"]==true,"abnormal":true}),false).unwrap();
                graph
                    .relations(
                        1,
                        Phase::Chord,
                        vec![(ck, hk, json!({"kind":"Containment"}))],
                    )
                    .unwrap();
                graph
                    .relations(
                        1,
                        Phase::Symbol,
                        vec![(dk, hk, json!({"kind":"AugmentationRelation","grade":0.7}))],
                    )
                    .unwrap();
                dots.push(dk);
            }
            if c["remaining_dot_manual"] != true {
                let mut frozen = graph.clone();
                let mut data = frozen.snapshot()["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == frozen.id(dots[1]).unwrap())
                    .unwrap()
                    .clone();
                data["frozen"] = json!(true);
                frozen.vertex(dots[1], data, false).unwrap();
                let before = frozen.snapshot();
                assert_eq!(
                    frozen.remove_music_inter(dots[0]).unwrap_err(),
                    "music_removal_frozen_inter"
                );
                assert_eq!(frozen.snapshot(), before);
            }
            graph.remove_music_inter(dots[0]).unwrap();
            observe(&graph, &dots, cid, &c["after_first_dot_vertex_remove"]);
            assert!(
                check_targets(&mut graph).unwrap()["removed_orphan_dot_ids"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
            observe(&graph, &dots, cid, &c["after_target_check"]);
        }
    }
    #[test]
    fn weak_rest_removal_marks_and_cleans_its_actual_orphan_dot() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rest-dot-orphan/observed.json"
        ))
        .unwrap();
        let expected = &original["cases"][0];
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-augmentation/half-rest-augmentation.png"
        ))
        .unwrap();
        let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
        let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
        let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
        let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
        let owner = crate::music_links::finalize_stems(owner).unwrap();
        let m = crate::music_measures::continue_measures(owner).unwrap();
        let mut m = crate::music_symbols::continue_symbols(m).unwrap();
        let snapshot = m.owner.graph.snapshot();
        let rest = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "RestInter" && n["bounds"] == expected["rest_bounds"])
            .unwrap();
        let dot = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "AugmentationDotInter" && n["bounds"] == expected["dot_bounds"])
            .unwrap();
        assert_eq!(dot["abnormal"], expected["before_dot_abnormal"]);
        let gid = dot["glyph_id"].as_u64().unwrap() as usize;
        let physical =
            crate::beam_dispatch::glyph_json(&m.owner.prepared.registry.get(gid).unwrap().glyph);
        let mut target = rest.clone();
        target["grade"] = expected["declared_target_grade"].clone();
        m.owner
            .graph
            .vertex(key(rest).unwrap(), target, false)
            .unwrap();
        crate::music_symbol_reduction::reduce(&mut m.owner.graph).unwrap();
        let after = m.owner.graph.snapshot();
        assert!(
            after["inters"]
                .as_array()
                .unwrap()
                .iter()
                .all(|n| n["id"] != rest["id"])
        );
        let orphan = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == dot["id"])
            .unwrap();
        assert_eq!(
            orphan["abnormal"],
            expected["after_original_contextualizeAndPurge"]["dot_abnormal"]
        );
        assert!(
            after["relations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| e["kind"] != "AugmentationRelation")
        );
        let containers = crate::music_containers::refresh(
            &mut m.owner.graph,
            &m.measures,
            &mut m.chords,
            &mut m.voices,
        )
        .unwrap();
        assert_eq!(containers.removed_chords.len(), 1);
        let mut frozen = m.owner.graph.clone();
        let mut data = orphan.clone();
        data["frozen"] = json!(true);
        frozen.vertex(key(dot).unwrap(), data, false).unwrap();
        let before = frozen.snapshot();
        assert_eq!(
            check_targets(&mut frozen).unwrap_err(),
            "music_removal_frozen_inter"
        );
        assert_eq!(frozen.snapshot(), before);
        assert_eq!(
            check_targets(&mut m.owner.graph).unwrap()["removed_orphan_dot_ids"],
            json!([dot["id"]])
        );
        assert_eq!(
            check_targets(&mut m.owner.graph).unwrap()["removed_orphan_dot_ids"],
            json!([])
        );
        assert!(
            m.owner.graph.snapshot()["inters"]
                .as_array()
                .unwrap()
                .iter()
                .all(|n| n["id"] != dot["id"])
        );
        assert_eq!(
            crate::beam_dispatch::glyph_json(&m.owner.prepared.registry.get(gid).unwrap().glyph),
            physical
        );
        contextualize_and_count(&mut m.owner.graph, &m.chords).unwrap();
        // Deleting a real weak interpretation does not synthesize missing beats.
        assert!(crate::music_xml::continue_export(m).is_err());
    }
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
    #[test]
    fn original_manual_added_support_counts_actual_owner_and_can_reenter_deletion() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-dot-support-added/observed.json"
        ))
        .unwrap();
        for c in original["cases"].as_array().unwrap().iter().rev() {
            let mut graph = Graph::default();
            let m = &c["measure"];
            graph
                .bind_dot_measures(&[Measure {
                    system: 1,
                    part: m["part_id"].as_u64().unwrap(),
                    staff: m["staff_id"].as_u64().unwrap(),
                    index: 1,
                    left: m["left"].as_i64().unwrap(),
                    right: m["right"].as_i64().unwrap(),
                    timing: Default::default(),
                }])
                .unwrap();
            let ck = Key {
                system: 1,
                phase: Phase::Chord,
                local: 1,
            };
            let cid=graph.vertex(ck,json!({"kind":"HeadChordInter","bounds":c["owner_bounds"],"part_id":1,"staff_id":1,"duration":null,"voice":null,"dots_number":0}),false).unwrap();
            let mut notes = Vec::new();
            let mut note_ids = Vec::new();
            for (i, h) in c["notes"].as_array().unwrap().iter().enumerate() {
                let k = Key {
                    system: 1,
                    phase: Phase::Head,
                    local: i + 1,
                };
                let mut n = h.clone();
                n["staff_id"] = json!(1);
                note_ids.push(graph.vertex(k, n, false).unwrap());
                notes.push(k);
                graph
                    .relations(
                        1,
                        Phase::Chord,
                        vec![(ck, k, json!({"kind":"Containment"}))],
                    )
                    .unwrap();
            }
            let dk = Key {
                system: 1,
                phase: Phase::Symbol,
                local: 1,
            };
            let did=graph.vertex(dk,json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":c["dot_bounds"],"grade":0.8,"staff_id":1,"glyph_id":1,"manual":c["mode"]=="manual_dot"}),false).unwrap();
            graph.enable_dot_callback_trace();
            graph.relations(1,Phase::Symbol,vec![(dk,notes[0],json!({"kind":"AugmentationRelation","grade":0.7,"manual":c["mode"]=="manual_relation"}))]).unwrap();
            let edge_ids = vec![graph.dot_callback_trace()[0]["id"].as_u64().unwrap() as usize];
            let after = graph.snapshot();
            let live = after["inters"].as_array().unwrap();
            let expected = &c["after"]["owners"];
            assert_eq!(
                live.iter().any(|n| n["id"] == did),
                expected["dots"][0]["registered"] == true,
                "{}",
                c["mode"]
            );
            assert_eq!(
                live.iter().find(|n| n["id"] == cid).unwrap()["dots_number"],
                expected["chord_dot_counts"][0]
            );
            let mut trace =
                normalized_dot_callback_trace(&graph, &note_ids, &[cid], &[did], &edge_ids);
            for (entry, raw) in trace.iter_mut().zip(graph.dot_callback_trace()) {
                let snap = &raw["state"];
                let n = snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == did)
                    .unwrap();
                let owner = &snap["augmentation_dot_ownership"];
                let part = owner["part_cache"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["id"] == did)
                    .unwrap();
                let member = owner["measures"].as_array().unwrap().iter().any(|m| {
                    m["augmentation_dot_ids"]
                        .as_array()
                        .unwrap()
                        .contains(&json!(did))
                });
                entry["state"] = json!({"owners":entry["state"],
                    "ownership":[{"cached_part_id":part["part_id"],"staff_id":n["staff_id"],"measure_member":member,"registered":n["registered"],"removed":n["lifecycle_removed"]}],
                    "dot_fields":[{"bounds":n["bounds"],"grade":n["grade"],"contextual_grade":n["contextual_grade"],"manual":n["manual"]}]});
            }
            assert_eq!(json!(trace), c["trace"], "{}", c["mode"]);
            let receipts = reduce_consistencies(&mut graph, &[]).unwrap();
            let gone = receipts["counting"]["discarded_inconsistent_dots"]
                .as_array()
                .unwrap();
            assert_eq!(gone.len(), usize::from(c["mode"] == "manual_relation"));
            if let Some(g) = gone.first() {
                assert_eq!(g["member_dot_counts"], json!([1, 0]));
                assert_eq!(g["rounded_chord_count"], 0);
                assert_eq!(g["id"], did);
            }
        }
    }
}
