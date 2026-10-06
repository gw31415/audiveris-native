// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image continuation's automatic existing-seed branch of CLinker.retrieveStump.
//! The section fallback and subsequent items/cache/SIG lifecycle remain pending.
use crate::{
    GrayImage,
    beam_dispatch::{self, Group, Registry},
    beam_structure, stem_image_head_plan,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn array<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|v| v.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image seed {label}"))
}
fn overlaps(a: [f64; 4], b: [f64; 4]) -> bool {
    a[2] > 0.
        && a[3] > 0.
        && b[2] > 0.
        && b[3] > 0.
        && a[0] < b[0] + b[2]
        && b[0] < a[0] + a[2]
        && a[1] < b[1] + b[3]
        && b[1] < a[1] + a[3]
}
// java.awt.geom.Line2D.ptSegDistSq operation order, including endpoints.
fn distance(line: [f64; 4], p: [f64; 2]) -> f64 {
    let dx = line[2] - line[0];
    let dy = line[3] - line[1];
    let mut px = p[0] - line[0];
    let mut py = p[1] - line[1];
    let mut dot = px * dx + py * dy;
    let projection = if dot <= 0. {
        0.
    } else {
        px = dx - px;
        py = dy - py;
        dot = px * dx + py * dy;
        if dot <= 0. {
            0.
        } else {
            dot * dot / (dx * dx + dy * dy)
        }
    };
    (px * px + py * py - projection).max(0.)
}
// Original retrieveNoStemAreas: connected bar medians plus each gap ribbon.
// Axis-aligned ribbons are exact rectangles. Other geometry is refused, rather
// than approximated by its bounds during the original Area intersection.
fn no_stem_areas(graph: &Value, system: &Value) -> Result<Vec<[f64; 4]>, String> {
    let vertices = array(&graph["inters"], 100_000, "SIG vertices")?;
    let mut by_id = BTreeMap::new();
    for v in vertices {
        let id = v["id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or("invalid image seed SIG identity")?;
        if by_id.insert(id, v).is_some() {
            return Err("duplicate image seed SIG identity".into());
        }
    }
    let mut result = Vec::new();
    for edge in array(&graph["relations"], 200_000, "SIG relations")? {
        if edge["kind"] != "BarConnectionRelation" || edge["system_id"] != *system {
            continue;
        }
        let mut pair = Vec::new();
        for key in ["source", "target"] {
            let id = edge[key]
                .as_u64()
                .ok_or("invalid image seed bar connection identity")?;
            let v = *by_id
                .get(&id)
                .ok_or("unknown image seed bar connection vertex")?;
            if v["kind"] != "BarlineInter" || v["origin"]["system"] != *system {
                return Err("invalid image seed bar connection".into());
            }
            let line: [f64; 4] = serde_json::from_value(v["median"].clone())
                .map_err(|_| "invalid image seed bar median")?;
            let width = v["width"]
                .as_f64()
                .filter(|v| v.is_finite() && *v > 0.)
                .ok_or("invalid image seed bar width")?;
            if line.iter().any(|v| !v.is_finite()) || line[0] != line[2] || line[1] >= line[3] {
                return Err("unsupported image seed angled/empty bar area".into());
            }
            result.push([line[0] - width / 2., line[1], width, line[3] - line[1]]);
            pair.push((line, width));
        }
        let (a, aw) = pair[0];
        let (b, bw) = pair[1];
        if a[2] != b[0] || a[3] >= b[1] {
            return Err("unsupported image seed angled/overlapping bar gap".into());
        }
        let width = (aw + bw) / 2.;
        result.push([a[2] - width / 2., a[3], width, b[1] - a[3]]);
    }
    Ok(result)
}
fn purged(bounds: [f64; 4], areas: &[[f64; 4]], limit: f64) -> bool {
    areas.iter().any(|a| {
        if !overlaps(*a, bounds) {
            return false;
        }
        let width = (bounds[0] + bounds[2]).min(a[0] + a[2]) - bounds[0].max(a[0]);
        let height = (bounds[1] + bounds[3]).min(a[1] + a[3]) - bounds[1].max(a[1]);
        (width * height) / ((bounds[2] * bounds[3]).min(a[2] * a[3])) > limit
    })
}
/// StemsRetriever.finalizeBeams constructs a fresh retriever and obtains a fresh
/// VERTICAL_SEED list from the system's free glyphs. Purging affects this local
/// list only: canonical glyphs, groups, cached stems and SIG nodes remain owned.
pub(crate) fn repurge_system_seeds(
    graph: &Value,
    registry: &Registry,
    free: &beam_dispatch::SystemSpots,
) -> Result<Value, String> {
    let mut areas = no_stem_areas(graph, &json!(free.system_id))?;
    areas.sort_by(|a, b| a[0].total_cmp(&b[0]));
    if free.glyph_ids.len() > 100_000 {
        return Err("unsupported epilog free glyph count".into());
    }
    let mut seen = BTreeSet::new();
    let mut seeds = Vec::new();
    for &id in &free.glyph_ids {
        let entry = registry.get(id)?;
        if !seen.insert(id) {
            return Err("duplicate epilog free glyph".into());
        }
        if entry.groups.contains(&Group::VerticalSeed) {
            seeds.push((id, &entry.glyph));
        }
    }
    if areas
        .len()
        .checked_mul(seeds.len())
        .is_none_or(|n| n > 20_000_000)
    {
        return Err("unsupported epilog seed area work budget".into());
    }
    let mut removed = Vec::new();
    let mut surviving = Vec::new();
    for (id, glyph) in &seeds {
        let bounds = [
            f64::from(glyph.left),
            f64::from(glyph.top),
            glyph.width as f64,
            glyph.height as f64,
        ];
        if purged(bounds, &areas, 0.25) {
            removed.push(*id);
        } else {
            surviving.push(*id);
        }
    }
    Ok(
        json!({"system":free.system_id,"no_stem_areas":areas,"grouped_seed_ids":seeds.iter().map(|s|s.0).collect::<Vec<_>>(),"purged_seed_ids":removed,"surviving_seed_ids":surviving,"scope":"fresh retriever-local VERTICAL_SEED list; canonical registry/free groups/SIG/cache retained"}),
    )
}
#[allow(clippy::too_many_arguments)]
fn decision(
    glyph: &audiveris_staff_removal::GlyphPixels,
    p: [f64; 2],
    x: f64,
    x_dir: i32,
    y_dir: i32,
    in_dx: i32,
    out_dx: i32,
    min_stump: i32,
) -> (i32, bool, bool) {
    let gap = (f64::from(x_dir) * (x - p[0]) + 0.5).floor() as i32;
    let eligible = (gap >= 0 && gap <= out_dx) || (gap <= 0 && -gap <= in_dx);
    let ref_y = p[1].round_ties_even() as i32;
    let extent = if y_dir > 0 {
        glyph.top + glyph.height as i32 - 1 - ref_y
    } else {
        ref_y - glyph.top
    };
    (gap, eligible, extent >= min_stump)
}
/// Derives all conditions from native image HEADS; selected IDs belong to a
/// replayed, strongly owned physical page registry, preserving upstream groups.
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    let plan = stem_image_head_plan::analyze(image, page)?;
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let interline = scale.interline.main as f64;
    let in_dx = (interline * 0.4).round_ties_even() as i32;
    let out_dx = (interline * 0.25).round_ties_even() as i32;
    let seed_dy = (interline * 0.25).round_ties_even();
    let min_stump = (interline * 0.5).round_ties_even() as i32;
    let upstream = array(
        &page["head_registration"]["live_glyph_retrieval"]["registry"],
        100_000,
        "registry",
    )?;
    let mut registry = Registry::default();
    for r in upstream {
        let g = beam_dispatch::glyph_from_json(&r["glyph"])?;
        let id = registry.register(&g)?;
        if r["id"].as_u64() != Some(id as u64) {
            return Err("invalid image seed registry identity/order".into());
        }
        for group in array(&r["groups"], 4, "registry groups")? {
            registry.add_group(
                id,
                match group.as_str() {
                    Some("VERTICAL_SEED") => Group::VerticalSeed,
                    Some("BEAM_SPOT") => Group::BeamSpot,
                    Some("BLACK_HEAD_SPOT") => Group::BlackHeadSpot,
                    Some("BLACK_STACK_SPOT") => Group::BlackStackSpot,
                    _ => return Err("unsupported image seed registry group".into()),
                },
            )?;
        }
    }
    let initial_count = registry.len();
    let sources = array(&page["stem_seeds"]["systems"], 64, "systems")?;
    let systems = array(&plan["systems"], 64, "head systems")?;
    let layout = crate::system_area_geometry::build(
        &page["grid_ir"],
        image.width,
        image.height,
        scale.interline.main,
    )?;
    if sources.len() != systems.len() {
        return Err("invalid image seed system count".into());
    }
    let mut result = Vec::new();
    let mut selected_count = 0;
    let mut total = 0;
    let mut work = 0usize;
    for (index, (s, context)) in sources.iter().zip(systems).enumerate() {
        if s["system_id"] != context["system"] {
            return Err("invalid image seed system order".into());
        }
        let areas = no_stem_areas(&page["head_registration"]["shared_sig"], &context["system"])?;
        let mut purged_ids = Vec::new();
        let mut seeds = Vec::new();
        let mut ids = BTreeSet::new();
        for v in array(&s["accepted_seeds"], 4096, "accepted seeds")? {
            let glyph = beam_dispatch::glyph_from_json(v)?;
            let id = registry.register(&glyph)?;
            if registry.len() != initial_count
                || !registry.get(id)?.groups.contains(&Group::VerticalSeed)
                || !ids.insert(id)
            {
                return Err("invalid image seed upstream glyph identity/group/duplicate".into());
            }
            let line = beam_structure::glyph_geometry_signed(&glyph)?
                .center_line
                .ok_or("unsupported image seed single pixel line")?;
            if !line.iter().all(|v| v.is_finite()) || line[1] == line[3] {
                return Err("unsupported image seed horizontal center line".into());
            }
            let bounds = [
                f64::from(glyph.left),
                f64::from(glyph.top),
                glyph.width as f64,
                glyph.height as f64,
            ];
            if purged(bounds, &areas, 0.25) {
                purged_ids.push(id);
            } else {
                seeds.push((id, glyph, line, bounds));
            }
        }
        let corners = array(&context["image_context"]["corners"], 8192, "corners")?;
        work = work
            .checked_add(corners.len() * seeds.len() + areas.len() * ids.len())
            .filter(|v| *v <= 20_000_000)
            .ok_or("unsupported image seed work budget")?;
        let sys = layout.system_areas[index].bounds()?;
        let mut cases = Vec::new();
        for head in array(&context["image_context"]["corners"], 8192, "corners")? {
            let b: [i32; 4] = serde_json::from_value(head["head_bounds"].clone())
                .map_err(|_| "invalid image seed head bounds")?;
            let p: [f64; 2] = serde_json::from_value(head["reference"].clone())
                .map_err(|_| "invalid image seed reference")?;
            let x_dir = if head["side"] == "LEFT" { -1 } else { 1 };
            let y_dir = if head["vertical"] == "TOP" { -1 } else { 1 };
            let neighbor_rect = [
                f64::from(b[0]) - interline,
                f64::from(sys[1]),
                f64::from(b[2]) + 2. * interline,
                f64::from(sys[3]),
            ];
            let inside = p[0] - f64::from(x_dir * in_dx);
            let outside = p[0] + f64::from(x_dir * out_dx);
            let area = [
                inside.min(outside),
                p[1] - seed_dy,
                (outside - inside).abs(),
                2. * seed_dy,
            ];
            let neighbors = seeds
                .iter()
                .filter(|s| overlaps(s.3, neighbor_rect))
                .collect::<Vec<_>>();
            let mut candidates = neighbors
                .iter()
                .copied()
                .filter(|s| overlaps(s.3, area))
                .map(|s| (s, distance(s.2, p)))
                .collect::<Vec<_>>();
            candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
            // The Java set's tie order is not a portable native identity contract.
            // Refuse ambiguous physical ties until their original ordering is ported.
            if candidates.windows(2).any(|w| w[0].1 == w[1].1) {
                return Err("unsupported image seed equal-distance physical tie".into());
            }
            let mut chosen = None;
            let mut rows = Vec::new();
            for (s, _) in candidates {
                let x = crate::stem_checks::x_at_y([s.2[0], s.2[1]], [s.2[2], s.2[3]], p[1]);
                let (gap, eligible, stands_out) =
                    decision(&s.1, p, x, x_dir, y_dir, in_dx, out_dx, min_stump);
                if chosen.is_none() && eligible && stands_out {
                    chosen = Some(s.0);
                }
                rows.push(json!({"registry_id":s.0,"glyph":beam_dispatch::glyph_json(&s.1),"gap":gap,"eligible":eligible,"stands_out":stands_out}));
            }
            if chosen.is_some() {
                selected_count += 1;
            }
            total += 1;
            cases.push(json!({"head":head,"neighbor_seed_ids":neighbors.iter().map(|s|s.0).collect::<Vec<_>>(),"candidates":rows,"selected_seed_id":chosen,"selected_seed":chosen.map(|id|beam_dispatch::glyph_json(&registry.get(id).expect("owned selected seed").glyph)),"section_fallback_required":chosen.is_none()}));
        }
        result.push(json!({"no_stem_areas":areas,"purged_seed_ids":purged_ids,"surviving_seed_ids":seeds.iter().map(|s|s.0).collect::<Vec<_>>(),"system":context["system"],"max_head_in_dx":in_dx,"max_head_out_dx":out_dx,"max_head_seed_dy":seed_dy as i32,"min_head_stump_dy":min_stump,"cases":cases}));
    }
    if registry.len() != initial_count {
        return Err("image seed selection unexpectedly mutated physical registry".into());
    }
    Ok(
        json!({"schema":"audiveris-native.stem-image-seeds.v1","head_plan":plan,"systems":result,"physical_registry":registry.snapshot(),"upstream_registry_preserved":true,"corner_count":total,"selected_seed_corners":selected_count,"section_fallback_corners":total-selected_count,"image_seed_branch_complete":true,"automatic_stump_selection_complete":false,"automatic_raster_pipeline_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["image section fallback stump assembly/registration","remaining other-head and beam linker discovery; automatic items/chunks/gap/length/final glyph","whole-page owned cache/head/SIG integration","original cleaner/epilog; transient context, skew, scans, special staff and full24 pages"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_epilog_seed_list_matches_original_laura_and_preserves_physical_groups() {
        let output = std::process::Command::new("python3").args(["-c", "import gzip,sys;sys.stdout.buffer.write(gzip.decompress(open(sys.argv[1],'rb').read()))", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/official-stem-head-reference-context/native-laura-report.json.gz")]).output().unwrap();
        assert!(output.status.success());
        let page: Value = serde_json::from_slice(&output.stdout).unwrap();
        let mut registry = Registry::default();
        for r in page["head_registration"]["live_glyph_retrieval"]["registry"]
            .as_array()
            .unwrap()
        {
            let id = registry
                .register(&beam_dispatch::glyph_from_json(&r["glyph"]).unwrap())
                .unwrap();
            assert_eq!(json!(id), r["id"]);
            for group in r["groups"].as_array().unwrap() {
                registry
                    .add_group(
                        id,
                        match group.as_str().unwrap() {
                            "VERTICAL_SEED" => Group::VerticalSeed,
                            "BEAM_SPOT" => Group::BeamSpot,
                            "BLACK_HEAD_SPOT" => Group::BlackHeadSpot,
                            "BLACK_STACK_SPOT" => Group::BlackStackSpot,
                            _ => panic!("unexpected upstream glyph group"),
                        },
                    )
                    .unwrap();
            }
        }
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-seeds/controls.json"
        ))
        .unwrap();
        let graph = &page["head_registration"]["shared_sig"];
        let before = registry.snapshot();
        let mut removed = 0;
        let canonical = |rows: &[Value]| {
            rows.iter()
                .map(|r| serde_json::to_string(r).unwrap())
                .collect::<BTreeSet<_>>()
        };
        for (free, original) in page["beam_prolog"]["free_glyphs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(oracle["systems"].as_array().unwrap())
        {
            let free = beam_dispatch::SystemSpots {
                system_id: free["system_id"].as_u64().unwrap(),
                glyph_ids: serde_json::from_value(free["glyph_ids"].clone()).unwrap(),
            };
            let result = repurge_system_seeds(graph, &registry, &free).unwrap();
            let mut areas: Vec<[f64; 4]> =
                serde_json::from_value(original["no_stem_areas"].clone()).unwrap();
            areas.sort_by(|a, b| a[0].total_cmp(&b[0]));
            assert_eq!(result["no_stem_areas"], json!(areas));
            let surviving = result["surviving_seed_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| {
                    beam_dispatch::glyph_json(
                        &registry.get(id.as_u64().unwrap() as usize).unwrap().glyph,
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                canonical(&surviving),
                canonical(original["surviving_seeds"].as_array().unwrap())
            );
            removed += result["purged_seed_ids"].as_array().unwrap().len();
            for id in result["purged_seed_ids"].as_array().unwrap() {
                assert!(free.glyph_ids.contains(&(id.as_u64().unwrap() as usize)));
                assert!(
                    registry
                        .get(id.as_u64().unwrap() as usize)
                        .unwrap()
                        .groups
                        .contains(&Group::VerticalSeed)
                );
            }
            let mut stale = free;
            stale.glyph_ids.push(registry.len() + 1);
            assert!(repurge_system_seeds(graph, &registry, &stale).is_err());
        }
        assert_eq!(removed, 3);
        assert_eq!(registry.snapshot(), before);
    }
    #[test]
    fn original_seed_rounding_and_strict_bar_overlap_boundaries() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-seeds/controls.json"
        ))
        .unwrap();
        let params = &o["systems"][0];
        for c in o["seed_boundaries"].as_array().unwrap() {
            let g = beam_dispatch::glyph_from_json(&c["glyph"]).unwrap();
            let line = beam_structure::glyph_geometry_signed(&g)
                .unwrap()
                .center_line
                .unwrap();
            let p: [f64; 2] = serde_json::from_value(c["reference"].clone()).unwrap();
            let x = crate::stem_checks::x_at_y([line[0], line[1]], [line[2], line[3]], p[1]);
            let (gap, eligible, out) = decision(
                &g,
                p,
                x,
                c["x_direction"].as_i64().unwrap() as i32,
                c["y_direction"].as_i64().unwrap() as i32,
                params["max_head_in_dx"].as_i64().unwrap() as i32,
                params["max_head_out_dx"].as_i64().unwrap() as i32,
                params["min_head_stump_dy"].as_i64().unwrap() as i32,
            );
            assert_eq!(
                json!([gap, eligible, out]),
                json!([c["gap"], c["eligible"], c["stands_out"]])
            );
        }
        let areas: Vec<[f64; 4]> = serde_json::from_value(params["no_stem_areas"].clone()).unwrap();
        for c in o["bar_overlap_boundaries"].as_array().unwrap() {
            let b: [f64; 4] = serde_json::from_value(c["glyph"]["bounds"].clone()).unwrap();
            assert_eq!(
                purged(b, &areas, c["limit"].as_f64().unwrap()),
                c["removed"].as_bool().unwrap()
            );
        }
        assert!(!overlaps([0., 0., 1., 1.], [1., 0., 1., 1.]));
    }
}
