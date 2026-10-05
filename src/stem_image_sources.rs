// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image-derived CLinker seed input collection before builder parts/items.
use crate::{GrayImage, beam_dispatch, stem_image_stumps, stem_parts_alignment};
use audiveris_area::LinearArea;
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn overlap(a: [i32; 4], b: [i32; 4]) -> i32 {
    (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1])
}
fn bounds(g: &GlyphPixels) -> [i32; 4] {
    [g.left, g.top, g.width as i32, g.height as i32]
}
fn distance(line: [f64; 4], point: [i32; 2]) -> f64 {
    let (dx, dy) = (line[2] - line[0], line[3] - line[1]);
    let (px, py) = (f64::from(point[0]) - line[0], f64::from(point[1]) - line[1]);
    let dot = px * dx + py * dy;
    let projection = dot * dot / (dx * dx + dy * dy);
    (px * px + py * py - projection).max(0.).sqrt()
}
struct Parameters<'a> {
    polygon: &'a [[f64; 2]],
    line: [f64; 4],
    y_range: [i32; 4],
    min_contribution: i32,
    max_distance: f64,
}
fn choose(
    candidates: &[(usize, GlyphPixels)],
    stump: Option<&GlyphPixels>,
    p: &Parameters<'_>,
) -> Result<Vec<usize>, String> {
    if candidates.len() > 4096
        || p.polygon.len() > 128
        || p.line
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || (p.line[0] == p.line[2] && p.line[1] == p.line[3])
        || !(0..=1024).contains(&p.min_contribution)
        || !p.max_distance.is_finite()
        || !(0.0..=1024.0).contains(&p.max_distance)
        || p.y_range.iter().any(|v| !(0..=4_000_000).contains(v))
        || p.y_range[1] + p.y_range[3] > 4_000_000
    {
        return Err("unsupported image seed source parameters".into());
    }
    let area = if p.polygon.is_empty() {
        None
    } else {
        Some(LinearArea::polygon(p.polygon)?)
    };
    let mut ids = BTreeSet::new();
    let mut eligible = Vec::new();
    for (id, g) in candidates {
        if *id == 0 || !ids.insert(*id) {
            return Err("invalid image seed source identity".into());
        }
        let center = beam_dispatch::centroid(g)?;
        let b = bounds(g);
        if !match &area {
            Some(a) => a.intersects_rect(b.map(f64::from))?,
            None => false,
        } || stump.is_some_and(|s| overlap(b, bounds(s)) > 0)
        {
            continue;
        }
        let contribution = overlap(p.y_range, b).max(0);
        if contribution < p.min_contribution || distance(p.line, center) > p.max_distance {
            continue;
        }
        eligible.push((*id, b, contribution));
    }
    eligible.sort_by_key(|v| std::cmp::Reverse(v.2));
    // Hash-set insertion order on an equal-contribution overlapping pair can
    // change the retained glyph. That original ordering is not yet certified.
    for (i, a) in eligible.iter().enumerate() {
        if eligible[i + 1..]
            .iter()
            .take_while(|b| b.2 == a.2)
            .any(|b| overlap(a.1, b.1) > 0)
        {
            return Err("unsupported image seed source overlapping contribution tie".into());
        }
    }
    let mut selected: Vec<(usize, [i32; 4])> = Vec::new();
    for (id, b, _) in eligible {
        if selected.iter().any(|(_, old)| overlap(b, *old) > 0) {
            continue;
        }
        selected.push((id, b));
    }
    Ok(selected.into_iter().map(|v| v.0).collect())
}
fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|v| v.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image sources {label}"))
}
/// All candidates/geometry come from the actual native image HEADS path.
/// The page-owned registry produced during stump preparation remains owned.
pub fn prepare(image: &GrayImage, page: &Value) -> Result<stem_image_stumps::Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<crate::beam_dispatch::Registry>,
) -> Result<stem_image_stumps::Prepared, String> {
    let mut owned = stem_image_stumps::prepare_with_registry(image, page, registry)?;
    let stump_plan = owned.report.clone();
    let seed_plan = &stump_plan["seed_plan"];
    let head_plan = &seed_plan["head_plan"];
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let interline = scale.interline.main as f64;
    let min = (interline * 0.5).round_ties_even() as i32;
    let max_dx = interline * 0.15;
    let systems = rows(&stump_plan["systems"], 64, "systems")?;
    let seeds = rows(&seed_plan["systems"], 64, "seed systems")?;
    let heads = rows(&head_plan["systems"], 64, "head plans")?;
    if systems.len() != seeds.len() || systems.len() != heads.len() {
        return Err("invalid image sources system count".into());
    }
    let mut result = Vec::new();
    let mut selected_count = 0;
    let mut aligned_count = 0;
    let mut work = 0usize;
    for ((sys, seed), head) in systems.iter().zip(seeds).zip(heads) {
        if sys["system"] != seed["system"] || sys["system"] != head["system"] {
            return Err("invalid image sources system order".into());
        }
        let cases = rows(&sys["cases"], 8192, "stump cases")?;
        let seed_cases = rows(&seed["cases"], 8192, "seed cases")?;
        let targets = rows(&head["cases"], 8192, "target cases")?;
        if cases.len() != seed_cases.len() || cases.len() != targets.len() {
            return Err("invalid image sources corner count".into());
        }
        let mut out = Vec::new();
        for ((c, s), t) in cases.iter().zip(seed_cases).zip(targets) {
            if c["head"] != s["head"] {
                return Err("invalid image sources corner order".into());
            }
            let reference: [f64; 2] = serde_json::from_value(c["head"]["reference"].clone())
                .map_err(|_| "invalid image sources reference")?;
            let target: [f64; 2] = serde_json::from_value(t["target"]["target"].clone())
                .map_err(|_| "invalid image sources target")?;
            let polygon: Vec<[f64; 2]> =
                serde_json::from_value(t["target"]["area"]["polygon"].clone())
                    .map_err(|_| "invalid image sources area")?;
            // CLinker replaces its line with refPt->targetPt after buildLuArea;
            // this differs from the area's extended collector endpoint.
            let line = [reference[0], reference[1], target[0], target[1]];
            let top = if c["head"]["vertical"] == "TOP" {
                target[1]
            } else {
                reference[1]
            };
            let y_range = [
                0,
                top.round_ties_even() as i32,
                0,
                (target[1] - reference[1]).abs().round_ties_even() as i32,
            ];
            let stump = c["selected_stump_id"]
                .as_u64()
                .map(|id| owned.registry.get(id as usize).map(|e| &e.glyph))
                .transpose()?;
            let mut candidates = Vec::new();
            for id in rows(&s["neighbor_seed_ids"], 4096, "neighbor seed IDs")? {
                let id = id
                    .as_u64()
                    .and_then(|v| usize::try_from(v).ok())
                    .filter(|v| *v > 0)
                    .ok_or("invalid image sources physical identity")?;
                candidates.push((id, owned.registry.get(id)?.glyph.clone()));
            }
            work = work
                .checked_add(candidates.len())
                .filter(|v| *v <= 20_000_000)
                .ok_or("image sources work budget")?;
            let selected = choose(
                &candidates,
                stump,
                &Parameters {
                    polygon: &polygon,
                    line,
                    y_range,
                    min_contribution: min,
                    max_distance: max_dx,
                },
            )?;
            selected_count += selected.len();
            let glyphs = selected
                .iter()
                .map(|id| {
                    beam_dispatch::glyph_json(
                        &owned.registry.get(*id).expect("owned selected seed").glyph,
                    )
                })
                .collect::<Vec<_>>();
            let parts = selected
                .iter()
                .map(|id| stem_parts_alignment::Part {
                    id: *id,
                    glyph: &owned.registry.get(*id).expect("owned selected seed").glyph,
                })
                .collect::<Vec<_>>();
            let start = c["selected_stump_id"]
                .as_u64()
                .map(|id| stem_parts_alignment::Part {
                    id: id as usize,
                    glyph: stump.expect("selected stump"),
                });
            let direction = if c["head"]["vertical"] == "TOP" {
                -1
            } else {
                1
            };
            let aligned = stem_parts_alignment::filter(
                &parts,
                start,
                &stem_parts_alignment::Parameters::new(
                    direction,
                    interline * 0.15,
                    interline * 4.,
                    0.,
                )?,
            )?;
            aligned_count += aligned.retained_order.len();
            let aligned_glyphs = aligned
                .retained_order
                .iter()
                .map(|id| {
                    beam_dispatch::glyph_json(
                        &owned.registry.get(*id).expect("owned aligned seed").glyph,
                    )
                })
                .collect::<Vec<_>>();
            out.push(json!({"head":c["head"],"theoretical_line":line,"y_range":y_range,"polygon":polygon,"stump":c["selected_stump"],"selected_seed_ids":selected,"selected_seeds":glyphs,"aligned_seed_ids":aligned.retained_order,"aligned_seeds":aligned_glyphs,"alignment_removed_ids":aligned.removed_order}));
        }
        result.push(json!({"system":sys["system"],"min_seed_contribution":min,"max_line_seed_dx":max_dx,"max_alignment_dx":interline*0.15,"max_alignment_dy":interline*4.,"cases":out}));
    }
    owned.report = json!({"schema":"audiveris-native.stem-image-sources.v1","stump_plan":stump_plan,"systems":result,"corner_count":stump_plan["corner_count"],"selected_builder_seed_occurrences":selected_count,"aligned_builder_seed_occurrences":aligned_count,"image_builder_seed_selection_complete":true,"image_seed_alignment_complete":true,"other_head_linker_selection_complete":false,"beam_sibling_linker_selection_complete":false,"final_stem_glyph_selection_complete":false,"automatic_raster_pipeline_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["other-head/beam sibling target linkers and builder section/chunk/items/final prefix glyph","same owned live HEADS Graph and retriever/cache/head attachments","contextual cleaner/checkNeededStems/beam epilog","fresh transient lifecycle; nonzero skew/scans/special staff/full24 pages"]});
    Ok(owned)
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_seed_source_filter_boundaries_and_unverified_ties() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-sources/controls.json"
        ))
        .unwrap();
        for c in o["seed_source_controls"].as_array().unwrap() {
            let candidates = c["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, g)| (i + 1, beam_dispatch::glyph_from_json(g).unwrap()))
                .collect::<Vec<_>>();
            let stump = (!c["stump"].is_null())
                .then(|| beam_dispatch::glyph_from_json(&c["stump"]).unwrap());
            let polygon = serde_json::from_value::<Vec<[f64; 2]>>(c["polygon"].clone()).unwrap();
            let p = Parameters {
                polygon: &polygon,
                line: serde_json::from_value(c["theoretical_line"].clone()).unwrap(),
                y_range: serde_json::from_value(c["y_range"].clone()).unwrap(),
                min_contribution: c["min_seed_contribution"].as_i64().unwrap() as i32,
                max_distance: c["max_line_seed_dx"].as_f64().unwrap(),
            };
            let selected = choose(&candidates, stump.as_ref(), &p).unwrap();
            let glyphs = selected
                .iter()
                .map(|id| beam_dispatch::glyph_json(&candidates[*id - 1].1))
                .collect::<Vec<_>>();
            assert_eq!(json!(glyphs), c["selected"], "{}", c["name"]);
        }
        let c = &o["seed_source_controls"][1];
        let g = beam_dispatch::glyph_from_json(&c["candidates"][0]).unwrap();
        let mut alias = g.clone();
        alias.left += 1;
        let polygon = serde_json::from_value::<Vec<[f64; 2]>>(c["polygon"].clone()).unwrap();
        let p = Parameters {
            polygon: &polygon,
            line: [100., 100., 100., 140.],
            y_range: [0, 100, 0, 40],
            min_contribution: 6,
            max_distance: 20.,
        };
        assert!(
            choose(&[(1, g.clone()), (2, alias)], None, &p)
                .unwrap_err()
                .contains("tie")
        );
        assert!(
            choose(&[(1, g.clone()), (1, g)], None, &p)
                .unwrap_err()
                .contains("identity")
        );
    }
}
