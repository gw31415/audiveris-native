// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original HeadInter.retrieveGlyph: template foreground keypoints in the full
//! binary image, one vertical run-table glyph, and glyph bounds replacing slim bounds.
use crate::{
    GrayImage,
    beam_dispatch::{self, Registry},
    beam_structure,
    curves::Curve,
    head_candidate_filter, head_seed_search, head_seed_veto,
};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use audiveris_template_bank::Bank;
use audiveris_template_match::DistanceTable;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Original LineUtil.intersection evaluation order for a horizontal or vertical line.
fn intersection(line: [f64; 4], other: [f64; 4]) -> Result<[f64; 2], String> {
    if line
        .iter()
        .chain(other.iter())
        .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
    {
        return Err("invalid_head_seed_intersection".into());
    }
    let [x1, y1, x2, y2] = line;
    let [x3, y3, x4, y4] = other;
    let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if den == 0. {
        return Err("parallel_head_seed_intersection".into());
    }
    let v12 = x1 * y2 - y1 * x2;
    let v34 = x3 * y4 - y3 * x4;
    Ok([
        (v12 * (x3 - x4) - (x1 - x2) * v34) / den,
        (v12 * (y3 - y4) - (y1 - y2) * v34) / den,
    ])
}
pub fn seed_abscissa(line: [f64; 4], ordinate: f64) -> Result<i32, String> {
    Ok(intersection(line, [0., ordinate, 1000., ordinate])?[0].round_ties_even() as i32)
}
fn ledger_y(line: [f64; 4], x: f64) -> Result<f64, String> {
    Ok(intersection(line, [x, 0., x, 1000.])?[1])
}
pub fn retrieve(
    binary: &GrayImage,
    bank: &Bank,
    candidate: &Value,
) -> Result<Option<GlyphPixels>, String> {
    retrieve_at_size(binary, bank, 51, candidate)
}
/// Physical HeadInter glyph extraction with the same exact catalog as matching.
/// The historic wrapper and certified page path continue to select 51pt.
pub fn retrieve_at_size(
    binary: &GrayImage,
    bank: &Bank,
    point_size: u16,
    candidate: &Value,
) -> Result<Option<GlyphPixels>, String> {
    let shape = candidate["shape"]
        .as_str()
        .ok_or("invalid_head_glyph_shape")?;
    let asset = bank.get(shape, point_size).map_err(|e| e.to_string())?;
    let bounds: [i32; 4] = serde_json::from_value(candidate["bounds"].clone())
        .map_err(|_| "invalid_head_glyph_bounds")?;
    if bounds.iter().any(|v| v.unsigned_abs() > 4_000_000) || bounds[2] <= 0 || bounds[3] <= 0 {
        return Err("invalid_head_glyph_bounds".into());
    }
    let slim = asset.slim_bounds();
    let origin = [bounds[0] - slim[0] as i32, bounds[1] - slim[1] as i32];
    let mut pixels = BTreeSet::new();
    for point in asset.template().points() {
        if point.distance != 0.0 {
            continue;
        }
        let x = origin[0] + point.x;
        let y = origin[1] + point.y;
        if x >= 0
            && y >= 0
            && (x as usize) < binary.width
            && (y as usize) < binary.height
            && binary.pixels[y as usize * binary.width + x as usize] == 0
        {
            pixels.insert((point.x, point.y));
        }
    }
    if pixels.is_empty() {
        return Ok(None);
    }
    let left = pixels.iter().map(|p| p.0).min().unwrap();
    let top = pixels.iter().map(|p| p.1).min().unwrap();
    let right = pixels.iter().map(|p| p.0).max().unwrap();
    let bottom = pixels.iter().map(|p| p.1).max().unwrap();
    let width = (right - left + 1) as usize;
    let height = (bottom - top + 1) as usize;
    let mut runs = Vec::new();
    for x in left..=right {
        let mut y = top;
        while y <= bottom {
            if !pixels.contains(&(x, y)) {
                y += 1;
                continue;
            }
            let start = y;
            while y <= bottom && pixels.contains(&(x, y)) {
                y += 1;
            }
            runs.push(Run {
                position: (x - left) as usize,
                start: (start - top) as usize,
                length: (y - start) as usize,
            });
        }
    }
    Ok(Some(GlyphPixels {
        left: origin[0] + left,
        top: origin[1] + top,
        width,
        height,
        orientation: Orientation::Vertical,
        runs,
    }))
}
fn head(
    binary: &GrayImage,
    catalog: (&Bank, u16),
    registry: &mut Registry,
    candidate: &Value,
    staff: u64,
    pitch: i32,
    seed_x: Option<i32>,
) -> Result<Option<Value>, String> {
    let Some(glyph) = retrieve_at_size(binary, catalog.0, catalog.1, candidate)? else {
        return Ok(None);
    };
    let id = registry.register(&glyph)?;
    let mut row = candidate.clone();
    row["slim_bounds"] = row["bounds"].clone();
    row["glyph"] = beam_dispatch::glyph_json(&glyph);
    row["bounds"] = row["glyph"]["bounds"].clone();
    row["glyph_id"] = json!(id);
    row["glyph_id_namespace"] = json!("native sheet glyph registry (BEAMS/HEADS)");
    row["kind"] = json!("HeadInter");
    row["staff_id"] = json!(staff);
    row["pitch"] = json!(pitch as f64);
    row["left_dx"] = Value::Null;
    row["right_dx"] = Value::Null;
    if let Some(x) = seed_x
        && row["grade"].as_f64().ok_or("invalid_head_glyph_grade")? >= 0.4
    {
        if row["anchor"] == "LEFT_STEM" {
            row["left_dx"] = json!(glyph.left as f64 - x as f64 + 0.5);
        } else if row["anchor"] == "RIGHT_STEM" {
            row["right_dx"] = json!(x as f64 + 0.5 - (glyph.left as f64 + glyph.width as f64 - 1.));
        }
    }
    Ok(Some(row))
}
pub struct Input<'a> {
    pub binary: &'a GrayImage,
    pub grid: &'a Value,
    pub seed_veto: &'a Value,
    pub catalog: &'a Value,
    pub main_interline: usize,
    pub table: &'a DistanceTable,
    pub shared_sig: &'a Value,
    pub max_stem: usize,
}
pub fn analyze(input: Input<'_>, registry: &mut Registry) -> Result<Value, String> {
    analyze_impl(input, registry, false)
}
/// Original staff lifecycle: purge a staff before the next staff's competitor slice.
pub fn analyze_for_registration(
    input: Input<'_>,
    registry: &mut Registry,
) -> Result<Value, String> {
    analyze_impl(input, registry, true)
}
fn analyze_impl(input: Input<'_>, registry: &mut Registry, retire: bool) -> Result<Value, String> {
    let Input {
        binary,
        grid,
        seed_veto,
        catalog,
        main_interline,
        table,
        shared_sig,
        max_stem,
    } = input;
    if seed_veto["seed_veto_complete"] != true
        || catalog["range_catalog_candidates_complete"] != true
        || !(1..=128).contains(&main_interline)
    {
        return Err("incomplete_head_glyph_dependencies".into());
    }
    let declared = crate::head_staff_catalog::TemplateCatalog::from_report(seed_veto)?;
    if declared != crate::head_staff_catalog::TemplateCatalog::from_report(catalog)? {
        return Err("inconsistent_head_template_catalog".into());
    }
    let bank = if let Some(catalog) = &declared {
        catalog.bank()?
    } else {
        Bank::load_builtin().map_err(|e| e.to_string())?
    };
    let mut trial = registry.clone();
    let mut systems = Vec::new();
    let (mut seed_count, mut range_count, mut conflicts) = (0, 0, 0);
    let max_dx = (0.375 * main_interline as f64).round_ties_even() as i32;
    for system in catalog["systems"]
        .as_array()
        .ok_or("invalid_head_glyph_systems")?
    {
        let sid = system["system_id"]
            .as_u64()
            .ok_or("invalid_head_glyph_system")?;
        let vs = seed_veto["systems"]
            .as_array()
            .ok_or("invalid_head_glyph_seeds")?
            .iter()
            .find(|s| s["system_id"] == sid)
            .ok_or("missing_head_glyph_system")?;
        let inters = shared_sig["inters"]
            .as_array()
            .ok_or("invalid_head_glyph_sig")?
            .iter()
            .filter(|v| v["origin"]["system"] == sid)
            .cloned()
            .collect::<Vec<_>>();
        let context = head_seed_veto::Context::from_inters(&inters, main_interline, max_stem)?;
        let mut system_seeds = Vec::<Value>::new();
        let mut staffs = Vec::new();
        for f in system["staffs"]
            .as_array()
            .ok_or("invalid_head_glyph_staffs")?
        {
            let staff = f["staff_id"].as_u64().ok_or("invalid_head_glyph_staff")?;
            let vf = vs["staffs"]
                .as_array()
                .ok_or("invalid_head_glyph_seed_staffs")?
                .iter()
                .find(|f| f["staff_id"] == staff)
                .ok_or("missing_head_glyph_seed_staff")?;
            let gs = grid["staffs"]
                .as_array()
                .ok_or("invalid_head_glyph_grid_staffs")?
                .iter()
                .find(|f| f["id"] == staff)
                .ok_or("missing_head_glyph_grid_staff")?;
            let interline = crate::ledger_registration::staffs_from_grid(grid, sid)?
                .into_iter()
                .find(|f| f.id == staff)
                .ok_or("missing_head_glyph_staff_geometry")?
                .specific_interline;
            let point_size = if let Some(catalog) = &declared {
                let size = catalog.staff_point_size(interline, main_interline)?;
                if vf["point_size"].as_u64() != Some(u64::from(size))
                    || f["point_size"] != vf["point_size"]
                {
                    return Err("inconsistent_head_seed_catalog".into());
                }
                size
            } else {
                51
            };
            let lines = gs["lines"]
                .as_array()
                .ok_or("invalid_head_glyph_lines")?
                .iter()
                .map(|l| {
                    Curve::interpolate(
                        serde_json::from_value(l["points"].clone())
                            .map_err(|_| "invalid_head_glyph_curve")?,
                    )
                    .ok_or_else(|| "invalid_head_glyph_curve".to_string())
                })
                .collect::<Result<Vec<_>, String>>()?;
            let mut seeds = Vec::new();
            for key in ["scanners", "ledger_scanners"] {
                for c in vf[key]
                    .as_array()
                    .ok_or("invalid_head_glyph_seed_scanners")?
                {
                    let pitch = c["pitch"].as_i64().ok_or("invalid_head_glyph_pitch")? as i32;
                    let dir = c["dir"].as_i64().ok_or("invalid_head_glyph_dir")? as i32;
                    let index = c["line_index"].as_u64().unwrap_or(0) as usize;
                    let ledger: Option<[f64; 4]> = if key == "ledger_scanners" {
                        Some(
                            serde_json::from_value(c["main_line"].clone())
                                .map_err(|_| "invalid_head_glyph_ledger")?,
                        )
                    } else {
                        None
                    };
                    let line = if let Some(l) = ledger {
                        Curve::interpolate(vec![[l[0], l[1]], [l[2], l[3]]])
                            .ok_or("invalid_head_glyph_ledger")?
                    } else {
                        lines.get(index).ok_or("missing_head_glyph_line")?.clone()
                    };
                    let further: Vec<[f64; 4]> = if ledger.is_some() {
                        serde_json::from_value(c["further_lines"].clone())
                            .map_err(|_| "invalid_head_glyph_further_ledgers")?
                    } else {
                        Vec::new()
                    };
                    let slice = context.slice(&line, interline, dir)?;
                    let xo: Vec<i32> = serde_json::from_value(c["x_offsets"].clone())
                        .map_err(|_| "invalid_head_glyph_offsets")?;
                    let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone())
                        .map_err(|_| "invalid_head_glyph_offsets")?;
                    for seed in c["seeds"]
                        .as_array()
                        .ok_or("invalid_head_glyph_seed_list")?
                    {
                        let glyph = beam_dispatch::glyph_from_json(&seed["seed"])?;
                        let rough = (glyph.left as f64 + glyph.width as f64 / 2.).round_ties_even();
                        let yl = if let Some(l) = ledger {
                            ledger_y(l, rough)?
                        } else {
                            line.y(rough)
                        };
                        let fit = beam_structure::glyph_stem_line(&glyph)?
                            .ok_or("head_seed_without_line")?;
                        // Scanner calls the int overload of LineAdapter.yAt before xAtY.
                        let x0 = seed_abscissa(fit, yl.round_ties_even())?;
                        let y = if let Some(l) = ledger {
                            let y = ledger_y(l, x0 as f64)?;
                            if pitch % 2 == 0 {
                                y
                            } else if let Some(l) = further
                                .iter()
                                .find(|l| x0 as f64 >= l[0].ceil() && x0 as f64 <= l[2].floor())
                            {
                                (y + ledger_y(*l, x0 as f64)?) / 2.
                            } else {
                                y + pitch.signum() as f64 * interline as f64 / 2.
                            }
                        } else if dir == -1 && index > 0 {
                            (line.y(x0 as f64) + lines[index - 1].y(x0 as f64)) / 2.
                        } else {
                            line.y(x0 as f64) + dir as f64 * interline as f64 / 2.
                        };
                        let y0 = y.round_ties_even() as i32;
                        let mut matches = Vec::new();
                        for anchor in ["LEFT_STEM", "RIGHT_STEM"] {
                            matches.extend(head_seed_search::search_selected_with_veto(
                                table,
                                (&bank, point_size),
                                [x0, y0],
                                (&xo, &yo),
                                anchor,
                                &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"],
                                &mut |b| slice.veto(b),
                            )?);
                        }
                        matches.sort_by_key(|m| {
                            (
                                if m["anchor"] == "LEFT_STEM" { 0 } else { 1 },
                                if m["source_shape"] == "NOTEHEAD_VOID" {
                                    0
                                } else {
                                    1
                                },
                            )
                        });
                        for m in matches {
                            if let Some(row) = head(
                                binary,
                                (&bank, point_size),
                                &mut trial,
                                &m,
                                staff,
                                pitch,
                                Some(x0),
                            )? {
                                seeds.push(row);
                            }
                        }
                    }
                }
            }
            seed_count += seeds.len();
            system_seeds.extend(seeds.iter().enumerate().map(|(index, h)| {
                let mut h = h.clone();
                if retire {
                    h["native_staff_seed_index"] = json!(index);
                }
                h
            }));
            system_seeds.sort_by_key(|h| h["bounds"][1].as_i64().unwrap());
            let mut row =
                json!({"staff_id":staff,"seed_heads":seeds,"scanners":[],"ledger_scanners":[]});
            for key in ["scanners", "ledger_scanners"] {
                for (index, c) in f[key]
                    .as_array()
                    .ok_or("invalid_head_glyph_range_scanners")?
                    .iter()
                    .enumerate()
                {
                    let vc = &vf[key][index];
                    let dir = c["dir"].as_i64().ok_or("invalid_head_glyph_dir")? as i32;
                    let pitch = c["pitch"].as_i64().ok_or("invalid_head_glyph_pitch")? as i32;
                    let line = if key == "ledger_scanners" {
                        let l: [f64; 4] = serde_json::from_value(vc["main_line"].clone())
                            .map_err(|_| "invalid_head_glyph_ledger")?;
                        Curve::interpolate(vec![[l[0], l[1]], [l[2], l[3]]])
                            .ok_or("invalid_head_glyph_ledger")?
                    } else {
                        lines.get(c["line_index"].as_u64().ok_or("invalid_head_glyph_line")? as usize).ok_or("missing_head_glyph_line")?.clone()
                    };
                    let competitors = live_seed_slice(&system_seeds, &line, interline, dir)?;
                    let matches = c["matches"]
                        .as_array()
                        .ok_or("invalid_head_glyph_range_matches")?;
                    let a = head_candidate_filter::aggregate_indices(matches, max_dx)?;
                    let aggregated = a.iter().map(|&i| matches[i].clone()).collect::<Vec<_>>();
                    let f = head_candidate_filter::filter_seed_indices(&aggregated, &competitors)?;
                    conflicts += a.len() - f.len();
                    let indices = f.iter().map(|&i| a[i]).collect::<Vec<_>>();
                    let mut glyphs = Vec::new();
                    for &i in &indices {
                        let result = head(
                            binary,
                            (&bank, point_size),
                            &mut trial,
                            &matches[i],
                            staff,
                            pitch,
                            None,
                        )?;
                        if result.is_some() {
                            range_count += 1;
                        }
                        glyphs.push(json!({"candidate_index":i,"result":result}));
                    }
                    let mut out = json!({"seed_competitors":competitors,"filtered_indices":indices,"range_glyphs":glyphs});
                    for k in ["line_index", "ledger_index", "ledger_order", "dir", "pitch"] {
                        if !c[k].is_null() {
                            out[k] = c[k].clone();
                        }
                    }
                    row[key].as_array_mut().unwrap().push(out);
                }
            }
            if retire {
                let retired = retire_staff_seeds(&row)?;
                for seed in &mut system_seeds {
                    if seed["staff_id"] == staff {
                        let index = seed["native_staff_seed_index"]
                            .as_u64()
                            .ok_or("invalid_staff_seed_origin")?
                            as usize;
                        let after = retired.get(index).ok_or("invalid_staff_seed_origin")?;
                        for k in ["removed", "left_dx", "right_dx"] {
                            seed[k] = after[k].clone();
                        }
                    }
                }
            }
            staffs.push(row);
        }
        systems.push(json!({"system_id":sid,"staffs":staffs}));
    }
    let snapshot = trial.snapshot();
    *registry = trial;
    let mut report = json!({"head_glyph_retrieval_complete":true,"live_seed_conflicts_complete":true,"heads_complete":false,"seed_heads":seed_count,"range_heads":range_count,"seed_conflicts":conflicts,"systems":systems,"registry":snapshot,"scope":"Original seed enum order, exact full-binary template foreground glyphs, real glyph bounds, stable aggregation and live seed conflicts; no SIG insertion","pending":["HeadInter/SIG registration","duplicates, overlaps, staff notes, small-beam purge and head seed tally"]});
    if retire {
        report["staff_dependency_complete"] = json!(true);
        report["scope"] = json!(
            "Original physical seed/range search with previous staff duplicate/overlap purge and tally retirement before next staff competitor slicing; glyph IDs share the existing canonical registry."
        );
        report["pending"] = json!([
            "SIG continuation and system small-beam/epilog processing consume this staff-certified physical report"
        ]);
    }
    if let Some(declared) = declared {
        report["template_catalog"] = json!(declared);
        report["template_catalog_required"] = json!(true);
        report["scope"] = json!(
            "Explicit exact oval family/staff point catalog through physical seed/range glyphs and live previous-staff retirement. Controlled HEADS propagation; actual other-font/special-staff upstream coverage remains uncertified."
        );
    }
    Ok(report)
}
/// Inters.intersectedInters skips removed references before checking good seed slices.
pub fn live_seed_slice(
    seeds: &[Value],
    line: &Curve,
    interline: usize,
    dir: i32,
) -> Result<Vec<Value>, String> {
    if interline == 0 || interline > 1000 || !(-1..=1).contains(&dir) || seeds.len() > 100_000 {
        return Err("invalid_live_staff_seed_slice".into());
    }
    let mid = interline as f64 * dir as f64 / 2.;
    let mut out = Vec::new();
    for seed in seeds {
        if seed["removed"] == true {
            continue;
        }
        if seed["kind"] != "HeadInter" {
            return Err("invalid_live_staff_seed_kind".into());
        }
        let grade = seed["grade"]
            .as_f64()
            .filter(|g| (0.0..=1.0).contains(g))
            .ok_or("invalid_live_staff_seed_grade")?;
        if grade < 0.4 {
            continue;
        }
        let bounds: [i32; 4] = serde_json::from_value(seed["bounds"].clone())
            .map_err(|_| "invalid_live_staff_seed_bounds")?;
        if bounds[2] <= 0 || bounds[3] <= 0 || bounds.iter().any(|x| x.unsigned_abs() > 4_000_000) {
            return Err("invalid_live_staff_seed_bounds".into());
        }
        if head_seed_search::intersects_seed(
            line,
            bounds.map(f64::from),
            mid - interline as f64 / 4.,
            mid + interline as f64 / 4.,
        )? {
            let mut row = seed.clone();
            row.as_object_mut()
                .unwrap()
                .remove("native_staff_seed_index");
            out.push(row);
        }
    }
    out.sort_by_key(|h| h["bounds"][0].as_i64().unwrap());
    Ok(out)
}
/// Original prior-staff purge/tally state used by later staff seed competitor references.
pub fn retire_staff_seeds(staff: &Value) -> Result<Vec<Value>, String> {
    let mut heads = staff["seed_heads"]
        .as_array()
        .ok_or("invalid_retired_staff_seed_heads")?
        .clone();
    let n = heads.len();
    for key in ["scanners", "ledger_scanners"] {
        for sc in staff[key]
            .as_array()
            .ok_or("invalid_retired_staff_scanners")?
        {
            for row in sc["range_glyphs"]
                .as_array()
                .ok_or("invalid_retired_staff_glyphs")?
            {
                if !row["result"].is_null() {
                    heads.push(row["result"].clone());
                }
            }
        }
    }
    for (i, h) in heads.iter_mut().enumerate() {
        h["id"] = json!(i + 1);
        h["removed"] = json!(false);
    }
    crate::head_registration::purge(&mut heads, true)?;
    crate::head_registration::purge(&mut heads, false)?;
    crate::head_registration::purge_removed_tally(&mut heads);
    heads.truncate(n);
    Ok(heads)
}
