// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image-selected head/beam linker and seed inputs -> actual head builder filter.
//! Before lookupChunks/items/gaps/final glyph and owned graph/cache/epilog.
use crate::{
    GrayImage, beam_dispatch, stem_image_beam_generation, stem_image_stumps, stem_length_map,
    stem_parts_alignment, stem_parts_filter,
};
use serde_json::{Value, json};
fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|a| a.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid head parts {label}"))
}
fn canonical(owned: &mut stem_image_stumps::Prepared, v: &Value) -> Result<Option<usize>, String> {
    if v.is_null() {
        return Ok(None);
    }
    let g = beam_dispatch::glyph_from_json(v)?;
    let before = owned.registry.len();
    let id = owned.registry.register(&g)?;
    if before != owned.registry.len() {
        return Err("unregistered image head builder part".into());
    }
    Ok(Some(id))
}
/// Uses the same retained registry, current CLinker seed/other-head/beam choices.
/// No caller-supplied filter cases and no original oracle at runtime.
pub fn prepare(image: &GrayImage, page: &Value) -> Result<stem_image_stumps::Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<crate::beam_dispatch::Registry>,
) -> Result<stem_image_stumps::Prepared, String> {
    let mut owned = stem_image_beam_generation::prepare_with_registry(image, page, registry)?;
    let plan = owned.report.clone();
    let heads = &plan["beam_target_plan"]["head_linker_plan"];
    let source = &heads["source_plan"];
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let il = scale.interline.main as f64;
    let mut systems = Vec::new();
    let mut total_linkers = 0;
    let mut kept_linkers = 0;
    let mut total_seeds = 0;
    let mut total_items = 0;
    for (si, sys) in rows(&plan["systems"], 64, "systems")?.iter().enumerate() {
        let cs = rows(&sys["cases"], 16384, "head cases")?;
        let sc = rows(&source["systems"][si]["cases"], 16384, "source cases")?;
        let hc = rows(&heads["systems"][si]["cases"], 16384, "other-head cases")?;
        if cs.len() != sc.len() || cs.len() != hc.len() {
            return Err("inconsistent image head parts count".into());
        }
        let mut out = Vec::new();
        for ((c, s), h) in cs.iter().zip(sc).zip(hc) {
            if c["head"] != s["head"] || c["head"] != h["head"] {
                return Err("inconsistent image head parts identity".into());
            }
            let line: [f64; 4] = serde_json::from_value(s["theoretical_line"].clone())
                .map_err(|_| "invalid head builder line")?;
            let y_range = stem_length_map::line_bounds(line)?;
            let direction = if line[3] > line[1] { 1 } else { -1 };
            let stump_id = canonical(&mut owned, &s["stump"])?;
            let seed_ids: Vec<usize> = serde_json::from_value(s["selected_seed_ids"].clone())
                .map_err(|_| "invalid head builder seed IDs")?;
            if seed_ids.len() > 4096 {
                return Err("head builder seed budget".into());
            }
            let mut metadata = Vec::new();
            let mut identities = Vec::new();
            for l in rows(
                &h["selected_other_head_linkers"],
                2048,
                "head linker candidates",
            )? {
                let id = canonical(&mut owned, &l["stump"])?;
                identities.push((stem_parts_filter::Kind::HeadCorner, id));
                metadata.push(json!({"kind":"HeadCorner","head_bounds":l["head_bounds"],"shape":l["shape"],"side":l["side"],"vertical":l["vertical"],"beam_height":null,"reference":l["reference"],"stump":l["stump"],"native_head_origin":l["head_origin"]}));
            }
            for l in rows(&c["selected_blinkers"], 2048, "beam linker candidates")? {
                let beam = rows(&sys["beam_catalog"], 2048, "beam catalog")?
                    .iter()
                    .find(|b| b["beam"]["id"] == l["beam_id"])
                    .ok_or("missing head builder beam")?;
                let id = canonical(&mut owned, &l["linker"]["stump"])?;
                identities.push((stem_parts_filter::Kind::Beam, id));
                metadata.push(json!({"kind":"Beam","beam_median":beam["beam"]["median"],"beam_height":beam["beam"]["height"],"reference":l["linker"]["reference"],"stump":l["linker"]["stump"],"native_beam_id":l["beam_id"],"native_linker_id":l["linker_id"]}));
            }
            let parts = seed_ids
                .iter()
                .map(|id| {
                    Ok(stem_parts_alignment::Part {
                        id: *id,
                        glyph: &owned.registry.get(*id)?.glyph,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let start = stump_id
                .map(|id| {
                    owned.registry.get(id).map(|e| stem_parts_alignment::Part {
                        id,
                        glyph: &e.glyph,
                    })
                })
                .transpose()?;
            let links = identities
                .iter()
                .zip(&metadata)
                .enumerate()
                .map(|(i, ((kind, id), m))| {
                    let stump = id
                        .map(|id| {
                            owned.registry.get(id).map(|e| stem_parts_alignment::Part {
                                id,
                                glyph: &e.glyph,
                            })
                        })
                        .transpose()?;
                    Ok(stem_parts_filter::Linker {
                        id: i + 1,
                        kind: *kind,
                        stump,
                        reference: serde_json::from_value(m["reference"].clone())
                            .map_err(|_| "invalid head builder linker reference")?,
                        beam_height: m["beam_height"].as_f64(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let head_box = start.map(|p| {
                [
                    p.glyph.left,
                    p.glyph.top,
                    p.glyph.width as i32,
                    p.glyph.height as i32,
                ]
            });
            let filtered = stem_parts_filter::filter(
                &parts,
                start,
                &links,
                &stem_parts_filter::Parameters {
                    alignment: stem_parts_alignment::Parameters::new(
                        direction,
                        il * 0.15,
                        il * 4.,
                        0.,
                    )?,
                    y_range,
                    start_is_beam: false,
                    head_box,
                },
            )?;
            let retained = filtered
                .retained_part_order
                .iter()
                .map(|id| {
                    owned
                        .registry
                        .get(*id)
                        .map(|e| beam_dispatch::glyph_json(&e.glyph))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let items=filtered.items.iter().map(|i|{let g=i.glyph_id.map(|id|owned.registry.get(id).map(|e|beam_dispatch::glyph_json(&e.glyph))).transpose()?;Ok(json!({"kind":i.kind,"contrib":i.contrib,"line":i.line,"glyph":g,"linker_index":i.linker_id}))}).collect::<Result<Vec<_>,String>>()?;
            total_seeds += seed_ids.len();
            total_linkers += links.len();
            kept_linkers += filtered.retained_linker_order.len();
            total_items += items.len();
            out.push(json!({"head":c["head"],"theoretical_line":line,"builder_y_range":y_range,"start_stump":s["stump"],"initial_seeds":s["selected_seeds"],"selected_linkers":metadata,"retained_seeds":retained,"retained_linker_indices":filtered.retained_linker_order,"last_head_y":filtered.last_head_y,"pre_chunk_items":items}));
        }
        systems.push(json!({"system":sys["system"],"cases":out}));
    }
    owned.report = json!({"schema":"audiveris-native.stem-image-head-parts.v1","beam_generation_plan":plan,"systems":systems,"corner_count":plan["corner_count"],"initial_seed_occurrences":total_seeds,"initial_linker_occurrences":total_linkers,"retained_linker_occurrences":kept_linkers,"pre_chunk_item_count":total_items,"head_builder_pre_chunk_inputs_complete":true,"automatic_raster_pipeline_complete":false,"final_stem_glyph_selection_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["complete beam-origin geometry/seeds/head builders","automatic section/chunk registration/items/gaps/length/final glyph","actual owned liveGraph/cache/head/cleaner/epilog; original transient/skew/scans/special/full24pages"]});
    Ok(owned)
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}
