// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    music_measures::{ChordKind, OwnedMusic},
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};

pub(crate) fn lookup(
    music: &OwnedMusic,
    system: u64,
    b: [i64; 4],
    shape: &str,
) -> Result<Option<(Key, Value)>, String> {
    if !music
        .chords
        .iter()
        .any(|c| c.key.system == system && c.kind == ChordKind::Head)
    {
        return Ok(None);
    }
    let (above, below) = match shape {
        "MARCATO" | "STACCATISSIMO" => (true, false),
        "MARCATO_BELOW" | "STACCATISSIMO_BELOW" => (false, true),
        "STACCATO" | "ACCENT" | "TENUTO" => (false, false),
        _ => return Err("music_articulation_shape_required".into()),
    };
    let profile = music
        .owner
        .systems
        .iter()
        .find(|s| s.id == system)
        .ok_or("music_articulation_actual_System_required")?
        .profile;
    let (xmax, ymax, ymin) = match profile {
        0 => (0.8, 3., 0.1),
        1 => (1., 5., 0.05),
        _ => return Err("music_articulation_profile_pending".into()),
    };
    let il = music.owner.interline as f64;
    let dx = (xmax * il).round_ties_even() as i64;
    let dy = (ymax * il).round_ties_even() as i64;
    let min_dy = (ymin * il).round_ties_even() as i64;
    let p = [b[0] + b[2] / 2, b[1] + b[3] / 2];
    let around = crate::grid_staff_neighborhood::around(
        &music.owner.staffs,
        &music.owner.staff_areas,
        &music.owner.parts,
        system,
        p.map(|v| v as f64),
    )?;
    let top = around
        .staffs
        .first()
        .ok_or("music_articulation_actual_Staff_source_required")?;
    let ordinate = |staff: &crate::grid_ir::StaffGeometry, index: usize| -> Result<i64, String> {
        let line = staff
            .lines
            .get(index)
            .ok_or("music_articulation_standard_Staff_required")?;
        Ok(line.y(p[0] as f64).round_ties_even() as i64)
    };
    let (min_y, max_y) = if around.staffs.len() == 2 {
        (
            if above { p[1] } else { ordinate(top, 0)? },
            if below {
                p[1]
            } else {
                ordinate(&around.staffs[1], 4)?
            },
        )
    } else if (p[1] as f64) < top.lines[0].y(p[0] as f64) {
        if below {
            return Ok(None);
        }
        (if above { p[1] } else { p[1] - dy }, ordinate(top, 4)?)
    } else if (p[1] as f64) > top.lines[4].y(p[0] as f64) {
        if above {
            return Ok(None);
        }
        (ordinate(top, 0)?, if below { p[1] } else { p[1] + dy })
    } else {
        (
            if above { p[1] } else { ordinate(top, 0)? },
            if below { p[1] } else { ordinate(top, 4)? },
        )
    };
    let lu = [p[0] - dx, min_y, 2 * dx + 1, max_y - min_y + 1];
    let mut chords = music
        .chords
        .iter()
        .filter(|c| c.key.system == system && c.kind == ChordKind::Head)
        .collect::<Vec<_>>();
    chords.sort_by_key(|c| c.bounds[0]);
    let intersects = |a: [i64; 4], b: [i64; 4]| {
        a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
    };
    let mut best = None;
    for chord in chords {
        let cb = chord.bounds;
        if !intersects(cb, lu) || intersects(cb, b) {
            continue;
        }
        let center = [cb[0] + cb[2] / 2, cb[1] + cb[3] / 2];
        let gap = if p[1] > center[1] {
            p[1] - (cb[1] + cb[3])
        } else {
            cb[1] - p[1]
        };
        if gap < min_dy {
            continue;
        }
        let xgap = (center[0] - p[0]).abs() as f64 / il;
        let ygap = gap.abs() as f64 / il;
        let grade = crate::music_dots::gap_grade(xgap, ygap, xmax, ymax, 3., 1.);
        if grade >= 0.1 && best.as_ref().is_none_or(|(_, _, old)| ygap < *old) {
            best = Some((
                chord.key,
                json!({"kind":"ChordArticulationRelation","grade":grade,"dx":xgap,"dy":ygap,"source_ratio":1.,"target_ratio":1.+2.5*grade,"manual":false}),
                ygap,
            ));
        }
    }
    Ok(best.map(|(key, relation, _)| (key, relation)))
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
    let data = create(glyph_id, glyph.clone(), shape, grade)?;
    let bounds = serde_json::from_value(glyph["bounds"].clone())
        .map_err(|_| "music_articulation_bounds_required")?;
    let Some((chord, relation)) = lookup(music, system, bounds, shape)? else {
        return Ok(None);
    };
    let mut next = graph.clone();
    let key = next.next_symbol_key(system)?;
    next.vertex(key, data, false)?;
    next.relations(system, Phase::Symbol, vec![(chord, key, relation)])?;
    *graph = next;
    Ok(Some(key))
}
pub(crate) fn create(
    glyph_id: usize,
    glyph: Value,
    shape: &str,
    grade: f64,
) -> Result<Value, String> {
    if !grade.is_finite() || !(0.0..=1.0).contains(&grade) {
        return Err("music_articulation_grade_required".into());
    }
    Ok(
        json!({"kind":"ArticulationInter","shape":shape,"bounds":glyph["bounds"],"glyph_id":glyph_id,"glyph":glyph,"grade":grade,"manual":false,"abnormal":false,"staff_id":null,"contextual_grade":null}),
    )
}

pub(crate) fn contextualize_and_purge(
    graph: &mut Graph,
    chords: &[crate::music_measures::Chord],
) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut staged = graph.clone();
    let mut contexts = Vec::new();
    let mut weak = Vec::new();
    for n in nodes.iter().filter(|n| n["kind"] == "ArticulationInter") {
        let mut sources = Vec::new();
        let supports = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "ChordArticulationRelation" && e["target"] == n["id"])
            .collect::<Vec<_>>();
        if supports.len() > 1 {
            return Err("music_articulation_single_Chord_source_required".into());
        }
        let mut sum = 0.;
        for e in supports {
            let chord = chords
                .iter()
                .find(|c| c.id == e["source"] && c.kind == ChordKind::Head)
                .ok_or("music_articulation_actual_Chord_required")?;
            if graph.id(chord.key) != Some(chord.id) {
                return Err("music_articulation_actual_Chord_required".into());
            }
            let partner = nodes
                .iter()
                .find(|c| c["id"] == chord.id)
                .ok_or("music_articulation_live_Chord_required")?;
            // Chord grade is the mean cached at actual membership invalidation.
            // Recomputing it from today's Head grades changes original support.
            let grade = partner["grade"]
                .as_f64()
                .filter(|g| (0.0..=1.0).contains(g))
                .ok_or("music_articulation_cached_Chord_grade_required")?;
            sum += (e["target_ratio"]
                .as_f64()
                .ok_or("music_articulation_support_ratio_required")?
                - 1.)
                * grade;
            sources.push(json!({"relation_grade":e["grade"],"partner_grade":grade,"partner_bounds":partner["bounds"]}));
        }
        let grade = n["grade"]
            .as_f64()
            .filter(|g| (0.0..=1.0).contains(g))
            .ok_or("music_articulation_grade_required")?;
        let contextual = ((1. + sum) * grade) / (1. + sum * grade);
        let key: Key = serde_json::from_value(n["origin"].clone())
            .map_err(|_| "music_articulation_origin_required")?;
        let mut data = n.clone();
        data["contextual_grade"] = json!(contextual);
        staged.vertex(key, data, false)?;
        contexts.push(json!({"id":n["id"],"bounds":n["bounds"],"shape":n["shape"],"grade":grade,"contextual_grade":contextual,"sources":sources}));
        if contextual < 0.5 && n["frozen"] != true {
            staged.remove_music_inter(key)?;
            weak.push(n["id"].clone());
        }
    }
    *graph = staged;
    Ok(json!({"contextualized":contexts,"removed_weak_articulation_ids":weak}))
}

pub(crate) fn check_redundancy(
    graph: &mut Graph,
    chords: &[crate::music_measures::Chord],
) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let nodes = snapshot["inters"].as_array().unwrap();
    let mut staged = graph.clone();
    let mut removed = Vec::new();
    for chord in chords.iter().filter(|c| c.kind == ChordKind::Head) {
        let mut shapes = Vec::<(String, Vec<&Value>)>::new();
        for e in snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "ChordArticulationRelation" && e["source"] == chord.id)
        {
            let n = nodes
                .iter()
                .find(|n| n["id"] == e["target"])
                .ok_or("music_articulation_live_target_required")?;
            let shape = n["shape"]
                .as_str()
                .ok_or("music_articulation_shape_required")?;
            if let Some((_, list)) = shapes.iter_mut().find(|(s, _)| s == shape) {
                list.push(n);
            } else {
                shapes.push((shape.to_owned(), vec![n]));
            }
        }
        for (_, mut list) in shapes {
            for n in &list {
                n["grade"]
                    .as_f64()
                    .filter(|g| (0.0..=1.0).contains(g))
                    .ok_or("music_articulation_grade_required")?;
            }
            // checkArticulations uses stable reverse intrinsic grade, even if
            // contextual grades disagree. Keep actual relation encounter order.
            list.sort_by(|a, b| {
                b["grade"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&a["grade"].as_f64().unwrap())
            });
            for n in list.into_iter().skip(1) {
                let key = serde_json::from_value(n["origin"].clone())
                    .map_err(|_| "music_articulation_origin_required")?;
                staged.remove_music_inter(key)?;
                removed.push(n["id"].clone());
            }
        }
    }
    *graph = staged;
    Ok(json!({"removed_redundant_articulation_ids":removed}))
}

#[cfg(test)]
pub(crate) mod image_tests;
#[cfg(test)]
pub(crate) use image_tests::assert_actual_factory;
