// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original NoteHeadsBuilder aggregation and seed-head conflict gates.
//! Inputs retain their intrinsic grades and slim bounds until glyph retrieval.
use serde_json::{Value, json};

fn bounds(v: &Value) -> Result<[i32; 4], String> {
    let b: [i32; 4] =
        serde_json::from_value(v["bounds"].clone()).map_err(|_| "invalid_head_candidate_bounds")?;
    if b.iter().any(|x| x.unsigned_abs() > 4_000_000)
        || b[2] <= 0
        || b[3] <= 0
        || b[2] as i64 * b[3] as i64 > i32::MAX as i64 / 2
    {
        return Err("invalid_head_candidate_bounds".into());
    }
    Ok(b)
}
fn grade(v: &Value) -> Result<f64, String> {
    let g = v["grade"].as_f64().ok_or("missing_head_candidate_grade")?;
    if !g.is_finite() || !(0.0..=1.0).contains(&g) {
        return Err("invalid_head_candidate_grade".into());
    }
    Ok(g)
}
/// Stable decreasing-grade order, first matching representative, inclusive dx.
/// Representatives never move, and ordinate/shape do not affect aggregation.
pub fn aggregate_indices(heads: &[Value], max_dx: i32) -> Result<Vec<usize>, String> {
    if !(0..=128).contains(&max_dx) || heads.len() > 100_000 {
        return Err("invalid_head_aggregate_parameters".into());
    }
    let data = heads
        .iter()
        .map(|h| Ok((bounds(h)?, grade(h)?)))
        .collect::<Result<Vec<_>, String>>()?;
    let mut order: Vec<usize> = (0..heads.len()).collect();
    order.sort_by(|a, b| data[*b].1.total_cmp(&data[*a].1));
    let mut main: Vec<usize> = Vec::new();
    let mut work = 0usize;
    for i in order {
        let center = data[i].0[0] as f64 + data[i].0[2] as f64 / 2.;
        let mut found = false;
        for &m in &main {
            work += 1;
            if work > 1_000_000 {
                return Err("head_aggregate_work_budget".into());
            }
            let representative = data[m].0[0] as f64 + data[m].0[2] as f64 / 2.;
            if (center - representative).abs() <= max_dx as f64 {
                found = true;
                break;
            }
        }
        if !found {
            main.push(i);
        }
    }
    Ok(main)
}
/// Competitors must be the original abscissa-sorted good competitor slice.
/// Non-head objects are ignored. Touching rectangles have no positive overlap.
pub fn filter_seed_indices(heads: &[Value], competitors: &[Value]) -> Result<Vec<usize>, String> {
    if heads.len() > 100_000 || competitors.len() > 100_000 {
        return Err("head_conflict_input_budget".into());
    }
    let mut comps = Vec::new();
    for c in competitors {
        if c["kind"] != "HeadInter" {
            continue;
        }
        let b = bounds(c)?;
        if comps
            .last()
            .is_some_and(|(p, _): &([i32; 4], f64)| p[0] > b[0])
        {
            return Err("unsorted_head_seed_competitors".into());
        }
        comps.push((b, grade(c)?));
    }
    let mut kept = Vec::new();
    let mut work = 0usize;
    for (i, h) in heads.iter().enumerate() {
        let b = bounds(h)?;
        let lowered = grade(h)? * (1.0 - 0.1);
        let mut conflict = false;
        for (c, g) in &comps {
            work += 1;
            if work > 1_000_000 {
                return Err("head_conflict_work_budget".into());
            }
            let iw = (b[0] + b[2]).min(c[0] + c[2]) - b[0].max(c[0]);
            let ih = (b[1] + b[3]).min(c[1] + c[3]) - b[1].max(c[1]);
            if iw > 0 && ih > 0 {
                let intersection = iw * ih;
                let union = b[2] * b[3] + c[2] * c[3] - intersection;
                if intersection as f64 / union as f64 >= 0.1 && *g >= lowered {
                    conflict = true;
                    break;
                }
            } else if c[0] > b[0] + b[2] {
                break;
            }
        }
        if !conflict {
            kept.push(i);
        }
    }
    Ok(kept)
}
pub fn analyze(catalog: &Value, main_interline: usize) -> Result<Value, String> {
    if catalog["range_catalog_candidates_complete"] != true || !(1..=128).contains(&main_interline)
    {
        return Err("incomplete_head_candidate_dependencies".into());
    }
    let declared = crate::head_staff_catalog::TemplateCatalog::from_report(catalog)?;
    let max_dx = (0.375 * main_interline as f64).round_ties_even() as i32;
    let mut systems = catalog["systems"].clone();
    let mut count = 0usize;
    for s in systems
        .as_array_mut()
        .ok_or("invalid_head_candidate_systems")?
    {
        for staff in s["staffs"]
            .as_array_mut()
            .ok_or("invalid_head_candidate_staffs")?
        {
            for key in ["scanners", "ledger_scanners"] {
                for scan in staff[key]
                    .as_array_mut()
                    .ok_or("invalid_head_candidate_scanners")?
                {
                    let heads = scan["matches"]
                        .as_array()
                        .ok_or("invalid_head_candidate_matches")?;
                    let indices = aggregate_indices(heads, max_dx)?;
                    let matches: Vec<Value> = indices.iter().map(|&i| heads[i].clone()).collect();
                    count += matches.len();
                    scan["candidate_filter"] = json!({"aggregate_indices":indices,"max_template_dx":max_dx,"aggregated":matches});
                }
            }
        }
    }
    let mut report = json!({"range_aggregation_complete":true,"seed_conflict_kernel_complete":true,"live_seed_conflicts_complete":false,
        "heads_complete":false,"aggregated_count":count,"systems":systems,
        "scope":"Original per-scanner fixed-center stable aggregation; seed conflict kernel certified separately, live seed glyph bounds awaited",
        "pending":["seed glyph retrieval and actual competitor slices","live seed conflicts","range glyph retrieval and HeadInter/SIG registration","duplicates, overlaps, seed tally and epilog"]});
    if let Some(declared) = declared {
        report["template_catalog"] = json!(declared);
        report["template_catalog_required"] = json!(true);
    }
    Ok(report)
}
