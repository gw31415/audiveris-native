// SPDX-License-Identifier: AGPL-3.0-or-later
//! BasicTimeColumn failed whole-symbol columns on the held image owners.
use crate::{
    beam_dispatch,
    music_measures::OwnedMusic,
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn key(n: &Value) -> Result<Key, String> {
    serde_json::from_value(n["origin"].clone())
        .map_err(|_| "music_time_actual_origin_required".into())
}
fn bounds(n: &Value) -> Result<[i64; 4], String> {
    let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
        .map_err(|_| "music_time_actual_bounds_required")?;
    if b[2] <= 0 || b[3] <= 0 || b.iter().any(|v| v.unsigned_abs() > 4_000_000) {
        return Err("music_time_invalid_bounds".into());
    }
    Ok(b)
}
fn intersects(a: [i64; 4], b: [i64; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
fn grade(n: &Value) -> Result<f64, String> {
    n["grade"]
        .as_f64()
        .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
        .ok_or("music_time_actual_grade_required".into())
}
fn glyph(
    music: &OwnedMusic,
    graph: &Graph,
    n: &Value,
) -> Result<audiveris_staff_removal::GlyphPixels, String> {
    let k = key(n)?;
    if n["kind"] == "StemInter" {
        let cached = music
            .owner
            .caches
            .get(&k.system)
            .ok_or("music_time_actual_stem_cache_required")?
            .registered_stem(graph, k.local)?;
        return Ok(music
            .owner
            .prepared
            .registry
            .get(cached.glyph_id())?
            .glyph
            .clone());
    }
    if !matches!(n["kind"].as_str(), Some("FlagInter" | "TimeWholeInter")) {
        return Err("music_time_neighbor_overlap_pending".into());
    }
    let id = n["glyph_id"]
        .as_u64()
        .ok_or("music_time_actual_glyph_required")? as usize;
    let held = &music.owner.prepared.registry.get(id)?.glyph;
    if beam_dispatch::glyph_json(held) != n["glyph"] {
        return Err("music_time_stale_glyph_projection".into());
    }
    Ok(held.clone())
}
fn overlaps(
    music: &OwnedMusic,
    graph: &Graph,
    neighbor: &Value,
    time: &Value,
) -> Result<bool, String> {
    if neighbor["implicit"] == true || time["implicit"] == true {
        return Ok(false);
    }
    let a = glyph(music, graph, neighbor)?;
    let b = glyph(music, graph, time)?;
    let pixels = crate::clef_candidates::pixels(&a).collect::<BTreeSet<_>>();
    Ok(crate::clef_candidates::pixels(&b).any(|p| pixels.contains(&p)))
}
pub(crate) fn handle_times(graph: &mut Graph, music: &OwnedMusic) -> Result<Value, String> {
    let mut staged = graph.clone();
    let initial = graph.snapshot();
    let nodes = initial["inters"]
        .as_array()
        .ok_or("music_time_actual_SIG_required")?;
    let mut groups: BTreeMap<(u64, usize), Vec<Key>> = BTreeMap::new();
    for n in nodes.iter().filter(|n| {
        matches!(
            n["kind"].as_str(),
            Some("TimeWholeInter" | "TimeCustomInter" | "TimeNumberInter")
        )
    }) {
        let k = key(n)?;
        let staff = music
            .owner
            .staffs
            .iter()
            .find(|s| s.system == k.system && Some(s.id) == n["staff_id"].as_u64())
            .ok_or("music_time_actual_Staff_required")?;
        let clef_key = Key {
            system: k.system,
            phase: Phase::MusicHeader,
            local: staff.id as usize,
        };
        let (header, removed) = graph.retained_inter(clef_key)?;
        if removed || header["staff_id"] != staff.id {
            return Err("music_time_actual_header_required".into());
        }
        let stop = header["header_stop"]
            .as_i64()
            .ok_or("music_time_retained_header_stop_required")?;
        let b = bounds(n)?;
        let point = [b[0] + b[2] / 2, b[1] + b[3] / 2];
        if point[0] < stop {
            continue;
        }
        let Some(stack) = graph.actual_music_stack_at(k.system, Some(point.map(|v| v as f64)))?
        else {
            continue;
        };
        if !music
            .measure_layout
            .stacks
            .iter()
            .any(|s| std::sync::Arc::ptr_eq(&s.stack, &stack))
        {
            return Err("music_time_Stack_owner_mismatch".into());
        }
        groups.entry((k.system, stack.index)).or_default().push(k);
    }
    let mut columns = Vec::new();
    let mut isolated_positive_columns = Vec::new();
    for ((system, index), set) in groups {
        if set.iter().any(|k| {
            staged
                .retained_inter(*k)
                .is_ok_and(|(n, _)| n["kind"] != "TimeWholeInter")
        }) {
            return Err("music_time_basic_numbers_pending".into());
        }
        let mut trace = Vec::new();
        let mut failed = false;
        for staff in music.owner.staffs.iter().filter(|s| s.system == system) {
            let mut candidates = set
                .iter()
                .filter(|k| {
                    staged
                        .retained_inter(**k)
                        .is_ok_and(|(n, removed)| !removed && n["staff_id"] == staff.id)
                })
                .copied()
                .collect::<Vec<_>>();
            let mut removed_by_filter = Vec::new();
            let before_candidates=candidates.iter().map(|k|{let (n,removed)=staged.retained_inter(*k).unwrap();json!({"kind":n["kind"],"bounds":n["bounds"],"grade":n["grade"],"manual":n["manual"]==true,"removed":removed,"staff":n["staff_id"]})}).collect::<Vec<_>>();
            for time in candidates.clone() {
                let (n, _) = staged.retained_inter(time)?;
                let n = n.clone();
                let box_time = bounds(&n)?;
                let time_grade = grade(&n)?;
                let current = staged.snapshot();
                for neighbor in current["inters"].as_array().unwrap() {
                    let nk = key(neighbor)?;
                    if nk.system != system
                        || set.contains(&nk)
                        || neighbor["member_ids"].is_array()
                        || neighbor["members"].is_array()
                        || neighbor["kind"] == "BeamGroupInter"
                        || !intersects(bounds(neighbor)?, box_time)
                    {
                        continue;
                    }
                    if overlaps(music, &staged, neighbor, &n)? {
                        if grade(neighbor)? <= time_grade {
                            return Err("music_time_lower_neighbor_callback_pending".into());
                        }
                        staged.remove_music_inter(time)?;
                        removed_by_filter.push(
                            json!({"kind":n["kind"],"bounds":n["bounds"],"staff":n["staff_id"]}),
                        );
                        candidates.retain(|k| *k != time);
                        break;
                    }
                }
            }
            let ok = !candidates.is_empty();
            trace.push(json!({"staff":staff.id,"candidates":before_candidates,"filter_result":ok,"after_filter":set.iter().map(|k|{let (n,removed)=staged.retained_inter(*k).unwrap();json!({"kind":n["kind"],"bounds":n["bounds"],"staff":n["staff_id"],"removed":removed})}).collect::<Vec<_>>(),"removed_by_filter":removed_by_filter,"remaining_candidates":candidates.iter().map(|k|staged.id(*k)).collect::<Vec<_>>() }));
            if !ok {
                for k in &set {
                    staged.remove_music_inter(*k)?;
                }
                failed = true;
                break;
            }
        }
        if !failed {
            if set.len() != 1
                || music
                    .owner
                    .staffs
                    .iter()
                    .filter(|s| s.system == system)
                    .count()
                    != 1
            {
                return Err("music_time_positive_column_alignment_pending".into());
            }
            let (time, removed) = staged.retained_inter(set[0])?;
            if removed || grade(time)? <= 0.0 {
                return Err("music_time_positive_column_consistency_pending".into());
            }
            // A single whole on the sole Staff forms the sole alignment/value
            // vector. Selection re-registers its existing glyph and Staff;
            // every overlapping lower neighbor remains guarded above.
            glyph(music, &staged, time)?;
            isolated_positive_columns.push(json!({"system":system,"stack_index":index,"selected_id":staged.id(set[0]),"kind":time["kind"],"bounds":time["bounds"],"grade":time["grade"],"staff":time["staff_id"],"trace":trace}));
            continue;
        }
        columns.push(json!({"system":system,"stack_index":index,"trace":trace,"removed_whole_times":set.iter().map(|k|{let (n,removed)=staged.retained_inter(*k).unwrap();json!({"kind":n["kind"],"bounds":n["bounds"],"staff":n["staff_id"],"removed":removed})}).collect::<Vec<_>>() }));
    }
    *graph = staged;
    Ok(
        json!({"failed_columns":columns,"isolated_positive_columns":isolated_positive_columns,"general_positive_columns_complete":false}),
    )
}

pub(crate) fn continue_time_checks(mut music: OwnedMusic) -> Result<OwnedMusic, String> {
    let mut graph = music.owner.graph.clone();
    let receipt = handle_times(&mut graph, &music)?;
    music.owner.graph = graph;
    music.owner.prepared.report["time_late_checks"] = receipt;
    music.owner.prepared.report["shared_sig"] = music.owner.graph.snapshot();
    Ok(music)
}
#[cfg(test)]
mod image_tests;
#[cfg(test)]
pub(crate) use image_tests::{assert_isolated_positive_column, assert_real_time_columns};
