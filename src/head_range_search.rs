// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original HeadSpotsBuilder, black/void prefix and default oval range catalog. Candidates
//! precede aggregation, seed conflict filtering, glyph retrieval and SIG insertion.
use crate::{
    GrayImage, beam_dispatch, beam_spots, curves::Curve, head_ledger_search, head_seed_search,
    head_seed_veto,
};
use audiveris_staff_removal::GlyphPixels;
use audiveris_template_bank::Bank;
use audiveris_template_match::DistanceTable;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// HEAD_SPOTS are temporary GlyphFactory products, not registered free glyphs.
/// An index identifies the same sheet spot when two systems share it.
pub fn dispatch(image: &GrayImage, geometry: &beam_dispatch::Geometry) -> Result<Value, String> {
    let spots = beam_spots::extract_spots(image)?;
    geometry.validate_point_work(spots.len())?;
    let mut systems: Vec<Value> = geometry
        .systems
        .iter()
        .map(|s| json!({"system_id":s.id,"spots":[]}))
        .collect();
    let mut references = 0usize;
    for (index, spot) in spots.iter().enumerate() {
        let center = beam_dispatch::centroid(spot)?;
        for id in geometry.eligible(center)? {
            references += 1;
            if references > beam_dispatch::MAX_REGISTERED_GLYPHS {
                return Err("head_spot_reference_budget".into());
            }
            let row = systems.iter_mut().find(|s| s["system_id"] == id).unwrap();
            row["spots"]
                .as_array_mut()
                .unwrap()
                .push(json!({"sheet_spot_index":index,
                "glyph":beam_dispatch::glyph_json(spot),"centroid":center,"groups":["HEAD_SPOT"]}));
        }
    }
    Ok(
        json!({"head_spot_dispatch_complete":true,"head_spots_sha256":format!("{:x}",Sha256::digest(&image.pixels)),
        "sheet_spot_count":spots.len(),"system_spot_references":references,"systems":systems}),
    )
}

/// Original competitorsArea slice followed by horizontal growth of one specific
/// interline on each side. Positive-area intersection and half-open spot bounds.
pub fn relevant(
    line: &Curve,
    interline: usize,
    dir: i32,
    range: [i32; 2],
    spots: &[GlyphPixels],
) -> Result<(Vec<bool>, Vec<Value>), String> {
    if !(1..=128).contains(&interline)
        || !(-1..=1).contains(&dir)
        || range.iter().any(|x| x.unsigned_abs() > 4_000_000)
        || spots.len() > 100_000
    {
        return Err("invalid_head_range_relevance".into());
    }
    let len = (range[1] as i64 - range[0] as i64 + 1).max(0) as usize;
    if len > 4_000_000 {
        return Err("head_range_width_budget".into());
    }
    let mut flags = vec![false; len];
    let mut selected = Vec::new();
    let mid = interline as f64 * dir as f64 / 2.;
    for g in spots {
        // Also validate runs and bounds before their geometry enters the slice.
        beam_dispatch::centroid(g)?;
        if head_seed_search::intersects_seed(
            line,
            [g.left as f64, g.top as f64, g.width as f64, g.height as f64],
            mid - interline as f64 / 4.,
            mid + interline as f64 / 4.,
        )? {
            selected.push(beam_dispatch::glyph_json(g));
            let left = (g.left - interline as i32).max(range[0]);
            let right =
                (g.left as i64 + g.width as i64 + interline as i64).min(range[1] as i64 + 1);
            for x in left as i64..right {
                flags[(x - range[0] as i64) as usize] = true;
            }
        }
    }
    selected.sort_by_key(|g| g["bounds"][0].as_i64().unwrap());
    Ok((flags, selected))
}
pub fn spans(flags: &[bool], left: i32) -> Vec<[i32; 2]> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < flags.len() {
        if !flags[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < flags.len() && flags[i] {
            i += 1;
        }
        out.push([left + start as i32, left + i as i32 - 1]);
    }
    out
}

pub struct Scan<'a> {
    pub table: &'a DistanceTable,
    pub bank: &'a Bank,
    pub line: &'a Curve,
    pub interline: usize,
    pub dir: i32,
    pub range: [i32; 2],
    pub main_interline: usize,
    pub y_offsets: &'a [i32],
    pub spots: &'a [GlyphPixels],
}
/// The range loop's integer chamfer safety check skips exactly 2*templateHalf
/// pixels, including the loop increment. UNKNOWN (-1) is retained as foreground.
pub fn scan(
    input: Scan<'_>,
    ordinate: &mut dyn FnMut(i32) -> Result<i32, String>,
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Value, String> {
    scan_mode(input, ordinate, veto, false, 51)
}
/// Default original oval catalog in Shape enum order. Hollow templates remain
/// relevant without a HEAD_SPOT. Candidate grades precede the later staff boost.
/// Exact matched shapes with kernel oracle coverage. Default page catalogs
/// remain ALL_SHAPES; special staff planners must explicitly select their set.
pub const SUPPORTED_SHAPES: &[&str] = &[
    "BREVE",
    "BREVE_SMALL",
    "BREVE_CROSS",
    "BREVE_DIAMOND",
    "BREVE_TRIANGLE_DOWN",
    "BREVE_CIRCLE_X",
    "WHOLE_NOTE",
    "WHOLE_NOTE_SMALL",
    "WHOLE_NOTE_CROSS",
    "WHOLE_NOTE_DIAMOND",
    "WHOLE_NOTE_TRIANGLE_DOWN",
    "WHOLE_NOTE_CIRCLE_X",
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
pub const ALL_SHAPES: &[&str] = &["BREVE", "WHOLE_NOTE", "NOTEHEAD_VOID", "NOTEHEAD_BLACK"];
pub const HOLLOW_SHAPES: &[&str] = &["BREVE", "WHOLE_NOTE", "NOTEHEAD_VOID"];
pub const STEMLESS_BOOST: f64 = 0.;
pub const MIN_CONTEXTUAL_GRADE: f64 = 0.5;
pub const MIN_INTER_GRADE: f64 = 0.8 * 0.1;

pub fn candidate_decision(shape: &str, score: f64) -> Result<Value, String> {
    if !ALL_SHAPES.contains(&shape) {
        return Err("invalid_head_range_candidate".into());
    }
    candidate_decision_for_shape(shape, score)
}
pub fn candidate_decision_for_shape(shape: &str, score: f64) -> Result<Value, String> {
    if !SUPPORTED_SHAPES.contains(&shape) || !score.is_finite() || !(0.0..=1.0).contains(&score) {
        return Err("invalid_head_range_candidate".into());
    }
    let stemless = !crate::head_seed_search::ALL_STEM_SHAPES.contains(&shape);
    let grade = 0.8 * (1.0 - score / 0.5).max(0.0);
    let future_grade = if stemless && grade < 0.8 {
        grade + STEMLESS_BOOST * (0.8 - grade)
    } else {
        grade
    };
    let weak = stemless && future_grade < MIN_CONTEXTUAL_GRADE;
    let create = grade >= MIN_INTER_GRADE;
    Ok(
        json!({"stemless":stemless,"intrinsic_grade":grade,"future_staff_grade":future_grade,
        "pending_stemless_boost":if stemless {STEMLESS_BOOST}else{0.},
        "weak_stemless":weak,"create_inter_eligible":create,"range_eligible":!weak&&create}),
    )
}
/// Original fixed-x best locations and subsequent weak-stemless/createInter gate.
/// The raw source score is retained; no aggregation, glyph or SIG identity exists.
pub fn candidates_at(
    table: &DistanceTable,
    bank: &Bank,
    pivot: [i32; 2],
    y_offsets: &[i32],
    black_relevant: bool,
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Value, String> {
    candidates_at_size(table, (bank, 51), pivot, y_offsets, black_relevant, veto)
}
/// Shared range kernel using an exact, caller-selected font/point-size catalog.
/// Page orchestration still checks its separately certified coverage.
pub fn candidates_at_size(
    table: &DistanceTable,
    catalog: (&Bank, u16),
    pivot: [i32; 2],
    y_offsets: &[i32],
    black_relevant: bool,
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Value, String> {
    let shapes = if black_relevant {
        ALL_SHAPES
    } else {
        HOLLOW_SHAPES
    };
    candidates_for_shapes(table, catalog, pivot, y_offsets, shapes, veto)
}
/// The planner supplies original enum-ordered all/hollow drum shapes. Empty
/// catalog is a no-op; unported shapes and duplicates fail before any veto call.
pub fn candidates_for_shapes(
    table: &DistanceTable,
    catalog: (&Bank, u16),
    pivot: [i32; 2],
    y_offsets: &[i32],
    shapes: &[&str],
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Value, String> {
    if shapes.len() > SUPPORTED_SHAPES.len()
        || shapes
            .iter()
            .enumerate()
            .any(|(i, shape)| !SUPPORTED_SHAPES.contains(shape) || shapes[..i].contains(shape))
    {
        return Err("unsupported_head_selected_shapes".into());
    }
    for &shape in shapes.iter().chain(
        shapes
            .contains(&"NOTEHEAD_BLACK")
            .then_some(&"NOTEHEAD_VOID")
            .iter()
            .copied(),
    ) {
        catalog
            .0
            .get(shape, catalog.1)
            .map_err(|e| e.to_string())?
            .anchor("MIDDLE_LEFT")
            .map_err(|e| e.to_string())?
            .rounded()
            .map_err(|e| e.to_string())?;
    }
    let raw = head_seed_search::search_shapes_at_size_with_veto(
        table,
        catalog,
        pivot,
        (&[0], y_offsets),
        "MIDDLE_LEFT",
        shapes,
        veto,
    )?;
    let mut matches = Vec::new();
    let mut rejected = Vec::new();
    for mut candidate in raw {
        let decision = candidate_decision_for_shape(
            candidate["shape"].as_str().unwrap(),
            candidate["score"].as_f64().unwrap(),
        )?;
        let accepted = decision["range_eligible"] == true;
        candidate["eligibility"] = decision;
        if accepted {
            matches.push(candidate);
        } else {
            rejected.push(candidate);
        }
    }
    Ok(json!({"matches":matches,"rejected":rejected}))
}
pub fn scan_catalog(
    input: Scan<'_>,
    ordinate: &mut dyn FnMut(i32) -> Result<i32, String>,
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Value, String> {
    scan_mode(input, ordinate, veto, true, 51)
}
pub fn scan_catalog_at_size(
    input: Scan<'_>,
    point_size: u16,
    ordinate: &mut dyn FnMut(i32) -> Result<i32, String>,
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
) -> Result<Value, String> {
    scan_mode(input, ordinate, veto, true, point_size)
}
fn scan_mode(
    input: Scan<'_>,
    ordinate: &mut dyn FnMut(i32) -> Result<i32, String>,
    veto: &mut dyn FnMut([i32; 4]) -> Result<bool, String>,
    catalog: bool,
    point_size: u16,
) -> Result<Value, String> {
    let Scan {
        table,
        bank,
        line,
        interline,
        dir,
        range,
        main_interline,
        y_offsets,
        spots,
    } = input;
    if !(1..=128).contains(&main_interline)
        || y_offsets.is_empty()
        || y_offsets.len() > 129
        || y_offsets[0] != 0
        || y_offsets.iter().any(|y| y.unsigned_abs() > 128)
        || range[0] < 0
    {
        return Err("invalid_head_range_scan".into());
    }
    for shape in if catalog {
        ALL_SHAPES
    } else {
        &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"]
    } {
        bank.get(shape, point_size).map_err(|e| e.to_string())?;
    }
    let (flags, selected) = relevant(line, interline, dir, range, spots)?;
    let half = (3 * main_interline / 2) as i32;
    let mut x = range[0];
    let mut visited = Vec::new();
    let mut skipped = Vec::new();
    let mut matches = Vec::new();
    let mut eval_work = 0usize;
    let mut rejected = Vec::new();
    while x <= range[1] {
        let y = ordinate(x)?;
        let probe_x = x + half;
        let reason = if probe_x as usize >= table.width() {
            Some("right_limit")
        } else if y < 0 || y as usize >= table.height() {
            Some("ordinate_limit")
        } else if table.get(probe_x as usize, y as usize).unwrap() / 3 > half {
            Some("no_foreground")
        } else {
            None
        };
        if let Some(reason) = reason {
            skipped.push(json!([x, y, reason]));
            x += 2 * half;
            continue;
        }
        let black = flags[(x - range[0]) as usize];
        visited.push(json!([x, y, black]));
        let shapes: &[&str] = if catalog {
            if black { ALL_SHAPES } else { HOLLOW_SHAPES }
        } else if black {
            &["NOTEHEAD_BLACK", "NOTEHEAD_VOID"]
        } else {
            &["NOTEHEAD_VOID"]
        };
        eval_work += shapes.len() * y_offsets.len();
        if eval_work > 1_000_000 {
            return Err("head_range_eval_budget".into());
        }
        if catalog {
            let mut out =
                candidates_at_size(table, (bank, point_size), [x, y], y_offsets, black, veto)?;
            matches.append(out["matches"].as_array_mut().unwrap());
            rejected.append(out["rejected"].as_array_mut().unwrap());
        } else {
            matches.extend(head_seed_search::search_shapes_at_size_with_veto(
                table,
                (bank, point_size),
                [x, y],
                (&[0], y_offsets),
                "MIDDLE_LEFT",
                shapes,
                veto,
            )?);
        }
        x += 1;
    }
    let mut report = json!({"scan_range":range,"template_half":half,"selected_spots":selected,
        "black_relevant_spans":spans(&flags,range[0]),"visited":visited,"skipped":skipped,
        "matches":matches,"eval_work_bound":eval_work});
    if catalog {
        report["rejected"] = json!(rejected);
        report["y_offsets"] = json!(y_offsets);
        report["all_shapes"] = json!(ALL_SHAPES);
        report["hollow_shapes"] = json!(HOLLOW_SHAPES);
    }
    Ok(report)
}

pub struct Input<'a> {
    pub binary: &'a GrayImage,
    pub head_spots: &'a GrayImage,
    pub grid: &'a Value,
    pub headers: &'a Value,
    pub seeds: &'a Value,
    pub ledgers: &'a Value,
    pub shared_sig: &'a Value,
    pub seed_veto: &'a Value,
    pub geometry: &'a beam_dispatch::Geometry,
    pub main_interline: usize,
    pub max_stem: usize,
}
pub fn analyze(input: Input<'_>) -> Result<Value, String> {
    analyze_mode(input, false)
}
pub fn analyze_catalog(input: Input<'_>) -> Result<Value, String> {
    analyze_mode(input, true)
}
fn analyze_mode(input: Input<'_>, catalog: bool) -> Result<Value, String> {
    let Input {
        binary,
        head_spots,
        grid,
        headers,
        seeds,
        ledgers,
        shared_sig,
        seed_veto,
        geometry,
        main_interline,
        max_stem,
    } = input;
    if seed_veto["seed_veto_complete"] != true
        || headers["headers_complete"] != true
        || binary.width != head_spots.width
        || binary.height != head_spots.height
    {
        return Err("incomplete_head_range_dependencies".into());
    }
    let declared = crate::head_staff_catalog::TemplateCatalog::from_report(seed_veto)?;
    let dispatch = dispatch(head_spots, geometry)?;
    let (table, mask) = head_seed_search::prepare(binary, grid, seeds, ledgers)?;
    let bank = if let Some(catalog) = &declared {
        catalog.bank()?
    } else {
        Bank::load_builtin().map_err(|e| e.to_string())?
    };
    let mut systems = Vec::new();
    let mut eval_work = 0usize;
    let mut rejected_count = 0usize;
    let (mut total, mut count, mut visited_count, mut skipped_count) = (0, 0, 0, 0);
    for sys in seed_veto["systems"]
        .as_array()
        .ok_or("invalid_head_range_systems")?
    {
        let sid = sys["system_id"]
            .as_u64()
            .ok_or("invalid_head_range_system")?;
        let spots = dispatch["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["system_id"] == sid)
            .ok_or("missing_head_range_spot_system")?["spots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| beam_dispatch::glyph_from_json(&s["glyph"]))
            .collect::<Result<Vec<_>, _>>()?;
        let inters: Vec<Value> = shared_sig["inters"]
            .as_array()
            .ok_or("invalid_head_range_sig")?
            .iter()
            .filter(|v| v["origin"]["system"].as_u64() == Some(sid))
            .cloned()
            .collect();
        let context = head_seed_veto::Context::from_inters(&inters, main_interline, max_stem)?;
        let staffs = crate::ledger_registration::staffs_from_grid(grid, sid)?;
        let mut rows = Vec::new();
        for f in sys["staffs"]
            .as_array()
            .ok_or("invalid_head_range_staffs")?
        {
            if declared.is_none() && f["point_size"] != 51 {
                return Err("unsupported_head_range_catalog".into());
            }
            let staff = staffs
                .iter()
                .find(|s| Some(s.id) == f["staff_id"].as_u64())
                .ok_or("missing_head_range_staff")?;
            let point_size = if let Some(catalog) = &declared {
                let size = catalog.staff_point_size(staff.specific_interline, main_interline)?;
                if f["point_size"].as_u64() != Some(u64::from(size)) {
                    return Err("inconsistent_head_seed_catalog".into());
                }
                size
            } else {
                51
            };
            let header = headers["staffs"]
                .as_array()
                .ok_or("invalid_head_range_headers")?
                .iter()
                .find(|h| h["staff_id"] == staff.id)
                .ok_or("missing_head_range_header")?;
            let stop = header["stop"]
                .as_i64()
                .filter(|v| v.unsigned_abs() <= 4_000_000)
                .ok_or("invalid_head_range_header_stop")? as i32;
            let lines = staff
                .lines
                .iter()
                .map(|p| Curve::interpolate(p.clone()).ok_or("invalid_head_range_curve"))
                .collect::<Result<Vec<_>, _>>()?;
            let mut row = json!({"staff_id":staff.id,"point_size":point_size,"scanners":[],"ledger_scanners":[]});
            if catalog {
                row["header_stop"] = json!(stop);
            }
            for key in ["scanners", "ledger_scanners"] {
                for c in f[key].as_array().ok_or("invalid_head_range_scanners")? {
                    count += 1;
                    if count > 4096 {
                        return Err("head_range_scanner_budget".into());
                    }
                    let dir = c["dir"]
                        .as_i64()
                        .filter(|v| (-1..=1).contains(v))
                        .ok_or("invalid_head_range_dir")? as i32;
                    let pitch = c["pitch"].as_i64().ok_or("invalid_head_range_pitch")? as i32;
                    let ledger: Option<[f64; 4]> = if key == "ledger_scanners" {
                        Some(
                            serde_json::from_value(c["main_line"].clone())
                                .map_err(|_| "invalid_head_range_ledger")?,
                        )
                    } else {
                        None
                    };
                    let index = c["line_index"].as_u64().unwrap_or(0) as usize;
                    let line = if let Some(l) = ledger {
                        Curve::interpolate(vec![[l[0], l[1]], [l[2], l[3]]])
                            .ok_or("invalid_head_range_ledger")?
                    } else {
                        lines
                            .get(index)
                            .ok_or("invalid_head_range_line_index")?
                            .clone()
                    };
                    let left = if ledger.is_some() {
                        line.points[0][0].ceil()
                    } else {
                        line.points[0][0].floor()
                    } as i32;
                    let right =
                        line.points.last().unwrap()[0].floor() as i32 - main_interline as i32;
                    let further: Vec<[f64; 4]> = if ledger.is_some() {
                        serde_json::from_value(c["further_lines"].clone())
                            .map_err(|_| "invalid_head_range_further")?
                    } else {
                        Vec::new()
                    };
                    let yo: Vec<i32> = serde_json::from_value(c["y_offsets"].clone())
                        .map_err(|_| "invalid_head_range_offsets")?;
                    let slice = context.slice(&line, staff.specific_interline, dir)?;
                    let mut report = scan_mode(
                        Scan {
                            table: &table,
                            bank: &bank,
                            line: &line,
                            interline: staff.specific_interline,
                            dir,
                            range: [left.max(stop), right],
                            main_interline,
                            y_offsets: &yo,
                            spots: &spots,
                        },
                        &mut |x| {
                            if let Some(l) = ledger {
                                head_ledger_search::ordinate(
                                    l,
                                    &further,
                                    pitch,
                                    staff.specific_interline,
                                    x,
                                )
                            } else {
                                Ok((if dir == -1 && index > 0 {
                                    (line.y(x as f64) + lines[index - 1].y(x as f64)) / 2.
                                } else {
                                    line.y(x as f64)
                                        + dir as f64 * staff.specific_interline as f64 / 2.
                                })
                                .round_ties_even() as i32)
                            }
                        },
                        &mut |b| slice.veto(b),
                        catalog,
                        point_size,
                    )?;
                    for k in ["line_index", "ledger_index", "ledger_order", "dir", "pitch"] {
                        if !c[k].is_null() {
                            report[k] = c[k].clone();
                        }
                    }
                    eval_work += report["eval_work_bound"].as_u64().unwrap() as usize;
                    if eval_work > 1_000_000 {
                        return Err("head_range_page_eval_budget".into());
                    }
                    if catalog {
                        rejected_count += report["rejected"].as_array().unwrap().len();
                    }
                    total += report["matches"].as_array().unwrap().len();
                    visited_count += report["visited"].as_array().unwrap().len();
                    skipped_count += report["skipped"].as_array().unwrap().len();
                    row[key].as_array_mut().unwrap().push(report);
                }
            }
            rows.push(row);
        }
        systems.push(json!({"system_id":sid,"staffs":rows}));
    }
    let mut report = json!({"range_black_void_prefix_complete":true,"range_search_complete":false,"heads_complete":false,
        "dispatch":dispatch,"distance_mask":mask,"scanner_count":count,"visited_abscissae":visited_count,
        "skipped_windows":skipped_count,"proposal_count":total,"systems":systems,
        "scope":"Bravura 51pt black/void MIDDLE_LEFT prefix before candidate aggregation and seed conflict filtering",
        "pending":["whole/breve and other catalog shapes","range candidate aggregation and seed conflicts",
            "glyph retrieval and HeadInter/SIG registration","duplicate/overlap purge and seed tally","stem attachment, rhythm and MusicXML"]});
    if catalog {
        report
            .as_object_mut()
            .unwrap()
            .remove("range_black_void_prefix_complete");
        report["range_catalog_candidates_complete"] = json!(true);
        report["raw_proposal_count"] = json!(total + rejected_count);
        report["rejected_candidate_count"] = json!(rejected_count);
        report["scope"] = json!(
            "Default oval Bravura 51pt catalog in original enum order; weak-stemless and createInter grade gates; before aggregation, seed conflict filtering, glyph retrieval or SIG insertion"
        );
        report["eligibility_parameters"] = json!({"stemless_boost":STEMLESS_BOOST,"min_contextual_grade":MIN_CONTEXTUAL_GRADE,"min_inter_grade":MIN_INTER_GRADE});
        report["pending"] = json!([
            "non-default small/cross/drum shapes and other catalogs",
            "range candidate aggregation and seed conflicts",
            "glyph retrieval and HeadInter/SIG registration",
            "duplicate/overlap purge and seed tally",
            "stem attachment, rhythm and MusicXML"
        ]);
    }
    if let Some(declared) = declared {
        report["template_catalog"] = json!(declared);
        report["template_catalog_required"] = json!(true);
        report["scope"] = json!(
            "Explicit exact oval family and per-staff point catalog through original range browsing and eligibility gates. Controlled HEADS catalog propagation; actual other-font/special-staff upstream coverage remains uncertified."
        );
    }
    Ok(report)
}
