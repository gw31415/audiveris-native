// SPDX-License-Identifier: AGPL-3.0-or-later
//! Supplied section catalog/geometry through registered and filtered chunks.
//! Independent per-search registries; no automatic page, cache, SIG or epilog.
use crate::{
    beam_dispatch::{self, Registry},
    stem_chunks, stem_parts_alignment, stem_parts_cli, stem_parts_lookup,
    vertical_sticks::SeedSection,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub const REQUEST_SCHEMA: &str = "audiveris-native.stem-chunks-lookup-request.v1";
pub const REPORT_SCHEMA: &str = "audiveris-native.stem-chunks-lookup-report.v1";
pub const MAX_REQUEST_BYTES: usize = 8_000_000;
pub const MAX_REPORT_BYTES: usize = 32_000_000;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Search {
    pub polygon: Vec<[f64; 2]>,
    pub lookup_parameters: stem_parts_lookup::Parameters,
    pub initial_catalog: Vec<Value>,
    pub existing_ids: Vec<usize>,
    pub stump_id: Option<usize>,
    pub head_glyph: Option<Value>,
    pub vip: bool,
    pub max_alignment_dx: f64,
    pub max_alignment_dy: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub vertical_sections: Vec<SeedSection>,
    pub horizontal_sections: Vec<SeedSection>,
    pub searches: Vec<Search>,
}
#[derive(Serialize)]
pub struct Case {
    pub lookup: stem_parts_lookup::Selected,
    pub chunks: stem_chunks::Selected,
    pub registry: Value,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub lookup_geometry: &'static str,
    pub registry_scope: &'static str,
    pub sections: usize,
    pub cases: Vec<Case>,
    pub registered_to_sig: bool,
    pub heads_complete: bool,
    pub stems_complete: bool,
}
fn charge(g: &audiveris_staff_removal::GlyphPixels, pixels: &mut usize) -> Result<(), String> {
    beam_dispatch::centroid(g)?;
    for r in &g.runs {
        *pixels = pixels
            .checked_add(r.length)
            .filter(|n| *n <= 8_000_000)
            .ok_or("stem_chunks_request_pixel_budget")?;
    }
    Ok(())
}
pub fn evaluate(r: &Request) -> Result<Report, String> {
    if r.schema != REQUEST_SCHEMA {
        return Err("unsupported_stem_chunks_request_schema".into());
    }
    if r.searches.len() > 4096 {
        return Err("stem_chunks_search_budget".into());
    }
    // Reuse the certified CLI catalog validator even when there are no searches.
    let catalog = stem_parts_cli::Request {
        schema: stem_parts_cli::REQUEST_SCHEMA.into(),
        vertical_sections: r.vertical_sections.clone(),
        horizontal_sections: r.horizontal_sections.clone(),
        searches: Vec::new(),
    };
    let sections = stem_parts_cli::evaluate(&catalog)?.sections;
    let mut pixels = r
        .vertical_sections
        .iter()
        .chain(&r.horizontal_sections)
        .map(|s| s.weight() as usize)
        .sum::<usize>();
    if (pixels as u64) * (r.searches.len() as u64) > 200_000_000 {
        return Err("stem_chunks_request_work_budget".into());
    }
    let (mut cases, mut output_runs) = (Vec::with_capacity(r.searches.len()), 0usize);
    for (i, s) in r.searches.iter().enumerate() {
        if s.initial_catalog.len() > 4096 {
            return Err("stem_chunks_initial_catalog_budget".into());
        }
        let mut registry = Registry::default();
        for (j, v) in s.initial_catalog.iter().enumerate() {
            let g = beam_dispatch::glyph_from_json(v)?;
            charge(&g, &mut pixels)?;
            if registry.register(&g)? != j + 1 {
                return Err("noncanonical_stem_chunks_initial_catalog".into());
            }
        }
        let head = s
            .head_glyph
            .as_ref()
            .map(beam_dispatch::glyph_from_json)
            .transpose()?;
        if let Some(g) = head.as_ref() {
            charge(g, &mut pixels)?;
        }
        let stump = s
            .stump_id
            .map(|id| registry.get(id).map(|e| &e.glyph))
            .transpose()?;
        let lookup = stem_parts_lookup::collect(
            &r.vertical_sections,
            &r.horizontal_sections,
            &s.polygon,
            stump,
            &s.lookup_parameters,
        )
        .map_err(|e| format!("search {i}: {e}"))?;
        let raw = lookup
            .raw_glyphs
            .iter()
            .map(beam_dispatch::glyph_from_json)
            .collect::<Result<Vec<_>, _>>()?;
        let alignment = stem_parts_alignment::Parameters::new(
            s.lookup_parameters.direction,
            s.max_alignment_dx,
            s.max_alignment_dy,
            s.lookup_parameters.slope,
        )?;
        let chunks = stem_chunks::select(
            &mut registry,
            &raw,
            &s.existing_ids,
            s.stump_id,
            head.as_ref().map(|h| (h, s.vip)),
            &alignment,
        )
        .map_err(|e| format!("search {i}: {e}"))?;
        output_runs += raw.iter().map(|g| g.runs.len()).sum::<usize>();
        let snapshot = registry.snapshot();
        output_runs += snapshot
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["glyph"]["runs"].as_array().unwrap().len())
            .sum::<usize>();
        if output_runs > 1_000_000 {
            return Err("stem_chunks_report_run_budget".into());
        }
        cases.push(Case {
            lookup,
            chunks,
            registry: snapshot,
        });
    }
    Ok(Report {
        schema: REPORT_SCHEMA,
        lookup_geometry: "caller_supplied_linear_polygons",
        registry_scope: "independent_per_search_canonical_catalogs",
        sections,
        cases,
        registered_to_sig: false,
        heads_complete: false,
        stems_complete: false,
    })
}
pub fn encode_report(r: &Report) -> Result<Vec<u8>, String> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if b.len() > MAX_REPORT_BYTES - self.0.len() {
                return Err(std::io::Error::other("stem_chunks_report_byte_budget"));
            }
            self.0.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut out = Bounded(Vec::new());
    serde_json::to_writer(&mut out, r).map_err(|e| e.to_string())?;
    if out.0.len() >= MAX_REPORT_BYTES {
        return Err("stem_chunks_report_byte_budget".into());
    }
    out.0.push(b'\n');
    Ok(out.0)
}
