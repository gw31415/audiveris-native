// Copyright © Audiveris 2026. All rights reserved.
// Rust adaptation 2026-10-02; SPDX-License-Identifier: AGPL-3.0-or-later
// SectionFactory.Build and JunctionRatioPolicy/JunctionShiftPolicy, upstream 9e1e55cd.
use crate::{GrayImage, Run, position_runs, scale::Scale};
use std::io::Write;
pub const MAX_RUNS: usize = 250_000;
pub const MAX_SECTIONS: usize = 100_000;
pub const MAX_DIMENSION: usize = 16_384;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
impl Orientation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
#[derive(Debug)]
pub struct RunTable {
    pub orientation: Orientation,
    pub width: usize,
    pub height: usize,
    pub sequences: Vec<Vec<Run>>,
}
impl RunTable {
    pub fn from_binary(
        image: &GrayImage,
        orientation: Orientation,
        mut accept: impl FnMut(&Run) -> bool,
    ) -> Result<Self, String> {
        if image.width > MAX_DIMENSION || image.height > MAX_DIMENSION {
            return Err("sections_dimension_limit".into());
        }
        let vertical = orientation == Orientation::Vertical;
        let count = if vertical { image.width } else { image.height };
        let mut total = 0;
        let mut sequences = Vec::with_capacity(count);
        for pos in 0..count {
            let mut row = Vec::new();
            for run in position_runs(image, vertical, pos).filter(|r| accept(r)) {
                total += 1;
                if total > MAX_RUNS {
                    return Err("sections_run_limit".into());
                }
                row.push(run);
            }
            sequences.push(row);
        }
        Ok(Self {
            orientation,
            width: image.width,
            height: image.height,
            sequences,
        })
    }
    pub fn run_count(&self) -> usize {
        self.sequences.iter().map(Vec::len).sum()
    }
}
#[derive(Debug)]
pub struct Section {
    pub id: SectionId,
    pub orientation: Orientation,
    pub first_pos: usize,
    pub runs: Vec<Run>,
    pub weight: usize,
}
impl Section {
    pub fn bounds(&self) -> Bounds {
        let min = self.runs.iter().map(|r| r.start).min().unwrap();
        let stop = self.runs.iter().map(|r| r.start + r.length).max().unwrap();
        match self.orientation {
            Orientation::Horizontal => Bounds {
                x: min,
                y: self.first_pos,
                width: stop - min,
                height: self.runs.len(),
            },
            Orientation::Vertical => Bounds {
                x: self.first_pos,
                y: min,
                width: self.runs.len(),
                height: stop - min,
            },
        }
    }
    pub fn mean_run_length(&self) -> usize {
        self.weight / self.runs.len()
    }
    pub fn last(&self) -> &Run {
        self.runs.last().unwrap()
    }
}
#[derive(Clone, Copy)]
pub enum JunctionPolicy {
    Ratio(f64),
    Shift(usize),
}
impl JunctionPolicy {
    fn consistent(self, run: &Run, section: &Section) -> bool {
        match self {
            Self::Ratio(max) => {
                let ratio = run.length as f64 / section.mean_run_length() as f64;
                (1.0 / max) <= ratio && ratio <= max
            }
            Self::Shift(max) => {
                run.start.abs_diff(section.last().start) <= max
                    && (run.start + run.length - 1)
                        .abs_diff(section.last().start + section.last().length - 1)
                        <= max
            }
        }
    }
}
fn overlaps(a: &Run, b: &Run) -> bool {
    a.start < b.start + b.length && b.start < a.start + a.length
}
/// Reconstructs section IDs/order/partition from the original one-shot SectionFactory.
pub fn build(table: &RunTable, policy: JunctionPolicy) -> Result<Vec<Section>, String> {
    let mut sections: Vec<Section> = Vec::new();
    let mut processed = Vec::new();
    let mut active: Vec<usize> = Vec::new();
    for (pos, row) in table.sequences.iter().enumerate() {
        if row.is_empty() {
            active.clear();
            continue;
        }
        let previous = std::mem::take(&mut active);
        for &id in &previous {
            let section = &sections[id];
            let mut matching = row.iter().filter(|r| overlaps(section.last(), r));
            if let Some(run) = matching.next()
                && (matching.next().is_some() || !policy.consistent(run, section))
            {
                processed[id] = true;
            }
        }
        for run in row {
            let mut matching = previous
                .iter()
                .copied()
                .filter(|&id| overlaps(sections[id].last(), run));
            let first = matching.next();
            let second = matching.next();
            if let Some(id) = first.filter(|&id| second.is_none() && !processed[id]) {
                sections[id].runs.push(Run {
                    start: run.start,
                    length: run.length,
                });
                sections[id].weight += run.length;
                active.push(id);
            } else {
                if sections.len() >= MAX_SECTIONS {
                    return Err("sections_count_limit".into());
                }
                let id = sections.len();
                sections.push(Section {
                    id: SectionId(id as u32 + 1),
                    orientation: table.orientation,
                    first_pos: pos,
                    runs: vec![Run {
                        start: run.start,
                        length: run.length,
                    }],
                    weight: run.length,
                });
                processed.push(false);
                active.push(id);
            }
        }
    }
    Ok(sections)
}
/// Public classifier boundary: absolute pixel origin + local 8-bit mask + contributing section refs.
/// It does not claim that a section is already a musical glyph or provide fabricated features.
#[derive(Debug)]
pub struct GlyphCandidate {
    pub bounds: Bounds,
    pub pixels: Vec<u8>,
    pub sources: Vec<(Orientation, SectionId)>,
}
impl Section {
    pub fn as_glyph_candidate(&self) -> GlyphCandidate {
        let bounds = self.bounds();
        let mut pixels = vec![255; bounds.width * bounds.height];
        for (i, r) in self.runs.iter().enumerate() {
            for c in r.start..r.start + r.length {
                let (x, y) = match self.orientation {
                    Orientation::Horizontal => (c, self.first_pos + i),
                    Orientation::Vertical => (self.first_pos + i, c),
                };
                pixels[(y - bounds.y) * bounds.width + x - bounds.x] = 0;
            }
        }
        GlyphCandidate {
            bounds,
            pixels,
            sources: vec![(self.orientation, self.id)],
        }
    }
}
pub struct GridPreparation {
    pub vertical: Vec<Section>,
    pub horizontal: Vec<Section>,
    pub short_horizontal_runs: usize,
    pub long_vertical_runs: usize,
    pub long_horizontal_runs: usize,
    pub min_vertical_length: usize,
    pub min_horizontal_length: usize,
    pub max_vertical_shift: usize,
}
/// LagManager.dispatchRuns + LinesRetriever.createBothLags table partition + the two SectionFactory policies.
/// Output horizontal sections are filament inputs, not staff lines or assembled staves.
pub fn prepare(binary: &GrayImage, scale: &Scale) -> Result<GridPreparation, String> {
    let min_vertical_length = 1 + (scale.line.max as f64 * 1.2).round_ties_even() as usize;
    let min_horizontal_length = (scale.interline.main as f64 * 0.25).round_ties_even() as usize;
    let max_vertical_shift = (scale.interline.main as f64 * 0.05).round_ties_even() as usize;
    let vertical_table = RunTable::from_binary(binary, Orientation::Vertical, |r| {
        r.length >= min_vertical_length
    })?;
    let mut short = GrayImage::new(binary.width, binary.height, vec![255; binary.pixels.len()])?;
    for x in 0..binary.width {
        for r in position_runs(binary, true, x).filter(|r| r.length < min_vertical_length) {
            for y in r.start..r.start + r.length {
                short.pixels[y * binary.width + x] = 0;
            }
        }
    }
    let mut short_horizontal_runs = 0;
    let horizontal_table = RunTable::from_binary(&short, Orientation::Horizontal, |r| {
        if r.length < min_horizontal_length {
            short_horizontal_runs += 1;
            false
        } else {
            true
        }
    })?;
    let long_vertical_runs = vertical_table.run_count();
    let long_horizontal_runs = horizontal_table.run_count();
    let vertical = build(&vertical_table, JunctionPolicy::Shift(max_vertical_shift))?;
    let horizontal = build(&horizontal_table, JunctionPolicy::Ratio(1.25))?;
    Ok(GridPreparation {
        vertical,
        horizontal,
        short_horizontal_runs,
        long_vertical_runs,
        long_horizontal_runs,
        min_vertical_length,
        min_horizontal_length,
        max_vertical_shift,
    })
}
pub fn write_sections(out: &mut impl Write, sections: &[Section]) -> std::io::Result<()> {
    write!(out, "[")?;
    for (i, s) in sections.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        let b = s.bounds();
        write!(
            out,
            "{{\"id\":{},\"first_pos\":{},\"weight\":{},\"mean_run_length\":{},\"bounds\":[{},{},{},{}],\"runs\":[",
            s.id.0,
            s.first_pos,
            s.weight,
            s.mean_run_length(),
            b.x,
            b.y,
            b.width,
            b.height
        )?;
        for (j, r) in s.runs.iter().enumerate() {
            if j > 0 {
                write!(out, ",")?;
            }
            write!(out, "[{},{}]", r.start, r.length)?;
        }
        write!(out, "]}}")?;
    }
    write!(out, "]")
}
impl GridPreparation {
    pub fn write_json(&self, out: &mut impl Write) -> std::io::Result<()> {
        write!(
            out,
            "{{\"stage\":\"lag_sections\",\"staff_detection\":\"not_implemented\",\"min_vertical_length\":{},\"min_horizontal_length\":{},\"max_vertical_shift\":{},\"short_horizontal_runs\":{},\"long_vertical_runs\":{},\"long_horizontal_runs\":{},\"horizontal_sections\":",
            self.min_vertical_length,
            self.min_horizontal_length,
            self.max_vertical_shift,
            self.short_horizontal_runs,
            self.long_vertical_runs,
            self.long_horizontal_runs
        )?;
        write_sections(out, &self.horizontal)?;
        write!(out, ",\"vertical_sections\":")?;
        write_sections(out, &self.vertical)?;
        write!(out, "}}")
    }
}
