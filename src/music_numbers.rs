// SPDX-License-Identifier: AGPL-3.0-or-later
//! Temporary NumberInter factories and the ordinary LINKS no-partner lifecycle.
use crate::{
    beam_dispatch,
    grid_ir::StaffGeometry,
    music_measures::OwnedMusic,
    shared_sig::{Graph, Key},
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) fn value(shape: &str) -> Option<u32> {
    Some(match shape {
        "TIME_TWO" => 2,
        "TIME_THREE" => 3,
        "TIME_FOUR" => 4,
        "TIME_FIVE" => 5,
        "TIME_SIX" => 6,
        "TIME_SEVEN" => 7,
        "TIME_EIGHT" => 8,
        "TIME_NINE" => 9,
        "TIME_TWELVE" => 12,
        "TIME_SIXTEEN" => 16,
        _ => return None,
    })
}
pub(crate) fn create(
    m: &OwnedMusic,
    glyph: &GlyphPixels,
    staff: &Arc<StaffGeometry>,
    shape: &str,
    grade: f64,
    glyph_id: usize,
) -> Result<Option<Value>, String> {
    if staff.lines.len() != 5 {
        return Err("music_number_non_five_line_staff_pending".into());
    }
    // Factory admission uses the integer mass centroid; lookup later uses
    // the rectangle center. Near pitch 10 these accept different candidates.
    let centroid = beam_dispatch::centroid(glyph)?;
    let pitch = m
        .owner
        .ledger_memberships
        .pitch_at(staff, centroid.map(|v| v as f64))?;
    if pitch.abs() > 10. {
        return Ok(None);
    }
    Ok(Some(
        json!({"kind":"NumberInter","shape":shape,"value":value(shape).ok_or("music_number_shape_required")?,"bounds":[glyph.left,glyph.top,glyph.width,glyph.height],"glyph":beam_dispatch::glyph_json(glyph),"glyph_id":glyph_id,"staff_id":staff.id,"grade":grade,"manual":false,"abnormal":true,"frozen":false}),
    ))
}
fn ordinate(m: &OwnedMusic, staff: &Arc<StaffGeometry>, x: f64, pitch: f64) -> Result<f64, String> {
    let span = (staff.lines.len() - 1) as f64;
    let top = staff.lines.first().unwrap().y(x);
    let bottom = staff.lines.last().unwrap().y(x);
    if pitch.abs() <= span {
        return Ok(0.5 * (top + bottom + pitch * (bottom - top) / span));
    }
    let dir = if pitch < 0. { -1 } else { 1 };
    let (mut y, mut pp) = if dir < 0 {
        (top, -span)
    } else {
        (bottom, span)
    };
    let map = m
        .owner
        .ledger_memberships
        .maps
        .iter()
        .find(|map| Arc::ptr_eq(&map.staff, staff))
        .ok_or("music_number_actual_ledger_map_required")?;
    for level in 1..=1024 {
        let Some(line) = map.virtual_lines.get(&(dir * level)) else {
            break;
        };
        let next_pp = pp + 2. * dir as f64;
        let next_y = line.y(x);
        if (next_pp - pitch) * dir as f64 >= 0. {
            return Ok(y + (next_y - y) * (pitch - pp) / (next_pp - pp));
        }
        y = next_y;
        pp = next_pp;
    }
    Ok(y + 0.5 * (pitch - pp) * staff.specific_interline as f64)
}
fn search(m: &OwnedMusic, graph: &Graph, key: Key) -> Result<Value, String> {
    let n = graph.retained_inter(key)?.0;
    let b: [i64; 4] =
        serde_json::from_value(n["bounds"].clone()).map_err(|_| "music_number_bounds_required")?;
    let p = [(b[0] + b[2] / 2) as f64, (b[1] + b[3] / 2) as f64];
    let system = m
        .owner
        .systems
        .iter()
        .find(|s| s.id == key.system)
        .ok_or("music_number_actual_system_required")?;
    let Some(staff) = crate::music_symbols::geometry::closest(&m.owner, system, p)? else {
        return Ok(json!({"id":graph.id(key),"closest_staff":null,"search_links":[]}));
    };
    if staff.lines.len() != 5 {
        return Err("music_number_non_five_line_staff_pending".into());
    }
    let pitch = m.owner.ledger_memberships.pitch_at(&staff, p)?;
    let distance = (staff.lines.first().unwrap().y(p[0]) - p[1])
        .max(p[1] - staff.lines.last().unwrap().y(p[0])) as i32;
    let inside =
        p[0] >= staff.abscissae[0] as f64 && p[0] <= staff.abscissae[1] as f64 && distance <= 0;
    if inside {
        let y = ordinate(m, &staff, p[0], -pitch)?.round_ties_even();
        for partner in graph.live_inter_keys(key.system, "TimeNumberInter") {
            let tn = graph.retained_inter(partner)?.0;
            let b: [i64; 4] = serde_json::from_value(tn["bounds"].clone())
                .map_err(|_| "music_number_partner_bounds_required")?;
            if p[0] >= b[0] as f64
                && p[0] < (b[0] + b[2]) as f64
                && y >= b[1] as f64
                && y < (b[1] + b[3]) as f64
            {
                return Err("music_number_time_pair_conversion_pending".into());
            }
        }
    } else if pitch.abs() <= 10. {
        for kind in ["MultipleRestInter", "MeasureRepeatInter"] {
            for partner in graph.live_inter_keys(key.system, kind) {
                let pn = graph.retained_inter(partner)?.0;
                if pn["staff_id"] != staff.id {
                    continue;
                }
                let b: [i64; 4] = serde_json::from_value(pn["bounds"].clone())
                    .map_err(|_| "music_number_partner_bounds_required")?;
                if p[0] < b[0] as f64 || p[0] >= (b[0] + b[2]) as f64 {
                    continue;
                }
                if kind == "MeasureRepeatInter" && !pn["shape"].is_null() {
                    let count = match pn["shape"].as_str() {
                        Some("REPEAT_ONE_BAR") => 1,
                        Some("REPEAT_TWO_BARS") => 2,
                        Some("REPEAT_FOUR_BARS") => 4,
                        _ => return Err("music_number_repeat_shape_required".into()),
                    };
                    if n["value"] != count {
                        continue;
                    }
                }
                return Err("music_number_measure_count_conversion_pending".into());
            }
        }
    }
    Ok(
        json!({"id":graph.id(key),"closest_staff":staff.id,"center_pitch":pitch,"center_inside_staff":inside,"search_links":[]}),
    )
}
pub(crate) fn link_and_remove(m: &OwnedMusic, graph: &mut Graph) -> Result<Value, String> {
    let mut next = graph.clone();
    let mut removed = Vec::new();
    for system in &m.owner.systems {
        for key in graph.live_inter_keys(system.id, "NumberInter") {
            let observation = search(m, &next, key)?;
            // SymbolsLinker removes even an unlinked temporary NumberInter.
            // Retained vertex/edge identities and callback clocks belong to SIG.
            next.remove_music_inter(key)?;
            removed.push(observation);
        }
    }
    *graph = next;
    Ok(json!({"removed_numbers":removed,"positive_conversion_complete":false}))
}

#[cfg(test)]
mod tests;
