// SPDX-License-Identifier: AGPL-3.0-or-later
//! Image GRID + retained head filter seeds -> automatic lookupChunks.
//! Physical registration includes rejected chunks. No reconstructed live Graph.
use crate::{
    GrayImage, beam_dispatch, stem_chunks, stem_image_head_parts, stem_image_stumps,
    stem_parts_alignment, stem_parts_lookup, vertical_sticks::SeedSection,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|a| a.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image chunks {label}"))
}
pub(crate) fn sections(v: &Value, vertical: bool) -> Result<BTreeMap<u64, SeedSection>, String> {
    let mut out = BTreeMap::new();
    let mut runs = 0;
    for v in rows(v, 32768, "lag sections")? {
        let s = SeedSection {
            id: v["id"]
                .as_u64()
                .filter(|i| *i > 0)
                .ok_or("invalid image chunk section id")?,
            vertical,
            first_pos: v["first_pos"]
                .as_i64()
                .ok_or("invalid image chunk section position")?,
            runs: serde_json::from_value(v["runs"].clone())
                .map_err(|_| "invalid image chunk section runs")?,
        };
        if s.runs.is_empty() || s.runs.len() > 8192 {
            return Err("invalid image chunk section size".into());
        }
        stem_parts_lookup::section_centroid(&s)?;
        runs += s.runs.len();
        if runs > 1_048_576 || json!(s.bounds()) != v["bounds"] || out.insert(s.id, s).is_some() {
            return Err("invalid image chunk lag identity/bounds/budget".into());
        }
    }
    Ok(out)
}
pub(crate) fn members(
    all: &BTreeMap<u64, SeedSection>,
    v: &Value,
) -> Result<Vec<SeedSection>, String> {
    let mut seen = BTreeSet::new();
    rows(v, 32768, "system sections")?
        .iter()
        .map(|v| {
            let n = v.as_u64().ok_or("invalid image chunk member")?;
            if !seen.insert(n) {
                return Err("duplicate image chunk system section".into());
            }
            all.get(&n)
                .cloned()
                .ok_or("missing image chunk section".into())
        })
        .collect()
}
fn canonical(owned: &mut stem_image_stumps::Prepared, v: &Value) -> Result<Option<usize>, String> {
    if v.is_null() {
        return Ok(None);
    }
    let g = beam_dispatch::glyph_from_json(v)?;
    let before = owned.registry.len();
    let id = owned.registry.register(&g)?;
    if before != owned.registry.len() {
        return Err("unknown image head chunk upstream glyph".into());
    }
    Ok(Some(id))
}
/// Uses only current native image output, GRID section membership and retained
/// physical registry. Existing excludes exactly the filter-mutated seed list,
/// matching retrieveAllItems -> lookupChunks(parts), not every linker glyph.
pub fn prepare(image: &GrayImage, page: &Value) -> Result<stem_image_stumps::Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<crate::beam_dispatch::Registry>,
) -> Result<stem_image_stumps::Prepared, String> {
    let mut owned = stem_image_head_parts::prepare_with_registry(image, page, registry)?;
    let plan = owned.report.clone();
    let source =
        &plan["beam_generation_plan"]["beam_target_plan"]["head_linker_plan"]["source_plan"];
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let il = scale.interline.main as f64;
    let max = page["stem_seeds"]["stem_scaling"]["stem_scale"]["max"]
        .as_i64()
        .filter(|n| (1..=128).contains(n))
        .ok_or("invalid image chunk max stem thickness")?;
    let vertical = sections(&page["grid_ir"]["vlag"], true)?;
    let horizontal = sections(&page["grid_ir"]["hlag"], false)?;
    let grid = rows(&page["grid_ir"]["systems"], 64, "grid systems")?;
    let heads = rows(&page["head_registration"]["systems"], 64, "head systems")?;
    let sys = rows(&plan["systems"], 64, "plan systems")?;
    if grid.len() != sys.len() || heads.len() != sys.len() {
        return Err("invalid image chunk system count".into());
    }
    let before = owned.registry.len();
    let mut out = Vec::new();
    let (mut raw_count, mut kept_count, mut selected_v, mut selected_h, mut work) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for (si, system) in sys.iter().enumerate() {
        let g = &grid[si];
        let h = &heads[si];
        if g["id"] != system["system"] || h["system_id"] != system["system"] {
            return Err("invalid image chunk system order".into());
        }
        let v = members(&vertical, &g["vertical_section_ids"])?;
        let hor = members(&horizontal, &g["horizontal_section_ids"])?;
        let cases = rows(&system["cases"], 16384, "head cases")?;
        let sc = rows(&source["systems"][si]["cases"], 16384, "source cases")?;
        if cases.len() != sc.len() {
            return Err("invalid image chunk source count".into());
        }
        work = work
            .checked_add(cases.len() * (v.len() + hor.len()))
            .filter(|w| *w <= 20_000_000)
            .ok_or("image chunk lookup work budget")?;
        let mut cs = Vec::new();
        for (c, s) in cases.iter().zip(sc) {
            if c["head"] != s["head"] {
                return Err("invalid image chunk head identity".into());
            }
            let line: [f64; 4] = serde_json::from_value(c["theoretical_line"].clone())
                .map_err(|_| "invalid image chunk line")?;
            let direction = if line[3] > line[1] { 1 } else { -1 };
            let polygon: Vec<[f64; 2]> = serde_json::from_value(s["polygon"].clone())
                .map_err(|_| "invalid image chunk polygon")?;
            let stump_id = canonical(&mut owned, &c["start_stump"])?;
            let stump = stump_id
                .map(|id| owned.registry.get(id).map(|e| e.glyph.clone()))
                .transpose()?;
            let raw = stem_parts_lookup::collect(
                &v,
                &hor,
                &polygon,
                stump.as_ref(),
                &stem_parts_lookup::Parameters {
                    direction,
                    slope: 0.,
                    max_stem_thickness: max,
                    max_line_section_dx: il * 0.3,
                    last_head_y: c["last_head_y"].as_f64(),
                    theoretical_line: line,
                },
            )?;
            let existing = rows(&c["retained_seeds"], 4096, "retained seeds")?
                .iter()
                .map(|v| canonical(&mut owned, v)?.ok_or("missing retained chunk seed".into()))
                .collect::<Result<Vec<_>, String>>()?;
            let head = rows(&h["heads"], 8192, "heads")?
                .iter()
                .find(|h| h["id"] == c["head"]["head_id"])
                .ok_or("missing image chunk head glyph")?;
            if head["removed"] != false
                || head["bounds"] != c["head"]["head_bounds"]
                || head["shape"] != c["head"]["shape"]
            {
                return Err("invalid image chunk head context".into());
            }
            let head_id = head["glyph_id"]
                .as_u64()
                .ok_or("invalid image chunk head glyph id")? as usize;
            let head_glyph = beam_dispatch::glyph_from_json(&head["glyph"])?;
            if owned.registry.get(head_id)?.glyph != head_glyph {
                return Err("inconsistent image chunk owned head glyph".into());
            }
            let glyphs = raw
                .raw_glyphs
                .iter()
                .map(beam_dispatch::glyph_from_json)
                .collect::<Result<Vec<_>, _>>()?;
            let selected = stem_chunks::select(
                &mut owned.registry,
                &glyphs,
                &existing,
                stump_id,
                Some((&head_glyph, false)),
                &stem_parts_alignment::Parameters::new(direction, il * 0.15, il * 4., 0.)?,
            )?;
            let physical = |ids: &[usize]| {
                ids.iter()
                    .map(|id| {
                        owned
                            .registry
                            .get(*id)
                            .map(|e| beam_dispatch::glyph_json(&e.glyph))
                    })
                    .collect::<Result<Vec<_>, String>>()
            };
            let registered = physical(&selected.registered_ids)?;
            let retained = physical(&selected.retained_ids)?;
            let section_json = |all: &BTreeMap<u64, SeedSection>,
                                ids: &[u64]|
             -> Result<Vec<Value>, String> {
                ids.iter().map(|id| { let section = all.get(id).ok_or("missing selected image chunk section")?; Ok(json!({"vertical":section.vertical,"first_pos":section.first_pos,"bounds":section.bounds(),"runs":section.runs})) }).collect()
            };
            let selected_vertical_sections = section_json(&vertical, &raw.vertical_ids)?;
            let selected_horizontal_sections = section_json(&horizontal, &raw.horizontal_ids)?;
            raw_count += glyphs.len();
            kept_count += retained.len();
            selected_v += raw.vertical_ids.len();
            selected_h += raw.horizontal_ids.len();
            cs.push(json!({"head":c["head"],"chunk_lookup":{"selected_vertical_sections":selected_vertical_sections,"selected_horizontal_sections":selected_horizontal_sections,"section_id_namespace":"native GRID per-orientation section IDs; ordered IDs and physical sections certified on Laura","selected_vertical_ids":raw.vertical_ids,"selected_horizontal_ids":raw.horizontal_ids,"raw_glyphs":raw.raw_glyphs,"registered_glyphs":registered,"registered_native_ids":selected.registered_ids,"retained_chunks":retained,"retained_native_ids":selected.retained_ids,"existing_seed_native_ids":existing,"head_glyph":head["glyph"],"head_vip":false,"max_stem_thickness":max,"max_line_section_dx":il*0.3}}));
        }
        out.push(json!({"system":system["system"],"cases":cs}));
    }
    let registry = owned.snapshot_registry();
    owned.report = json!({"schema":"audiveris-native.stem-image-head-chunks.v1","head_parts_plan":plan,"systems":out,"corner_count":plan["corner_count"],"selected_vertical_section_occurrences":selected_v,"selected_horizontal_section_occurrences":selected_h,"raw_chunk_occurrences":raw_count,"retained_chunk_occurrences":kept_count,"registry_before_chunk_lookup":before,"new_chunk_registrations":owned.registry.len()-before,"physical_registry":registry,"automatic_head_chunk_lookup_complete":true,"automatic_raster_pipeline_complete":false,"final_stem_glyph_selection_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["complete beam-origin builders","final items/gaps/length and prefix glyph","actual upstream owned Graph/cache/head/cleaner/epilog; transient/skew/scans/special/full24pages"]});
    Ok(owned)
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}
