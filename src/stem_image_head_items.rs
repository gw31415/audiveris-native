// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual image head filter/chunks -> final StemBuilder items/gaps/lengths.
//! No caller-defined item source or endpoint; stem/glyph/cache/SIG pending.
use crate::{
    GrayImage, beam_dispatch, stem_image_head_chunks, stem_image_stumps,
    stem_item_events::{self, Kind, Source},
    stem_length_map,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|a| a.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image head items {label}"))
}
fn glyph(v: &Value) -> Option<Value> {
    (!v.is_null()).then(|| v.clone())
}
fn reference(v: &Value) -> Result<[f64; 2], String> {
    serde_json::from_value(v.clone()).map_err(|_| "invalid image builder item reference".into())
}
/// Same current physical registry retained. Original default XML book profile1
/// is the existing image CLI context; other profiles are not inferred from JSON.
pub fn prepare(image: &GrayImage, page: &Value) -> Result<stem_image_stumps::Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<crate::beam_dispatch::Registry>,
) -> Result<stem_image_stumps::Prepared, String> {
    let mut owned = stem_image_head_chunks::prepare_with_registry(image, page, registry)?;
    let plan = owned.report.clone();
    let parts = &plan["head_parts_plan"];
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let il = scale.interline.main as f64;
    let gap_map: [f64; 5] = [0., 0.3, 0.6, 2., 4.];
    let gap_map: BTreeMap<i32, i32> = gap_map
        .into_iter()
        .enumerate()
        .map(|(p, f)| (p as i32, (il * f).round_ties_even() as i32))
        .collect();
    let profile = 1;
    let mut systems = Vec::new();
    let (mut item_count, mut gap_count, mut truncated_count) = (0, 0, 0);
    for (si, sys) in rows(&plan["systems"], 64, "systems")?.iter().enumerate() {
        let cs = rows(&sys["cases"], 16384, "chunk cases")?;
        let ps = rows(&parts["systems"][si]["cases"], 16384, "part cases")?;
        if cs.len() != ps.len() {
            return Err("inconsistent image builder item count".into());
        }
        let mut out = Vec::new();
        for (c, p) in cs.iter().zip(ps) {
            if c["head"] != p["head"] {
                return Err("inconsistent image builder item identity".into());
            }
            let bounds: [i32; 4] = serde_json::from_value(c["head"]["head_bounds"].clone())
                .map_err(|_| "invalid image starting head bounds")?;
            let start = Source {
                token: 1,
                kind: Kind::HalfLinkerItem,
                glyph: glyph(&p["start_stump"]),
                reference: Some(reference(&c["head"]["reference"])?),
                beam_height: None,
                head_bounds: Some(bounds),
            };
            let mut sources = vec![start];
            let mut origins = vec![json!({"kind":"StartingHeadCorner","head":c["head"]})];
            for i in rows(&p["pre_chunk_items"], 32, "filtered items")? {
                let kind: Kind = serde_json::from_value(i["kind"].clone())
                    .map_err(|_| "invalid image item kind")?;
                let m = if let Some(index) = i["linker_index"].as_u64() {
                    p["selected_linkers"]
                        .get(index as usize - 1)
                        .ok_or("invalid image item target")?
                } else {
                    &Value::Null
                };
                let hb = if kind == Kind::HalfLinkerItem {
                    Some(
                        serde_json::from_value(m["head_bounds"].clone())
                            .map_err(|_| "invalid target head bounds")?,
                    )
                } else {
                    None
                };
                let source = Source {
                    token: sources.len() + 1,
                    kind,
                    glyph: glyph(&i["glyph"]),
                    reference: if kind == Kind::GlyphItem {
                        None
                    } else {
                        Some(reference(&m["reference"])?)
                    },
                    beam_height: m["beam_height"].as_f64(),
                    head_bounds: hb,
                };
                sources.push(source);
                origins.push(m.clone());
            }
            for (id, g) in rows(&c["chunk_lookup"]["retained_native_ids"], 4096, "chunk ids")?
                .iter()
                .zip(rows(&c["chunk_lookup"]["retained_chunks"], 4096, "chunks")?)
            {
                sources.push(Source {
                    token: sources.len() + 1,
                    kind: Kind::GlyphItem,
                    glyph: Some(g.clone()),
                    reference: None,
                    beam_height: None,
                    head_bounds: None,
                });
                origins.push(json!({"kind":"Chunk","native_glyph_id":id}));
            }
            let range: [i32; 4] = serde_json::from_value(p["builder_y_range"].clone())
                .map_err(|_| "invalid image builder yRange")?;
            let prepared = stem_item_events::construct(&sources, range)?;
            let line: [f64; 4] = serde_json::from_value(p["theoretical_line"].clone())
                .map_err(|_| "invalid image builder line")?;
            let dir = if line[3] > line[1] { 1 } else { -1 };
            let events = stem_item_events::events(&prepared, dir, gap_map[&profile])?;
            let native = events
                .items
                .iter()
                .map(|i| Some(i.length_item()))
                .collect::<Vec<_>>();
            let prefix_lengths = stem_length_map::prefix_lengths(&native, dir, line[1])?;
            let lengths = stem_length_map::retrieve_lengths(
                &native,
                dir,
                line[1],
                &gap_map,
                profile,
                &BTreeMap::new(),
            )?;
            let items=events.items.iter().map(|i|{let(source_glyph,origin)=if let Some(t)=i.token{(sources[t-1].glyph.clone(),origins[t-1].clone())}else{(None,Value::Null)};let glyph_id=if let Some(g)=&source_glyph{let before=owned.registry.len();let id=owned.registry.register(&beam_dispatch::glyph_from_json(g)?)?;if owned.registry.len()!=before{return Err("unregistered image final item glyph".into());}Some(id)}else{None};Ok(json!({"kind":i.kind,"gap":i.kind==Kind::GapItem,"contrib":i.contrib,"line":i.line,"line_bounds":stem_length_map::line_bounds(i.line)?,"head_bounds":i.head_bounds,"reference":i.reference,"glyph":source_glyph,"glyph_id":glyph_id,"source_token":i.token,"native_source":origin}))}).collect::<Result<Vec<Value>,String>>()?;
            item_count += items.len();
            gap_count += items.iter().filter(|i| i["gap"] == true).count();
            truncated_count += usize::from(events.truncated);
            out.push(json!({"head":c["head"],"builder_items":{"items":items,"prefix_lengths":prefix_lengths,"length_map":lengths,"profile":profile,"gap_map":gap_map,"sorted_source_tokens":events.sorted_tokens,"truncated":events.truncated}}));
        }
        systems.push(json!({"system":sys["system"],"cases":out}));
    }
    owned.report = json!({"schema":"audiveris-native.stem-image-head-items.v1","head_chunks_plan":plan,"systems":systems,"corner_count":plan["corner_count"],"final_head_item_count":item_count,"gap_item_count":gap_count,"truncated_head_builders":truncated_count,"head_builder_items_stage_complete":true,"head_builder_context_profile":profile,"automatic_raster_pipeline_complete":false,"final_stem_glyph_selection_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["complete beam-origin builders and automatic endpoint/physical glyph/grade selection","actual upstream owned Graph/cache/head/cleaner/epilog","default XML-restored profile1/zero skew; transient/scans/special/full24pages unverified"]});
    Ok(owned)
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}
