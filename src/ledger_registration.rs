// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fresh five-line LedgersBuilder acceptance/overlap reduction and registration.
use crate::{
    GrayImage,
    beam_dispatch::Registry,
    curves::Curve,
    ledger_checks::{LedgerStick, Parameters},
    ledger_reference::PreviousLedger,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Staff {
    pub id: u64,
    pub specific_interline: usize,
    pub lines: Vec<Vec<[f64; 2]>>,
    pub merged_first: bool,
    pub merged_last: bool,
    pub tablature: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ledger {
    pub id: usize,
    pub candidate_index: usize,
    pub glyph_id: usize,
    pub index: i32,
    pub staff_id: Option<u64>,
    pub bounds: [i64; 4],
    pub grade: f64,
    pub removed: bool,
    pub glyph: Value,
}
fn line_bounds(points: &[[f64; 2]]) -> [i64; 4] {
    let mut l = points[0][0].round_ties_even() as i64;
    let mut t = points[0][1].round_ties_even() as i64;
    let (mut r, mut b) = (l, t);
    for p in &points[1..] {
        l = (l as f64).min(p[0]).floor() as i64;
        t = (t as f64).min(p[1]).floor() as i64;
        r = (r as f64).max(p[0]).ceil() as i64;
        b = (b as f64).max(p[1]).ceil() as i64;
    }
    [l, t, r - l, (b - t).max(1)]
}
/// Fresh ledger vertices have no supports or ensembles. Exclusion endpoint order
/// follows registration IDs; on equal grades the higher ID is deleted.
pub fn reduce(ledgers: &mut [Ledger], indices: &mut Vec<usize>) -> Vec<usize> {
    indices.sort_by_key(|&i| ledgers[i].bounds[0]);
    let mut edges = Vec::new();
    for (pos, &a) in indices.iter().enumerate() {
        for &b in &indices[pos + 1..] {
            let x = ledgers[a].bounds;
            let y = ledgers[b].bounds;
            if x[0] + x[2] <= y[0] || y[0] + y[2] <= x[0] {
                break;
            }
            edges.push(if ledgers[a].id < ledgers[b].id {
                (a, b)
            } else {
                (b, a)
            });
        }
    }
    let mut deleted = Vec::new();
    loop {
        let mut best = None;
        let mut grade = 0.;
        for &(a, b) in &edges {
            if ledgers[a].removed || ledgers[b].removed {
                continue;
            }
            let g = ledgers[a].grade.max(ledgers[b].grade);
            if g > grade {
                grade = g;
                best = Some((a, b));
            }
        }
        let Some((a, b)) = best else {
            break;
        };
        let loser = if ledgers[a].grade < ledgers[b].grade {
            a
        } else {
            b
        };
        ledgers[loser].removed = true;
        deleted.push(loser);
    }
    indices.retain(|&i| !ledgers[i].removed);
    deleted
}
pub fn build(
    image: &GrayImage,
    candidates: &[Value],
    staffs: &[Staff],
    main_interline: usize,
    fore: usize,
    registry: &mut Registry,
) -> Result<Value, String> {
    build_resume(
        image,
        candidates,
        staffs,
        main_interline,
        fore,
        registry,
        Resume::default(),
    )
}
#[derive(Default)]
pub struct Resume {
    pub nodes: Vec<Ledger>,
    pub maps: Vec<Value>,
    pub blocked: std::collections::BTreeSet<usize>,
}
pub fn build_resume(
    image: &GrayImage,
    candidates: &[Value],
    staffs: &[Staff],
    main_interline: usize,
    fore: usize,
    registry: &mut Registry,
    resume: Resume,
) -> Result<Value, String> {
    build_resume_observed(
        image,
        candidates,
        staffs,
        main_interline,
        fore,
        registry,
        (resume, &mut |_| {}),
    )
}
#[derive(Clone, Debug)]
pub(crate) struct StaffMembership {
    pub(crate) staff_id: u64,
    pub(crate) lines: Vec<(i32, Vec<usize>)>,
}
pub(crate) fn build_resume_observed(
    image: &GrayImage,
    candidates: &[Value],
    staffs: &[Staff],
    main_interline: usize,
    fore: usize,
    registry: &mut Registry,
    retained: (Resume, &mut impl FnMut(StaffMembership)),
) -> Result<Value, String> {
    let (resume, observe) = retained;
    if candidates.len() > 32768 || staffs.len() > 128 {
        return Err("ledger_registration_budget".into());
    }
    let mut sticks = Vec::new();
    for c in candidates {
        sticks.push(
            serde_json::from_value::<LedgerStick>(c.clone())
                .map_err(|_| "invalid_native_ledger_candidate")?,
        );
    }
    let mut nodes: Vec<Ledger> = resume.nodes;
    let mut candidate_nodes = vec![None; candidates.len()];
    for (n, node) in nodes.iter().enumerate() {
        candidate_nodes[node.candidate_index - 1] = Some(n);
    }
    let mut maps = resume.maps;
    let mut attempts = 0;
    for staff in staffs {
        if staff.id == 0 || staff.lines.len() != 5 || !(1..=128).contains(&staff.specific_interline)
        {
            return Err("unsupported_native_ledger_staff".into());
        }
        let first =
            Curve::interpolate(staff.lines[0].clone()).ok_or("invalid_ledger_staff_curve")?;
        let last =
            Curve::interpolate(staff.lines[4].clone()).ok_or("invalid_ledger_staff_curve")?;
        let mut prior = Vec::<PreviousLedger>::new();
        let mut lines = Vec::new();
        let mut actual_lines = Vec::new();
        if !staff.tablature {
            for sign in [-1_i32, 1] {
                if sign < 0 && staff.merged_last {
                    continue;
                }
                for level in 1..=1024 {
                    if sign > 0 && staff.merged_first && level > 1 {
                        break;
                    }
                    let index = sign * level;
                    let il = staff.specific_interline;
                    let points = &staff.lines[if sign < 0 { 0 } else { 4 }];
                    let mut bounds = line_bounds(points);
                    let margin = (0.35 * il as f64).round_ties_even() as i64;
                    bounds[1] += index as i64 * il as i64 - 2 * margin;
                    bounds[3] += 4 * margin;
                    let mut accepted = Vec::new();
                    for (ci, stick) in sticks.iter().enumerate() {
                        if resume.blocked.contains(&(ci + 1)) {
                            continue;
                        }
                        attempts += 1;
                        if attempts > 8_000_000 {
                            return Err("ledger_lookup_work_budget".into());
                        }
                        let x = (stick.start[0] + stick.stop[0]) / 2.;
                        let y = (stick.start[1] + stick.stop[1]) / 2.;
                        if x < bounds[0] as f64
                            || x >= (bounds[0] + bounds[2]) as f64
                            || y < bounds[1] as f64
                            || y >= (bounds[1] + bounds[3]) as f64
                        {
                            continue;
                        }
                        let Some(reference) = crate::ledger_reference::select(
                            &first,
                            &last,
                            index,
                            stick.bounds,
                            main_interline,
                            &prior,
                        )?
                        else {
                            continue;
                        };
                        let wide = reference
                            .previous_id
                            .and_then(|id| prior.iter().find(|p| p.id == id))
                            .is_some_and(|p| {
                                p.bounds[2] >= (1.5 * il as f64).round_ties_even() as i64
                            });
                        let checks = crate::ledger_checks::check(
                            image,
                            stick,
                            &Parameters {
                                main_interline,
                                specific_interline: il,
                                fore,
                                target_y: reference.y + sign as f64 * il as f64,
                                wide_previous: wide,
                            },
                        )?;
                        if !checks.accepted {
                            continue;
                        }
                        let n = if let Some(n) = candidate_nodes[ci] {
                            n
                        } else {
                            let glyph =
                                crate::beam_dispatch::glyph_from_json(&candidates[ci]["glyph"])?;
                            let glyph_id = registry.register(&glyph)?;
                            let geometry = crate::beam_structure::glyph_geometry(&glyph)?;
                            let [x1, y1, x2, y2] =
                                geometry.center_line.ok_or("missing_ledger_center_line")?;
                            let h = geometry.mean_thickness / 2.;
                            let ledger_bounds = audiveris_area::LinearArea::polygon(&[
                                [x1, y1 - h],
                                [x2, y2 - h],
                                [x2, y2 + h],
                                [x1, y1 + h],
                            ])?
                            .bounds()
                            .map(i64::from);
                            let n = nodes.len();
                            nodes.push(Ledger {
                                id: n + 1,
                                candidate_index: ci + 1,
                                glyph_id,
                                index,
                                staff_id: None,
                                bounds: ledger_bounds,
                                grade: checks.grade,
                                removed: false,
                                glyph: candidates[ci]["glyph"].clone(),
                            });
                            candidate_nodes[ci] = Some(n);
                            n
                        };
                        if !nodes[n].removed {
                            accepted.push(n);
                        }
                    }
                    if accepted.is_empty() {
                        break;
                    }
                    reduce(&mut nodes, &mut accepted);
                    if accepted.is_empty() {
                        break;
                    }
                    for &n in &accepted {
                        nodes[n].staff_id = Some(staff.id);
                        let glyph = crate::beam_dispatch::glyph_from_json(&nodes[n].glyph)?;
                        let center_line = crate::beam_structure::glyph_geometry(&glyph)?
                            .center_line
                            .ok_or("missing_ledger_center_line")?;
                        let line = crate::beam_structure::glyph_stem_line(&glyph)?
                            .ok_or("missing_ledger_glyph_line")?;
                        prior.push(PreviousLedger {
                            id: nodes[n].id as u64,
                            index,
                            bounds: nodes[n].bounds,
                            center_line,
                            start: [line[0], line[1]],
                            stop: [line[2], line[3]],
                        });
                    }
                    let members = accepted.iter().map(|&n| nodes[n].id).collect::<Vec<_>>();
                    actual_lines.push((index, members.clone()));
                    lines.push(json!({"index":index,"ledger_ids":members}));
                }
            }
        }
        observe(StaffMembership {
            staff_id: staff.id,
            lines: actual_lines,
        });
        let value = json!({"staff_id":staff.id,"lines":lines});
        if let Some(row) = maps
            .iter_mut()
            .find(|m| m["staff_id"].as_u64() == Some(staff.id))
        {
            *row = value;
        } else {
            maps.push(value);
        }
    }
    Ok(
        json!({"builder_complete":true,"ledger_registration_complete":true,"post_analysis_complete":false,"ledgers_complete":false,"inters":nodes,"staff_maps":maps,"lookup_attempts":attempts}),
    )
}
pub fn staffs_from_grid(grid: &Value, system: u64) -> Result<Vec<Staff>, String> {
    let mut output = Vec::new();
    for s in grid["staffs"]
        .as_array()
        .ok_or("missing_ledger_grid_staffs")?
        .iter()
        .filter(|s| s["system_id"].as_u64() == Some(system))
    {
        let lines = s["lines"]
            .as_array()
            .ok_or("missing_ledger_grid_lines")?
            .iter()
            .map(|l| {
                serde_json::from_value(l["points"].clone())
                    .map_err(|_| "invalid_ledger_grid_points".into())
            })
            .collect::<Result<Vec<_>, String>>()?;
        output.push(Staff {
            id: s["id"].as_u64().ok_or("invalid_ledger_grid_staff")?,
            specific_interline: s["specific_interline"]
                .as_u64()
                .ok_or("invalid_ledger_specific_interline")?
                as usize,
            lines,
            merged_first: false,
            merged_last: false,
            tablature: s["tablature"].as_bool().unwrap_or(false),
        });
    }
    Ok(output)
}
