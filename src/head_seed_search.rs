// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original HEADS distance neutralization and automatic staff seed-template
//! proposals. This precedes competitor/bar veto and HeadInter registration.
use crate::{GrayImage, beam_dispatch, beam_structure, curves::Curve, template_probes};
use audiveris_template_bank::Bank;
use audiveris_template_match::{DistanceTable, Weights};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub fn x_offsets(max_stem: usize) -> Result<Vec<i32>, String> {
    if !(1..=128).contains(&max_stem) {
        return Err("unsupported_head_stem_width".into());
    }
    let n = max_stem + usize::from(max_stem.is_multiple_of(2));
    Ok((0..n)
        .map(|i| {
            if i % 2 == 0 {
                -(i as i32 / 2)
            } else {
                (i as i32 + 1) / 2
            }
        })
        .collect())
}
pub fn y_offsets(interline: usize, dir: i32, open: bool) -> Result<Vec<i32>, String> {
    if !(1..=128).contains(&interline) || !(-1..=1).contains(&dir) || (open && dir == 0) {
        return Err("unsupported_head_pitch".into());
    }
    let n = if open {
        ((0.2 * interline as f64).round_ties_even() as usize).max(1)
    } else {
        1
    };
    Ok((0..=n)
        .map(|i| {
            if !open {
                if i % 2 == 0 {
                    i as i32 / 2
                } else {
                    -((i as i32 + 1) / 2)
                }
            } else {
                match i {
                    0 => 0,
                    1 => dir,
                    2 => -dir,
                    _ => dir * (i as i32 - 1),
                }
            }
        })
        .collect())
}
/// Best location per source shape/anchor, strict improvement preserves original
/// search order on equal distances. Hollow reassignment retains black's score.
pub fn search(
    table: &DistanceTable,
    bank: &Bank,
    pivot: [i32; 2],
    offsets: (&[i32], &[i32]),
    anchor: &str,
) -> Result<Vec<Value>, String> {
    search_with_veto(table, bank, pivot, offsets, anchor, &mut |_| Ok(false))
}
/// Original Scanner.eval veto runs on the source template's slim bounds before
/// scoring. A veto at the first pivot abandons this shape/anchor search window.
pub fn search_with_veto(
    table: &DistanceTable,
    bank: &Bank,
    pivot: [i32; 2],
    offsets: (&[i32], &[i32]),
    anchor: &str,
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Vec<Value>, String> {
    if !matches!(anchor, "LEFT_STEM" | "RIGHT_STEM") {
        return Err("invalid_head_search_window".into());
    }
    search_shapes_with_veto(
        table,
        bank,
        pivot,
        offsets,
        anchor,
        &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"],
        veto,
    )
}
/// Legacy exact seven-shape selection; every asset is preflighted before veto.
pub fn search_selected_with_veto(
    table: &DistanceTable,
    catalog: (&Bank, u16),
    pivot: [i32; 2],
    offsets: (&[i32], &[i32]),
    anchor: &str,
    shapes: &[&str],
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Vec<Value>, String> {
    search_selected_impl(
        table,
        catalog,
        pivot,
        offsets,
        anchor,
        shapes,
        &STEM_SHAPES,
        veto,
    )
}
/// Opt-in all twelve original stem shapes; no interpolation or default catalog change.
pub fn search_all_heads_with_veto(
    table: &DistanceTable,
    catalog: (&Bank, u16),
    pivot: [i32; 2],
    offsets: (&[i32], &[i32]),
    anchor: &str,
    shapes: &[&str],
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Vec<Value>, String> {
    search_selected_impl(
        table,
        catalog,
        pivot,
        offsets,
        anchor,
        shapes,
        &ALL_STEM_SHAPES,
        veto,
    )
}
#[allow(clippy::too_many_arguments)]
fn search_selected_impl(
    table: &DistanceTable,
    catalog: (&Bank, u16),
    pivot: [i32; 2],
    offsets: (&[i32], &[i32]),
    anchor: &str,
    shapes: &[&str],
    allowed: &[&str],
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Vec<Value>, String> {
    if !matches!(anchor, "LEFT_STEM" | "RIGHT_STEM") {
        return Err("invalid_head_search_window".into());
    }
    let mut selected = std::collections::BTreeSet::new();
    for &shape in shapes {
        if !allowed.contains(&shape) || !selected.insert(shape) {
            return Err("unsupported_head_seed_shapes".into());
        }
    }
    for &shape in shapes {
        catalog
            .0
            .get(shape, catalog.1)
            .map_err(|e| e.to_string())?
            .anchor(anchor)
            .map_err(|e| e.to_string())?
            .rounded()
            .map_err(|e| e.to_string())?;
    }
    if selected.contains("NOTEHEAD_BLACK") {
        catalog
            .0
            .get("NOTEHEAD_VOID", catalog.1)
            .map_err(|e| e.to_string())?
            .anchor(anchor)
            .map_err(|e| e.to_string())?
            .rounded()
            .map_err(|e| e.to_string())?;
    }
    search_shapes_at_size_with_veto(table, catalog, pivot, offsets, anchor, shapes, veto)
}
/// Historical seven-shape selection remains available unchanged.
pub const STEM_SHAPES: [&str; 7] = [
    "NOTEHEAD_VOID",
    "NOTEHEAD_BLACK",
    "NOTEHEAD_BLACK_SMALL",
    "NOTEHEAD_CROSS",
    "NOTEHEAD_DIAMOND_FILLED",
    "NOTEHEAD_TRIANGLE_DOWN_FILLED",
    "NOTEHEAD_CIRCLE_X",
];
/// Full original StemHeads enum order for opt-in all-head banks.
pub const ALL_STEM_SHAPES: [&str; 12] = [
    "NOTEHEAD_VOID",
    "NOTEHEAD_VOID_SMALL",
    "NOTEHEAD_CROSS_VOID",
    "NOTEHEAD_DIAMOND_VOID",
    "NOTEHEAD_TRIANGLE_DOWN_VOID",
    "NOTEHEAD_CIRCLE_X_VOID",
    "NOTEHEAD_BLACK",
    "NOTEHEAD_BLACK_SMALL",
    "NOTEHEAD_CROSS",
    "NOTEHEAD_DIAMOND_FILLED",
    "NOTEHEAD_TRIANGLE_DOWN_FILLED",
    "NOTEHEAD_CIRCLE_X",
];
/// Shared original location search; range browsing has a fixed MIDDLE_LEFT x
/// and restricts black source templates to abscissae covered by head spots.
pub(crate) fn search_shapes_with_veto(
    table: &DistanceTable,
    bank: &Bank,
    pivot: [i32; 2],
    offsets: (&[i32], &[i32]),
    anchor: &str,
    shapes: &[&str],
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Vec<Value>, String> {
    search_shapes_at_size_with_veto(table, (bank, 51), pivot, offsets, anchor, shapes, veto)
}
/// Exact catalog selection for additional fonts and sizes. Search geometry,
/// strict ties, central abandonment and black-to-void conversion stay shared.
pub(crate) fn search_shapes_at_size_with_veto(
    table: &DistanceTable,
    catalog: (&Bank, u16),
    pivot: [i32; 2],
    offsets: (&[i32], &[i32]),
    anchor: &str,
    shapes: &[&str],
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Vec<Value>, String> {
    let (bank, point_size) = catalog;
    let (xo, yo) = offsets;
    if xo.is_empty()
        || yo.is_empty()
        || xo.len() > 129
        || yo.len() > 129
        || xo[0] != 0
        || yo[0] != 0
        || xo.iter().chain(yo).any(|v| v.unsigned_abs() > 128)
        || pivot.iter().any(|v| v.unsigned_abs() > 4_000_000)
        || !matches!(anchor, "LEFT_STEM" | "RIGHT_STEM" | "MIDDLE_LEFT")
    {
        return Err("invalid_head_search_window".into());
    }
    let mut out = Vec::new();
    for &source_shape in shapes {
        let asset = bank
            .get(source_shape, point_size)
            .map_err(|e| e.to_string())?;
        let a = asset.anchor(anchor).map_err(|e| e.to_string())?;
        let (ax, ay) = a.rounded().map_err(|e| e.to_string())?;
        let slim = asset.slim_bounds();
        let mut best = None::<(i32, i32, f64)>;
        'window: for &dy in yo {
            for &dx in xo {
                let x = pivot[0] + dx;
                let y = pivot[1] + dy;
                if veto([
                    x - ax + slim[0] as i32,
                    y - ay + slim[1] as i32,
                    slim[2] as i32,
                    slim[3] as i32,
                ])? {
                    if x == pivot[0] && y == pivot[1] {
                        break 'window;
                    }
                    continue;
                }
                let score = asset
                    .template()
                    .evaluate(table, x, y, Some(a), Weights::default())
                    .map_err(|e| e.to_string())?
                    .score;
                if score <= 0.4 {
                    if best.is_none_or(|v| v.2 > score) {
                        best = Some((x, y, score));
                    }
                } else if x == pivot[0] && y == pivot[1] && score >= 1.0 {
                    break 'window;
                }
            }
        }
        if let Some((x, y, score)) = best {
            let mut shape = source_shape;
            if shape == "NOTEHEAD_BLACK" {
                let v = bank
                    .get("NOTEHEAD_VOID", point_size)
                    .map_err(|e| e.to_string())?;
                if v.template()
                    .evaluate(
                        table,
                        x,
                        y,
                        Some(v.anchor(anchor).map_err(|e| e.to_string())?),
                        Weights::default(),
                    )
                    .map_err(|e| e.to_string())?
                    .hole_white_ratio
                    >= 0.2
                {
                    shape = "NOTEHEAD_VOID";
                }
            }
            let selected = bank.get(shape, point_size).map_err(|e| e.to_string())?;
            let (dx, dy) = selected
                .anchor(anchor)
                .map_err(|e| e.to_string())?
                .rounded()
                .map_err(|e| e.to_string())?;
            let b = selected.slim_bounds();
            out.push(json!({"shape":shape,"source_shape":source_shape,"anchor":anchor,"x":x,"y":y,"score":score,"grade":0.8*(1.0-score/0.5),"bounds":[x-dx+b[0] as i32,y-dy+b[1] as i32,b[2],b[3]]}));
        }
    }
    Ok(out)
}
fn paint(mask: &mut GrayImage, raw: &Value) -> Result<(), String> {
    let g = beam_dispatch::glyph_from_json(raw)?;
    for r in &g.runs {
        for v in r.start..r.start + r.length {
            let (x, y) = if g.orientation == audiveris_staff_removal::Orientation::Vertical {
                (g.left as i64 + r.position as i64, g.top as i64 + v as i64)
            } else {
                (g.left as i64 + v as i64, g.top as i64 + r.position as i64)
            };
            if x < 0 || y < 0 || x >= mask.width as i64 || y >= mask.height as i64 {
                return Err("head_mask_pixel_outside_page".into());
            }
            mask.pixels[y as usize * mask.width + x as usize] = 0;
        }
    }
    Ok(())
}
pub fn prepare(
    binary: &GrayImage,
    grid: &Value,
    seeds: &Value,
    ledgers: &Value,
) -> Result<(DistanceTable, Value), String> {
    if seeds["stem_seeds_complete"] != true || ledgers["ledgers_complete"] != true {
        return Err("incomplete_head_mask_dependencies".into());
    }
    let (mut mask, _) = crate::note_mask::from_grid(binary, grid)?;
    let mut seed_count = 0;
    let mut ledger_count = 0;
    for sys in seeds["systems"].as_array().ok_or("invalid_head_seeds")? {
        for g in sys["accepted_seeds"]
            .as_array()
            .ok_or("invalid_head_seeds")?
        {
            paint(&mut mask, g)?;
            seed_count += 1;
        }
    }
    for sys in ledgers["systems"]
        .as_array()
        .ok_or("invalid_head_ledgers")?
    {
        for g in sys["registration"]["inters"]
            .as_array()
            .ok_or("invalid_head_ledgers")?
        {
            if g["removed"] == false {
                paint(&mut mask, &g["glyph"])?;
                ledger_count += 1;
            }
        }
    }
    let table = template_probes::distances(binary, Some(&mask))?;
    let mut digest = Sha256::new();
    for v in table.values() {
        digest.update((v as i16).to_be_bytes());
    }
    Ok((
        table,
        json!({"distance_mask_complete":true,"mask_sha256":format!("{:x}",Sha256::digest(&mask.pixels)),"distance_sha256_be_i16":format!("{:x}",digest.finalize()),"masked_pixel_count":mask.pixels.iter().filter(|&&v|v==0).count(),"vertical_seed_count":seed_count,"active_ledger_count":ledger_count}),
    ))
}
/// Area.intersects(Rectangle) for the original constant-offset staff-spline
/// band. The bottom offset includes the original +1 pixel. Clipped monotone
/// Bezier extrema decide strict positive-area overlap, rather than sampling.
pub fn intersects_seed(
    curve: &Curve,
    bounds: [f64; 4],
    above: f64,
    below: f64,
) -> Result<bool, String> {
    if bounds
        .iter()
        .chain([above, below].iter())
        .any(|v| !v.is_finite())
        || bounds[2] <= 0.
        || bounds[3] <= 0.
        || above >= below + 1.
    {
        return Err("invalid_head_seed_area".into());
    }
    for seg in curve.path_segments() {
        let left = bounds[0].max(seg[0][0]);
        let right = (bounds[0] + bounds[2]).min(seg.last().unwrap()[0]);
        if left < right {
            let e = audiveris_area::monotone_clip_y_bounds(seg, left, right)?;
            if e[0] + above < bounds[1] + bounds[3] && e[1] + below + 1. > bounds[1] {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
#[allow(clippy::too_many_arguments)]
pub fn analyze(
    binary: &GrayImage,
    grid: &Value,
    seeds: &Value,
    ledgers: &Value,
    head_scale: &Value,
    max_stem: usize,
    main_interline: usize,
) -> Result<Value, String> {
    analyze_impl(
        binary,
        grid,
        seeds,
        ledgers,
        head_scale,
        max_stem,
        main_interline,
        None,
    )
}
/// Opt-in ordinary five-line seed dependency with exact declared family/size.
/// Later range/glyph dependencies reject this metadata until they propagate it.
#[allow(clippy::too_many_arguments)]
pub fn analyze_catalog(
    binary: &GrayImage,
    grid: &Value,
    seeds: &Value,
    ledgers: &Value,
    head_scale: &Value,
    max_stem: usize,
    main_interline: usize,
) -> Result<Value, String> {
    let catalog = crate::head_staff_catalog::TemplateCatalog::from_head_scale(head_scale)?;
    analyze_impl(
        binary,
        grid,
        seeds,
        ledgers,
        head_scale,
        max_stem,
        main_interline,
        Some(&catalog),
    )
}
#[allow(clippy::too_many_arguments)]
fn analyze_impl(
    binary: &GrayImage,
    grid: &Value,
    seeds: &Value,
    ledgers: &Value,
    head_scale: &Value,
    max_stem: usize,
    main_interline: usize,
    declared: Option<&crate::head_staff_catalog::TemplateCatalog>,
) -> Result<Value, String> {
    if declared.is_none()
        && (head_scale["music_font"]["name"] != "Bravura"
            || head_scale["music_font"]["pointSize"] != 51)
    {
        return Err("unsupported_head_template_catalog".into());
    }
    if !(1..=128).contains(&main_interline) {
        return Err("unsupported_head_sheet_scale".into());
    }
    let bank = if let Some(catalog) = declared {
        catalog.bank()?
    } else {
        Bank::load_builtin().map_err(|e| e.to_string())?
    };
    // Preflight every staff catalog before preparing pixels. The legacy entry
    // point retains its certified 51pt scope; explicit catalogs stay opt-in.
    for staff in grid["staffs"]
        .as_array()
        .ok_or("invalid_head_grid_staffs")?
    {
        if staff["tablature"].as_bool().unwrap_or(false) {
            continue;
        }
        if declared.is_some() && staff["drum"] == true {
            return Err("unsupported_head_seed_staff_catalog".into());
        }
        let specific = staff["specific_interline"]
            .as_u64()
            .ok_or("invalid_head_staff_scale")?;
        let point_size = if let Some(catalog) = declared {
            catalog.staff_point_size(specific as usize, main_interline)?
        } else {
            51
        };
        if specific == 0
            || specific > 128
            || declared.is_none()
                && Bank::staff_point_size(specific as f64, main_interline as f64, 51)
                    .map_err(|e| e.to_string())?
                    != 51
        {
            return Err("unsupported_head_staff_template_catalog".into());
        }
        for shape in ["NOTEHEAD_BLACK", "NOTEHEAD_VOID"] {
            bank.get(shape, point_size).map_err(|e| e.to_string())?;
        }
    }
    let (table, mask) = prepare(binary, grid, seeds, ledgers)?;
    let xo = x_offsets(max_stem)?;
    let mut systems = Vec::new();
    let mut total = 0;
    let mut attempts = 0;
    let mut eval_budget = 0usize;
    let mut ledger_total = 0usize;
    let mut ledger_pairs = 0usize;
    for sys in seeds["systems"].as_array().ok_or("invalid_head_systems")? {
        let id = sys["system_id"].as_u64().ok_or("invalid_head_system_id")?;
        let specs = crate::ledger_registration::staffs_from_grid(grid, id)?;
        let mut gs = sys["accepted_seeds"]
            .as_array()
            .ok_or("invalid_head_seeds")?
            .iter()
            .map(beam_dispatch::glyph_from_json)
            .collect::<Result<Vec<_>, String>>()?;
        // Original first sorts by ordinate, then each selected slice by abscissa;
        // both sorts are stable, with identical abscissa preserving ordinate.
        gs.sort_by_key(|g| g.top);
        let mut staffs = Vec::new();
        for staff in specs {
            if staff.tablature {
                continue;
            }
            if staff.lines.len() != 5 {
                return Err("unsupported_head_staff".into());
            }
            let il = staff.specific_interline;
            let point_size = if let Some(catalog) = declared {
                catalog.staff_point_size(il, main_interline)?
            } else {
                51
            };
            let lines = staff
                .lines
                .iter()
                .map(|p| Curve::interpolate(p.clone()).ok_or("invalid_head_staff_curve".into()))
                .collect::<Result<Vec<_>, String>>()?;
            let mut scanners = Vec::new();
            for position in
                crate::head_staff_catalog::positions(staff.lines.len(), false, main_interline)?
            {
                let li = position.line_index;
                let dir = position.dir;
                let pitch = position.pitch;
                let line = &lines[li];
                let yo = position.y_offsets;
                let above = il as f64 * (dir as f64 - 0.75) / 2.;
                let below = il as f64 * (dir as f64 + 0.75) / 2.;
                let mut selected = Vec::new();
                for g in &gs {
                    if intersects_seed(
                        line,
                        [g.left as f64, g.top as f64, g.width as f64, g.height as f64],
                        above,
                        below,
                    )? {
                        selected.push(g);
                    }
                }
                selected.sort_by_key(|g| g.left);
                let mut records = Vec::new();
                for g in selected {
                    attempts += 1;
                    eval_budget = eval_budget
                        .checked_add(xo.len() * yo.len() * 4)
                        .ok_or("head_search_eval_budget")?;
                    if eval_budget > 1_000_000 {
                        return Err("head_search_eval_budget".into());
                    }
                    if attempts > 100_000 {
                        return Err("head_search_seed_budget".into());
                    }
                    let rough = (g.left as f64 + g.width as f64 / 2.).round_ties_even();
                    let yline = line.y(rough);
                    let fit =
                        beam_structure::glyph_stem_line(g)?.ok_or("head_seed_without_line")?;
                    if fit[3] == fit[1] {
                        return Err("head_seed_not_vertical".into());
                    }
                    let x0 = (fit[0] + (fit[2] - fit[0]) * (yline - fit[1]) / (fit[3] - fit[1]))
                        .round_ties_even() as i32;
                    let y0 = if dir == -1 && li > 0 {
                        (line.y(x0 as f64) + lines[li - 1].y(x0 as f64)) / 2.
                    } else {
                        line.y(x0 as f64) + dir as f64 * il as f64 / 2.
                    };
                    let y0 = y0.round_ties_even() as i32;
                    let mut matches = Vec::new();
                    for a in ["LEFT_STEM", "RIGHT_STEM"] {
                        matches.extend(search_selected_with_veto(
                            &table,
                            (&bank, point_size),
                            [x0, y0],
                            (&xo, &yo),
                            a,
                            &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"],
                            &mut |_| Ok(false),
                        )?);
                    }
                    total += matches.len();
                    records.push(json!({"seed":beam_dispatch::glyph_json(g),"x0":x0,"y0":y0,"matches":matches}));
                }
                scanners.push(json!({"line_index":li,"dir":dir,"pitch":pitch,"x_offsets":xo,"y_offsets":yo,"seeds":records}));
            }
            let post_system = ledgers["systems"]
                .as_array()
                .ok_or("invalid_head_ledger_systems")?
                .iter()
                .find(|s| s["system_id"].as_u64() == Some(id))
                .ok_or("missing_head_ledger_system")?;
            let input = crate::head_ledger_search::from_registration(&staff, post_system)?;
            let ledger_report = crate::head_ledger_search::scan_at_size(
                &input,
                &crate::head_ledger_search::Search {
                    table: &table,
                    main_interline,
                    bank: &bank,
                    seeds: &gs,
                    x_offsets: &xo,
                },
                point_size,
            )?;
            ledger_total += ledger_report["proposal_count"].as_u64().unwrap() as usize;
            ledger_pairs += ledger_report["seed_pitch_pairs"].as_u64().unwrap() as usize;
            staffs.push(json!({"staff_id":staff.id,"point_size":point_size,"scanners":scanners,"ledger_scanners":ledger_report["ledger_scanners"]}));
        }
        systems.push(json!({"system_id":id,"staffs":staffs}));
    }
    let mut report = json!({"stage":"HEADS.ALL_SEED_TEMPLATE_SEARCH","seed_template_search_complete":true,"ledger_seed_search_complete":true,"heads_complete":false,"note_recognition_complete":false,"musicxml_complete":false,"distance_preparation":mask,"seed_pitch_pairs":attempts+ledger_pairs,"proposal_count":total+ledger_total,"staff_seed_pitch_pairs":attempts,"staff_proposal_count":total,"ledger_seed_pitch_pairs":ledger_pairs,"ledger_proposal_count":ledger_total,"systems":systems,"scope":"Bravura 51pt, fresh native five-line staff and signed ledger pitches; automatic stem seed black/void raw template proposals before competitor/bar veto, glyph retrieval or graph registration","pending":["frozen bar/competitor veto","horizontal range search","glyph retrieval and HeadInter registration","duplicate/overlap purge and tally","stem attachment, rhythm and MusicXML"]});
    if let Some(catalog) = declared {
        report["template_catalog"] = json!(catalog);
        report["template_catalog_required"] = json!(true);
        report["scope"] = json!(
            "Explicit exact oval family/point catalog for ordinary five-line staff and ledger raw seed matching. Upstream font inference and later range/glyph/registration catalog propagation are not certified."
        );
    }
    Ok(report)
}
