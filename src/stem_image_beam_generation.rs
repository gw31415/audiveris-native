// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image-owned beam seeds, side stumps, BLinkers and head-origin anchors.
//! Inspection only: final stems, owned SIG and epilog are still pending.
use crate::{GrayImage, beam_dispatch, stem_glyph, stem_image_beam_targets, stem_image_stumps};
use audiveris_area::LinearArea;
use audiveris_staff_removal::GlyphPixels;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Deserialize, Serialize)]
struct Beam {
    id: usize,
    median: [f64; 4],
    top: [f64; 4],
    bottom: [f64; 4],
    height: f64,
    group: usize,
    #[serde(default)]
    glyph_id: usize,
}
#[derive(Clone, Serialize, Deserialize)]
struct Link {
    side: Option<String>,
    reference: [f64; 2],
    stump: Value,
    anchor: bool,
    #[serde(default)]
    directions: Vec<String>,
}
fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|a| a.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid beam generation {label}"))
}
fn inter(a: [f64; 4], b: [f64; 4]) -> Result<[f64; 2], String> {
    let [x1, y1, x2, y2] = a;
    let [x3, y3, x4, y4] = b;
    let d = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let p = x1 * y2 - y1 * x2;
    let q = x3 * y4 - y3 * x4;
    let v = [
        (p * (x3 - x4) - (x1 - x2) * q) / d,
        (p * (y3 - y4) - (y1 - y2) * q) / d,
    ];
    if v.iter().any(|x| !x.is_finite() || x.abs() > 4_000_000.) {
        return Err("unsupported beam generation parallel/nonfinite line".into());
    }
    Ok(v)
}
fn at_x(l: [f64; 4], x: f64) -> f64 {
    l[1] + (x - l[0]) * (l[3] - l[1]) / (l[2] - l[0])
}
fn bounds(g: &GlyphPixels) -> [f64; 4] {
    [g.left as f64, g.top as f64, g.width as f64, g.height as f64]
}
fn polygon(l: [f64; 4], h: f64) -> [[f64; 2]; 4] {
    [
        [l[0], l[1] - h / 2.],
        [l[2], l[3] - h / 2.],
        [l[2], l[3] + h / 2.],
        [l[0], l[1] + h / 2.],
    ]
}
fn seg_distance(l: [f64; 4], p: [f64; 2]) -> f64 {
    let dx = l[2] - l[0];
    let dy = l[3] - l[1];
    let px = p[0] - l[0];
    let py = p[1] - l[1];
    let dot = px * dx + py * dy;
    let t = (dot / (dx * dx + dy * dy)).clamp(0., 1.);
    let x = px - t * dx;
    let y = py - t * dy;
    x * x + y * y
}
fn purge(gs: &mut Vec<GlyphPixels>, median: [f64; 4], dx: i32) -> Result<(), String> {
    if gs.len() > 4096 || !(0..=1024).contains(&dx) {
        return Err("invalid beam purge budget".into());
    }
    let mut i = 0;
    while i < gs.len() {
        let a = stem_glyph::geometry(&gs[i])?.median;
        let pa = inter(a, median)?;
        let mut j = i + 1;
        let mut removed_i = false;
        while j < gs.len() {
            let b = stem_glyph::geometry(&gs[j])?.median;
            let pb = inter(b, median)?;
            if pb[0] - pa[0] >= f64::from(dx) {
                break;
            }
            let overlap = (gs[i].top + gs[i].height as i32).min(gs[j].top + gs[j].height as i32)
                - gs[i].top.max(gs[j].top);
            let keep_a = if overlap > 0 {
                gs[i].height >= gs[j].height
            } else {
                seg_distance(a, pa) <= seg_distance(b, pb)
            };
            if keep_a {
                gs.remove(j);
                j += 1;
            } else {
                gs.remove(i);
                removed_i = true;
                break;
            }
        }
        // Original increments j after removal, skipping the shifted entry. If i is removed it revisits the shifted i.
        if !removed_i {
            i += 1;
        }
    }
    Ok(())
}
fn siblings<'a>(
    b: &Beam,
    all: &'a [Beam],
    members: &[usize],
    p: [f64; 2],
    dx: i32,
) -> Result<Vec<&'a Beam>, String> {
    let mut v = Vec::new();
    for id in members {
        let m = all
            .iter()
            .find(|b| b.id == *id)
            .ok_or("unknown beam member")?;
        if m.group != b.group {
            return Err("inconsistent beam generation group".into());
        }
        let q = inter([p[0], p[1], p[0], p[1] + 1000.], m.median)?;
        if m.median[0] - dx as f64 <= q[0] && q[0] <= m.median[2] + dx as f64 {
            v.push((m, q[1]));
        }
    }
    v.sort_by(|a, b| a.1.total_cmp(&b.1));
    Ok(v.into_iter().map(|v| v.0).collect())
}
fn directions(
    b: &Beam,
    all: &[Beam],
    members: &[usize],
    g: &GlyphPixels,
    side_dx: i32,
    min_dy: i32,
) -> Result<Vec<String>, String> {
    let p = [
        g.left as f64 + g.width as f64 / 2.,
        g.top as f64 + g.height as f64 / 2.,
    ];
    let bs = siblings(b, all, members, p, side_dx)?;
    let Some(first) = bs.first() else {
        return Ok(Vec::new());
    };
    let last = bs.last().expect("nonempty");
    if b.id != first.id
        && b.id != last.id
        && b.glyph_id != first.glyph_id
        && b.glyph_id != last.glyph_id
    {
        return Ok(Vec::new());
    }
    let l = stem_glyph::geometry(g)?.median;
    let mut d = Vec::new();
    if (at_x(first.top, p[0]) - l[1]).max(0.) >= min_dy as f64 {
        d.push("TOP".into());
    }
    if (l[3] - at_x(last.bottom, p[0])).max(0.) >= min_dy as f64 {
        d.push("BOTTOM".into());
    }
    Ok(d)
}
fn portion(b: &Beam, x: f64, margin: i32) -> Option<&'static str> {
    if x < b.median[0] + margin as f64 {
        Some("LEFT")
    } else if x > b.median[2] - margin as f64 {
        Some("RIGHT")
    } else {
        None
    }
}
fn side_glyph(
    sections: &[Value],
    b: &Beam,
    side: usize,
    thickness: i32,
) -> Result<Option<GlyphPixels>, String> {
    let endpoint = if side == 0 { b.median[0] } else { b.median[2] };
    let other = endpoint
        + if side == 0 {
            thickness as f64
        } else {
            -f64::from(thickness)
        };
    let l = if side == 0 {
        [endpoint, b.median[1], other, at_x(b.median, other)]
    } else {
        [other, at_x(b.median, other), endpoint, b.median[3]]
    };
    let area = LinearArea::polygon(&polygon(l, b.height))?;
    let desired = endpoint
        + if side == 0 {
            thickness as f64 / 2.
        } else {
            -f64::from(thickness) / 2.
        };
    let mut selected = Vec::new();
    for s in sections {
        let first = s["first_pos"]
            .as_i64()
            .ok_or("invalid beam section position")?;
        let rs = rows(&s["runs"], 8192, "section runs")?;
        let bb: [i32; 4] = serde_json::from_value(s["bounds"].clone())
            .map_err(|_| "invalid beam section bounds")?;
        let mut hit = false;
        for (i, r) in rs.iter().enumerate() {
            let pair: [i32; 2] =
                serde_json::from_value(r.clone()).map_err(|_| "invalid beam section run")?;
            if pair[0] < 0 || pair[1] <= 0 || first < 0 || first + i as i64 > 4_000_000 {
                return Err("invalid beam section bounds".into());
            }
            if area.intersects_rect([
                first as f64 + i as f64,
                pair[0] as f64,
                1.,
                pair[1] as f64,
            ])? {
                hit = true;
            }
        }
        if hit {
            let runs: Vec<_> = rs
                .iter()
                .enumerate()
                .map(|(i, r)| json!([i, r[0].as_i64().unwrap() - bb[1] as i64, r[1]]))
                .collect();
            let g = beam_dispatch::glyph_from_json(
                &json!({"bounds":bb,"orientation":"VERTICAL","runs":runs}),
            )?;
            selected.push((g, ((bb[0] + bb[2] / 2) as f64 - desired).abs()));
        }
    }
    selected.sort_by(|a, b| a.1.total_cmp(&b.1));
    // Equal-distance section HashSet ordering may affect where the width stop occurs; refuse uncertified productive ties.
    for w in selected.windows(2) {
        if w[0].1 == w[1].1 {
            return Err("unsupported beam side section distance tie".into());
        }
    }
    let mut kept = Vec::new();
    let mut left = i32::MAX;
    let mut right = i32::MIN;
    for (g, _) in selected {
        let nl = left.min(g.left);
        let nr = right.max(g.left + g.width as i32);
        if nr - nl > thickness {
            break;
        }
        left = nl;
        right = nr;
        kept.push(g);
    }
    if kept.is_empty() {
        Ok(None)
    } else {
        Ok(Some(stem_glyph::assemble(&kept)?))
    }
}
fn nearest(
    ls: &mut Vec<Link>,
    median: [f64; 4],
    line: [f64; 4],
    max_dx: i32,
) -> Result<usize, String> {
    if ls.len() > 4096 || !(0..=1024).contains(&max_dx) {
        return Err("invalid beam linker budget".into());
    }
    let p = inter(line, median)?;
    let mut best = None;
    let mut dx = f64::MAX;
    for (i, l) in ls.iter().enumerate() {
        let d = (l.reference[0] - p[0]).abs();
        if d < dx {
            dx = d;
            best = Some(i);
        }
    }
    if dx <= max_dx as f64 {
        return best.ok_or("empty nearest beam linker".into());
    }
    ls.push(Link {
        side: None,
        reference: p,
        stump: Value::Null,
        anchor: true,
        directions: Vec::new(),
    });
    Ok(ls.len() - 1)
}
/// Produces beam catalog from image GRID/HEADS and the same owned physical registry.
pub fn prepare(image: &GrayImage, page: &Value) -> Result<stem_image_stumps::Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<crate::beam_dispatch::Registry>,
) -> Result<stem_image_stumps::Prepared, String> {
    let mut owned = stem_image_beam_targets::prepare_with_registry(image, page, registry)?;
    let plan = owned.report.clone();
    let source = &plan["head_linker_plan"]["source_plan"];
    let seeds = &source["stump_plan"]["seed_plan"];
    let head = &seeds["head_plan"];
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let il = scale.interline.main as f64;
    let dx = (il * 0.1).round_ties_even();
    let side_dx = (il * 0.25).round_ties_even() as i32;
    let min_dx = il.round_ties_even() as i32;
    let min_dy = (il * 0.5).round_ties_even() as i32;
    let margin = min_dy;
    let main = page["stem_seeds"]["stem_scaling"]["stem_scale"]["main"]
        .as_i64()
        .filter(|x| (1..=128).contains(x))
        .ok_or("invalid beam stem main thickness")? as i32;
    let max = page["stem_seeds"]["stem_scaling"]["stem_scale"]["max"]
        .as_i64()
        .filter(|x| (*x >= main as i64) && *x <= 128)
        .ok_or("invalid beam stem max thickness")? as i32;
    let layout = crate::system_area_geometry::build(
        &page["grid_ir"],
        image.width,
        image.height,
        scale.interline.main,
    )?;
    let grid = rows(&page["grid_ir"]["systems"], 64, "GRID systems")?;
    let lag = rows(&page["grid_ir"]["vlag"], 100_000, "GRID sections")?;
    let mut section_map = BTreeMap::new();
    for sec in lag {
        let id = sec["id"].as_u64().ok_or("invalid beam section ID")?;
        if section_map.insert(id, sec.clone()).is_some() {
            return Err("duplicate beam section ID".into());
        }
    }
    let mut result = Vec::new();
    let mut stump_total = 0;
    let mut base_total = 0;
    let mut final_total = 0;
    let mut selected_total = 0;
    for (i, h) in rows(&head["systems"], 64, "head systems")?
        .iter()
        .enumerate()
    {
        let context = &h["image_context"];
        if context["profile"] != 1 {
            return Err("unsupported beam generation profile".into());
        }
        let mut beams: Vec<Beam> = serde_json::from_value(context["beams"].clone())
            .map_err(|_| "invalid beam generation catalog")?;
        let groups = rows(&context["groups"], 2048, "groups")?;
        let bb = rows(&context["beam_bounds"], 2048, "bounds")?;
        let system = grid
            .iter()
            .find(|s| s["id"] == h["system"])
            .ok_or("missing beam GRID system")?;
        let system_bounds = layout
            .system_areas
            .get(i)
            .ok_or("missing beam system area")?
            .bounds()?;
        let mut sections = Vec::new();
        for id in rows(&system["vertical_section_ids"], 100_000, "system sections")? {
            let id = id.as_u64().ok_or("invalid beam section membership")?;
            sections.push(section_map.get(&id).ok_or("missing beam section")?.clone());
        }
        let live = rows(
            &page["head_registration"]["shared_sig"]["inters"],
            100_000,
            "live SIG",
        )?;
        let mut ids = BTreeSet::new();
        for b in &mut beams {
            if !ids.insert(b.id)
                || b.median
                    .iter()
                    .chain(b.top.iter())
                    .chain(b.bottom.iter())
                    .any(|x| !x.is_finite())
                || b.median[2] <= b.median[0]
                || !b.height.is_finite()
                || b.height <= 0.
            {
                return Err("invalid beam generation geometry/ID".into());
            }
            let mut matches = live.iter().filter(|v| {
                v["origin"]["phase"] == "BEAM"
                    && v["origin"]["system"] == h["system"]
                    && v["origin"]["local"] == b.id
            });
            let v = matches.next().ok_or("missing live beam origin")?;
            if matches.next().is_some() {
                return Err("duplicate live beam origin".into());
            }
            b.glyph_id = v["glyph_id"]
                .as_u64()
                .filter(|x| *x > 0)
                .ok_or("invalid live beam glyph ID")? as usize;
            owned.registry.get(b.glyph_id)?;
        }
        beams.sort_by_key(|b| {
            bb.iter()
                .find(|v| v["id"] == b.id)
                .and_then(|v| v["bounds"][0].as_i64())
                .unwrap_or(i64::MAX)
        });
        let mut catalogs = Vec::new();
        let mut links: Vec<Vec<Link>> = Vec::new();
        for b in &beams {
            let bounds_v = bb
                .iter()
                .find(|v| v["id"] == b.id)
                .ok_or("missing beam bounds")?;
            let bounds: [i32; 4] = serde_json::from_value(bounds_v["bounds"].clone())
                .map_err(|_| "invalid beam bounds")?;
            let neighbors = [
                (bounds[0] - il as i32) as f64,
                system_bounds[1] as f64,
                (bounds[2] + 2 * il as i32) as f64,
                system_bounds[3] as f64,
            ];
            let slope = (b.median[3] - b.median[1]) / (b.median[2] - b.median[0]);
            let extended = [
                b.median[0] - dx,
                b.median[1] - slope * dx,
                b.median[2] + dx,
                b.median[3] + slope * dx,
            ];
            let dy = 0.25 * (il * 1.2).round_ties_even();
            let poly = polygon(extended, b.height + 2. * dy);
            let area = LinearArea::polygon(&poly)?;
            let mut admitted = Vec::new();
            for n in rows(
                &seeds["systems"][i]["surviving_seed_ids"],
                4096,
                "surviving seeds",
            )? {
                let g = &owned
                    .registry
                    .get(n.as_u64().ok_or("invalid beam seed ID")? as usize)?
                    .glyph;
                let r = bounds_g(g);
                if rect_intersects(neighbors, r) && area.intersects_rect(r)? {
                    admitted.push(g.clone());
                }
            }
            let mut sorted = Vec::new();
            for g in admitted {
                let hit = inter(stem_glyph::geometry(&g)?.median, b.median)?;
                sorted.push((g, hit[0]));
            }
            sorted.sort_by(|a, b| a.1.total_cmp(&b.1));
            if sorted.windows(2).any(|v| v[0].1 == v[1].1) {
                return Err("unsupported beam equal-abscissa seed tie".into());
            }
            let mut stumps: Vec<_> = sorted.into_iter().map(|v| v.0).collect();
            let admitted_json = stumps
                .iter()
                .map(beam_dispatch::glyph_json)
                .collect::<Vec<_>>();
            purge(&mut stumps, b.median, min_dx)?;
            let purged_json = stumps
                .iter()
                .map(beam_dispatch::glyph_json)
                .collect::<Vec<_>>();
            let group = groups
                .iter()
                .find(|g| g["id"] == b.group)
                .ok_or("missing beam group")?;
            let members: Vec<usize> = serde_json::from_value(group["members"].clone())
                .map_err(|_| "invalid beam group members")?;
            if members.len() > 2048
                || members.iter().copied().collect::<BTreeSet<_>>().len() != members.len()
            {
                return Err("invalid beam group membership".into());
            }
            let mut sides: [Option<Value>; 2] = [None, None];
            if !stumps.is_empty() {
                for (side, slot) in sides.iter_mut().enumerate() {
                    let g = if side == 0 {
                        &stumps[0]
                    } else {
                        stumps.last().expect("nonempty")
                    };
                    let hit = inter(stem_glyph::geometry(g)?.median, b.median)?;
                    if portion(b, hit[0], margin) == Some(if side == 0 { "LEFT" } else { "RIGHT" })
                    {
                        *slot = Some(beam_dispatch::glyph_json(g));
                    }
                }
            }
            let mut side_attempts = Vec::new();
            for (side, slot) in sides.iter_mut().enumerate() {
                if slot.is_none() {
                    let raw = side_glyph(&sections, b, side, max)?;
                    let d = if let Some(g) = &raw {
                        directions(b, &beams, &members, g, side_dx, min_dy)?
                    } else {
                        Vec::new()
                    };
                    side_attempts.push(json!({"side":if side==0{"LEFT"}else{"RIGHT"},"raw":raw.as_ref().map(beam_dispatch::glyph_json),"directions":d}));
                    if !d.is_empty() {
                        let g = raw.expect("directions require glyph");
                        let before = owned.registry.len();
                        owned.register_stump(&g)?;
                        if owned.registry.len() != before {
                            return Err("unsupported new beam side glyph: beam-before-head registration order required".into());
                        }
                        *slot = Some(beam_dispatch::glyph_json(&g));
                        if side == 0 {
                            stumps.insert(0, g);
                        } else {
                            stumps.push(g);
                        }
                    }
                }
            }
            let mut ls = Vec::new();
            for g in &stumps {
                let physical = beam_dispatch::glyph_json(g);
                let side = (0..2)
                    .find(|x| sides[*x].as_ref() == Some(&physical))
                    .map(|x| {
                        if x == 0 {
                            "LEFT".into()
                        } else {
                            "RIGHT".into()
                        }
                    });
                ls.push(Link {
                    side,
                    reference: inter(stem_glyph::geometry(g)?.median, b.median)?,
                    stump: physical,
                    anchor: false,
                    directions: directions(b, &beams, &members, g, side_dx, min_dy)?,
                });
            }
            for (side, stump) in sides.iter().enumerate() {
                if stump.is_none() {
                    let x = if side == 0 { b.median[0] } else { b.median[2] };
                    let p = [x, at_x(b.median, x)];
                    let sib = siblings(b, &beams, &members, p, side_dx)?;
                    if sib.first().is_some_and(|m| m.glyph_id == b.glyph_id)
                        || sib.last().is_some_and(|m| m.glyph_id == b.glyph_id)
                    {
                        let x = x + if side == 0 {
                            main as f64 / 2.
                        } else {
                            -f64::from(main) / 2.
                        };
                        ls.push(Link {
                            side: Some(if side == 0 {
                                "LEFT".into()
                            } else {
                                "RIGHT".into()
                            }),
                            reference: inter(b.median, [x, 0., x, 1.])?,
                            stump: Value::Null,
                            anchor: false,
                            directions: vec!["TOP".into(), "BOTTOM".into()],
                        });
                    }
                }
            }
            stump_total += stumps.len();
            base_total += ls.len();
            catalogs.push(json!({"beam":b,"admitted_seeds":admitted_json,"purged_seeds":purged_json,"seed_area_polygon":poly,"stumps":stumps.iter().map(beam_dispatch::glyph_json).collect::<Vec<_>>(),"base_blinkers":ls,"side_attempts":side_attempts}));
            links.push(ls);
        }
        // Original inspectVLinkers runs before head inspection. Each non-anchor VLinker
        // calls filterBeams on its sibling group, excluding its source and aliases
        // of the same beam glyph. Both directions use the same zero-skew x.
        let base_links = links.clone();
        let mut beam_origin = Vec::new();
        for (bi, base) in base_links.iter().enumerate() {
            let b = &beams[bi];
            let g = groups
                .iter()
                .find(|g| g["id"] == b.group)
                .ok_or("missing beam inspection group")?;
            let members: Vec<usize> = serde_json::from_value(g["members"].clone())
                .map_err(|_| "invalid beam inspection members")?;
            for (li, l) in base.iter().enumerate() {
                for direction in &l.directions {
                    let line = [
                        l.reference[0],
                        l.reference[1],
                        l.reference[0],
                        l.reference[1] + 1000.,
                    ];
                    let siblings = siblings(b, &beams, &members, l.reference, side_dx)?;
                    let mut selected = Vec::new();
                    for target in siblings {
                        if target.id == b.id || target.glyph_id == b.glyph_id {
                            continue;
                        }
                        let ti = beams
                            .iter()
                            .position(|v| v.id == target.id)
                            .expect("owned beam");
                        let index = nearest(&mut links[ti], target.median, line, side_dx)?;
                        selected.push(json!({"beam_id":target.id,"linker_id":index+1,"linker":links[ti][index]}));
                    }
                    beam_origin.push(json!({"beam_id":b.id,"linker_id":li+1,"direction":direction,"selected_sibling_blinkers":selected}));
                }
            }
        }
        // Original HeadCorner.values: TOP_RIGHT, BOTTOM_LEFT, TOP_LEFT, BOTTOM_RIGHT, after stable head bounds.x order.
        let cases = rows(&plan["systems"][i]["cases"], 16384, "member cases")?;
        let source_cases = rows(&source["systems"][i]["cases"], 16384, "source cases")?;
        let mut order: Vec<usize> = (0..cases.len()).collect();
        let rank = |c: &Value| match (c["head"]["side"].as_str(), c["head"]["vertical"].as_str()) {
            (Some("RIGHT"), Some("TOP")) => 0,
            (Some("LEFT"), Some("BOTTOM")) => 1,
            (Some("LEFT"), Some("TOP")) => 2,
            _ => 3,
        };
        order.sort_by_key(|k| {
            (
                cases[*k]["head"]["head_bounds"][0]
                    .as_i64()
                    .unwrap_or(i64::MAX),
                cases[*k]["head"]["head_id"].as_u64().unwrap_or(0),
                rank(&cases[*k]),
            )
        });
        let mut selections = vec![Value::Null; cases.len()];
        for k in order {
            let line: [f64; 4] =
                serde_json::from_value(source_cases[k]["theoretical_line"].clone())
                    .map_err(|_| "invalid beam source line")?;
            let mut selected = Vec::new();
            for id in rows(
                &cases[k]["selected_sibling_member_ids"],
                2048,
                "selected members",
            )? {
                let id = id.as_u64().ok_or("invalid selected beam ID")? as usize;
                let bi = beams
                    .iter()
                    .position(|b| b.id == id)
                    .ok_or("unknown selected beam")?;
                let li = nearest(&mut links[bi], beams[bi].median, line, side_dx)?;
                selected.push(json!({"beam_id":id,"linker_id":li+1,"linker":links[bi][li]}));
            }
            selected_total += selected.len();
            selections[k] = json!({"head":cases[k]["head"],"selected_blinkers":selected});
        }
        for (c, l) in catalogs.iter_mut().zip(links) {
            final_total += l.len();
            c["final_blinkers"] = json!(l);
        }
        result.push(json!({"system":h["system"],"beam_catalog":catalogs,"beam_origin_sibling_choices":beam_origin,"cases":selections,"main_stem_thickness":main,"max_stem_thickness":max,"min_beam_stems_dx":min_dx,"max_beam_seed_dx":dx,"max_beam_linker_dx":side_dx,"min_beam_stump_dy":min_dy}));
    }
    owned.report = json!({"schema":"audiveris-native.stem-image-beam-generation.v1","beam_target_plan":plan,"systems":result,"corner_count":plan["corner_count"],"beam_stump_occurrences":stump_total,"base_blinker_count":base_total,"final_blinker_count":final_total,"selected_blinker_occurrences":selected_total,"beam_catalog_generation_complete":true,"beam_origin_builder_inputs_complete":false,"automatic_raster_pipeline_complete":false,"final_stem_glyph_selection_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["beam-origin VLinker inspection/builders","section/chunk/items/final stem glyph; owned liveGraph/cache/head; cleaner/epilog","fresh transient HEADS; nonzero skew/scans/special staff/full24shape pages; HashSet ordering ties"]});
    Ok(owned)
}
fn bounds_g(g: &GlyphPixels) -> [f64; 4] {
    bounds(g)
}
fn rect_intersects(a: [f64; 4], b: [f64; 4]) -> bool {
    a[2] > 0.
        && a[3] > 0.
        && b[2] > 0.
        && b[3] > 0.
        && a[0] < b[0] + b[2]
        && b[0] < a[0] + a[2]
        && a[1] < b[1] + b[3]
        && b[1] < a[1] + a[3]
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn original() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-beam-generation/controls.json"
        ))
        .unwrap()
    }
    #[test]
    fn original_purge_overlap_touch_distance_threshold_and_removal_iteration() {
        for c in original()["purge_controls"].as_array().unwrap() {
            let mut gs = c["input"]
                .as_array()
                .unwrap()
                .iter()
                .map(|g| beam_dispatch::glyph_from_json(g).unwrap())
                .collect();
            purge(
                &mut gs,
                serde_json::from_value(c["median"].clone()).unwrap(),
                c["min_dx"].as_i64().unwrap() as i32,
            )
            .unwrap();
            assert_eq!(
                json!(gs.iter().map(beam_dispatch::glyph_json).collect::<Vec<_>>()),
                c["selected"],
                "{}",
                c["name"]
            );
        }
    }
    #[test]
    fn original_nearest_reuse_inclusive_ulps_and_new_anchor_registration() {
        for c in original()["nearest_controls"].as_array().unwrap() {
            let mut ls: Vec<Link> = serde_json::from_value(c["before"].clone()).unwrap();
            let i = nearest(
                &mut ls,
                serde_json::from_value(c["median"].clone()).unwrap(),
                serde_json::from_value(c["line"].clone()).unwrap(),
                c["max_dx"].as_i64().unwrap() as i32,
            )
            .unwrap();
            assert_eq!(json!(ls[i]), c["selected"]);
            assert_eq!(json!(ls), c["after"]);
        }
        let mut ls = Vec::new();
        assert_eq!(
            nearest(&mut ls, [0., 10., 100., 10.], [50., 0., 50., 20.], 3).unwrap(),
            0
        );
        assert!(ls[0].anchor);
        assert!(nearest(&mut ls, [0., 10., 100., 10.], [0., 0., 100., 0.], 3).is_err());
    }
    #[test]
    fn side_width_stops_at_first_overwide_section_and_portion_edges() {
        let b = Beam {
            id: 1,
            median: [100., 100., 120., 100.],
            top: [100., 98., 120., 98.],
            bottom: [100., 102., 120., 102.],
            height: 4.,
            group: 1,
            glyph_id: 1,
        };
        let a = json!({"id":1,"bounds":[102,95,1,12],"first_pos":102,"runs":[[95,12]]});
        let wide = json!({"id":2,"bounds":[100,95,6,12],"first_pos":100,"runs":[[95,12],[95,12],[95,12],[95,12],[95,12],[95,12]]});
        let later = json!({"id":3,"bounds":[100,95,1,12],"first_pos":100,"runs":[[95,12]]});
        let g = side_glyph(&[a, wide, later], &b, 0, 4).unwrap().unwrap();
        assert_eq!((g.left, g.width), (102, 1));
        assert_eq!(portion(&b, 105., 5), None);
        assert_eq!(
            portion(&b, f64::from_bits(105f64.to_bits() - 1), 5),
            Some("LEFT")
        );
        assert_eq!(portion(&b, 115., 5), None);
        assert_eq!(
            portion(&b, f64::from_bits(115f64.to_bits() + 1), 5),
            Some("RIGHT")
        );
        assert!(side_glyph(&[], &b, 0, 5).unwrap().is_none());
    }
}
