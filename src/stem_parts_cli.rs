// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit section-catalog/lookup-polygon CLI contract. Geometry is supplied by
//! the caller; this adapter cannot mark page HEADS/STEMS or SIG completion.
use crate::{
    beam_dispatch,
    stem_parts_lookup::{self, Parameters, Selected},
    vertical_sticks::SeedSection,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const REQUEST_SCHEMA: &str = "audiveris-native.stem-parts-lookup-request.v1";
pub const REPORT_SCHEMA: &str = "audiveris-native.stem-parts-lookup-report.v1";
pub const MAX_REQUEST_BYTES: usize = 8_000_000;
pub const MAX_REPORT_BYTES: usize = 32_000_000;
const MAX_SEARCHES: usize = 4096;
const MAX_WORK_PIXELS: u64 = 200_000_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Search {
    pub polygon: Vec<[f64; 2]>,
    pub stump: Option<Value>,
    pub parameters: Parameters,
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
pub struct Report {
    pub schema: &'static str,
    pub lookup_geometry: &'static str,
    pub sections: usize,
    pub cases: Vec<Selected>,
    pub registered_to_sig: bool,
    pub heads_complete: bool,
    pub stems_complete: bool,
}
pub fn evaluate(request: &Request) -> Result<Report, String> {
    if request.schema != REQUEST_SCHEMA {
        return Err("unsupported_stem_lookup_request_schema".into());
    }
    let sections = request.vertical_sections.len() + request.horizontal_sections.len();
    if sections > 32768 || request.searches.len() > MAX_SEARCHES {
        return Err("stem_lookup_request_budget".into());
    }
    // Validate before calling weight() or charging budgets: input integers can
    // be arbitrary JSON values. Catalog identity is checked even for zero searches.
    let mut pixels = 0u64;
    let mut runs = 0usize;
    let mut identities = std::collections::BTreeSet::new();
    for (wanted, all) in [
        (true, &request.vertical_sections),
        (false, &request.horizontal_sections),
    ] {
        for s in all {
            stem_parts_lookup::section_centroid(s)?;
            if s.vertical != wanted || !identities.insert((wanted, s.id)) {
                return Err("invalid_stem_lookup_section_identity".into());
            }
            pixels += s.weight() as u64;
            runs += s.runs.len();
            if pixels > 8_000_000 || runs > 1_048_576 {
                return Err("stem_lookup_pixel_budget".into());
            }
        }
    }
    if pixels * request.searches.len() as u64 > MAX_WORK_PIXELS {
        return Err("stem_lookup_request_work_budget".into());
    }
    let mut cases = Vec::with_capacity(request.searches.len());
    let mut output_runs = 0usize;
    for (index, search) in request.searches.iter().enumerate() {
        let stump = search
            .stump
            .as_ref()
            .map(beam_dispatch::glyph_from_json)
            .transpose()?;
        let selected = stem_parts_lookup::collect(
            &request.vertical_sections,
            &request.horizontal_sections,
            &search.polygon,
            stump.as_ref(),
            &search.parameters,
        )
        .map_err(|e| format!("search {index}: {e}"))?;
        output_runs += selected
            .raw_glyphs
            .iter()
            .map(|g| g["runs"].as_array().unwrap().len())
            .sum::<usize>();
        if output_runs > 1_000_000 {
            return Err("stem_lookup_report_run_budget".into());
        }
        cases.push(selected);
    }
    Ok(Report {
        schema: REPORT_SCHEMA,
        lookup_geometry: "caller_supplied_linear_polygons",
        sections,
        cases,
        registered_to_sig: false,
        heads_complete: false,
        stems_complete: false,
    })
}
/// Bound serialization before opening the destination so refusal leaves no
/// partial report. The CLI then uses create_new to prevent overwriting files.
pub fn encode_report(report: &Report) -> Result<Vec<u8>, String> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if b.len() > MAX_REPORT_BYTES - self.0.len() {
                return Err(std::io::Error::other("stem_lookup_report_byte_budget"));
            }
            self.0.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut out = Bounded(Vec::new());
    serde_json::to_writer(&mut out, report).map_err(|e| e.to_string())?;
    out.0.push(b'\n');
    if out.0.len() > MAX_REPORT_BYTES {
        return Err("stem_lookup_report_byte_budget".into());
    }
    Ok(out.0)
}
