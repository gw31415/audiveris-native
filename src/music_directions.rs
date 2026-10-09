// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original Dynamics/Pedal factories, complex absorption and actual Stack lookup.
use crate::{
    grid_ir::{StaffGeometry, SystemGeometry},
    music_measures::{Chord, ChordKind, OwnedMusic},
    shared_sig::{Graph, Key, Phase},
};
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) fn signature(shape: &str) -> Option<&'static str> {
    Some(match shape {
        "DYNAMICS_PPP" => "ppp",
        "DYNAMICS_PP" => "pp",
        "DYNAMICS_P" => "p",
        "DYNAMICS_MP" => "mp",
        "DYNAMICS_MF" => "mf",
        "DYNAMICS_F" => "f",
        "DYNAMICS_FF" => "ff",
        "DYNAMICS_FFF" => "fff",
        "DYNAMICS_FP" => "fp",
        "DYNAMICS_FZ" => "fz",
        "DYNAMICS_SF" => "sf",
        "DYNAMICS_SFZ" => "sfz",
        _ => return None,
    })
}
pub(crate) fn sound(shape: &str) -> Option<u32> {
    Some(match shape {
        "DYNAMICS_PPP" => 34,
        "DYNAMICS_PP" => 45,
        "DYNAMICS_P" => 56,
        "DYNAMICS_MP" => 67,
        "DYNAMICS_MF" => 89,
        "DYNAMICS_F" | "DYNAMICS_FP" | "DYNAMICS_FZ" | "DYNAMICS_SFZ" => 100,
        "DYNAMICS_FF" => 122,
        "DYNAMICS_FFF" => 144,
        _ => return None,
    })
}
fn origin(n: &Value) -> Result<Key, String> {
    serde_json::from_value(n["origin"].clone())
        .map_err(|_| "music_direction_origin_required".into())
}
fn bounds(n: &Value) -> Result<[i64; 4], String> {
    serde_json::from_value(n["bounds"].clone())
        .map_err(|_| "music_direction_bounds_required".into())
}
fn overlap(a: [i64; 4], b: [i64; 4], i: usize) -> i64 {
    (a[i] + a[i + 2]).min(b[i] + b[i + 2]) - a[i].max(b[i])
}
pub(crate) fn absorb(graph: &mut Graph) -> Result<Value, String> {
    let snap = graph.snapshot();
    let nodes = snap["inters"].as_array().unwrap();
    let dynamics = nodes
        .iter()
        .filter(|n| n["kind"] == "DynamicsInter")
        .collect::<Vec<_>>();
    let mut complexes = dynamics
        .iter()
        .copied()
        .filter(|n| signature(n["shape"].as_str().unwrap()).unwrap().len() > 1)
        .collect::<Vec<_>>();
    complexes.sort_by(|a, b| {
        signature(b["shape"].as_str().unwrap())
            .unwrap()
            .len()
            .cmp(&signature(a["shape"].as_str().unwrap()).unwrap().len())
            .then_with(|| {
                b["grade"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&a["grade"].as_f64().unwrap())
            })
    });
    let mut next = graph.clone();
    let mut swallowed = Vec::new();
    for c in complexes {
        let cb = bounds(c)?;
        let cs = signature(c["shape"].as_str().unwrap()).unwrap();
        let ck = origin(c)?;
        for s in &dynamics {
            if c["origin"]["system"] != s["origin"]["system"] {
                continue;
            }
            let ss = signature(s["shape"].as_str().unwrap()).unwrap();
            let sb = bounds(s)?;
            if ss.len() >= cs.len()
                || !cs.contains(ss)
                || overlap(cb, sb, 0) <= 0
                || overlap(cb, sb, 1) <= 0
            {
                continue;
            }
            let ratio = (overlap(cb, sb, 0) * overlap(cb, sb, 1)) as f64 / (sb[2] * sb[3]) as f64;
            if ratio >= 0.8 {
                let mut live = next.retained_inter(ck)?.0.clone();
                live["grade"] = json!(
                    live["grade"]
                        .as_f64()
                        .unwrap()
                        .max(s["grade"].as_f64().unwrap())
                );
                // The original collection retains removed candidates during this pass.
                // Reading them may still boost another complex; removal itself is idempotent.
                let removed = next.retained_inter(ck)?.1;
                next.vertex(ck, live, removed)?;
                next.remove_music_inter(origin(s)?)?;
                swallowed.push(
                    json!({"complex_id":c["id"],"shorter_id":s["id"],"intersection_ratio":ratio}),
                );
            }
        }
    }
    *graph = next;
    Ok(json!({"swallowed":swallowed}))
}
fn sided_staff(
    m: &OwnedMusic,
    system: &SystemGeometry,
    p: [f64; 2],
    above: bool,
) -> Result<Option<Arc<StaffGeometry>>, String> {
    let Some(s) = crate::music_symbols::geometry::closest(&m.owner, system, p)? else {
        return Ok(None);
    };
    let i = system
        .staffs
        .iter()
        .position(|h| Arc::ptr_eq(h, &s))
        .ok_or("music_direction_canonical_Staff_required")?;
    if above {
        if s.lines
            .first()
            .ok_or("music_direction_Staff_lines_required")?
            .y(p[0])
            <= p[1]
        {
            Ok(Some(s))
        } else {
            Ok(i.checked_sub(1).map(|i| system.staffs[i].clone()))
        }
    } else if s
        .lines
        .last()
        .ok_or("music_direction_Staff_lines_required")?
        .y(p[0])
        >= p[1]
    {
        Ok(Some(s))
    } else {
        Ok(system.staffs.get(i + 1).cloned())
    }
}
fn standard<'a>(
    m: &'a OwnedMusic,
    graph: &Graph,
    system: &SystemGeometry,
    stack: &Arc<crate::music_measure_layout::Stack>,
    p: [f64; 2],
    area: [i64; 4],
    above: bool,
) -> Result<Option<&'a Chord>, String> {
    let Some(staff) = sided_staff(m, system, p, above)? else {
        return Ok(None);
    };
    let Some((mi, _)) = m.measures.iter().enumerate().find(|(_, measure)| {
        measure.geometry.as_ref().is_some_and(|g| {
            Arc::ptr_eq(&g.stack, stack) && g.part.staffs.iter().any(|s| Arc::ptr_eq(s, &staff))
        })
    }) else {
        return Ok(None);
    };
    let mut best = None;
    let mut distance = f64::MAX;
    for c in m.chords.iter().filter(|c| c.measure == mi) {
        let g = m.measures[mi].geometry.as_ref().unwrap();
        let members = g
            .part
            .staffs
            .iter()
            .filter(|s| c.members.iter().any(|n| n["staff_id"] == s.id))
            .collect::<Vec<_>>();
        let held = if above {
            members.last()
        } else {
            members.first()
        };
        if !held.is_some_and(|h| Arc::ptr_eq(h, &staff))
            || overlap(c.bounds, area, 0) <= 0
            || overlap(c.bounds, area, 1) >= 0
        {
            continue;
        }
        let (node, removed) = graph.retained_inter(c.key)?;
        if removed {
            continue;
        }
        let hp = if c.kind == ChordKind::Head {
            if node["head_location"].is_null() {
                None
            } else {
                Some(
                    serde_json::from_value::<[f64; 2]>(node["head_location"].clone())
                        .map_err(|_| "music_direction_actual_Chord_head_required")?,
                )
            }
        } else {
            Some([
                (c.bounds[0] + c.bounds[2] / 2) as f64,
                (c.bounds[1] + c.bounds[3] / 2) as f64,
            ])
        };
        let Some(hp) = hp else { continue };
        if (above && hp[1] >= p[1]) || (!above && hp[1] <= p[1]) {
            continue;
        }
        let b = bounds(node)?;
        let dx = (b[0] as f64 - p[0])
            .max(p[0] - (b[0] + b[2] - 1) as f64)
            .max(0.);
        let dy = (b[1] as f64 - p[1])
            .max(p[1] - (b[1] + b[3] - 1) as f64)
            .max(0.);
        let d = dx * dx + dy * dy;
        if d < distance {
            distance = d;
            best = Some(c);
        }
    }
    Ok(best)
}
fn lookup(m: &OwnedMusic, graph: &Graph, n: &Value) -> Result<Option<Key>, String> {
    let key = origin(n)?;
    let b = bounds(n)?;
    let p = [(b[0] + b[2] / 2) as f64, (b[1] + b[3] / 2) as f64];
    let system = m
        .owner
        .systems
        .iter()
        .find(|s| s.id == key.system)
        .ok_or("music_direction_actual_System_required")?;
    let around = crate::grid_staff_neighborhood::around(
        &m.owner.staffs,
        &m.owner.staff_areas,
        &m.owner.parts,
        key.system,
        p,
    )?;
    let Some(staff) = around.staffs.first() else {
        return Ok(None);
    };
    let mut selected = None;
    for stack in m
        .measure_layout
        .stacks
        .iter()
        .filter(|s| s.stack.system == key.system)
    {
        let Some(measure) = stack
            .measures
            .iter()
            .find(|g| g.part.staffs.iter().any(|s| Arc::ptr_eq(s, staff)))
        else {
            continue;
        };
        let [left, right] = measure.abscissae(staff)?;
        if p[0] > left as f64 && p[0] <= right as f64 {
            selected = Some(stack);
            break;
        }
    }
    let Some(stack) = selected else {
        return Ok(None);
    };
    let mut area = b;
    if n["kind"] == "PedalInter" {
        if n["shape"] == "PEDAL_UP_MARK" {
            let Some(s) = sided_staff(m, system, p, true)? else {
                return Ok(None);
            };
            let Some(g) = stack
                .measures
                .iter()
                .find(|g| g.part.staffs.iter().any(|h| Arc::ptr_eq(h, &s)))
            else {
                return Ok(None);
            };
            let left = g.abscissae(&s)?[0];
            area[2] += area[0] - left;
            area[0] = left;
        }
        return Ok(standard(m, graph, system, &stack.stack, p, area, true)?.map(|c| c.key));
    }
    let il = m.owner.interline as i64;
    area[0] -= il;
    area[2] += 2 * il;
    // Original default ProcessingSwitches enables both sides, in TOP/BOTTOM order.
    for above in [true, false] {
        let Some(mut c) = standard(m, graph, system, &stack.stack, p, area, above)? else {
            continue;
        };
        if c.kind == ChordKind::Rest {
            continue;
        }
        let mut dy = -overlap(area, c.bounds, 1);
        let snap = graph.snapshot();
        if let Some(rel) =
            snap["relations"].as_array().unwrap().iter().find(|e| {
                e["kind"] == "MirrorRelation" && (e["source"] == c.id || e["target"] == c.id)
            })
        {
            let other = if rel["source"] == c.id {
                &rel["target"]
            } else {
                &rel["source"]
            };
            let mirror = m
                .chords
                .iter()
                .find(|h| h.id == other.as_u64().unwrap() as usize)
                .ok_or("music_direction_actual_mirror_Chord_required")?;
            let gap = -overlap(area, mirror.bounds, 1);
            if gap < dy {
                dy = gap;
                c = mirror;
            }
        }
        if dy > 7 * il {
            let g = m.measures[c.measure]
                .geometry
                .as_ref()
                .ok_or("music_direction_actual_Measure_required")?;
            let held = g
                .part
                .staffs
                .iter()
                .filter(|s| c.members.iter().any(|n| n["staff_id"] == s.id))
                .collect::<Vec<_>>();
            let s = if above { held.last() } else { held.first() }
                .ok_or("music_direction_Chord_Staff_required")?;
            let x = area[0] as f64 + area[2] as f64 / 2.;
            let top = s.lines.first().unwrap().y(x);
            let bottom = s.lines.last().unwrap().y(x);
            let gap = (top - (area[1] + area[3]) as f64)
                .max(area[1] as f64 - bottom)
                .max(0.);
            if gap > 7. * il as f64 {
                continue;
            }
        }
        return Ok(Some(c.key));
    }
    Ok(None)
}
pub(crate) fn link_and_contextualize(m: &OwnedMusic, graph: &mut Graph) -> Result<Value, String> {
    let mut next = graph.clone();
    let mut observations = Vec::new();
    for kind in ["DynamicsInter", "PedalInter"] {
        let snap = next.snapshot();
        let nodes = snap["inters"].as_array().unwrap();
        for n in nodes.iter().filter(|n| n["kind"] == kind) {
            let key = origin(n)?;
            if let Some(chord) = lookup(m, &next, n)? {
                let dynamics = kind == "DynamicsInter";
                next.relations(key.system,Phase::Symbol,vec![(chord,key,json!({"kind":if dynamics{"ChordDynamicsRelation"}else{"ChordPedalRelation"},"grade":1.,"source_ratio":1.,"target_ratio":if dynamics{3.5}else{1.},"manual":false}))])?;
                let mut live = next.retained_inter(key)?.0.clone();
                if dynamics && live["staff_id"].is_null() {
                    live["staff_id"] = next.retained_inter(chord)?.0["staff_id"].clone();
                }
                next.vertex(key, live, false)?;
            }
            if kind == "PedalInter" && next.retained_inter(key)?.0["staff_id"].is_null() {
                let mut live = next.retained_inter(key)?.0.clone();
                let b = bounds(n)?;
                let system = m.owner.systems.iter().find(|s| s.id == key.system).unwrap();
                live["staff_id"] = json!(
                    sided_staff(
                        m,
                        system,
                        [(b[0] + b[2] / 2) as f64, (b[1] + b[3] / 2) as f64],
                        true
                    )?
                    .map(|s| s.id)
                );
                next.vertex(key, live, false)?;
            }
        }
    }
    let snap = next.snapshot();
    let nodes = snap["inters"].as_array().unwrap();
    for n in nodes
        .iter()
        .filter(|n| matches!(n["kind"].as_str(), Some("DynamicsInter" | "PedalInter")))
    {
        let mut sum = 0.;
        for r in snap["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["target"] == n["id"] && e["kind"] == "ChordDynamicsRelation")
        {
            let partner = nodes
                .iter()
                .find(|c| c["id"] == r["source"])
                .ok_or("music_direction_actual_Chord_required")?;
            sum += 2.5
                * r["grade"]
                    .as_f64()
                    .ok_or("music_direction_relation_grade_required")?
                * partner["grade"]
                    .as_f64()
                    .ok_or("music_direction_cached_Chord_grade_required")?;
        }
        let g = n["grade"]
            .as_f64()
            .ok_or("music_direction_grade_required")?;
        let contextual = (1. + sum) * g / (1. + sum * g);
        let k = origin(n)?;
        let weak = contextual < 0.5 && n["frozen"] != true;
        if weak {
            next.remove_music_inter(k)?;
        } else {
            let mut live = n.clone();
            live["contextual_grade"] = json!(contextual);
            next.vertex(k, live, false)?;
        }
        observations.push(json!({"id":n["id"],"contextual_grade":contextual,"removed":weak}));
    }
    *graph = next;
    Ok(json!({"ordered_dynamics_then_pedal":true,"contextualization":observations}))
}

#[cfg(test)]
mod image_tests;
