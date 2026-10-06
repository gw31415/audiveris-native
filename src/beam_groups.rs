// SPDX-License-Identifier: AGPL-3.0-or-later
// Pinned BeamGroupInter.populateSystem/groupBeams before rhythm processing.
use crate::{beam_candidates::SystemCandidates, beam_structure::BeamItem};
#[derive(Clone, Debug, PartialEq)]
pub struct GroupBeam {
    pub id: usize,
    pub bounds: [i32; 4],
    pub item: BeamItem,
}
#[derive(Clone, Debug, PartialEq)]
pub struct BeamGroup {
    pub id: usize,
    pub members: Vec<usize>,
    pub bounds: [i32; 4],
}
#[derive(Clone, Debug, PartialEq)]
pub struct GroupPass {
    pub ordered_ids: Vec<usize>,
    pub groups: Vec<BeamGroup>,
    pub supports: Vec<(usize, usize)>,
}
pub fn populate_system(state: &SystemCandidates, interline: usize) -> Result<GroupPass, String> {
    let beams = state
        .inters()
        .iter()
        .filter(|b| !b.is_removed())
        .map(|b| GroupBeam {
            id: b.id(),
            bounds: b.bounds,
            item: b.item.clone(),
        })
        .collect::<Vec<_>>();
    populate(&beams, state.exclusions(), interline)
}
fn y(m: [f64; 4], x: f64) -> f64 {
    m[1] + (m[3] - m[1]) * (x - m[0]) / (m[2] - m[0])
}
fn neighbors(a: &GroupBeam, b: &GroupBeam, min_x: f64, max_y: f64) -> bool {
    let m = a.item.median;
    let n = b.item.median;
    let l = m[0].max(n[0]);
    let r = m[2].min(n[2]);
    if r - l < min_x {
        return false;
    }
    let x = (l + r) / 2.;
    (y(m, x) - y(n, x)).abs() <= max_y
        && ((m[3] - m[1]) / (m[2] - m[0]) - (n[3] - n[1]) / (n[2] - n[0])).abs() <= 0.065
}
fn intersects(a: [i64; 4], b: [i32; 4]) -> bool {
    a[2] > 0
        && a[3] > 0
        && b[2] > 0
        && b[3] > 0
        && a[0] < i64::from(b[0]) + i64::from(b[2])
        && i64::from(b[0]) < a[0] + a[2]
        && a[1] < i64::from(b[1]) + i64::from(b[3])
        && i64::from(b[1]) < a[1] + a[3]
}
/// Fresh pre-rhythm system only. Stable bound-ordinate order, first-ensemble
/// reuse, shared-membership merge, and insertion-ordered pairwise support.
pub fn populate(
    beams: &[GroupBeam],
    exclusions: &[(usize, usize)],
    interline: usize,
) -> Result<GroupPass, String> {
    if interline == 0
        || interline > 10000
        || exclusions.len() > beams.len().saturating_mul(beams.len())
        || beams.len().checked_pow(4).is_none_or(|n| n > 1_000_000_000)
    {
        return Err("unsupported_beam_group_work".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    for b in beams {
        let m = b.item.median;
        if !ids.insert(b.id)
            || b.id == 0
            || b.bounds[2] <= 0
            || b.bounds[3] <= 0
            || !m.iter().all(|v| v.is_finite() && v.abs() <= 10_000_000.)
            || b.bounds.iter().any(|&v| i64::from(v).abs() > 10_000_000)
            || m[2] <= m[0]
            || !b.item.height.is_finite()
            || b.item.height <= 0.
            || b.item.height > 20000.
        {
            return Err("invalid_beam_group_input".into());
        }
    }
    let mut ordered = beams.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|b| b.bounds[1]);
    let min_x = (0.7 * interline as f64).round_ties_even();
    let max_y = (1.2 * interline as f64).round_ties_even();
    let mut groups: Vec<Option<Vec<usize>>> = Vec::new();
    let mut memberships: std::collections::BTreeMap<usize, Vec<usize>> =
        beams.iter().map(|b| (b.id, Vec::new())).collect();
    let mut supports = Vec::new();
    let excluded = exclusions
        .iter()
        .map(|&(a, b)| (a.min(b), a.max(b)))
        .collect::<std::collections::BTreeSet<_>>();
    let add = |g: usize,
               id: usize,
               groups: &mut Vec<Option<Vec<usize>>>,
               memberships: &mut std::collections::BTreeMap<usize, Vec<usize>>,
               supports: &mut Vec<(usize, usize)>| {
        let members = groups[g].as_mut().unwrap();
        if !members.contains(&id) {
            members.push(id);
            memberships.get_mut(&id).unwrap().push(g);
        }
        for &other in members.iter() {
            if other == id {
                continue;
            }
            let p = (id.min(other), id.max(other));
            if !excluded.contains(&p) && !supports.contains(&p) {
                supports.push(p);
            }
        }
    };
    for (i, a) in ordered.iter().enumerate() {
        let g = match memberships[&a.id].first() {
            Some(&g) => g,
            None => {
                let g = groups.len();
                groups.push(Some(Vec::new()));
                add(g, a.id, &mut groups, &mut memberships, &mut supports);
                g
            }
        };
        let grow = (max_y - a.item.height / 2.).ceil() as i64;
        let mut band = a.bounds.map(i64::from);
        band[1] -= grow;
        band[3] += 2 * grow;
        for b in ordered.iter().skip(i + 1) {
            if intersects(band, b.bounds) {
                if neighbors(a, b, min_x, max_y) {
                    add(g, b.id, &mut groups, &mut memberships, &mut supports);
                }
            } else if i64::from(b.bounds[1]) >= band[1] + band[3] {
                break;
            }
        }
    }
    for b in &ordered {
        let ensemble = memberships[&b.id].clone();
        if ensemble.len() > 1 {
            let target = ensemble[0];
            for &old in ensemble.iter().skip(1) {
                let members = groups[old].clone().unwrap();
                for id in members {
                    add(target, id, &mut groups, &mut memberships, &mut supports);
                    memberships.get_mut(&id).unwrap().retain(|&g| g != old);
                }
                groups[old] = None;
            }
        }
    }
    let result = groups
        .into_iter()
        .enumerate()
        .filter_map(|(g, v)| {
            v.map(|members| {
                let mut l = i32::MAX;
                let mut t = i32::MAX;
                let mut r = i64::MIN;
                let mut d = i64::MIN;
                for id in &members {
                    let b = beams.iter().find(|b| b.id == *id).unwrap().bounds;
                    l = l.min(b[0]);
                    t = t.min(b[1]);
                    r = r.max(i64::from(b[0]) + i64::from(b[2]));
                    d = d.max(i64::from(b[1]) + i64::from(b[3]));
                }
                BeamGroup {
                    id: g + 1,
                    members,
                    bounds: [l, t, (r - i64::from(l)) as i32, (d - i64::from(t)) as i32],
                }
            })
        })
        .collect();
    Ok(GroupPass {
        ordered_ids: ordered.iter().map(|b| b.id).collect(),
        groups: result,
        supports,
    })
}

/// Original graph deletion removes incident supports and empty ensembles;
/// surviving ensembles retain their identity and member order (no regrouping).
pub fn remove_members(
    pass: &GroupPass,
    beams: &[GroupBeam],
    removed: &[usize],
) -> Result<GroupPass, String> {
    let ids = removed
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let bounds = beams
        .iter()
        .map(|b| (b.id, b.bounds))
        .collect::<std::collections::BTreeMap<_, _>>();
    if ids.iter().any(|id| !bounds.contains_key(id)) {
        return Err("unknown_group_cleanup_beam".into());
    }
    let mut result = pass.clone();
    result.ordered_ids.retain(|id| !ids.contains(id));
    result
        .supports
        .retain(|(a, b)| !ids.contains(a) && !ids.contains(b));
    for group in &mut result.groups {
        group.members.retain(|id| !ids.contains(id));
        if group.members.is_empty() {
            continue;
        }
        group.bounds = union_bounds(group.members.iter().map(|id| {
            bounds
                .get(id)
                .copied()
                .ok_or("unknown_group_cleanup_member")
        }))?;
    }
    result.groups.retain(|g| !g.members.is_empty());
    Ok(result)
}

fn union_bounds(
    bounds: impl Iterator<Item = Result<[i32; 4], &'static str>>,
) -> Result<[i32; 4], String> {
    let mut l = i32::MAX;
    let mut t = i32::MAX;
    let mut r = i64::MIN;
    let mut d = i64::MIN;
    for b in bounds {
        let b = b?;
        if b[2] <= 0 || b[3] <= 0 {
            return Err("group_cleanup_bounds".into());
        }
        l = l.min(b[0]);
        t = t.min(b[1]);
        r = r.max(i64::from(b[0]) + i64::from(b[2]));
        d = d.max(i64::from(b[1]) + i64::from(b[3]));
    }
    Ok([
        l,
        t,
        i32::try_from(r - i64::from(l)).map_err(|_| "group_cleanup_bounds")?,
        i32::try_from(d - i64::from(t)).map_err(|_| "group_cleanup_bounds")?,
    ])
}
fn owned_members(
    g: &serde_json::Value,
    group: &serde_json::Value,
    removed: &[u64],
) -> Result<Vec<(u64, [i32; 4])>, String> {
    let nodes = g["inters"]
        .as_array()
        .ok_or("owned beam group nodes required")?;
    let edges = g["relations"]
        .as_array()
        .ok_or("owned beam group edges required")?;
    let id = group["id"].as_u64().ok_or("owned beam group ID required")?;
    let members = group["members"]
        .as_array()
        .ok_or("owned beam group members required")?;
    let mut seen = std::collections::BTreeSet::new();
    let mut kept = Vec::new();
    for m in members {
        let bid = m.as_u64().ok_or("owned beam group member ID required")?;
        if !seen.insert(bid) {
            return Err("duplicate owned beam group member".into());
        }
        if removed.contains(&bid) {
            continue;
        }
        let beam = nodes
            .iter()
            .find(|n| n["id"] == bid)
            .ok_or("owned beam group live member required")?;
        if !matches!(
            beam["kind"].as_str(),
            Some("BeamInter" | "BeamHookInter" | "SmallBeamInter")
        ) || !edges
            .iter()
            .any(|e| e["kind"] == "Containment" && e["source"] == id && e["target"] == bid)
        {
            return Err("owned beam group typed containment required".into());
        }
        let bounds = serde_json::from_value(beam["bounds"].clone())
            .map_err(|_| "owned beam member bounds required")?;
        kept.push((bid, bounds));
    }
    Ok(kept)
}
/// Validate ownership/geometry before modeled reducer callbacks start.
pub(crate) fn validate_owned_groups(g: &serde_json::Value) -> Result<(), String> {
    for group in g["inters"]
        .as_array()
        .ok_or("owned beam group nodes required")?
        .iter()
        .filter(|n| n["kind"] == "BeamGroupInter")
    {
        let kept = owned_members(g, group, &[])?;
        if !kept.is_empty() {
            union_bounds(kept.iter().map(|(_, b)| Ok(*b)))?;
        }
    }
    Ok(())
}
/// Original Containment.removed -> BeamGroupInter.invalidateCache/getBounds,
/// and AbstractInter.remove's dying-ensemble callback, on an owned SIG snapshot.
/// Existing identities/order are retained; this never regroups survivors.
pub(crate) fn remove_owned_members(
    g: &mut serde_json::Value,
    removed: &[u64],
) -> Result<Vec<u64>, String> {
    use serde_json::json;
    let nodes = g["inters"]
        .as_array()
        .ok_or("owned beam group nodes required")?;
    let mut updates = Vec::new();
    let mut dying = Vec::new();
    for group in nodes.iter().filter(|n| n["kind"] == "BeamGroupInter") {
        let members = group["members"]
            .as_array()
            .ok_or("owned beam group members required")?;
        if !members
            .iter()
            .any(|id| id.as_u64().is_some_and(|id| removed.contains(&id)))
        {
            continue;
        }
        let id = group["id"].as_u64().ok_or("owned beam group ID required")?;
        let kept = owned_members(g, group, removed)?;
        if kept.is_empty() {
            dying.push(id);
            continue;
        }
        let bounds = union_bounds(kept.iter().map(|(_, b)| Ok(*b)))?;
        let mut next = group.clone();
        next["members"] = json!(kept.iter().map(|(id, _)| *id).collect::<Vec<_>>());
        next["bounds"] = json!(bounds);
        updates.push(next);
    }
    for next in updates {
        let id = next["id"].clone();
        *g["inters"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["id"] == id)
            .unwrap() = next;
    }
    g["inters"]
        .as_array_mut()
        .unwrap()
        .retain(|n| !dying.iter().any(|id| n["id"] == *id));
    g["relations"]
        .as_array_mut()
        .ok_or("owned beam group edges required")?
        .retain(|e| {
            !dying
                .iter()
                .any(|id| e["source"] == *id || e["target"] == *id)
        });
    Ok(dying)
}
