// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image-derived head CLinker expand/createStem at the inspection boundary.
//! No selected endpoint/parts input. This is not the original linkStems scheduler.
use crate::{
    GrayImage,
    beam_dispatch::{self, Registry},
    head_stem_relation::{self, ConnectionInput},
    stem_glyph,
    stem_image_owned::OwnedItems,
};
use audiveris_staff_removal::{GlyphPixels, Orientation};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn arr(v: &Value) -> Result<&[Value], String> {
    v.as_array()
        .map(Vec::as_slice)
        .ok_or("invalid image expansion array".into())
}
pub(crate) fn line(v: &Value) -> Result<[f64; 4], String> {
    serde_json::from_value(v.clone()).map_err(|_| "invalid image expansion line".into())
}
pub(crate) fn bounds(v: &Value) -> Result<[i32; 4], String> {
    serde_json::from_value(v.clone()).map_err(|_| "invalid image expansion bounds".into())
}
fn centroid(g: &GlyphPixels) -> Result<[f64; 2], String> {
    beam_dispatch::centroid(g)?;
    let (mut n, mut sx, mut sy) = (0i64, 0i64, 0i64);
    for r in &g.runs {
        let k = r.length as i64;
        let a = k * (2 * r.start as i64 + k - 1) / 2;
        let b = k * r.position as i64;
        n += k;
        if g.orientation == Orientation::Vertical {
            sx += b;
            sy += a
        } else {
            sx += a;
            sy += b
        }
    }
    Ok([
        g.left as f64 + sx as f64 / n as f64,
        g.top as f64 + sy as f64 / n as f64,
    ])
}
/// Append only a new canonical physical glyph; line translation uses the union
/// centroid, not an average of part centroids (overlap must count once).
fn update(
    registry: &Registry,
    id: Option<usize>,
    ids: &mut Vec<usize>,
    l: &mut [f64; 4],
    max_dx: Option<f64>,
) -> Result<bool, String> {
    let Some(id) = id else { return Ok(true) };
    if ids.contains(&id) {
        return Ok(true);
    }
    let g = &registry.get(id)?.glyph;
    if let Some(max) = max_dx
        && !ids.is_empty()
    {
        let c = centroid(g)?;
        let x = crate::stem_checks::x_at_y([l[0], l[1]], [l[2], l[3]], c[1]);
        if (c[0] - x).abs() > max {
            return Ok(false);
        }
    }
    let mut next = ids.clone();
    next.push(id);
    let gs = next
        .iter()
        .map(|id| registry.get(*id).map(|g| g.glyph.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let union = if gs.len() == 1 {
        gs[0].clone()
    } else {
        stem_glyph::assemble(&gs)?
    };
    let c = centroid(&union)?;
    let dx = c[0] - crate::stem_checks::x_at_y([l[0], l[1]], [l[2], l[3]], c[1]);
    l[0] += dx;
    l[2] += dx;
    *ids = next;
    Ok(true)
}
fn beam_relation(
    m: &Value,
    l: [f64; 4],
    dir: i32,
    il: usize,
    thickness: usize,
) -> Result<Option<Value>, String> {
    let mut b = line(&m["beam_median"])?;
    let h = m["beam_height"]
        .as_f64()
        .ok_or("missing expansion beam height")?;
    b[1] -= dir as f64 * h / 2.;
    b[3] -= dir as f64 * h / 2.;
    let den = (l[0] - l[2]) * (b[1] - b[3]) - (l[1] - l[3]) * (b[0] - b[2]);
    if den == 0. {
        return Err("unsupported parallel expansion beam".into());
    }
    let a = l[0] * l[3] - l[1] * l[2];
    let q = b[0] * b[3] - b[1] * b[2];
    let x = (a * (b[0] - b[2]) - (l[0] - l[2]) * q) / den;
    let y = (a * (b[1] - b[3]) - (l[1] - l[3]) * q) / den;
    let median = line(&m["beam_median"])?;
    let margin = (il as f64 * 0.5).round_ties_even();
    let portion = if x < median[0] + margin {
        "LEFT"
    } else if x > median[2] - margin {
        "RIGHT"
    } else {
        "CENTER"
    };
    let dx = match portion {
        "LEFT" => b[0] + thickness as f64 / 2. - x,
        "RIGHT" => x - b[2] + thickness as f64 / 2.,
        _ => 0.,
    } / il as f64;
    let dy = (l[1] - y).max(0.).max((y - l[3]).max(0.)) / il as f64;
    // AbstractConnection.setOutGaps preserves the absolute horizontal gap.
    let dx = dx.abs();
    let grade = (((0.2 - dx) / 0.2).clamp(0., 1.) * ((1.2 - dy) / 1.2).clamp(0., 1.).powf(4.))
        .powf(1. / 5.);
    Ok((grade>=0.1).then(||json!({"kind":"BeamStemRelation","grade":grade,"dx":dx,"dy":dy,"beam_portion":portion,"source_ratio":1.+4.*grade,"target_ratio":1.+if portion=="CENTER"{3.*grade}else{10.*grade},"extension_point":[x,y+dir as f64*(h-1.)]})))
}
pub(crate) struct Context<'a> {
    pub(crate) bank: Bank,
    pub(crate) profile: u8,
    pub(crate) point_size: u16,
    pub(crate) il: usize,
    pub(crate) thickness: usize,
    pub(crate) dx: BTreeMap<String, [Option<f64>; 2]>,
    pub(crate) heads: BTreeMap<String, f64>,
    pub(crate) all: &'a [Value],
}
impl Context<'_> {
    fn head_relation(
        &self,
        h: &Value,
        stump: &Value,
        l: [f64; 4],
    ) -> Result<Option<Value>, String> {
        let shape = h["shape"].as_str().ok_or("invalid expansion head shape")?;
        head_stem_relation::check_relation(
            &self.bank,
            self.point_size,
            ConnectionInput {
                shape,
                bounds: bounds(&h["head_bounds"])?,
                stem: l,
                glyph_bounds: if stump.is_null() {
                    None
                } else {
                    Some(bounds(&stump["bounds"])?)
                },
                vertical: serde_json::from_value(h["vertical"].clone())
                    .map_err(|_| "invalid expansion vertical")?,
                interline: self.il,
                profile: 1,
                learned_dx: self.dx.get(shape).copied().unwrap_or([None, None]),
            },
        )
    }
    fn concrete_opposite(&self, h: &Value, min: i32) -> Result<bool, String> {
        let side = if h["side"] == "LEFT" { "RIGHT" } else { "LEFT" };
        let c = self
            .all
            .iter()
            .find(|c| {
                c["head"]["head_bounds"] == h["head_bounds"]
                    && c["head"]["shape"] == h["shape"]
                    && c["head"]["side"] == side
                    && c["head"]["vertical"] == h["vertical"]
            })
            .ok_or("missing opposite image head builder")?;
        Ok(c["builder_items"]["length_map"]["1"]
            .as_i64()
            .ok_or("missing expansion concrete length")?
            >= min as i64)
    }
}
fn link_metadata(h: &Value, stump: &Value) -> Value {
    json!({"kind":"HeadCorner","head_bounds":h["head_bounds"],"shape":h["shape"],"side":h["side"],"vertical":h["vertical"],"beam_height":null,"reference":h["reference"],"stump":stump})
}
pub(crate) struct Expansion {
    pub(crate) last: isize,
    pub(crate) ids: Vec<usize>,
    pub(crate) relations: Vec<Value>,
    pub(crate) line: [f64; 4],
}
pub(crate) fn expand(
    registry: &Registry,
    c: &Value,
    p: &Value,
    ctx: &Context<'_>,
) -> Result<Expansion, String> {
    let head = &c["head"];
    let items = arr(&c["builder_items"]["items"])?;
    if items.is_empty() || items.len() > 4096 {
        return Err("unsupported expansion item count".into());
    }
    let initial = line(&p["theoretical_line"])?;
    if initial[1] == initial[3] || initial.iter().any(|x| !x.is_finite()) {
        return Err("unsupported expansion theoretical line".into());
    }
    let dir = if initial[3] > initial[1] { 1 } else { -1 };
    let mut l = if dir > 0 {
        initial
    } else {
        [initial[2], initial[3], initial[0], initial[1]]
    };
    let mut last_y = initial[1];
    let reference = head["reference"][1]
        .as_f64()
        .ok_or("missing expansion reference")?;
    let min = (ctx.il as f64 * 0.85).round_ties_even() as i32;
    let hard = reference + dir as f64 * (ctx.il as f64 * 1.75).round_ties_even();
    let mut soft = reference + dir as f64 * (ctx.il as f64 * 2.5).round_ties_even();
    let max_gap = c["builder_items"]["gap_map"][ctx.profile.to_string()]
        .as_i64()
        .ok_or("missing expansion gap")?;
    let target_beam = items
        .iter()
        .rfind(|i| {
            i["native_source"]["kind"] == "Beam" || i["native_source"]["kind"] == "HeadCorner"
        })
        .is_some_and(|i| i["native_source"]["kind"] == "Beam");
    let mut ids = Vec::new();
    let mut relations = Vec::new();
    let mut last = None;
    for (i, item) in items.iter().enumerate() {
        let id = item["glyph_id"].as_u64().map(|n| n as usize);
        let m = &item["native_source"];
        let kind = m["kind"].as_str();
        if item["gap"] == true {
            if item["contrib"]
                .as_i64()
                .ok_or("missing expansion gap contribution")?
                > max_gap
            {
                last = Some(if dir as f64 * (last_y - hard) < 0. {
                    -1
                } else {
                    i as isize - 1
                });
                break;
            }
            if !target_beam && dir as f64 * (last_y - soft) >= 0. {
                if let Some(next) = items.get(i + 1)
                    && next["kind"] == "GlyphItem"
                    && update(
                        registry,
                        next["glyph_id"].as_u64().map(|n| n as usize),
                        &mut ids,
                        &mut l,
                        Some(ctx.il as f64 * 0.2),
                    )?
                {
                    last = Some(i as isize + 1)
                } else {
                    last = Some(i as isize - 1)
                }
                break;
            }
        } else if matches!(kind, Some("StartingHeadCorner" | "HeadCorner")) {
            let h = if kind == Some("StartingHeadCorner") {
                head
            } else {
                m
            };
            let stump = if kind == Some("StartingHeadCorner") {
                &p["start_stump"]
            } else {
                &m["stump"]
            };
            if kind == Some("HeadCorner")
                && let Some(g) = (0..i).rev().find(|&g| items[g]["gap"] == true)
            {
                let gap = line(&items[g]["line"])?;
                let y = h["reference"][1]
                    .as_f64()
                    .ok_or("missing target head reference")?;
                let dy = if dir > 0 { y - gap[3] } else { gap[1] - y };
                if dy < min as f64 && ctx.concrete_opposite(h, min)? {
                    last = Some(g as isize - 1);
                    break;
                }
            }
            let Some(rel) = ctx.head_relation(h, stump, l)? else {
                continue;
            };
            let meta = link_metadata(h, stump);
            if let Some(r) = relations
                .iter_mut()
                .find(|r: &&mut Value| r["linker"] == meta)
            {
                r["relation"] = rel.clone()
            } else {
                relations.push(json!({"linker":meta,"relation":rel}))
            }
            update(registry, id, &mut ids, &mut l, None)?;
            let grade = *ctx
                .heads
                .get(&json!([h["head_bounds"], h["shape"]]).to_string())
                .ok_or("missing actual expansion head grade")?;
            let contrib = 0.8 * rel["target_ratio"].as_f64().unwrap() - 1.;
            let cg = ((1. + contrib) * grade) / (1. + contrib * grade);
            if cg >= 0.1 {
                soft = h["reference"][1].as_f64().unwrap()
                    + dir as f64 * (ctx.il as f64 * 2.5).round_ties_even()
            }
        } else if kind == Some("Beam") {
            update(registry, id, &mut ids, &mut l, None)?;
            let rel = beam_relation(m, l, dir, ctx.il, ctx.thickness)?;
            let meta = m.clone();
            relations.push(json!({"linker":meta,"relation":rel}));
            // Original 5.11 reads sb.get(i), not sb.get(j), in its group lookahead.
            // Consequently any later item suppresses this stop. Preserve this behavior.
            if i + 1 == items.len() {
                last = Some(i as isize);
                break;
            }
        } else if item["kind"] == "GlyphItem"
            && !update(registry, id, &mut ids, &mut l, Some(ctx.il as f64 * 0.2))?
        {
            last = Some(i as isize - 1);
            break;
        }
        if item["gap"] != true {
            let il = line(&item["line"])?;
            last_y = if dir > 0 {
                last_y.max(il[3])
            } else {
                last_y.min(il[1])
            }
        }
    }
    let last = if let Some(last) = last {
        last
    } else if dir as f64 * (last_y - hard) < 0. {
        -1
    } else if let Some(rel) = ctx.head_relation(head, &p["start_stump"], l)? {
        let meta = link_metadata(head, &p["start_stump"]);
        if let Some(r) = relations.iter_mut().find(|r| r["linker"] == meta) {
            r["relation"] = rel
        } else {
            relations.push(json!({"linker":meta,"relation":rel}))
        };
        items.len() as isize - 1
    } else {
        -1
    };
    Ok(Expansion {
        last,
        ids,
        relations,
        line: if dir > 0 { l } else { initial },
    })
}
/// All decisions originate in the same native image owner; cache creation uses
/// its NO_STAFF raster. Selected stems are not yet registered as SIG vertices.
pub fn continue_expansion(mut owned: OwnedItems) -> Result<OwnedItems, String> {
    let items_report = owned.prepared.report.clone();
    let parts = &items_report["head_chunks_plan"]["head_parts_plan"];
    let raster = GrayImage::decode(&owned.upstream_output_pixels)?;
    let scale = &owned.upstream_report["stem_seeds"]["stem_scaling"]["stem_scale"];

    let il = owned.interline;
    let thickness = scale["main"]
        .as_u64()
        .ok_or("missing expansion stem thickness")? as usize;
    let max = scale["max"].as_u64().ok_or("missing expansion max stem")? as usize;
    let parameters = crate::stem_checks::Parameters::new(il, max, 0., 1)?;
    let mut dx = BTreeMap::new();
    for r in arr(&owned.upstream_report["head_registration"]["head_seed_scale"])? {
        let shape = r["shape"]
            .as_str()
            .ok_or("invalid expansion DX shape")?
            .to_owned();
        let slot = if r["side"] == "LEFT" { 0 } else { 1 };
        let values = dx.entry(shape).or_insert([None, None]);
        values[slot] = r["dx"].as_f64()
    }
    for values in dx.values_mut() {
        *values = crate::stem_head_reference_context::restored_dx(*values)?
    }
    let graph = owned.graph.snapshot();
    let heads = arr(&graph["inters"])?
        .iter()
        .filter(|h| h["kind"] == "HeadInter")
        .map(|h| {
            Ok((
                json!([h["bounds"], h["shape"]]).to_string(),
                h["grade"].as_f64().ok_or("missing image head grade")?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let mut systems = Vec::new();
    let mut accepted = 0;
    let mut count = 0;
    for (si, sys) in arr(&items_report["systems"])?.iter().enumerate() {
        let all = arr(&sys["cases"])?;
        let p = arr(&parts["systems"][si]["cases"])?;
        let sid = sys["system"].as_u64().ok_or("missing expansion system")?;
        let ctx = Context {
            profile: 1,
            bank: if owned.catalog.stored_point_size == 51 {
                Bank::load_all_head_coverage("Bravura").map_err(|e| e.to_string())?
            } else {
                owned.catalog.bank()?
            },
            point_size: owned.catalog.stored_point_size,
            il,
            thickness,
            dx: dx.clone(),
            heads: heads.clone(),
            all,
        };
        let mut order = (0..all.len()).collect::<Vec<_>>();
        // Match original stable head x order and HeadCorner enum TR,BL,TL,BR.
        order.sort_by_key(|&i| {
            let h = &all[i]["head"];
            let rank = match (h["side"].as_str(), h["vertical"].as_str()) {
                (Some("RIGHT"), Some("TOP")) => 0,
                (Some("LEFT"), Some("BOTTOM")) => 1,
                (Some("LEFT"), Some("TOP")) => 2,
                _ => 3,
            };
            (
                h["head_bounds"][0].as_i64().unwrap(),
                h["head_id"].as_u64().unwrap(),
                rank,
            )
        });
        let cache = owned
            .caches
            .get_mut(&sid)
            .ok_or("missing actual system cache")?;
        let mut cases = Vec::new();
        let mut cache_size = 0;
        for i in order {
            let c = &all[i];
            if c["head"] != p[i]["head"] {
                return Err("mismatched expansion builder identity".into());
            }
            let e = expand(&owned.prepared.registry, c, &p[i], &ctx)?;
            let gs = e
                .ids
                .iter()
                .map(|id| owned.prepared.registry.get(*id).map(|g| g.glyph.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            let mut created = Value::Null;
            let mut hit = false;
            let mut glyph_id = None;
            if e.last != -1 && !gs.is_empty() {
                let creation = cache.create_selected(
                    &mut owned.prepared.registry,
                    &gs,
                    &raster,
                    &parameters,
                )?;
                glyph_id = Some(creation.glyph_id);
                if let Some(stem) = creation.stem {
                    created = stem.snapshot()?;
                    hit = creation.cache_hit;
                    accepted += 1;
                    if !hit {
                        cache_size += 1
                    }
                }
            }
            let h = &c["head"];
            cases.push(json!({"head":h,"head_bounds":h["head_bounds"],"shape":h["shape"],"side":h["side"],"vertical":h["vertical"],"last_index":e.last,"selected_parts":gs.iter().map(beam_dispatch::glyph_json).collect::<Vec<_>>(),"selected_native_ids":e.ids,"expanded_theoretical_line":e.line,"relations":e.relations,"created":created,"cache_hit":hit,"registered_glyph_id":glyph_id}));
            count += 1;
        }
        systems.push(json!({"system":sid,"cache_size":cache_size,"cases":cases}));
    }
    owned.prepared.report = json!({"schema":"audiveris-native.stem-image-head-expansion.v1","head_items_plan":items_report,"systems":systems,"corner_count":count,"accepted_creations":accepted,"image_head_expansion_cache_stage_complete":true,"comparison_boundary":"XML-loaded inspection replay auto head expand/create profile1 before SIG linking and epilog; full original scheduler pending","final_stem_glyph_selection_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"automatic_raster_pipeline_complete":false});
    Ok(owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_update_line_overlap_duplicates_null_and_threshold_ulps() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-head-expansion/controls.json"
        ))
        .unwrap();
        for c in arr(&oracle["update_controls"]).unwrap() {
            let mut registry = Registry::default();
            let mut ids = Vec::new();
            let mut l = line(&c["initial_line"]).unwrap();
            for e in arr(&c["events"]).unwrap() {
                let id = if e["glyph"].is_null() {
                    None
                } else {
                    Some(
                        registry
                            .register(&beam_dispatch::glyph_from_json(&e["glyph"]).unwrap())
                            .unwrap(),
                    )
                };
                let accepted =
                    update(&registry, id, &mut ids, &mut l, e["max_dx"].as_f64()).unwrap();
                assert_eq!(accepted, e["accepted"].as_bool().unwrap());
                assert_eq!(
                    json!(
                        ids.iter()
                            .map(|id| beam_dispatch::glyph_json(&registry.get(*id).unwrap().glyph))
                            .collect::<Vec<_>>()
                    ),
                    e["selected"]
                );
                assert_eq!(json!(l), e["line"]);
            }
        }
    }
    #[test]
    fn unknown_physical_id_refuses_without_changing_selected_parts_or_line() {
        let registry = Registry::default();
        let mut ids = Vec::new();
        let mut l = [100., 90., 100., 140.];
        assert!(update(&registry, Some(1), &mut ids, &mut l, Some(2.)).is_err());
        assert!(ids.is_empty());
        assert_eq!(l, [100., 90., 100., 140.]);
        assert!(update(&registry, None, &mut ids, &mut l, None).unwrap());
        assert_eq!(l, [100., 90., 100., 140.]);
    }
}
