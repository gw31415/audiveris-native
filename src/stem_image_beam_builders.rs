// SPDX-License-Identifier: AGPL-3.0-or-later
//! BeamLinker.VLinker geometry/filterHeads/StemBuilder and expand on image-owned
//! physical glyphs. Ordinary beam groups with actual alien search limits.
use crate::{
    beam_dispatch::{self, Registry},
    stem_beam_closer_limit, stem_beam_lookup_area, stem_chunks, stem_glyph,
    stem_image_head_chunks::{members, sections},
    stem_image_head_expansion::{Context, Expansion, arr, bounds, line, update},
    stem_image_owned::OwnedItems,
    stem_item_events::{self, Kind, Source},
    stem_length_map,
    stem_parts_alignment::{self, Part},
    stem_parts_filter, stem_parts_lookup,
};
use audiveris_area::LinearArea;
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn canonical(registry: &mut Registry, v: &Value) -> Result<Option<usize>, String> {
    if v.is_null() {
        return Ok(None);
    }
    let before = registry.len();
    let id = registry.register(&beam_dispatch::glyph_from_json(v)?)?;
    if registry.len() != before {
        return Err("beam builder unknown upstream glyph".into());
    }
    Ok(Some(id))
}
// Original Inters.byReverseWidth compares actual integer Rectangle widths.
// Median lengths can differ while bounds widths tie; preserve insertion order.
fn order_by_width(catalogs: &mut [&Value], actual: &[&Value]) -> Result<(), String> {
    let widths = actual
        .iter()
        .map(|b| {
            let id = b["origin"]["local"]
                .as_u64()
                .ok_or("beam sort actual ID required")?;
            let width = b["bounds"][2]
                .as_i64()
                .filter(|w| *w >= 0)
                .ok_or("beam sort actual bounds width required")?;
            Ok((id, width))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    for c in catalogs.iter() {
        if !c["beam"]["id"]
            .as_u64()
            .is_some_and(|id| widths.contains_key(&id))
        {
            return Err("beam sort catalog owner required".into());
        }
    }
    catalogs.sort_by_key(|c| std::cmp::Reverse(widths[&c["beam"]["id"].as_u64().unwrap()]));
    Ok(())
}
/// Input references are current native catalogs and actual retained registry,
/// never oracle data or selected caller coordinates.
pub(crate) fn prepare(o: &mut OwnedItems) -> Result<Vec<Vec<Value>>, String> {
    let report = o.prepared.report.clone();
    let generation = &report["head_chunks_plan"]["head_parts_plan"]["beam_generation_plan"];
    let source = &generation["beam_target_plan"]["head_linker_plan"]["source_plan"];
    let plan = &source["stump_plan"]["seed_plan"]["head_plan"];
    let seeds = &source["stump_plan"]["seed_plan"];
    let page = &o.upstream_report;
    let grid = &page["grid_ir"];
    let raster = crate::GrayImage::decode(&o.upstream_output_pixels)?;
    let layout =
        crate::system_area_geometry::build(grid, raster.width, raster.height, o.interline)?;
    let v = sections(&grid["vlag"], true)?;
    let h = sections(&grid["hlag"], false)?;
    let max = page["stem_seeds"]["stem_scaling"]["stem_scale"]["max"]
        .as_i64()
        .ok_or("beam builder max stem required")?;
    let il = o.interline as f64;
    let gap_map: BTreeMap<i32, i32> = [0., 0.3, 0.6, 2., 4.]
        .into_iter()
        .enumerate()
        .map(|(p, v)| (p as i32, (il * v).round_ties_even() as i32))
        .collect();
    let graph = o.graph.snapshot();
    let mut systems = Vec::new();
    for (si, system) in arr(&generation["systems"])?.iter().enumerate() {
        let catalogs = arr(&system["beam_catalog"])?;
        let sid = system["system"]
            .as_u64()
            .ok_or("beam builder system required")?;
        let live = arr(&graph["inters"])?
            .iter()
            .filter(|n| {
                n["origin"]["system"] == sid
                    && matches!(
                        n["kind"].as_str(),
                        Some("BeamInter" | "BeamHookInter" | "SmallBeamInter")
                    )
            })
            .collect::<Vec<_>>();
        if live.is_empty() {
            systems.push(Vec::new());
            continue;
        }
        if live.iter().any(|b| b["kind"] == "SmallBeamInter") {
            return Err("beam builder small beams pending".into());
        }
        let mut active = live
            .iter()
            .map(|b| {
                catalogs
                    .iter()
                    .find(|c| c["beam"]["id"] == b["origin"]["local"])
                    .ok_or("beam builder actual beam required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        // Admit only an actual same-glyph BeamInter/BeamHookInter exclusion.
        // Other aliases still require their original removal/callback handling.
        if active.iter().enumerate().any(|(i, c)| {
            active[..i].iter().any(|a| {
                let owners = [a, c].map(|cat| {
                    live.iter()
                        .find(|n| n["origin"]["local"] == cat["beam"]["id"])
                        .unwrap()
                });
                if owners[0]["glyph_id"] != owners[1]["glyph_id"] {
                    return false;
                }
                !owners.iter().any(|beam| {
                    crate::music_beams::competing_hook(&graph, beam).is_some_and(|hook| {
                        owners
                            .iter()
                            .any(|n| n["kind"] == "BeamHookInter" && n["origin"]["local"] == hook)
                    })
                })
            })
        }) {
            return Err("beam builder shared-glyph aliases pending".into());
        }
        let limit_beams = active
            .iter()
            .map(|c| {
                let n = live
                    .iter()
                    .find(|b| b["origin"]["local"] == c["beam"]["id"])
                    .ok_or("beam limit actual owner required")?;
                Ok(stem_beam_closer_limit::Beam {
                    id: c["beam"]["id"].as_u64().ok_or("beam limit ID required")?,
                    group: c["beam"]["group"]
                        .as_u64()
                        .ok_or("beam limit group required")?,
                    bounds: bounds(&n["bounds"])?,
                    median: line(&c["beam"]["median"])?,
                    height: c["beam"]["height"]
                        .as_f64()
                        .ok_or("beam limit height required")?,
                    grade: n["grade"]
                        .as_f64()
                        .ok_or("beam limit intrinsic grade required")?,
                    hook: n["kind"] == "BeamHookInter",
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        // StemsRetriever processes stable descending beam width.
        order_by_width(&mut active, &live)?;
        let ctx = &plan["systems"][si]["image_context"];
        let parts = arr(&ctx["parts"])?;
        if parts.len() != 1 {
            return Err("beam builder surrounding multiple parts pending".into());
        }
        let vertical = members(&v, &grid["systems"][si]["vertical_section_ids"])?;
        let horizontal = members(&h, &grid["systems"][si]["horizontal_section_ids"])?;
        let all = arr(&report["systems"][si]["cases"])?;
        let sources = arr(&source["systems"][si]["cases"])?;
        let mut heads = arr(&graph["inters"])?
            .iter()
            .filter(|n| n["origin"]["system"] == sid && n["kind"] == "HeadInter")
            .collect::<Vec<_>>();
        heads.sort_by_key(|n| n["bounds"][0].as_i64().unwrap());
        let candidates: Vec<usize> =
            serde_json::from_value(seeds["systems"][si]["surviving_seed_ids"].clone())
                .map_err(|_| "beam builder surviving seeds required")?;
        let mut cases = Vec::new();
        for cat in &active {
            for (bli, b) in arr(&cat["base_blinkers"])?.iter().enumerate() {
                if b["anchor"] == true {
                    return Err("beam builder anchor geometry pending".into());
                }
                for side in arr(&b["directions"])? {
                    let dir = if side == "BOTTOM" {
                        1
                    } else if side == "TOP" {
                        -1
                    } else {
                        return Err("beam builder direction required".into());
                    };
                    let mut lookup = stem_beam_lookup_area::Request {
                        reference: serde_json::from_value(b["reference"].clone())
                            .map_err(|_| "beam builder reference required")?,
                        border: line(&cat["beam"][if dir > 0 { "bottom" } else { "top" }])?,
                        direction: dir,
                        slope: 0.,
                        slope_margin: 0.015,
                        half_lookup_dx: il * 0.3,
                        max_seed_dy_ratio: 0.25,
                        gap_pixels: (il * 1.2).round_ties_even() as i32,
                        system_bounds: layout.system_areas[si].bounds()?,
                        surrounding_part_bounds: vec![bounds(&parts[0]["bounds"])?],
                        limit: None,
                    };
                    let initial = stem_beam_lookup_area::generate(&lookup)?;
                    let current = limit_beams
                        .iter()
                        .find(|a| a.id == cat["beam"]["id"].as_u64().unwrap())
                        .ok_or("beam limit current owner required")?;
                    let closer = stem_beam_closer_limit::select(
                        &stem_beam_closer_limit::Search {
                            group: current.group,
                            beam_bounds: current.bounds,
                            system_bounds: lookup.system_bounds,
                            reference: lookup.reference,
                            theoretical_line: initial.theoretical_line,
                            direction: dir,
                            left: b["side"] == "LEFT",
                            interline: o.interline as i32,
                        },
                        &limit_beams,
                    )?;
                    lookup.limit = closer.map(|(_, border)| border);
                    let geo = if closer.is_some() {
                        stem_beam_lookup_area::generate(&lookup)?
                    } else {
                        initial
                    };
                    let area = LinearArea::polygon(&geo.polygon)?;
                    let choices = arr(&system["beam_origin_sibling_choices"])?;
                    let chosen = choices
                        .iter()
                        .find(|s| {
                            s["beam_id"] == cat["beam"]["id"]
                                && s["linker_id"] == bli + 1
                                && s["direction"] == *side
                        })
                        .ok_or("beam sibling inspection choices required")?;
                    let mut metas = Vec::new();
                    let mut siblings = vec![*cat];
                    for selected in arr(&chosen["selected_sibling_blinkers"])? {
                        let sibling = active
                            .iter()
                            .find(|a| a["beam"]["id"] == selected["beam_id"])
                            .ok_or("beam sibling actual catalog required")?;
                        let mut m = selected["linker"].clone();
                        m["kind"] = json!("Beam");
                        m["native_beam_id"] = selected["beam_id"].clone();
                        m["native_linker_id"] = selected["linker_id"].clone();
                        m["beam_median"] = sibling["beam"]["median"].clone();
                        m["beam_height"] = sibling["beam"]["height"].clone();
                        metas.push(m);
                        siblings.push(*sibling);
                    }
                    let sibling_linkers = metas.clone();
                    siblings.sort_by(|a, b| {
                        a["beam"]["median"][1]
                            .as_f64()
                            .unwrap()
                            .total_cmp(&b["beam"]["median"][1].as_f64().unwrap())
                    });
                    let edge = if dir > 0 {
                        siblings.last().unwrap()
                    } else {
                        siblings.first().unwrap()
                    };
                    let border = line(&edge["beam"][if dir > 0 { "bottom" } else { "top" }])?;
                    let y = border[1]
                        + (geo.theoretical_line[0] - border[0]) * (border[3] - border[1])
                            / (border[2] - border[0]);
                    for head in &heads {
                        let hb = bounds(&head["bounds"])?;
                        if !area.intersects_rect(hb.map(f64::from))?
                            || f64::from(dir) * ((hb[1] + hb[3] / 2) as f64 - y)
                                < il.round_ties_even()
                        {
                            continue;
                        }
                        if arr(&graph["relations"])?.iter().any(|e| {
                            e["kind"] == "Exclusion"
                                && siblings.iter().any(|b| {
                                    let actual = live
                                        .iter()
                                        .find(|n| n["origin"]["local"] == b["beam"]["id"])
                                        .unwrap();
                                    (e["source"] == actual["id"] && e["target"] == head["id"])
                                        || (e["target"] == actual["id"]
                                            && e["source"] == head["id"])
                                })
                        }) {
                            continue;
                        }
                        if !matches!(
                            head["shape"].as_str(),
                            Some("NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
                        ) {
                            return Err("beam builder other head shapes pending".into());
                        }
                        for hori in ["LEFT", "RIGHT"] {
                            if head["shape"] == "NOTEHEAD_VOID"
                                && hori != if dir > 0 { "RIGHT" } else { "LEFT" }
                            {
                                continue;
                            }
                            let ci = all
                                .iter()
                                .position(|c| {
                                    c["head"]["head_id"] == head["id"]
                                        && c["head"]["side"] == hori
                                        && c["head"]["vertical"]
                                            == if dir > 0 { "TOP" } else { "BOTTOM" }
                                })
                                .ok_or("beam builder head corner required")?;
                            let hd = &all[ci]["head"];
                            let reference = serde_json::from_value(hd["reference"].clone())
                                .map_err(|_| "beam builder head reference required")?;
                            if area.contains(reference)? {
                                let mut m = hd.clone();
                                m["kind"] = json!("HeadCorner");
                                m["stump"] = sources[ci]["stump"].clone();
                                metas.push(m);
                            }
                        }
                    }
                    let registry = &mut o.prepared.registry;
                    let start_id = canonical(registry, &b["stump"])?;
                    let seed_ids = candidates
                        .iter()
                        .copied()
                        .filter_map(|id| {
                            match registry.get(id).and_then(|g| {
                                area.intersects_rect([
                                    g.glyph.left as f64,
                                    g.glyph.top as f64,
                                    g.glyph.width as f64,
                                    g.glyph.height as f64,
                                ])
                            }) {
                                Ok(true) => Some(Ok(id)),
                                Ok(false) => None,
                                Err(e) => Some(Err(e)),
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let ids = metas
                        .iter()
                        .map(|m| canonical(registry, &m["stump"]))
                        .collect::<Result<Vec<_>, _>>()?;
                    let ps = seed_ids
                        .iter()
                        .map(|id| {
                            Ok(Part {
                                id: *id,
                                glyph: &registry.get(*id)?.glyph,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    let start = start_id
                        .map(|id| {
                            registry.get(id).map(|g| Part {
                                id,
                                glyph: &g.glyph,
                            })
                        })
                        .transpose()?;
                    let links = metas
                        .iter()
                        .zip(&ids)
                        .enumerate()
                        .map(|(i, (m, id))| {
                            Ok(stem_parts_filter::Linker {
                                id: i + 1,
                                kind: if m["kind"] == "Beam" {
                                    stem_parts_filter::Kind::Beam
                                } else {
                                    stem_parts_filter::Kind::HeadCorner
                                },
                                stump: id
                                    .map(|id| {
                                        registry.get(id).map(|g| Part {
                                            id,
                                            glyph: &g.glyph,
                                        })
                                    })
                                    .transpose()?,
                                reference: serde_json::from_value(m["reference"].clone())
                                    .map_err(|_| "beam target reference required")?,
                                beam_height: m["beam_height"].as_f64(),
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    let range = stem_length_map::line_bounds(geo.theoretical_line)?;
                    let alignment =
                        stem_parts_alignment::Parameters::new(dir, il * 0.15, il * 4., 0.)?;
                    let filtered = stem_parts_filter::filter(
                        &ps,
                        start,
                        &links,
                        &stem_parts_filter::Parameters {
                            alignment: alignment.clone(),
                            y_range: range,
                            start_is_beam: true,
                            head_box: start.map(|p| {
                                [
                                    p.glyph.left,
                                    p.glyph.top,
                                    p.glyph.width as i32,
                                    p.glyph.height as i32,
                                ]
                            }),
                        },
                    )?;
                    let raw = stem_parts_lookup::collect(
                        &vertical,
                        &horizontal,
                        &geo.polygon,
                        start.map(|p| p.glyph),
                        &stem_parts_lookup::Parameters {
                            direction: dir,
                            slope: 0.,
                            max_stem_thickness: max,
                            max_line_section_dx: il * 0.3,
                            last_head_y: filtered.last_head_y,
                            theoretical_line: geo.theoretical_line,
                        },
                    )?;
                    let raw_gs = raw
                        .raw_glyphs
                        .iter()
                        .map(beam_dispatch::glyph_from_json)
                        .collect::<Result<Vec<_>, _>>()?;
                    let chunks = stem_chunks::select(
                        registry,
                        &raw_gs,
                        &filtered.retained_part_order,
                        start_id,
                        None,
                        &alignment,
                    )?;
                    let mut srcs = vec![Source {
                        token: 1,
                        kind: Kind::HalfLinkerItem,
                        glyph: (!b["stump"].is_null()).then(|| b["stump"].clone()),
                        reference: Some(
                            serde_json::from_value(b["reference"].clone())
                                .map_err(|_| "beam start reference required")?,
                        ),
                        beam_height: None,
                        head_bounds: None,
                    }];
                    let mut origins =
                        vec![json!({"kind":"StartingBeam","native_beam_id":cat["beam"]["id"]})];
                    for item in &filtered.items {
                        let m = item.linker_id.map(|id| &metas[id - 1]);
                        srcs.push(Source {
                            token: srcs.len() + 1,
                            kind: if m.is_some_and(|m| m["kind"] == "Beam") {
                                Kind::LinkerItem
                            } else if m.is_some() {
                                Kind::HalfLinkerItem
                            } else {
                                Kind::GlyphItem
                            },
                            glyph: item
                                .glyph_id
                                .map(|id| {
                                    registry
                                        .get(id)
                                        .map(|g| beam_dispatch::glyph_json(&g.glyph))
                                })
                                .transpose()?,
                            reference: m
                                .map(|m| {
                                    serde_json::from_value(m["reference"].clone())
                                        .map_err(|_| "beam item reference required")
                                })
                                .transpose()?,
                            beam_height: m.and_then(|m| m["beam_height"].as_f64()),
                            head_bounds: m
                                .filter(|m| m["kind"] == "HeadCorner")
                                .map(|m| bounds(&m["head_bounds"]))
                                .transpose()?,
                        });
                        origins.push(m.cloned().unwrap_or(Value::Null));
                    }
                    for id in chunks.retained_ids {
                        srcs.push(Source {
                            token: srcs.len() + 1,
                            kind: Kind::GlyphItem,
                            glyph: Some(beam_dispatch::glyph_json(&registry.get(id)?.glyph)),
                            reference: None,
                            beam_height: None,
                            head_bounds: None,
                        });
                        origins.push(json!({"kind":"Chunk","native_glyph_id":id}));
                    }
                    let profile = if b["side"].is_null() { 1 } else { 4 };
                    let events = stem_item_events::events(
                        &stem_item_events::construct(&srcs, range)?,
                        dir,
                        gap_map[&profile],
                    )?;
                    let items=events.items.iter().map(|e|{let t=e.token;let g=t.and_then(|t|srcs[t-1].glyph.clone());let id=g.as_ref().map(|g|canonical(registry,g)).transpose()?.flatten();Ok(json!({"kind":e.kind,"gap":e.kind==Kind::GapItem,"contrib":e.contrib,"line":e.line,"head_bounds":e.head_bounds,"glyph":g,"glyph_id":id,"native_source":t.map(|t|origins[t-1].clone())}))}).collect::<Result<Vec<_>,String>>()?;
                    let length_items = events
                        .items
                        .iter()
                        .map(|e| Some(e.length_item()))
                        .collect::<Vec<_>>();
                    let lengths = stem_length_map::retrieve_lengths(
                        &length_items,
                        dir,
                        geo.theoretical_line[1],
                        &gap_map,
                        profile,
                        &BTreeMap::new(),
                    )?;
                    let actual = live
                        .iter()
                        .find(|n| n["origin"]["local"] == cat["beam"]["id"])
                        .unwrap();
                    let competing = crate::music_beams::competing_hook(&graph, actual);
                    let link_profile = if !b["side"].is_null()
                        && (actual["kind"] == "BeamHookInter" || competing.is_some())
                    {
                        0
                    } else {
                        profile
                    };
                    let empty_sides = arr(&cat["final_blinkers"])?
                        .iter()
                        .filter(|l| {
                            !l["side"].is_null()
                                && l["directions"].as_array().is_some_and(Vec::is_empty)
                        })
                        .map(|l| l["side"].clone())
                        .collect::<Vec<_>>();
                    cases.push(json!({"empty_side_blinkers":empty_sides,"hook":actual["kind"] == "BeamHookInter","competing_hook_id":competing,"native_beam_id":cat["beam"]["id"],"native_linker_id":bli+1,"sibling_linkers":sibling_linkers,"closer_beam_id":closer.map(|(id,_)|id),"closer_limit":lookup.limit,"side":b["side"],"vertical":side,"profile":link_profile,"inspect_profile":profile,"beam_median":cat["beam"]["median"],"beam_height":cat["beam"]["height"],"reference":b["reference"],"polygon":geo.polygon,"theoretical_line":geo.theoretical_line,"items":items,"gap_map":gap_map,"length_map":lengths}));
                }
            }
        }
        systems.push(cases);
    }
    Ok(systems)
}
/// Original VLinker.expand. Stop snapshots exclude the coming head glyph;
/// after an end head the allowed gap returns to STANDARD (zero).
pub(crate) fn expand(
    registry: &Registry,
    c: &Value,
    ctx: &Context<'_>,
) -> Result<Expansion, String> {
    let initial = line(&c["theoretical_line"])?;
    let dir = if c["vertical"] == "BOTTOM" { 1 } else { -1 };
    let mut l = if dir > 0 {
        initial
    } else {
        [initial[2], initial[3], initial[0], initial[1]]
    };
    let mut gap = c["gap_map"][c["profile"].as_u64().unwrap().to_string()]
        .as_i64()
        .ok_or("beam expand gap required")?;
    let items = arr(&c["items"])?;
    let mut ids = Vec::new();
    let mut relations = Vec::new();
    let mut stopping: Option<(isize, Vec<usize>)> = None;
    for (i, item) in items.iter().enumerate() {
        if item["gap"] == true && item["contrib"].as_i64().unwrap() > gap {
            let (last, ids) = stopping.unwrap_or((-1, ids));
            return Ok(Expansion {
                last,
                ids,
                relations,
                line: l,
            });
        }
        let hd = &item["native_source"];
        if hd["kind"] == "HeadCorner" {
            if let Some((last, stopped)) = &stopping
                && let Some(ev) = items[..i].iter().rev().find(|e| e["gap"] == true)
            {
                let gl = line(&ev["line"])?;
                let y = hd["reference"][1].as_f64().unwrap();
                let dy = if dir > 0 { y - gl[3] } else { gl[1] - y };
                let opposite = ctx
                    .all
                    .iter()
                    .find(|c| {
                        c["head"]["head_id"] == hd["head_id"]
                            && c["head"]["side"] != hd["side"]
                            && c["head"]["vertical"] != hd["vertical"]
                    })
                    .ok_or("beam opposite corner required")?;
                if dy < (ctx.il as f64 * 0.85).round_ties_even()
                    && opposite["builder_items"]["length_map"]["1"]
                        .as_i64()
                        .unwrap()
                        >= (ctx.il as f64 * 0.85).round_ties_even() as i64
                {
                    return Ok(Expansion {
                        last: *last,
                        ids: stopped.clone(),
                        relations,
                        line: l,
                    });
                }
            }
            let Some(rel) = ctx.head_relation(hd, &hd["stump"], l)? else {
                continue;
            };
            if rel["head_side"] == if dir > 0 { "RIGHT" } else { "LEFT" } && !ids.is_empty() {
                let gs = ids
                    .iter()
                    .map(|id| registry.get(*id).map(|e| e.glyph.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                let g = if gs.len() == 1 {
                    gs[0].clone()
                } else {
                    stem_glyph::assemble(&gs)?
                };
                let portion = crate::head_stem_relation::stem_portion(
                    bounds(&hd["head_bounds"])?,
                    crate::stem_glyph::geometry(&g)?.median,
                    rel["extension_point"][1].as_f64().unwrap(),
                )?;
                if portion
                    == if dir > 0 {
                        crate::head_stem_relation::StemPortion::StemBottom
                    } else {
                        crate::head_stem_relation::StemPortion::StemTop
                    }
                {
                    stopping = Some((i as isize, ids.clone()));
                    gap = 0;
                }
            }
            relations.push(json!({"linker":hd,"relation":rel}));
        }
        update(
            registry,
            item["glyph_id"].as_u64().map(|id| id as usize),
            &mut ids,
            &mut l,
            None,
        )?;
    }
    Ok(Expansion {
        last: items.len() as isize - 1,
        ids,
        relations,
        line: l,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_reverse_actual_bounds_width_keeps_fractional_median_ties_stable() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-subdivisions/width-order-controls.json"
        ))
        .unwrap();
        let bs = o["beams"].as_array().unwrap();
        let catalogs = bs
            .iter()
            .map(|b| json!({"beam":{"id":b["id"],"median":b["median"]}}))
            .collect::<Vec<_>>();
        let actual = bs
            .iter()
            .map(|b| json!({"origin":{"local":b["id"]},"bounds":b["bounds"]}))
            .collect::<Vec<_>>();
        let mut ordered = catalogs.iter().collect::<Vec<_>>();
        order_by_width(&mut ordered, &actual.iter().collect::<Vec<_>>()).unwrap();
        assert_eq!(
            json!(
                ordered
                    .iter()
                    .map(|b| b["beam"]["id"].clone())
                    .collect::<Vec<_>>()
            ),
            o["sorted_ids"]
        );
    }
}
