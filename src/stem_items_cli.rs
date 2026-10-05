// SPDX-License-Identifier: AGPL-3.0-or-later
//! Declared selected glyph/linker sources through automatic items, events and
//! length maps. No image candidate discovery, physical cache/SIG or epilog.
use crate::{
    stem_item_events::{self, Events, Source},
    stem_length_map,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const REQUEST_SCHEMA: &str = "audiveris-native.stem-items-plan-request.v1";
pub const MAX_REQUEST_BYTES: usize = 16_000_000;
pub const MAX_REPORT_BYTES: usize = 32_000_000;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub direction: i8,
    pub start_y: f64,
    pub y_range: [i32; 4],
    pub sources: Vec<Source>,
    pub gap_map: BTreeMap<i32, i32>,
    pub stem_profile: i32,
    pub max_profile: i32,
    pub initial_lengths: BTreeMap<i32, i32>,
    pub skew: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub cases: Vec<Case>,
}
#[derive(Serialize)]
pub struct Selected {
    pub events: Events,
    pub prefix_lengths: Vec<i32>,
    pub lengths: BTreeMap<i32, i32>,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub source_selection: &'static str,
    pub cases: Vec<Selected>,
    pub registered_to_sig: bool,
    pub epilog_complete: bool,
    pub heads_complete: bool,
    pub stems_complete: bool,
}
pub fn evaluate(r: &Request) -> Result<Report, String> {
    if r.schema != REQUEST_SCHEMA || r.cases.len() > 8192 {
        return Err("unsupported item request schema/case budget".into());
    }
    let mut work = 0usize;
    let mut total_items = 0usize;
    let mut cases = Vec::new();
    for c in &r.cases {
        if c.skew != 0. {
            return Err("unsupported nonzero item request skew".into());
        }
        total_items = total_items
            .checked_add(c.sources.len())
            .filter(|n| *n <= 100_000)
            .ok_or("item request source budget")?;
        for s in &c.sources {
            if let Some(g) = &s.glyph {
                let g = crate::beam_dispatch::glyph_from_json(g)?;
                for run in &g.runs {
                    work = work
                        .checked_add(run.length)
                        .filter(|n| *n <= 8_000_000)
                        .ok_or("item request pixel budget")?;
                }
            }
        }
        let items = stem_item_events::construct(&c.sources, c.y_range)?;
        let gap = *c
            .gap_map
            .get(&c.stem_profile)
            .ok_or("stem profile absent from gap map")?;
        let events = stem_item_events::events(&items, c.direction, gap)?;
        let native = events
            .items
            .iter()
            .map(|i| Some(i.length_item()))
            .collect::<Vec<_>>();
        let prefix_lengths = stem_length_map::prefix_lengths(&native, c.direction, c.start_y)?;
        let lengths = stem_length_map::retrieve_lengths(
            &native,
            c.direction,
            c.start_y,
            &c.gap_map,
            c.max_profile,
            &c.initial_lengths,
        )?;
        cases.push(Selected {
            events,
            prefix_lengths,
            lengths,
        });
    }
    Ok(Report {
        schema: "audiveris-native.stem-items-plan-report.v1",
        source_selection: "caller_supplied_prefiltered_physical_glyphs_and_linkers",
        cases,
        registered_to_sig: false,
        epilog_complete: false,
        heads_complete: false,
        stems_complete: false,
    })
}
pub fn encode_report(report: &Report) -> Result<Vec<u8>, String> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if b.len() > MAX_REPORT_BYTES - self.0.len() {
                return Err(std::io::Error::other("item report byte budget"));
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
    if out.0.len() == MAX_REPORT_BYTES {
        return Err("item report byte budget".into());
    }
    out.0.push(b'\n');
    Ok(out.0)
}
