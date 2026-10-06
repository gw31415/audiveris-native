// SPDX-License-Identifier: AGPL-3.0-or-later
//! NoteHeadsBuilder staff duplicate/overlap purge, small beams and seed tally.
use crate::shared_sig::{Graph, Key, Phase};
use audiveris_area::LinearArea;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
/// Original Shape.isSmallHead / ShapeSet.HeadsOvalSmall, including stemless heads.
pub(crate) fn is_small_head(shape: &str) -> bool {
    matches!(
        shape,
        "NOTEHEAD_BLACK_SMALL" | "NOTEHEAD_VOID_SMALL" | "WHOLE_NOTE_SMALL" | "BREVE_SMALL"
    )
}
/// Shape.getNoteDuration in quarter-note units, not a glyph's final rhythmic
/// duration (beams, flags, dots and tuplets are applied later).
pub(crate) fn intrinsic_duration_quarters(shape: &str) -> Option<u32> {
    match shape {
        "BREVE"
        | "BREVE_SMALL"
        | "BREVE_CROSS"
        | "BREVE_DIAMOND"
        | "BREVE_TRIANGLE_DOWN"
        | "BREVE_CIRCLE_X" => Some(8),
        "WHOLE_NOTE"
        | "WHOLE_NOTE_SMALL"
        | "WHOLE_NOTE_CROSS"
        | "WHOLE_NOTE_DIAMOND"
        | "WHOLE_NOTE_TRIANGLE_DOWN"
        | "WHOLE_NOTE_CIRCLE_X" => Some(4),
        "NOTEHEAD_VOID"
        | "NOTEHEAD_VOID_SMALL"
        | "NOTEHEAD_CROSS_VOID"
        | "NOTEHEAD_DIAMOND_VOID"
        | "NOTEHEAD_TRIANGLE_DOWN_VOID"
        | "NOTEHEAD_CIRCLE_X_VOID" => Some(2),
        "NOTEHEAD_BLACK"
        | "NOTEHEAD_BLACK_SMALL"
        | "NOTEHEAD_CROSS"
        | "NOTEHEAD_DIAMOND_FILLED"
        | "NOTEHEAD_TRIANGLE_DOWN_FILLED"
        | "NOTEHEAD_CIRCLE_X" => Some(1),
        _ => None,
    }
}
fn bounds(v: &Value) -> Result<[i32; 4], String> {
    let b: [i32; 4] = serde_json::from_value(v["bounds"].clone())
        .map_err(|_| "invalid_registered_head_bounds")?;
    if b.iter().any(|x| x.unsigned_abs() > 4_000_000) || b[2] <= 0 || b[3] <= 0 {
        return Err("invalid_registered_head_bounds".into());
    }
    Ok(b)
}
fn grade(v: &Value) -> Result<f64, String> {
    v["grade"]
        .as_f64()
        .filter(|v| (0.0..=1.0).contains(v))
        .ok_or("invalid_registered_head_grade".into())
}
fn validate(v: &Value) -> Result<(), String> {
    bounds(v)?;
    grade(v)?;
    if !v["shape"]
        .as_str()
        .is_some_and(|shape| crate::head_range_search::SUPPORTED_SHAPES.contains(&shape))
    {
        return Err("unsupported_registered_head_shape".into());
    }
    if !v["pitch"]
        .as_f64()
        .is_some_and(|p| p.is_finite() && p.abs() < 1e6)
        || v["staff_id"].as_u64().is_none()
    {
        return Err("invalid_registered_head_staff_pitch".into());
    }
    for k in ["left_dx", "right_dx"] {
        if !v[k].is_null()
            && !v[k]
                .as_f64()
                .is_some_and(|x| x.is_finite() && x.abs() < 1e6)
        {
            return Err("invalid_registered_head_dx".into());
        }
    }
    Ok(())
}
fn intersects(a: [i32; 4], b: [i32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
pub fn same(a: &Value, b: &Value) -> Result<bool, String> {
    validate(a)?;
    validate(b)?;
    Ok(a["shape"] == b["shape"]
        && a["bounds"] == b["bounds"]
        && (a["glyph"].is_null() || b["glyph"].is_null() || a["glyph"] == b["glyph"]))
}
pub fn overlaps(a: &Value, b: &Value) -> Result<bool, String> {
    validate(a)?;
    validate(b)?;
    let x = bounds(a)?;
    let y = bounds(b)?;
    if !intersects(x, y) {
        return Ok(false);
    }
    let pitch = (a["staff_id"] == b["staff_id"]).then(|| {
        (a["pitch"].as_f64().unwrap().round_ties_even()
            - b["pitch"].as_f64().unwrap().round_ties_even())
        .abs()
    });
    if pitch.is_some_and(|p| p > 1.) {
        return Ok(false);
    }
    let w = (x[0] + x[2]).min(y[0] + y[2]) - x[0].max(y[0]);
    let h = (x[1] + x[3]).min(y[1] + y[3]) - x[1].max(y[1]);
    let ratio = w as f64 / x[2].min(y[2]) as f64;
    if ratio <= 0.2 {
        return Ok(false);
    }
    if ratio >= 0.8 && pitch.is_some_and(|p| p <= 1.) {
        return Ok(true);
    }
    Ok(w as f64 * h as f64 / (x[2] as f64 * x[3] as f64).min(y[2] as f64 * y[3] as f64) > 0.25)
}
fn equal_purged(heads: &mut [Value], a: usize, b: usize) -> usize {
    if heads[a]["left_dx"].is_null() && heads[a]["right_dx"].is_null() {
        return a;
    }
    if heads[b]["left_dx"].is_null() && heads[b]["right_dx"].is_null() {
        return b;
    }
    if heads[a]["shape"] != heads[b]["shape"] || heads[a]["bounds"] != heads[b]["bounds"] {
        return b;
    }
    if heads[a]["grade"].as_f64().unwrap() >= 0.4 && heads[b]["grade"].as_f64().unwrap() >= 0.4 {
        for k in ["left_dx", "right_dx"] {
            if heads[a][k].is_null() {
                heads[a][k] = heads[b][k].clone();
            }
        }
    }
    b
}
/// Returns original pair count and canonical local-ID exclusions. Errors do not mutate input.
pub fn purge(heads: &mut [Value], do_remove: bool) -> Result<(usize, Vec<(usize, usize)>), String> {
    if heads.len() > 100_000 {
        return Err("head_purge_budget".into());
    }
    let mut next = heads.to_vec();
    let mut ids = BTreeSet::new();
    for h in &next {
        validate(h)?;
        let id = h["id"]
            .as_u64()
            .filter(|i| *i > 0)
            .ok_or("invalid_registered_head_id")?;
        if !ids.insert(id) {
            return Err("duplicate_registered_head_id".into());
        }
    }
    let mut order = (0..next.len()).collect::<Vec<_>>();
    order.sort_by_key(|&i| {
        let b = bounds(&next[i]).unwrap();
        (b[0], b[1], next[i]["id"].as_u64().unwrap())
    });
    let mut count = 0;
    let mut edges = Vec::new();
    let mut seen = BTreeSet::new();
    let mut work = 0;
    'outer: for (p, &a) in order.iter().enumerate() {
        if next[a]["removed"] == true {
            continue;
        }
        let box_a = bounds(&next[a])?;
        let max = box_a[0] + box_a[2] - 1;
        for &b in &order[p + 1..] {
            work += 1;
            if work > 1_000_000 {
                return Err("head_purge_pair_budget".into());
            }
            if next[b]["removed"] == true {
                continue;
            }
            let box_b = bounds(&next[b])?;
            if intersects(box_a, box_b) {
                if if do_remove {
                    same(&next[a], &next[b])?
                } else {
                    overlaps(&next[a], &next[b])?
                } {
                    let ga = grade(&next[a])?;
                    let gb = grade(&next[b])?;
                    let removed = if (ga - gb).abs() < 1e-5 {
                        equal_purged(&mut next, a, b)
                    } else if ga < gb {
                        a
                    } else {
                        b
                    };
                    count += 1;
                    if do_remove {
                        next[removed]["removed"] = json!(true);
                        if removed == a {
                            continue 'outer;
                        }
                    } else {
                        let ia = next[a]["id"].as_u64().unwrap() as usize;
                        let ib = next[b]["id"].as_u64().unwrap() as usize;
                        let edge = (ia.min(ib), ia.max(ib));
                        if seen.insert(edge) {
                            edges.push(edge);
                        }
                    }
                }
            } else if box_b[0] > max {
                break;
            }
        }
    }
    heads.clone_from_slice(&next);
    Ok((count, edges))
}
/// Original HeadSeedTally.purgeRemovedHeads after the staff duplicate/overlap passes.
pub fn purge_removed_tally(heads: &mut [Value]) {
    for h in heads {
        if h["removed"] == true {
            h["left_dx"] = Value::Null;
            h["right_dx"] = Value::Null;
        }
    }
}
/// HEADS epilog shape/side means; original quorum is inclusive ten.
pub fn seed_scale(heads: &[Value]) -> Result<Value, String> {
    let mut data: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();
    for h in heads {
        validate(h)?;
        if h["removed"] == true {
            continue;
        }
        for (side, key) in [("LEFT", "left_dx"), ("RIGHT", "right_dx")] {
            if let Some(x) = h[key].as_f64() {
                data.entry((h["shape"].as_str().unwrap().into(), side.into()))
                    .or_default()
                    .push(x);
            }
        }
    }
    let mut result = Vec::new();
    for shape in ["NOTEHEAD_VOID", "NOTEHEAD_BLACK"] {
        for side in ["LEFT", "RIGHT"] {
            let a = data.get(&(shape.into(), side.into()));
            let count = a.map_or(0, Vec::len);
            let dx = if count >= 10 {
                json!(a.unwrap().iter().sum::<f64>() / count as f64)
            } else {
                Value::Null
            };
            result.push(json!({"shape":shape,"side":side,"dx":dx,"count":count}));
        }
    }
    Ok(json!(result))
}
/// Small beams use their actual area and original contextual grade. Equal grade removes the head.
pub fn purge_small_beams(
    heads: &mut [Value],
    beams: &mut [Value],
    relations: &[Value],
    main_interline: usize,
) -> Result<(usize, usize), String> {
    if main_interline == 0 || main_interline > 1000 {
        return Err("invalid_heads_interline".into());
    }
    let mut hs = heads.to_vec();
    let mut bs = beams.to_vec();
    for h in &hs {
        validate(h)?;
    }
    let mut order = (0..hs.len())
        .filter(|&i| hs[i]["removed"] != true)
        .collect::<Vec<_>>();
    order.sort_by_key(|&i| bounds(&hs[i]).unwrap()[1]);
    let minimum = (2.5 * main_interline as f64).round_ties_even();
    let mut removed_heads = 0;
    let mut removed_beams = 0;
    let mut work = 0;
    for i in 0..bs.len() {
        if bs[i]["removed"] == true {
            continue;
        }
        let b = bounds(&bs[i])?;
        if b[2] as f64 >= minimum {
            continue;
        }
        let live = bs
            .iter()
            .filter(|b| b["removed"] != true)
            .cloned()
            .collect::<Vec<_>>();
        let ids = live
            .iter()
            .filter_map(|b| b["id"].as_u64())
            .collect::<BTreeSet<_>>();
        let edges = relations
            .iter()
            .filter(|r| {
                r["source"].as_u64().is_some_and(|id| ids.contains(&id))
                    && r["target"].as_u64().is_some_and(|id| ids.contains(&id))
            })
            .cloned()
            .collect::<Vec<_>>();
        let cg = crate::head_beam_context::contextual_grade(
            bs[i]["id"].as_u64().ok_or("invalid_head_beam_id")?,
            &live,
            &edges,
        )?;
        bs[i]["contextual_grade"] = json!(cg);
        let m: [f64; 4] = serde_json::from_value(bs[i]["median"].clone())
            .map_err(|_| "invalid_head_beam_median")?;
        let half = bs[i]["height"]
            .as_f64()
            .filter(|x| x.is_finite() && *x > 0.)
            .ok_or("invalid_head_beam_height")?
            / 2.;
        let area = LinearArea::polygon(&[
            [m[0], m[1] - half],
            [m[2], m[3] - half],
            [m[2], m[3] + half],
            [m[0], m[1] + half],
        ])?;
        for &j in &order {
            if hs[j]["removed"] == true {
                continue;
            }
            work += 1;
            if work > 1_000_000 {
                return Err("head_small_beam_pair_budget".into());
            }
            let hb = bounds(&hs[j])?;
            if area.intersects_rect(hb.map(|x| x as f64))? {
                if grade(&hs[j])? > cg {
                    bs[i]["removed"] = json!(true);
                    removed_beams += 1;
                    break;
                }
                hs[j]["removed"] = json!(true);
                removed_heads += 1;
            } else if hb[1] > b[1] + b[3] - 1 {
                break;
            }
        }
    }
    heads.clone_from_slice(&hs);
    beams.clone_from_slice(&bs);
    Ok((removed_heads, removed_beams))
}
pub fn analyze(glyphs: &Value, graph: &mut Graph, main_interline: usize) -> Result<Value, String> {
    if glyphs["head_glyph_retrieval_complete"] != true
        || glyphs["live_seed_conflicts_complete"] != true
    {
        return Err("incomplete_head_registration_dependency".into());
    }
    let declared = crate::head_staff_catalog::TemplateCatalog::from_report(glyphs)?;
    let systems = glyphs["systems"]
        .as_array()
        .ok_or("invalid_head_registration_systems")?;
    let mut next = graph.clone();
    let mut records = Vec::new();
    let mut all = Vec::new();
    let mut systems_seen = BTreeSet::new();
    let mut total_duplicates = 0;
    let mut total_overlaps = 0;
    let mut small_heads = 0;
    let mut small_beams = 0;
    for s in systems {
        let system = s["system_id"]
            .as_u64()
            .filter(|x| *x > 0)
            .ok_or("invalid_head_registration_system")?;
        if !systems_seen.insert(system) {
            return Err("duplicate_head_registration_system".into());
        }
        let mut heads = Vec::new();
        let mut edges = Vec::new();
        let mut staff_order = Vec::new();
        let mut seen = BTreeSet::new();
        for staff in s["staffs"]
            .as_array()
            .ok_or("invalid_head_registration_staffs")?
        {
            let id = staff["staff_id"]
                .as_u64()
                .ok_or("invalid_head_registration_staff")?;
            if !seen.insert(id) {
                return Err("duplicate_head_registration_staff".into());
            }
            let start = heads.len();
            let mut candidates = staff["seed_heads"]
                .as_array()
                .ok_or("invalid_head_registration_seed_heads")?
                .clone();
            for key in ["scanners", "ledger_scanners"] {
                for sc in staff[key]
                    .as_array()
                    .ok_or("invalid_head_registration_scanners")?
                {
                    for c in sc["range_glyphs"]
                        .as_array()
                        .ok_or("invalid_head_registration_range_glyphs")?
                    {
                        if !c["result"].is_null() {
                            candidates.push(c["result"].clone());
                        }
                    }
                }
            }
            for mut h in candidates {
                validate(&h)?;
                if h["staff_id"] != id {
                    return Err("head_registration_staff_mismatch".into());
                }
                h["id"] = json!(heads.len() + 1);
                // HeadInter.added marks only ShapeSet.StemHeads abnormal before
                // any HeadStemRelation exists. Whole/breve remain normal.
                h["abnormal"] = json!(
                    crate::head_seed_search::ALL_STEM_SHAPES
                        .contains(&h["shape"].as_str().unwrap())
                );
                h["removed"] = json!(false);
                heads.push(h);
            }
            let slice = &mut heads[start..];
            total_duplicates += purge(slice, true)?.0;
            let (count, ex) = purge(slice, false)?;
            total_overlaps += count;
            edges.extend(ex);
            purge_removed_tally(slice);
            let mut notes = slice
                .iter()
                .filter(|h| h["removed"] != true)
                .collect::<Vec<_>>();
            // AbstractNoteInter.added inserts into Staff.notes at SIG registration.
            // The later sorted addNote loop does not reorder the LinkedHashSet.
            notes.sort_by_key(|h| h["id"].as_u64().unwrap());
            staff_order.push((
                id,
                notes
                    .iter()
                    .map(|h| h["id"].as_u64().unwrap() as usize)
                    .collect::<Vec<_>>(),
            ));
        }
        let base = next.snapshot();
        let mut beams = base["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["origin"]["system"] == system && b["origin"]["phase"] == "BEAM")
            .cloned()
            .collect::<Vec<_>>();
        let beam_ids = beams
            .iter()
            .filter_map(|b| b["id"].as_u64())
            .collect::<BTreeSet<_>>();
        let relations = base["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["source"]
                    .as_u64()
                    .is_some_and(|id| beam_ids.contains(&id))
                    && r["target"]
                        .as_u64()
                        .is_some_and(|id| beam_ids.contains(&id))
            })
            .cloned()
            .collect::<Vec<_>>();
        let (rh, rb) = purge_small_beams(&mut heads, &mut beams, &relations, main_interline)?;
        small_heads += rh;
        small_beams += rb;
        let mut edge_rows = Vec::new();
        for (a, b) in edges {
            if heads[a - 1]["removed"] != true && heads[b - 1]["removed"] != true {
                edge_rows.push(json!({"kind":"Exclusion","cause":"OVERLAP","source":a,"target":b}));
            }
        }
        next.sync_heads(system, &heads, &edge_rows, &beams)?;
        let id = |local| {
            next.id(Key {
                system,
                phase: Phase::Head,
                local,
            })
            .unwrap()
        };
        let live_heads = heads
            .iter()
            .filter(|h| h["removed"] != true)
            .map(|h| {
                let mut h = h.clone();
                h["id"] = json!(id(h["id"].as_u64().unwrap() as usize));
                h
            })
            .collect::<Vec<_>>();
        let notes=staff_order.into_iter().map(|(staff,ns)|json!({"staff_id":staff,"notes":ns.into_iter().filter(|&local|heads[local-1]["removed"]!=true).map(id).collect::<Vec<_>>()})).collect::<Vec<_>>();
        let mut exclusions = edge_rows;
        for e in &mut exclusions {
            e["source"] = json!(id(e["source"].as_u64().unwrap() as usize));
            e["target"] = json!(id(e["target"].as_u64().unwrap() as usize));
        }
        all.extend(heads);
        records.push(json!({"system_id":system,"heads":live_heads,"staffs":notes,"exclusions":exclusions,"small_beams":beams.iter().filter(|b|b.get("contextual_grade").is_some()).collect::<Vec<_>>()}));
    }
    let scale = seed_scale(&all)?;
    let count = all.iter().filter(|h| h["removed"] != true).count();
    let mut report = json!({"stage":"HEADS.REGISTRATION_AND_EPILOG","head_registration_complete":true,"heads_complete":false,"musicxml_complete":false,"systems":records,"retained_heads":count,"duplicates_removed":total_duplicates,"overlap_pairs":total_overlaps,"small_beam_removed_heads":small_heads,"small_beams_removed":small_beams,"head_seed_scale":scale,"shared_sig":next.snapshot(),"epilog_seed_scale_complete":true,"head_spots_persistence":"No native persistent HEAD_SPOTS image; transient binary is released after range search.","scope":"Original staff purge and note order, beam context/area purge and seed-tally epilog. Shared graph IDs remain stable across prior phases; original JVM IDs are oracle-only. Supported Bravura 51pt, ordinary five-line staff/ledger dependency pipeline; broader HEADS coverage is not certified."});
    *graph = next;
    if let Some(declared) = declared {
        report["template_catalog"] = json!(declared);
        report["template_catalog_required"] = json!(true);
        report["scope"] = json!(
            "Explicit exact oval family/staff point catalog through original registration, duplicates/overlaps, shared SIG/staff notes, small beams and seed tally. Controlled HEADS-only catalog changes on verified upstream states; actual other-font/special-staff page coverage remains uncertified."
        );
    }
    Ok(report)
}
