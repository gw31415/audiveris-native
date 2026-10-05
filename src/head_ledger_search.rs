// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original HEADS LedgerAdapter / signed outside-staff raw seed search.
//! No competitor veto, glyph retrieval or HeadInter graph registration yet.
use crate::{
    beam_dispatch, beam_structure, curves::Curve, head_seed_search, ledger_registration::Staff,
};
use audiveris_staff_removal::GlyphPixels;
use audiveris_template_bank::Bank;
use audiveris_template_match::DistanceTable;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LedgerLine {
    pub index: i32,
    pub line: [f64; 4],
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaffLedgers {
    pub staff_id: u64,
    pub interline: usize,
    pub merged_first: bool,
    pub merged_last: bool,
    pub ledger_lines: Vec<LedgerLine>,
}
pub struct Search<'a> {
    /// Original NoteHeadsBuilder parameters use sheet scale, not staff spacing.
    pub main_interline: usize,
    pub table: &'a DistanceTable,
    pub bank: &'a Bank,
    pub seeds: &'a [GlyphPixels],
    pub x_offsets: &'a [i32],
}
fn valid(line: &[f64; 4]) -> bool {
    line.iter().all(|v| v.is_finite() && v.abs() <= 4_000_000.) && line[0] < line[2]
}
fn y_at(line: [f64; 4], x: f64) -> f64 {
    line[1] + (line[3] - line[1]) * (x - line[0]) / (line[2] - line[0])
}
/// Original Scanner.getTheoreticalOrdinate: first further ledger embracing x,
/// with ceil/floor inclusive endpoints; otherwise signed half-interline fallback.
pub fn ordinate(
    main: [f64; 4],
    further: &[[f64; 4]],
    pitch: i32,
    interline: usize,
    x: i32,
) -> Result<i32, String> {
    if !valid(&main)
        || further.len() > 32768
        || further.iter().any(|l| !valid(l))
        || pitch.unsigned_abs() < 6
        || pitch.unsigned_abs() > 4096
        || !(1..=128).contains(&interline)
        || x.unsigned_abs() > 4_000_000
    {
        return Err("invalid_head_ledger_ordinate".into());
    }
    let y = y_at(main, x as f64);
    if pitch % 2 == 0 {
        return Ok(y.round_ties_even() as i32);
    }
    for &line in further {
        if x as f64 >= line[0].ceil() && x as f64 <= line[2].floor() {
            return Ok(((y + y_at(line, x as f64)) / 2.).round_ties_even() as i32);
        }
    }
    Ok((y + pitch.signum() as f64 * interline as f64 / 2.).round_ties_even() as i32)
}
pub fn from_registration(staff: &Staff, system: &Value) -> Result<StaffLedgers, String> {
    let registration = &system["registration"];
    let nodes = registration["inters"]
        .as_array()
        .ok_or("missing_head_ledger_nodes")?;
    let maps = registration["staff_maps"]
        .as_array()
        .ok_or("missing_head_ledger_maps")?;
    let map = maps
        .iter()
        .find(|r| r["staff_id"].as_u64() == Some(staff.id))
        .ok_or("missing_head_staff_ledger_map")?;
    let mut lines = Vec::new();
    for level in map["lines"]
        .as_array()
        .ok_or("invalid_head_staff_ledger_map")?
    {
        let index = level["index"]
            .as_i64()
            .filter(|i| i.unsigned_abs() <= 1024 && *i != 0)
            .ok_or("invalid_head_ledger_index")? as i32;
        for id in level["ledger_ids"]
            .as_array()
            .ok_or("invalid_head_ledger_ids")?
        {
            let node = nodes
                .iter()
                .find(|n| n["id"] == *id)
                .ok_or("missing_head_ledger_reference")?;
            if node["removed"] != false
                || node["index"].as_i64() != Some(index as i64)
                || node["staff_id"].as_u64() != Some(staff.id)
            {
                return Err("inconsistent_head_ledger_reference".into());
            }
            let g = beam_dispatch::glyph_from_json(&node["glyph"])?;
            let line = beam_structure::glyph_stem_line(&g)?.ok_or("head_ledger_without_line")?;
            lines.push(LedgerLine { index, line });
        }
    }
    Ok(StaffLedgers {
        staff_id: staff.id,
        interline: staff.specific_interline,
        merged_first: staff.merged_first,
        merged_last: staff.merged_last,
        ledger_lines: lines,
    })
}
pub fn scan(input: &StaffLedgers, search: &Search<'_>) -> Result<Value, String> {
    scan_at_size(input, search, 51)
}
/// Same signed ledger geometry, selecting the explicit staff's exact templates.
pub fn scan_at_size(
    input: &StaffLedgers,
    search: &Search<'_>,
    point_size: u16,
) -> Result<Value, String> {
    if !(1..=128).contains(&search.main_interline)
        || !(1..=128).contains(&input.interline)
        || input.ledger_lines.len() > 32768
        || search.seeds.len() > 8192
        || search.x_offsets.is_empty()
        || search.x_offsets.len() > 129
    {
        return Err("head_ledger_search_budget".into());
    }
    for shape in ["NOTEHEAD_BLACK", "NOTEHEAD_VOID"] {
        search
            .bank
            .get(shape, point_size)
            .map_err(|e| e.to_string())?;
    }
    let mut levels = BTreeMap::<i32, Vec<[f64; 4]>>::new();
    for l in &input.ledger_lines {
        if l.index == 0 || l.index.unsigned_abs() > 1024 || !valid(&l.line) {
            return Err("invalid_head_ledger_line".into());
        }
        levels.entry(l.index).or_default().push(l.line);
    }
    let mut seeds: Vec<_> = search.seeds.iter().collect();
    seeds.sort_by_key(|g| g.top);
    let mut scanners = Vec::new();
    let mut pairs = 0usize;
    let mut proposals = 0usize;
    let mut work = 0usize;
    for sign in [-1, 1] {
        let look_further = !(sign > 0 && input.merged_first || sign < 0 && input.merged_last);
        let mut index = sign;
        while let Some(lines) = levels.get(&index) {
            if lines.is_empty() {
                break;
            }
            let main_pitch = sign * (4 + 2 * index.abs());
            for (order, &line) in lines.iter().enumerate() {
                for dir in if look_further { vec![0, sign] } else { vec![0] } {
                    let pitch = main_pitch + dir;
                    let further = if dir == 0 {
                        Vec::new()
                    } else {
                        levels.get(&(index + sign)).cloned().unwrap_or_default()
                    };
                    let yo = head_seed_search::y_offsets(search.main_interline, dir, dir != 0)?;
                    let curve = Curve::interpolate(vec![[line[0], line[1]], [line[2], line[3]]])
                        .ok_or("invalid_head_ledger_curve")?;
                    let above = input.interline as f64 * (dir as f64 - 0.75) / 2.;
                    let below = input.interline as f64 * (dir as f64 + 0.75) / 2.;
                    let mut selected = Vec::new();
                    for g in &seeds {
                        work += 1;
                        if work > 1_000_000 {
                            return Err("head_ledger_search_work_budget".into());
                        }
                        if head_seed_search::intersects_seed(
                            &curve,
                            [g.left as f64, g.top as f64, g.width as f64, g.height as f64],
                            above,
                            below,
                        )? {
                            selected.push(*g);
                        }
                    }
                    selected.sort_by_key(|g| g.left);
                    let mut records = Vec::new();
                    for g in selected {
                        pairs += 1;
                        work = work
                            .checked_add(search.x_offsets.len() * yo.len() * 4)
                            .ok_or("head_ledger_search_work_budget")?;
                        if work > 1_000_000 {
                            return Err("head_ledger_search_work_budget".into());
                        }
                        let rough = (g.left as f64 + g.width as f64 / 2.).round_ties_even();
                        let yl = y_at(line, rough);
                        let fit =
                            beam_structure::glyph_stem_line(g)?.ok_or("head_seed_without_line")?;
                        if fit[1] == fit[3] {
                            return Err("head_seed_not_vertical".into());
                        }
                        let x0 = (fit[0] + (fit[2] - fit[0]) * (yl - fit[1]) / (fit[3] - fit[1]))
                            .round_ties_even() as i32;
                        let y0 = ordinate(line, &further, pitch, input.interline, x0)?;
                        let mut matches = Vec::new();
                        for a in ["LEFT_STEM", "RIGHT_STEM"] {
                            matches.extend(head_seed_search::search_selected_with_veto(
                                search.table,
                                (search.bank, point_size),
                                [x0, y0],
                                (search.x_offsets, &yo),
                                a,
                                &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"],
                                &mut |_| Ok(false),
                            )?);
                        }
                        proposals += matches.len();
                        records.push(json!({"seed":beam_dispatch::glyph_json(g),"x0":x0,"y0":y0,"matches":matches}));
                    }
                    let mut xs = BTreeSet::new();
                    xs.extend([
                        line[0].ceil() as i32 - 1,
                        line[0].ceil() as i32,
                        line[2].floor() as i32,
                        line[2].floor() as i32 + 1,
                        ((line[0] + line[2]) / 2.).round_ties_even() as i32,
                    ]);
                    for l in &further {
                        xs.extend([
                            l[0].ceil() as i32 - 1,
                            l[0].ceil() as i32,
                            l[2].floor() as i32,
                            l[2].floor() as i32 + 1,
                        ]);
                    }
                    let controls = xs
                        .into_iter()
                        .map(|x| {
                            Ok(json!({"x":x,"y":ordinate(line,&further,pitch,input.interline,x)?}))
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    scanners.push(json!({"ledger_index":index,"ledger_order":order,"dir":dir,"pitch":pitch,"main_line":line,"further_lines":further,"x_offsets":search.x_offsets,"y_offsets":yo,"ordinate_controls":controls,"seeds":records}));
                }
            }
            index += sign;
        }
    }
    Ok(json!({"ledger_scanners":scanners,"seed_pitch_pairs":pairs,"proposal_count":proposals}))
}
