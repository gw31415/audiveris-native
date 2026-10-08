// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original sheet-wide ledger statistics/filter/rebuild and translated paths.
use crate::{
    GrayImage,
    beam_dispatch::Registry,
    curves::Curve,
    ledger_registration::{Ledger, Resume, Staff},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Info {
    pub system_id: u64,
    pub ledger_id: usize,
    pub staff_id: u64,
    pub key: i32,
    pub interline: usize,
    pub delta: f64,
    pub height: f64,
    pub width: f64,
}
#[derive(Default)]
struct Population {
    n: usize,
    s: f64,
    s2: f64,
}
impl Population {
    fn add(&mut self, x: f64) {
        self.n += 1;
        self.s += x;
        self.s2 += x * x;
    }
    fn range(&self) -> Option<[f64; 2]> {
        if self.n == 0 {
            return None;
        }
        let n = self.n as f64;
        let mean = self.s / n;
        let variance = if self.n == 1 {
            0.
        } else {
            n * ((self.s2 - self.s * self.s / n) / n).max(0.) / (n - 1.)
        };
        let sigma = variance.sqrt();
        Some([mean - sigma, mean + sigma])
    }
}
pub fn filter(infos: &[Info]) -> Result<Value, String> {
    let mut above = Population::default();
    let mut below = Population::default();
    let mut heights = Population::default();
    let mut widths = Population::default();
    if infos.len() > 32768 {
        return Err("ledger_post_info_budget".into());
    }
    for i in infos {
        if !(1..=128).contains(&i.interline)
            || i.key == 0
            || [i.delta, i.height, i.width]
                .iter()
                .any(|x| !x.is_finite() || *x < 0.)
        {
            return Err("invalid_ledger_post_info".into());
        }
        let il = i.interline as f64;
        if i.key < 0 {
            above.add(i.delta / il)
        } else {
            below.add(i.delta / il)
        }
        heights.add(i.height / il);
        if i.width / il <= 3. {
            widths.add(i.width / il);
        }
    }
    let ar = above.range();
    let br = below.range();
    let hr = heights.range();
    let wr = widths.range();
    let mut discarded = Vec::new();
    for i in infos {
        let dr = if i.key < 0 { ar } else { br }.ok_or("missing_ledger_delta_population")?;
        let h = hr.unwrap();
        let il = i.interline as f64;
        if i.delta.ceil() < (dr[0] * il).floor()
            || i.delta.floor() > (dr[1] * il).ceil()
            || i.height.ceil() < (h[0] * il).floor()
            || i.height.floor() > (h[1] * il).ceil()
        {
            discarded.push(json!([i.system_id, i.ledger_id]));
        }
    }
    Ok(
        json!({"delta_above_range":ar,"delta_below_range":br,"height_range":hr,"width_range":wr,"discarded":discarded}),
    )
}
fn geometry(node: &Ledger) -> Result<([f64; 4], f64), String> {
    let glyph = crate::beam_dispatch::glyph_from_json(&node.glyph)?;
    let g = crate::beam_structure::glyph_geometry(&glyph)?;
    Ok((
        g.center_line.ok_or("missing_post_ledger_median")?,
        g.mean_thickness,
    ))
}
pub fn process(
    image: &GrayImage,
    systems: &[Value],
    staffs: &[(u64, Vec<Staff>)],
    candidates: &[Value],
    main_interline: usize,
    fore: usize,
    registry: &mut Registry,
) -> Result<Value, String> {
    process_observed(
        image,
        systems,
        staffs,
        candidates,
        main_interline,
        fore,
        (registry, &mut |_| {}),
    )
}
pub(crate) enum MembershipEvent {
    Retire {
        system: u64,
        ledgers: Vec<usize>,
    },
    Build {
        system: u64,
        membership: crate::ledger_registration::StaffMembership,
    },
}
pub(crate) fn process_observed(
    image: &GrayImage,
    systems: &[Value],
    staffs: &[(u64, Vec<Staff>)],
    candidates: &[Value],
    main_interline: usize,
    fore: usize,
    retained: (&mut Registry, &mut impl FnMut(MembershipEvent)),
) -> Result<Value, String> {
    let (registry, observe) = retained;
    let mut infos = Vec::new();
    let mut discarded = Vec::<(u64, usize)>::new();
    for sys in systems {
        let system = sys["system_id"]
            .as_u64()
            .ok_or("invalid_ledger_post_system")?;
        let report = &sys["registration"];
        let nodes: Vec<Ledger> = serde_json::from_value(report["inters"].clone())
            .map_err(|_| "invalid_ledger_post_nodes")?;
        let specs = &staffs
            .iter()
            .find(|s| s.0 == system)
            .ok_or("missing_ledger_post_staffs")?
            .1;
        for staff in specs {
            let map = report["staff_maps"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["staff_id"].as_u64() == Some(staff.id))
                .ok_or("missing_ledger_post_staff_map")?;
            let mut lines = map["lines"].as_array().unwrap().iter().collect::<Vec<_>>();
            lines.sort_by_key(|l| l["index"].as_i64().unwrap());
            let first = Curve::interpolate(staff.lines[0].clone()).ok_or("invalid_post_curve")?;
            let last = Curve::interpolate(staff.lines[4].clone()).ok_or("invalid_post_curve")?;
            for line in lines {
                let key = line["index"].as_i64().unwrap() as i32;
                for id in line["ledger_ids"].as_array().unwrap() {
                    let n = &nodes[id.as_u64().unwrap() as usize - 1];
                    let (median, height) = geometry(n)?;
                    let x = (median[0] + median[2]) / 2.;
                    let y = (median[1] + median[3]) / 2.;
                    let delta = if key.unsigned_abs() >= 2 {
                        let prev = key - key.signum();
                        let previous = map["lines"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|l| l["index"].as_i64() == Some(prev as i64))
                            .and_then(|l| l["ledger_ids"].as_array())
                            .and_then(|ids| {
                                ids.iter()
                                    .map(|id| &nodes[id.as_u64().unwrap() as usize - 1])
                                    .find(|n| {
                                        x >= n.bounds[0] as f64
                                            && x < (n.bounds[0] + n.bounds[2]) as f64
                                    })
                            });
                        let Some(p) = previous else {
                            discarded.push((system, n.id));
                            continue;
                        };
                        (y - crate::beam_structure::intersection_at_x(geometry(p)?.0, x)[1]).abs()
                    } else {
                        (first.y(x) - y).max(y - last.y(x))
                    };
                    infos.push(Info {
                        system_id: system,
                        ledger_id: n.id,
                        staff_id: staff.id,
                        key,
                        interline: staff.specific_interline,
                        delta,
                        height,
                        width: median[2] - median[0],
                    });
                }
            }
        }
    }
    let statistics = filter(&infos)?;
    let mut sorted = infos.clone();
    sorted.sort_by_key(|i| (i.system_id, i.ledger_id));
    for i in sorted {
        if statistics["discarded"]
            .as_array()
            .unwrap()
            .contains(&json!([i.system_id, i.ledger_id]))
        {
            discarded.push((i.system_id, i.ledger_id));
        }
    }
    let mut output = Vec::new();
    for sys in systems {
        let system = sys["system_id"].as_u64().unwrap();
        let specs = &staffs.iter().find(|s| s.0 == system).unwrap().1;
        let mut report = sys["registration"].clone();
        let mut nodes: Vec<Ledger> = serde_json::from_value(report["inters"].clone()).unwrap();
        let mut affected = Vec::new();
        let mut blocked = std::collections::BTreeSet::new();
        for &(owner, id) in &discarded {
            if owner == system {
                let n = &mut nodes[id - 1];
                let staff = n.staff_id.ok_or("discarded_ledger_without_staff")?;
                if !affected.contains(&staff) {
                    affected.push(staff);
                }
                blocked.insert(n.candidate_index);
                n.removed = true;
            }
        }
        if !affected.is_empty() {
            observe(MembershipEvent::Retire {
                system,
                ledgers: nodes.iter().filter(|n| n.removed).map(|n| n.id).collect(),
            });
            let selected = affected
                .iter()
                .map(|id| {
                    specs
                        .iter()
                        .find(|s| s.id == *id)
                        .cloned()
                        .ok_or("missing_rebuild_staff")
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut maps = report["staff_maps"].as_array().unwrap().clone();
            for m in &mut maps {
                m["lines"].as_array_mut().unwrap().iter_mut().for_each(|l| {
                    l["ledger_ids"]
                        .as_array_mut()
                        .unwrap()
                        .retain(|id| !nodes[id.as_u64().unwrap() as usize - 1].removed)
                });
            }
            let c = candidates
                .iter()
                .find(|c| c["system_id"].as_u64() == Some(system))
                .ok_or("missing_post_candidates")?;
            report = crate::ledger_registration::build_resume_observed(
                image,
                c["candidates"].as_array().unwrap(),
                &selected,
                main_interline,
                fore,
                registry,
                (
                    Resume {
                        nodes,
                        maps,
                        blocked,
                    },
                    &mut |membership| observe(MembershipEvent::Build { system, membership }),
                ),
            )?;
            let mut nodes: Vec<Ledger> = serde_json::from_value(report["inters"].clone()).unwrap();
            let active = report["staff_maps"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|m| m["lines"].as_array().unwrap())
                .flat_map(|l| l["ledger_ids"].as_array().unwrap())
                .filter_map(Value::as_u64)
                .collect::<std::collections::BTreeSet<_>>();
            for n in &mut nodes {
                if !active.contains(&(n.id as u64)) {
                    n.removed = true;
                }
            }
            report["inters"] = serde_json::to_value(nodes).unwrap();
        }
        let nodes: Vec<Ledger> = serde_json::from_value(report["inters"].clone()).unwrap();
        let mut paths = Vec::new();
        for staff in specs {
            let map = report["staff_maps"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["staff_id"].as_u64() == Some(staff.id))
                .unwrap();
            let mut ledger_paths = Vec::new();
            for sign in [-1_i32, 1] {
                let mut curve =
                    Curve::interpolate(staff.lines[if sign < 0 { 0 } else { 4 }].clone())
                        .ok_or("invalid_post_curve")?;
                for level in 1..=1024 {
                    let index = sign * level;
                    let Some(line) = map["lines"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|l| l["index"].as_i64() == Some(index as i64))
                    else {
                        break;
                    };
                    let ids = line["ledger_ids"].as_array().unwrap();
                    if ids.is_empty() {
                        break;
                    }
                    let mut delta = Population::default();
                    for id in ids {
                        let (m, _) = geometry(&nodes[id.as_u64().unwrap() as usize - 1])?;
                        let x = (m[0] + m[2]) / 2.;
                        delta.add((m[1] + m[3]) / 2. - curve.y(x));
                    }
                    curve = curve.translated_y(delta.s / delta.n as f64);
                    ledger_paths.push(json!({"index":index,"segments":curve.path_segments()}));
                }
            }
            ledger_paths.sort_by_key(|p| p["index"].as_i64().unwrap());
            paths.push(json!({"staff_id":staff.id,"ledger_paths":ledger_paths}));
        }
        report["post_analysis_complete"] = json!(true);
        report["ledgers_complete"] = json!(true);
        report["ledger_paths"] = json!(paths);
        output.push(json!({"system_id":system,"registration":report}));
    }
    Ok(
        json!({"stage":"LEDGERS.native_post_analysis","post_analysis_complete":true,"ledgers_complete":true,"infos":infos,"statistics":statistics,"discarded":discarded,"systems":output}),
    )
}
