// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native image HEADS output -> loaded-book-context beam target planning.
//! All corner references, margins, neighborhoods and targets are derived here;
//! neither oracle output nor a caller-supplied corner request is read at runtime.
use crate::{
    GrayImage,
    head_staff_catalog::TemplateCatalog,
    head_stem_relation::{HeadSide, VerticalSide},
    stem_head_neighborhood::{self, Bounds, Case, Request, System},
    stem_head_reference_context::{self, Head},
    stem_head_targets::{Beam, Group},
    stem_lookup_area::HeadRequest,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn rows<'a>(v: &'a Value, what: &str, max: usize) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|v| v.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image HEADS {what}"))
}
fn identity(v: &Value) -> Result<u64, String> {
    v.as_u64()
        .filter(|v| *v > 0)
        .ok_or("invalid image HEADS identity".into())
}
fn rectangle(v: &Value) -> Result<[i32; 4], String> {
    let r: [i32; 4] =
        serde_json::from_value(v.clone()).map_err(|_| "invalid image HEADS bounds")?;
    if r[..2].iter().any(|v| !(0..=4_000_000).contains(v))
        || r[2..].iter().any(|v| !(1..=4_000_000).contains(v))
        || i64::from(r[0]) + i64::from(r[2]) > 4_000_000
        || i64::from(r[1]) + i64::from(r[3]) > 4_000_000
    {
        return Err("invalid image HEADS bounds".into());
    }
    Ok(r)
}
fn persisted(v: f64) -> Result<f64, String> {
    stem_head_reference_context::to_xml(Some(v))?
        .ok_or("missing image HEADS XML value")?
        .parse::<f64>()
        .map_err(|e| e.to_string())
}
fn raw_dx(scale: &[Value], shape: &str) -> Result<[Option<f64>; 2], String> {
    let mut result = [None, None];
    for (i, side) in ["LEFT", "RIGHT"].iter().enumerate() {
        let found = scale
            .iter()
            .filter(|r| r["shape"] == shape && r["side"] == *side)
            .collect::<Vec<_>>();
        if found.len() > 1 {
            return Err("duplicate image HEADS seed scale".into());
        }
        if let Some(r) = found.first() {
            result[i] = if r["dx"].is_null() {
                None
            } else {
                Some(r["dx"].as_f64().ok_or("invalid image HEADS seed DX")?)
            };
        }
    }
    Ok(result)
}

/// The active HEADS catalog can differ from an unmeasured BEAMS font scale.
/// Carry the catalog selected by native HEADS; no new font inference is made.
pub(crate) fn image_catalog(page: &Value) -> Result<TemplateCatalog, String> {
    let catalog = match TemplateCatalog::from_report(&page["head_registration"])? {
        Some(catalog) => catalog,
        None => TemplateCatalog::from_head_scale(&page["beam_prolog"]["head_scale"])?,
    };
    if catalog.family != "Bravura" || !matches!(catalog.stored_point_size, 44 | 51) {
        return Err("unsupported image STEMS font/size context".into());
    }
    if page["beam_prolog"]["head_scale"]["music_font"]["name"]
        .as_str()
        .is_some_and(|family| family != catalog.family)
    {
        return Err("inconsistent image STEMS font/size context".into());
    }
    Ok(catalog)
}

/// Zero-skew, ordinary five-line, uniform-interline Bravura44/51 continuation.
/// Explicit XML restoration emulates the original oracle's loaded HEADS context.
/// It leaves the supplied image report and its raw physical registry/SIG intact.
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    if page["head_registration"]["head_registration_complete"] != true
        || page["head_registration"]["staff_dependency_complete"] != true
        || page["later_beams"]["groups_complete"] != true
        || page["stem_seeds"]["stem_seeds_complete"] != true
    {
        return Err("incomplete image HEADS/STEMS upstream dependency".into());
    }
    let slope = page["line_completion"]["global_slope"]
        .as_f64()
        .ok_or("missing image HEADS skew")?;
    if slope != 0. {
        return Err("unsupported image STEMS nonzero skew".into());
    }
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let sc = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let interline = sc.interline.main;
    let catalog = image_catalog(page)?;
    let bank = catalog.bank()?;
    let grid = &page["grid_ir"];
    let staffs = rows(&grid["staffs"], "staffs", 1024)?;
    let mut staff_indices = BTreeMap::new();
    for (i, s) in staffs.iter().enumerate() {
        let id = identity(&s["id"])?;
        if staff_indices.insert(id, i).is_some() {
            return Err("duplicate image HEADS staff".into());
        }
        if rows(&s["lines"], "staff lines", 6)?.len() != 5
            || s["specific_interline"].as_u64() != Some(interline as u64)
            || s["tablature"] == true
            || s["drum"] == true
        {
            return Err("unsupported image STEMS special/mixed staff".into());
        }
    }
    let layout = crate::system_area_geometry::build(grid, image.width, image.height, interline)?;
    let heads = &page["head_registration"];
    let scale = rows(&heads["head_seed_scale"], "seed scale", 64)?;
    let head_systems = rows(&heads["systems"], "head systems", 64)?;
    let beam_systems = rows(&page["later_beams"]["systems"], "beam systems", 64)?;
    let seeds = rows(&page["stem_seeds"]["systems"], "seed systems", 64)?;
    let vertices = rows(&heads["shared_sig"]["inters"], "SIG vertices", 100_000)?;
    let systems = rows(&grid["systems"], "systems", 64)?;
    if head_systems.len() != systems.len()
        || beam_systems.len() != systems.len()
        || seeds.len() != systems.len()
    {
        return Err("inconsistent image HEADS system catalogs".into());
    }
    let mut request_systems = Vec::new();
    let mut metadata = Vec::new();
    let mut total_corners = 0usize;
    let mut seen_systems = BTreeSet::new();
    for (i, s) in systems.iter().enumerate() {
        let sid = identity(&s["id"])?;
        if !seen_systems.insert(sid)
            || head_systems[i]["system_id"] != sid
            || beam_systems[i]["system_id"] != sid
            || seeds[i]["system_id"] != sid
        {
            return Err("inconsistent image HEADS system identity/order".into());
        }
        let profile = seeds[i]["profile"]
            .as_u64()
            .ok_or("missing image STEMS profile")?;
        if profile != 1 {
            return Err("unsupported image STEMS system profile".into());
        }
        let gaps = crate::head_stem_relation::gap_limits(profile as u8)?;
        let mut part_bounds = BTreeMap::new();
        let mut parts = Vec::new();
        let mut part_ids = BTreeSet::new();
        for part in rows(&s["parts"], "parts", 1024)? {
            let pid = identity(&part["id"])?;
            if !part_ids.insert(pid) {
                return Err("duplicate image HEADS part".into());
            }
            let members = rows(&part["staff_ids"], "part staff IDs", 1024)?;
            let mut union: Option<[i32; 4]> = None;
            let mut ids = Vec::new();
            for member in members {
                let id = identity(member)?;
                let index = *staff_indices
                    .get(&id)
                    .ok_or("unknown image HEADS part staff")?;
                if staffs[index]["system_id"] != sid || part_bounds.contains_key(&id) {
                    return Err("invalid image HEADS part membership".into());
                }
                let b = layout.staff_areas[index].bounds()?;
                union = Some(match union {
                    None => b,
                    Some(a) => {
                        let x = a[0].min(b[0]);
                        let y = a[1].min(b[1]);
                        [
                            x,
                            y,
                            (a[0] + a[2]).max(b[0] + b[2]) - x,
                            (a[1] + a[3]).max(b[1] + b[3]) - y,
                        ]
                    }
                });
                ids.push(id);
            }
            let b = union.ok_or("empty image HEADS part")?;
            for id in &ids {
                if part_bounds.insert(*id, b).is_some() {
                    return Err("duplicate image HEADS part staff".into());
                }
            }
            parts.push(json!({"id":pid,"staff_ids":ids,"bounds":b}));
        }
        let expected_staffs = rows(&s["staff_ids"], "system staff IDs", 1024)?
            .iter()
            .map(identity)
            .collect::<Result<BTreeSet<_>, _>>()?;
        if part_bounds.keys().copied().collect::<BTreeSet<_>>() != expected_staffs {
            return Err("incomplete image HEADS part coverage".into());
        }
        let groups: Vec<Group> = rows(&beam_systems[i]["groups"], "groups", 4096)?
            .iter()
            .map(|g| {
                let id = identity(&g["id"])? as usize;
                let members = rows(&g["members"], "group members", 4096)?
                    .iter()
                    .map(|v| identity(v).map(|v| v as usize))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Group { id, members })
            })
            .collect::<Result<_, String>>()?;
        let mut membership = BTreeMap::new();
        for g in &groups {
            for &id in &g.members {
                if membership.insert(id, g.id).is_some() {
                    return Err("duplicate image beam group membership".into());
                }
            }
        }
        let mut beams = Vec::new();
        let mut bounds = Vec::new();
        for b in vertices
            .iter()
            .filter(|b| b["origin"]["system"] == sid && b["origin"]["phase"] == "BEAM")
        {
            if !((b["kind"] == "BeamInter" && b["shape"] == "BEAM")
                || (b["kind"] == "BeamHookInter" && b["shape"] == "BEAM_HOOK"))
            {
                return Err("unsupported image STEMS small/unknown beam".into());
            }
            let id = identity(&b["origin"]["local"])? as usize;
            let mut median: [f64; 4] = serde_json::from_value(b["median"].clone())
                .map_err(|_| "invalid image beam median")?;
            for value in &mut median {
                *value = persisted(*value)?;
            }
            let height = persisted(b["height"].as_f64().ok_or("invalid image beam height")?)?;
            let mut top = median;
            let mut bottom = median;
            top[1] -= height / 2.;
            top[3] -= height / 2.;
            bottom[1] += height / 2.;
            bottom[3] += height / 2.;
            beams.push(Beam {
                id,
                group: membership.get(&id).copied(),
                median,
                top,
                bottom,
                height,
            });
            bounds.push(Bounds {
                id,
                bounds: rectangle(&b["bounds"])?,
                removed: false,
            });
        }
        let beam_ids = beams.iter().map(|b| b.id).collect();
        let mut cases = Vec::new();
        let mut corners = Vec::new();
        let mut unique_heads = BTreeSet::new();
        for h in rows(&head_systems[i]["heads"], "heads", 2048)? {
            if h["removed"] == true {
                continue;
            }
            let hid = identity(&h["id"])?;
            if !unique_heads.insert(hid) {
                return Err("duplicate image HEADS head identity".into());
            }
            let shape = h["shape"].as_str().ok_or("invalid image HEADS shape")?;
            if matches!(shape, "WHOLE_NOTE" | "BREVE") {
                continue;
            }
            if !matches!(shape, "NOTEHEAD_BLACK" | "NOTEHEAD_VOID") {
                return Err("unsupported image STEMS head shape".into());
            }
            let staff = identity(&h["staff_id"])?;
            let part = *part_bounds
                .get(&staff)
                .ok_or("unknown image HEADS head staff")?;
            let b = rectangle(&h["bounds"])?;
            let dx = raw_dx(scale, shape)?;
            for side in [HeadSide::Left, HeadSide::Right] {
                for vertical in [VerticalSide::Top, VerticalSide::Bottom] {
                    let reference = stem_head_reference_context::reference(
                        &Head {
                            bank: &bank,
                            point_size: catalog.stored_point_size,
                            shape,
                            bounds: b,
                            side,
                            vertical,
                        },
                        dx,
                    )?;
                    cases.push(Case {
                        head_bounds: b,
                        head: HeadRequest {
                            reference,
                            x_direction: if side == HeadSide::Left { -1 } else { 1 },
                            y_direction: if vertical == VerticalSide::Top { -1 } else { 1 },
                            slope,
                            slope_margin: 0.015,
                            max_head_in_dx: (interline as f64 * gaps.x_in_max).round_ties_even()
                                as i32,
                            max_head_out_dx: (interline as f64 * gaps.x_out_max).round_ties_even()
                                as i32,
                            part_bounds: part,
                            limit: None,
                        },
                        min_distance: interline as i32,
                        small_head: false,
                    });
                    corners.push(json!({"head_id":hid,"head_bounds":b,"shape":shape,"staff_id":staff,"side":side,"vertical":vertical,"reference":reference}));
                }
            }
        }
        total_corners += cases.len();
        let meta = json!({"system":sid,"profile":profile,"parts":parts,"beams":beams,"beam_bounds":bounds,"groups":groups,"corners":corners});
        request_systems.push(System {
            system: sid,
            system_bounds: layout.system_areas[i].bounds()?,
            vicinity_margin: interline as i32,
            beam_ids,
            beams,
            beam_bounds: bounds,
            groups,
            cases,
        });
        metadata.push(meta);
    }
    let report = stem_head_neighborhood::evaluate(&Request {
        schema: stem_head_neighborhood::REQUEST_SCHEMA.into(),
        systems: request_systems,
    })?;
    let mut planned = serde_json::to_value(report).map_err(|e| e.to_string())?;
    for (system, meta) in planned["systems"]
        .as_array_mut()
        .ok_or("missing image head plans")?
        .iter_mut()
        .zip(metadata)
    {
        system["image_context"] = meta;
    }
    planned["schema"] = json!("audiveris-native.stem-image-head-plan.v1");
    planned["input_context"] =
        json!("native_image_HEADS_and_BEAMS_with_explicit_XML_loaded_book_geometry_context");
    planned["image_head_target_selection_complete"] = json!(true);
    planned["corner_count"] = json!(total_corners);
    planned["raw_HEADS_report_preserved"] = json!(true);
    planned["pending"] = json!([
        "image linker/item/glyph selection",
        "whole-page owned cache/head/SIG integration",
        "original partitions/cleaner/checkNeededStems/finalizeBeams epilog",
        "fresh transient reference lifecycle; nonzero skew, real scans, special staff and full24 HEADS pages"
    ]);
    Ok(planned)
}

#[cfg(test)]
mod catalog_tests {
    use super::*;

    #[test]
    fn active_44_heads_catalog_survives_missing_beam_font_and_reaches_bank() {
        let mut page = json!({"head_registration": {
            "template_catalog_required": true,
            "template_catalog": {"family": "Bravura", "stored_point_size": 44}
        }});
        let before = page.clone();
        let catalog = image_catalog(&page).unwrap();
        assert_eq!(catalog.stored_point_size, 44);
        catalog.bank().unwrap();
        assert_eq!(page, before);
        page["beam_prolog"] = json!({"head_scale": {"music_font": {
            "name": "Bravura", "pointSize": 51
        }}});
        assert_eq!(image_catalog(&page).unwrap().stored_point_size, 44);
        page["beam_prolog"]["head_scale"]["music_font"]["name"] = json!("Leland");
        assert!(image_catalog(&page).unwrap_err().contains("font/size"));
    }

    #[test]
    fn legacy_catalog_missing_required_and_unsupported_contexts_are_distinct() {
        let mut page = json!({"beam_prolog": {"head_scale": {"music_font": {
            "name": "Bravura", "pointSize": 51
        }}}});
        assert_eq!(image_catalog(&page).unwrap().stored_point_size, 51);
        page["head_registration"] = json!({"template_catalog_required": true});
        assert!(image_catalog(&page).unwrap_err().contains("missing"));
        for (family, size) in [("Bravura", 40), ("Leland", 51)] {
            page["head_registration"]["template_catalog"] = json!({
                "family": family, "stored_point_size": size
            });
            assert!(image_catalog(&page).unwrap_err().contains("font/size"));
        }
    }
}
