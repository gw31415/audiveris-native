// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original pre-registration HEADS frozen-bar and good-competitor veto.
use crate::{GrayImage, curves::Curve, head_seed_search};
use audiveris_area::LinearArea;
use audiveris_template_bank::Bank;
use serde_json::{Value, json};

#[derive(Clone)]
struct Obstacle {
    inter: Value,
    area: Option<LinearArea>,
    points: Vec<[f64; 2]>,
    bounds: [f64; 4],
}
fn rect(v: &Value) -> Result<[f64; 4], String> {
    let a = v.as_array().ok_or("invalid_head_obstacle_bounds")?;
    if a.len() != 4 {
        return Err("invalid_head_obstacle_bounds".into());
    }
    let mut b = [0.; 4];
    for (n, x) in a.iter().enumerate() {
        b[n] = x.as_f64().ok_or("invalid_head_obstacle_bounds")?;
    }
    if b.iter().any(|v| !v.is_finite()) || b[2] <= 0. || b[3] <= 0. {
        return Err("invalid_head_obstacle_bounds".into());
    }
    Ok(b)
}
fn vertical(v: &Value) -> bool {
    matches!(
        v["kind"].as_str(),
        Some(
            "BarlineInter"
                | "BarConnectorInter"
                | "BracketInter"
                | "BracketConnectorInter"
                | "VerticalSerifInter"
        )
    )
}
fn beam(v: &Value) -> bool {
    matches!(
        v["shape"].as_str(),
        Some("BEAM" | "BEAM_HOOK" | "BEAM_SMALL" | "BEAM_HOOK_SMALL")
    )
}
fn competing(v: &Value) -> bool {
    matches!(
        v["shape"].as_str(),
        Some(
            "THICK_BARLINE"
                | "THIN_BARLINE"
                | "THICK_CONNECTOR"
                | "THIN_CONNECTOR"
                | "BEAM"
                | "BEAM_HOOK"
                | "BEAM_SMALL"
                | "BEAM_HOOK_SMALL"
                | "MULTIPLE_REST"
                | "TREMOLO_1"
                | "TREMOLO_2"
                | "TREMOLO_3"
                | "VERTICAL_SERIF"
        )
    )
}
fn good(v: &Value) -> Result<bool, String> {
    let threshold = if v["kind"] == "BarConnectorInter" {
        0.65
    } else if beam(v) {
        0.35
    } else {
        0.4
    };
    let grade = v["grade"].as_f64().ok_or("missing_head_competitor_grade")?;
    if !grade.is_finite() {
        return Err("invalid_head_competitor_grade".into());
    }
    Ok(grade >= threshold)
}
impl Obstacle {
    fn new(inter: &Value) -> Result<Self, String> {
        let bounds = rect(&inter["bounds"])?;
        let points = if vertical(inter) || beam(inter) {
            let m: Vec<f64> = serde_json::from_value(inter["median"].clone())
                .map_err(|_| "missing_head_obstacle_median")?;
            if m.len() != 4 || m.iter().any(|v| !v.is_finite()) {
                return Err("invalid_head_obstacle_median".into());
            }
            let half = inter[if vertical(inter) { "width" } else { "height" }]
                .as_f64()
                .ok_or("missing_head_obstacle_thickness")?
                / 2.;
            if !half.is_finite() || half <= 0. {
                return Err("invalid_head_obstacle_thickness".into());
            }
            if vertical(inter) {
                vec![
                    [m[0] - half, m[1]],
                    [m[2] - half, m[3]],
                    [m[2] + half, m[3]],
                    [m[0] + half, m[1]],
                ]
            } else {
                vec![
                    [m[0], m[1] - half],
                    [m[2], m[3] - half],
                    [m[2], m[3] + half],
                    [m[0], m[1] + half],
                ]
            }
        } else if let Some(a) = inter["area_points"].as_array() {
            a.iter()
                .map(|p| {
                    serde_json::from_value(p.clone())
                        .map_err(|_| "invalid_head_obstacle_area".into())
                })
                .collect::<Result<Vec<[f64; 2]>, String>>()?
        } else if inter["area_is_null"] == true {
            Vec::new()
        } else {
            // Future eligible inters must supply their original area or explicit
            // null-area metadata; never substitute a bbox for unknown geometry.
            return Err("unsupported_head_competitor_area".into());
        };
        let area = if points.is_empty() {
            None
        } else {
            Some(LinearArea::polygon(&points)?)
        };
        Ok(Self {
            inter: inter.clone(),
            area,
            points,
            bounds,
        })
    }
    fn intersects(&self, b: [f64; 4]) -> Result<bool, String> {
        if self.inter["kind"] == "HeadInter" {
            return Ok(false);
        }
        match &self.area {
            Some(a) => a.intersects_rect(b),
            None => Ok(self.bounds[0] < b[0] + b[2]
                && self.bounds[0] + self.bounds[2] > b[0]
                && self.bounds[1] < b[1] + b[3]
                && self.bounds[1] + self.bounds[3] > b[1]),
        }
    }
    fn area_report(&self) -> Value {
        json!({"points":self.points,"bounds":self.area.as_ref().unwrap().bounds()})
    }
}
pub struct Context {
    bars: Vec<Obstacle>,
    competitors: Vec<Obstacle>,
    main_interline: usize,
}
pub struct Slice<'a> {
    bars: Vec<&'a Obstacle>,
    competitors: Vec<&'a Obstacle>,
}
impl Context {
    pub fn from_inters(
        inters: &[Value],
        main_interline: usize,
        max_stem: usize,
    ) -> Result<Self, String> {
        if !(1..=128).contains(&main_interline)
            || !(1..=128).contains(&max_stem)
            || inters.len() > 100_000
        {
            return Err("invalid_head_veto_parameters".into());
        }
        let min_beam = (2.5 * main_interline as f64).round_ties_even();
        let mut bars = Vec::new();
        let mut competitors = Vec::new();
        for v in inters {
            if v["removed"] == true {
                continue;
            }
            if v["frozen"] == true
                && matches!(
                    v["kind"].as_str(),
                    Some("BarlineInter" | "BarConnectorInter")
                )
            {
                bars.push(Obstacle::new(v)?);
            }
            if !competing(v) || !good(v)? {
                continue;
            }
            if vertical(v)
                && v["width"]
                    .as_f64()
                    .ok_or("missing_head_vertical_width")?
                    .floor()
                    <= max_stem as f64
            {
                continue;
            }
            if beam(v) {
                let group = inters.iter().find(|g| {
                    g["kind"] == "BeamGroupInter"
                        && g["members"]
                            .as_array()
                            .is_some_and(|members| members.contains(&v["id"]))
                });
                let group = group.ok_or("head_competitor_beam_without_group")?;
                let mut long = false;
                for id in group["members"]
                    .as_array()
                    .ok_or("invalid_head_beam_group")?
                {
                    let member = inters
                        .iter()
                        .find(|n| n["id"] == *id)
                        .ok_or("missing_head_beam_group_member")?;
                    if rect(&member["bounds"])?[2] >= min_beam {
                        long = true;
                    }
                }
                if !long {
                    continue;
                }
            }
            competitors.push(Obstacle::new(v)?);
        }
        bars.sort_by(|a, b| a.bounds[1].total_cmp(&b.bounds[1]));
        competitors.sort_by(|a, b| a.bounds[1].total_cmp(&b.bounds[1]));
        Ok(Self {
            bars,
            competitors,
            main_interline,
        })
    }
    pub fn slice(&self, line: &Curve, interline: usize, dir: i32) -> Result<Slice<'_>, String> {
        if !(1..=128).contains(&interline) || !(-1..=1).contains(&dir) {
            return Err("invalid_head_veto_pitch".into());
        }
        let middle = interline as f64 * dir as f64 / 2.;
        let margin = 2. * self.main_interline as f64;
        let mut bars = Vec::new();
        let mut competitors = Vec::new();
        for b in &self.bars {
            let bounds = b.area.as_ref().unwrap().bounds().map(f64::from);
            if head_seed_search::intersects_seed(line, bounds, middle - margin, middle + margin)? {
                bars.push(b);
            }
        }
        for c in &self.competitors {
            if head_seed_search::intersects_seed(
                line,
                c.bounds,
                middle - interline as f64 / 4.,
                middle + interline as f64 / 4.,
            )? {
                competitors.push(c);
            }
        }
        competitors.sort_by(|a, b| a.bounds[0].total_cmp(&b.bounds[0]));
        Ok(Slice { bars, competitors })
    }
    pub fn report(&self) -> Value {
        json!({"bars":self.bars.iter().map(|b|&b.inter).collect::<Vec<_>>(),"competitors":self.competitors.iter().map(|c|&c.inter).collect::<Vec<_>>()})
    }
}
impl Slice<'_> {
    pub fn veto(&self, box_: [i32; 4]) -> Result<bool, String> {
        let b = box_.map(f64::from);
        if b[2] <= 0. || b[3] <= 0. {
            return Err("invalid_head_veto_box".into());
        }
        for o in self.bars.iter().chain(&self.competitors) {
            if o.intersects(b)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn report(&self) -> Value {
        json!({"bars":self.bars.iter().map(|b|b.area_report()).collect::<Vec<_>>(),"competitors":self.competitors.iter().map(|c|&c.inter).collect::<Vec<_>>()})
    }
}
pub struct Input<'a> {
    pub binary: &'a GrayImage,
    pub grid: &'a Value,
    pub seeds: &'a Value,
    pub ledgers: &'a Value,
    pub shared_sig: &'a Value,
    pub raw_search: &'a Value,
    pub main_interline: usize,
    pub max_stem: usize,
}
pub fn analyze(input: Input<'_>) -> Result<Value, String> {
    let Input {
        binary,
        grid,
        seeds,
        ledgers,
        shared_sig,
        raw_search,
        main_interline,
        max_stem,
    } = input;
    if raw_search["seed_template_search_complete"] != true
        || raw_search["ledger_seed_search_complete"] != true
    {
        return Err("incomplete_head_veto_dependencies".into());
    }
    let declared = crate::head_staff_catalog::TemplateCatalog::from_report(raw_search)?;
    let bank = if let Some(catalog) = &declared {
        catalog.bank()?
    } else {
        Bank::load_builtin().map_err(|e| e.to_string())?
    };
    let (table, mask) = head_seed_search::prepare(binary, grid, seeds, ledgers)?;
    let mut systems = raw_search["systems"]
        .as_array()
        .ok_or("invalid_head_veto_systems")?
        .clone();
    let mut staff_total = 0usize;
    let mut ledger_total = 0usize;
    let mut pairs = 0usize;
    for sys in &mut systems {
        let id = sys["system_id"]
            .as_u64()
            .ok_or("invalid_head_veto_system")?;
        let inters: Vec<Value> = shared_sig["inters"]
            .as_array()
            .ok_or("invalid_head_veto_sig")?
            .iter()
            .filter(|v| v["origin"]["system"].as_u64() == Some(id))
            .cloned()
            .collect();
        let context = Context::from_inters(&inters, main_interline, max_stem)?;
        sys["prolog"] = context.report();
        let staffs = crate::ledger_registration::staffs_from_grid(grid, id)?;
        for f in sys["staffs"]
            .as_array_mut()
            .ok_or("invalid_head_veto_staffs")?
        {
            let staff = staffs
                .iter()
                .find(|s| Some(s.id) == f["staff_id"].as_u64())
                .ok_or("missing_head_veto_staff")?;
            let point_size = if let Some(catalog) = &declared {
                let size = catalog.staff_point_size(staff.specific_interline, main_interline)?;
                if f["point_size"].as_u64() != Some(u64::from(size)) {
                    return Err("inconsistent_head_seed_catalog".into());
                }
                size
            } else {
                51
            };
            for key in ["scanners", "ledger_scanners"] {
                for c in f[key].as_array_mut().ok_or("invalid_head_veto_scanners")? {
                    let line = if key == "scanners" {
                        let index =
                            c["line_index"].as_u64().ok_or("invalid_head_veto_line")? as usize;
                        Curve::interpolate(
                            staff
                                .lines
                                .get(index)
                                .ok_or("missing_head_veto_line")?
                                .clone(),
                        )
                    } else {
                        let l: [f64; 4] = serde_json::from_value(c["main_line"].clone())
                            .map_err(|_| "invalid_head_veto_ledger_line")?;
                        if l.iter().any(|v| !v.is_finite()) || l[0] >= l[2] {
                            return Err("invalid_head_veto_ledger_line".into());
                        }
                        // main_line stores endpoint coordinates, not a bbox.
                        Curve::interpolate(vec![[l[0], l[1]], [l[2], l[3]]])
                    }
                    .ok_or("invalid_head_veto_curve")?;
                    let dir = c["dir"].as_i64().ok_or("invalid_head_veto_dir")? as i32;
                    let slice = context.slice(&line, staff.specific_interline, dir)?;
                    c["slice"] = slice.report();
                    let xo: Vec<i32> = serde_json::from_value(c["x_offsets"].clone())
                        .map_err(|_| "invalid_head_veto_offsets")?;
                    let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone())
                        .map_err(|_| "invalid_head_veto_offsets")?;
                    for z in c["seeds"].as_array_mut().ok_or("invalid_head_veto_seeds")? {
                        let pivot = [
                            z["x0"].as_i64().ok_or("invalid_head_veto_pivot")? as i32,
                            z["y0"].as_i64().ok_or("invalid_head_veto_pivot")? as i32,
                        ];
                        let mut matches = Vec::new();
                        for a in ["LEFT_STEM", "RIGHT_STEM"] {
                            matches.extend(head_seed_search::search_selected_with_veto(
                                &table,
                                (&bank, point_size),
                                pivot,
                                (&xo, &yo),
                                a,
                                &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"],
                                &mut |b| slice.veto(b),
                            )?);
                        }
                        if key == "scanners" {
                            staff_total += matches.len();
                        } else {
                            ledger_total += matches.len();
                        }
                        z["matches"] = json!(matches);
                        pairs += 1;
                        if pairs > 100_000 {
                            return Err("head_veto_seed_budget".into());
                        }
                    }
                }
            }
        }
    }
    let mut report = json!({"stage":"HEADS.ALL_SEED_VETO","seed_veto_complete":true,"heads_complete":false,"musicxml_complete":false,"distance_preparation":mask,"systems":systems,"seed_pitch_pairs":pairs,"proposal_count":staff_total+ledger_total,"staff_proposal_count":staff_total,"ledger_proposal_count":ledger_total,"scope":"Bravura 51pt staff/ledger seed proposals with frozen vertical bars and good grouped beam/bar competitors; unsupported eligible area types fail explicitly. Before horizontal range search, glyph retrieval, registration and duplicate purge."});
    if let Some(catalog) = declared {
        report["template_catalog"] = json!(catalog);
        report["template_catalog_required"] = json!(true);
        report["scope"] = json!(
            "Explicit exact oval family and staff point size preserved through seed and signed-ledger frozen bar/competitor veto. Later range/glyph/registration catalog propagation remains pending."
        );
    }
    Ok(report)
}
