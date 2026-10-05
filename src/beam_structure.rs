// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Rust translation of BeamStructure, BeamLine, BeamItem and BasicLine at
// 9e1e55cd2746037d059345881c53e6a6754bffbd (official 5.11.0 local bytecode).
// Geometry only: no BeamInter/SIG creation, production dispatch, or fixture lookup.
use crate::{
    MAX_PIXELS, Run, beam_dispatch,
    sections::{self, JunctionPolicy, Orientation, RunTable},
};
use audiveris_staff_removal::{GlyphPixels, Orientation as GlyphOrientation};
use serde::{Deserialize, Serialize};

const MAX_SECTION_SLOPE_GAP: f64 = 0.3;
const MAX_BORDER_JITTER: f64 = 0.8;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemParameters {
    pub is_small: bool,
    pub min_beam_width_low: f64,
    pub min_beam_width_high: f64,
    pub min_hook_width_low: f64,
    pub min_hook_width_high: f64,
    pub max_hook_width: f64,
    pub min_height_low: f64,
    pub typical_height: f64,
    pub max_height_high: f64,
    pub corner_margin: f64,
    pub max_item_x_gap: f64,
    pub core_section_width: usize,
}
impl ItemParameters {
    pub fn new(interline: usize, typical_height: f64, is_small: bool) -> Result<Self, String> {
        if interline == 0 || interline > sections::MAX_DIMENSION {
            return Err("invalid_beam_structure_interline".into());
        }
        let il = interline as f64;
        let value = Self {
            is_small,
            min_beam_width_low: il,
            min_beam_width_high: 4.0 * il,
            min_hook_width_low: 0.7 * il,
            min_hook_width_high: il,
            max_hook_width: 2.0 * il,
            min_height_low: typical_height * 0.7,
            typical_height,
            max_height_high: typical_height * 1.4,
            corner_margin: typical_height * 0.2,
            max_item_x_gap: il * 0.5,
            core_section_width: (il * 0.15).round_ties_even().max(2.0) as usize,
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(self) -> Result<(), String> {
        let values = [
            self.min_beam_width_low,
            self.min_beam_width_high,
            self.min_hook_width_low,
            self.min_hook_width_high,
            self.max_hook_width,
            self.min_height_low,
            self.typical_height,
            self.max_height_high,
            self.corner_margin,
            self.max_item_x_gap,
        ];
        if values
            .iter()
            .any(|x| !x.is_finite() || *x < 0.0 || *x > MAX_PIXELS as f64)
            || self.typical_height == 0.0
            || self.min_hook_width_low == 0.0
            || self.core_section_width < 2
            || self.core_section_width > sections::MAX_DIMENSION
            || self.min_beam_width_low > self.min_beam_width_high
            || self.min_hook_width_low > self.min_hook_width_high
            || self.min_hook_width_high > self.max_hook_width
            || self.min_height_low > self.typical_height
            || self.typical_height > self.max_height_high
        {
            return Err("invalid_beam_structure_parameters".into());
        }
        Ok(())
    }
}
/// Native resource bounds, not restrictions claimed for original Audiveris.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_sections: usize,
    pub max_border_lines: usize,
    pub max_work: usize,
    pub max_items: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_sections: 4096,
            max_border_lines: 1024,
            max_work: 16_000_000,
            max_items: 100_000,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeamItem {
    pub median: [f64; 4],
    pub height: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeamLine {
    pub median: [f64; 4],
    pub height: f64,
    pub items: Vec<BeamItem>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalSide {
    Top,
    Bottom,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BeamSection {
    pub first_pos: i32,
    pub bounds: [i32; 4],
    pub weight: usize,
    pub runs: Vec<[i32; 2]>,
}
impl BeamSection {
    // The vertical section polygon is the union of these half-open pixel cells.
    fn contains(&self, x: f64, y: f64) -> bool {
        let index = (x - self.first_pos as f64).floor();
        if index < 0.0 || index >= self.runs.len() as f64 {
            return false;
        }
        let [start, length] = self.runs[index as usize];
        y >= start as f64 && y < (start + length) as f64
    }
}
#[derive(Clone, Debug)]
struct BasicLine {
    n: usize,
    sx: f64,
    sy: f64,
    xx: f64,
    yy: f64,
    xy: f64,
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}
impl Default for BasicLine {
    fn default() -> Self {
        Self {
            n: 0,
            sx: 0.0,
            sy: 0.0,
            xx: 0.0,
            yy: 0.0,
            xy: 0.0,
            xmin: f64::INFINITY,
            xmax: f64::NEG_INFINITY,
            ymin: f64::INFINITY,
            ymax: f64::NEG_INFINITY,
        }
    }
}
impl BasicLine {
    fn include(&mut self, x: f64, y: f64) {
        self.n += 1;
        self.sx += x;
        self.sy += y;
        self.xx += x * x;
        self.yy += y * y;
        self.xy += x * y;
        self.xmin = self.xmin.min(x);
        self.xmax = self.xmax.max(x);
        self.ymin = self.ymin.min(y);
        self.ymax = self.ymax.max(y);
    }
    fn include_line(&mut self, l: &Self) {
        self.n += l.n;
        self.sx += l.sx;
        self.sy += l.sy;
        self.xx += l.xx;
        self.yy += l.yy;
        self.xy += l.xy;
        self.xmin = self.xmin.min(l.xmin);
        self.xmax = self.xmax.max(l.xmax);
        self.ymin = self.ymin.min(l.ymin);
        self.ymax = self.ymax.max(l.ymax);
    }
    fn coefficients(&self) -> Result<(f64, f64, f64, bool), String> {
        if self.n < 2 {
            return Err("undefined_beam_structure_line".into());
        }
        let n = self.n as f64;
        let h = n * self.xx - self.sx * self.sx;
        let v = n * self.yy - self.sy * self.sy;
        let vertical = h.abs() < v.abs();
        let (mut a, mut b, mut c) = if !vertical {
            (
                (n * self.xy - self.sx * self.sy) / h,
                -1.0,
                (self.sy * self.xx - self.sx * self.xy) / h,
            )
        } else {
            (
                -1.0,
                (n * self.xy - self.sx * self.sy) / v,
                (self.sx * self.yy - self.sy * self.xy) / v,
            )
        };
        // Match Java Math.hypot: even a one-ULP platform difference can
        // change strict split-height branches after raw-moment fitting.
        let norm = audiveris_area::hypot(a, b);
        a /= norm;
        b /= norm;
        c /= norm;
        if ![a, b, c].iter().all(|v| v.is_finite()) {
            return Err("undefined_beam_structure_line".into());
        }
        Ok((a, b, c, vertical))
    }
    fn slope(&self) -> Result<f64, String> {
        let (a, b, _, _) = self.coefficients()?;
        Ok(-a / b)
    }
    fn y(&self, x: f64) -> Result<f64, String> {
        let (a, b, c, _) = self.coefficients()?;
        if b == 0.0 {
            return Err("vertical_beam_structure_line".into());
        }
        Ok((-a * x - c) / b)
    }
    fn segment(&self) -> Result<[f64; 4], String> {
        let (a, b, c, vertical) = self.coefficients()?;
        Ok(if vertical {
            [
                (-b * self.ymin - c) / a,
                self.ymin,
                (-b * self.ymax - c) / a,
                self.ymax,
            ]
        } else {
            [
                self.xmin,
                (-a * self.xmin - c) / b,
                self.xmax,
                (-a * self.xmax - c) / b,
            ]
        })
    }
    fn mean_distance(&self) -> Result<f64, String> {
        let (a, b, c, _) = self.coefficients()?;
        let n = self.n as f64;
        Ok(((a * a * self.xx
            + b * b * self.yy
            + c * c * n
            + 2.0 * a * b * self.xy
            + 2.0 * a * c * self.sx
            + 2.0 * b * c * self.sy)
            / n)
            .max(0.0)
            .sqrt())
    }
}
// Keep the original determinant arithmetic, including the 1000-unit vertical
// line, rather than substituting slope interpolation (different roundoff).
#[allow(clippy::eq_op)] // Preserve the original determinant operation order.
pub(crate) fn intersection_at_x(l: [f64; 4], x: f64) -> [f64; 2] {
    let [x1, y1, x2, y2] = l;
    let denominator = (x1 - x2) * (0.0 - 1000.0) - (y1 - y2) * (x - x);
    let cross1 = x1 * y2 - y1 * x2;
    let cross2 = x * 1000.0 - 0.0 * x;
    [
        (cross1 * (x - x) - (x1 - x2) * cross2) / denominator,
        (cross1 * (0.0 - 1000.0) - (y1 - y2) * cross2) / denominator,
    ]
}
fn y_at_x(l: [f64; 4], x: f64) -> f64 {
    intersection_at_x(l, x)[1]
}
fn reference_y(center: [i32; 2], slope: f64, x: f64) -> f64 {
    y_at_x(
        [
            center[0] as f64,
            center[1] as f64,
            center[0] as f64 + 1000.0,
            center[1] as f64 + 1000.0 * slope,
        ],
        x,
    )
}
fn slice(l: [f64; 4], left: f64, right: f64) -> [f64; 4] {
    let p = intersection_at_x(l, left);
    let q = intersection_at_x(l, right);
    [p[0], p[1], q[0], q[1]]
}
fn slope(l: [f64; 4]) -> f64 {
    (l[3] - l[1]) / (l[2] - l[0])
}
fn put(map: &mut Vec<(f64, [f64; 4])>, key: f64, value: [f64; 4]) {
    match map.binary_search_by(|(k, _)| k.total_cmp(&key)) {
        Ok(i) => map[i] = (key, value),
        Err(i) => map.insert(i, (key, value)),
    }
}

#[derive(Debug)]
pub struct BeamStructure {
    bounds: [i32; 4],
    weight: usize,
    center: [i32; 2],
    params: ItemParameters,
    sections: Vec<BeamSection>,
    lines: Vec<BeamLine>,
    limits: Limits,
    computed: bool,
}
impl BeamStructure {
    pub fn new(glyph: &GlyphPixels, params: ItemParameters) -> Result<Self, String> {
        Self::with_limits(glyph, params, Limits::default())
    }
    pub fn with_limits(
        glyph: &GlyphPixels,
        params: ItemParameters,
        limits: Limits,
    ) -> Result<Self, String> {
        params.validate()?;
        if limits.max_sections == 0
            || limits.max_sections > sections::MAX_SECTIONS
            || limits.max_border_lines == 0
            || limits.max_border_lines > sections::MAX_SECTIONS
            || limits.max_work == 0
            || limits.max_work > 1_000_000_000
            || limits.max_items == 0
            || limits.max_items > 1_000_000
        {
            return Err("invalid_beam_structure_limits".into());
        }
        if glyph.orientation != GlyphOrientation::Vertical {
            return Err("invalid_beam_structure_orientation".into());
        }
        if glyph.width > sections::MAX_DIMENSION
            || glyph.height > sections::MAX_DIMENSION
            || glyph.runs.len() > sections::MAX_RUNS
        {
            return Err("unsupported_beam_structure_glyph_budget".into());
        }
        let center = beam_dispatch::centroid(glyph)?;
        // Restrict absolute coordinates to bounded page space so the original
        // BasicLine raw-moment arithmetic remains well-conditioned.
        if glyph.left < 0
            || glyph.top < 0
            || (glyph.left as usize)
                .checked_add(glyph.width)
                .is_none_or(|x| x > sections::MAX_DIMENSION)
            || (glyph.top as usize)
                .checked_add(glyph.height)
                .is_none_or(|x| x > sections::MAX_DIMENSION)
        {
            return Err("unsupported_beam_structure_coordinates".into());
        }
        let mut table = RunTable {
            orientation: Orientation::Vertical,
            width: glyph.width,
            height: glyph.height,
            sequences: (0..glyph.width).map(|_| Vec::new()).collect(),
        };
        for r in &glyph.runs {
            table.sequences[r.position].push(Run {
                start: r.start,
                length: r.length,
            });
        }
        let work = table
            .sequences
            .windows(2)
            .try_fold(0usize, |total, p| {
                total.checked_add(p[0].len().checked_mul(p[1].len())?.checked_mul(2)?)
            })
            .ok_or("unsupported_beam_structure_work_budget")?;
        if work > limits.max_work {
            return Err("unsupported_beam_structure_work_budget".into());
        }
        let local = sections::build(&table, JunctionPolicy::Ratio(1.25))?;
        if local.len() > limits.max_sections {
            return Err("unsupported_beam_structure_section_budget".into());
        }
        let sections = local
            .into_iter()
            .map(|s| {
                let b = s.bounds();
                BeamSection {
                    first_pos: glyph.left + s.first_pos as i32,
                    bounds: [
                        glyph.left + b.x as i32,
                        glyph.top + b.y as i32,
                        b.width as i32,
                        b.height as i32,
                    ],
                    weight: s.weight,
                    runs: s
                        .runs
                        .into_iter()
                        .map(|r| [glyph.top + r.start as i32, r.length as i32])
                        .collect(),
                }
            })
            .collect();
        Ok(Self {
            bounds: [
                glyph.left,
                glyph.top,
                glyph.width as i32,
                glyph.height as i32,
            ],
            weight: glyph.runs.iter().map(|r| r.length).sum(),
            center,
            params,
            sections,
            lines: Vec::new(),
            limits,
            computed: false,
        })
    }
    pub fn sections(&self) -> &[BeamSection] {
        &self.sections
    }
    pub fn lines(&self) -> &[BeamLine] {
        &self.lines
    }
    pub fn parameters(&self) -> ItemParameters {
        self.params
    }
    pub fn centroid(&self) -> [i32; 2] {
        self.center
    }
    pub fn width(&self) -> Option<f64> {
        if self.lines.is_empty() {
            return None;
        }
        let mut left = f64::MAX;
        let mut right = f64::from_bits(1);
        for l in &self.lines {
            left = left.min(l.median[0]);
            right = right.max(l.median[2]);
        }
        Some(right - left + 1.0)
    }
    fn border_lines(&self, side: VerticalSide) -> Result<Vec<BasicLine>, String> {
        let mut borders = Vec::new();
        for s in &self.sections {
            if s.runs.len() < self.params.core_section_width {
                continue;
            }
            let mut l = BasicLine::default();
            for (i, [start, len]) in s.runs.iter().enumerate() {
                let y = if side == VerticalSide::Top {
                    *start
                } else {
                    start + len
                };
                l.include((s.first_pos + i as i32) as f64, y as f64);
            }
            borders.push((s, l, 0.0));
        }
        // Stable ties preserve SectionFactory order, then the length ordering is
        // retained during ordinate-offset ties, as in Collections.sort.
        borders.sort_by_key(|(s, _, _)| std::cmp::Reverse(s.runs.len()));
        let (mut sum, mut count) = (0.0, 0usize);
        for (_, l, _) in &borders {
            let value = l.slope()?;
            if count > 0 && (value - sum / count as f64).abs() > MAX_SECTION_SLOPE_GAP {
                break;
            }
            count += l.n;
            sum += l.n as f64 * value;
        }
        let global = sum / count as f64;
        let mut selected = Vec::new();
        for (s, l, _) in borders {
            if (l.slope()? - global).abs() > MAX_SECTION_SLOPE_GAP {
                continue;
            }
            let x = s.bounds[0] as f64 + s.bounds[2] as f64 / 2.0;
            let dy = l.y(x)? - reference_y(self.center, global, x);
            selected.push((l, dy));
        }
        selected.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut lines: Vec<BasicLine> = Vec::new();
        let (mut offset_sum, mut offset_weight) = (0.0, 0usize);
        for (l, dy) in selected {
            if lines.is_empty()
                || dy - offset_sum / offset_weight as f64
                    > self.params.typical_height * MAX_BORDER_JITTER
            {
                if lines.len() >= self.limits.max_border_lines {
                    return Err("unsupported_beam_structure_border_budget".into());
                }
                lines.push(BasicLine::default());
                offset_sum = 0.0;
                offset_weight = 0;
            }
            offset_sum += l.n as f64 * dy;
            offset_weight += l.n;
            lines.last_mut().unwrap().include_line(&l);
        }
        let mut kept = Vec::new();
        for l in lines {
            let s = l.segment()?;
            if s[2] - s[0] >= self.params.min_hook_width_low {
                kept.push(l);
            }
        }
        Ok(kept)
    }
    fn lines_map(&self, global: f64, lines: &[BasicLine]) -> Result<Vec<(f64, [f64; 4])>, String> {
        let mut out = Vec::new();
        for l in lines {
            let segment = l.segment()?;
            let x = (segment[0] + segment[2]) / 2.0;
            put(
                &mut out,
                l.y(x)? - reference_y(self.center, global, x),
                segment,
            );
        }
        Ok(out)
    }
    fn complete_borders(
        &self,
        direction: f64,
        global: f64,
        source: &[(f64, [f64; 4])],
        target: &mut Vec<(f64, [f64; 4])>,
    ) {
        for (offset, l) in source {
            let desired = offset + direction * self.params.typical_height;
            let Some(i) = target.iter().position(|(dy, _)| {
                (dy - desired).abs() <= self.params.typical_height * MAX_BORDER_JITTER
            }) else {
                continue;
            };
            // allowBorderCreation is false in the pinned default configuration.
            let (_, other) = target.remove(i);
            let middle = (other[0] + other[2]) / 2.0;
            let dy = (other[1] + other[3]) / 2.0 - y_at_x(*l, middle);
            let p = if l[0] < other[0] {
                [l[0], l[1] + dy]
            } else {
                [other[0], other[1]]
            };
            let q = if l[2] > other[2] {
                [l[2], l[3] + dy]
            } else {
                [other[2], other[3]]
            };
            let extended = [p[0], p[1], q[0], q[1]];
            let x = (p[0] + q[0]) / 2.0;
            put(
                target,
                y_at_x(extended, x) - reference_y(self.center, global, x),
                extended,
            );
        }
    }
    fn retrieve_items(&self, line: &mut BeamLine) -> Result<(), String> {
        let mut span: Option<(i32, i32)> = None;
        for s in &self.sections {
            let x = s.bounds[0] as f64 + s.bounds[2] as f64 / 2.0;
            if !s.contains(x, y_at_x(line.median, x)) {
                continue;
            }
            let left = s.bounds[0];
            let right = left + s.bounds[2];
            if let Some((start, end)) = span {
                if (left - end) as f64 > self.params.max_item_x_gap {
                    line.items.push(BeamItem {
                        median: slice(line.median, start as f64, end as f64),
                        height: line.height,
                    });
                    span = Some((left, end.max(right)));
                } else {
                    span = Some((start, end.max(right)));
                }
            } else {
                span = Some((left, right));
            }
            if line.items.len() > self.limits.max_items {
                return Err("unsupported_beam_structure_item_budget".into());
            }
        }
        if let Some((left, right)) = span {
            line.items.push(BeamItem {
                median: slice(line.median, left as f64, right as f64),
                height: line.height,
            });
        }
        Ok(())
    }
    /// One-shot equivalent of original computeLines. A second call is rejected
    /// explicitly instead of silently appending duplicate lines to the phase IR.
    pub fn compute_lines(&mut self) -> Result<Option<f64>, String> {
        if self.computed {
            return Err("beam_structure_already_computed".into());
        }
        let top = self.border_lines(VerticalSide::Top)?;
        let bottom = self.border_lines(VerticalSide::Bottom)?;
        if top.is_empty() || bottom.is_empty() {
            self.computed = true;
            return Ok(None);
        }
        let mut total = 0.0;
        let mut count = 0;
        let mut longest: Option<&BasicLine> = None;
        let mut length = 0.0;
        for l in top.iter().chain(&bottom) {
            total += l.mean_distance()? * l.n as f64;
            count += l.n;
            let size = l.xmax - l.xmin + 1.0;
            if longest.is_none() || size > length {
                longest = Some(l);
                length = size;
            }
        }
        let global = longest.unwrap().slope()?;
        let mut top = self.lines_map(global, &top)?;
        let mut bottom = self.lines_map(global, &bottom)?;
        if top
            .len()
            .checked_mul(bottom.len())
            .and_then(|n| n.checked_mul(2))
            .is_none_or(|n| n > self.limits.max_work)
            || top
                .len()
                .checked_add(bottom.len())
                .and_then(|n| n.checked_mul(self.sections.len()))
                .is_none_or(|n| n > self.limits.max_work)
        {
            return Err("unsupported_beam_structure_work_budget".into());
        }
        self.complete_borders(1.0, global, &top, &mut bottom);
        self.complete_borders(-1.0, global, &bottom, &mut top);
        if top.len() != bottom.len() {
            self.computed = true;
            return Ok(None);
        }
        let mut lines = Vec::new();
        let mut item_count = 0usize;
        for ((_, a), (_, b)) in top.into_iter().zip(bottom) {
            let left = a[0].min(b[0]);
            let right = a[2].max(b[2]);
            let (y1, y2, y3, y4) = (
                y_at_x(a, left),
                y_at_x(b, left),
                y_at_x(a, right),
                y_at_x(b, right),
            );
            let height = ((y2 - y1) + (y4 - y3)) / 2.0;
            let mut line = BeamLine {
                median: [left, (y1 + y2) / 2.0, right, (y3 + y4) / 2.0],
                height,
                items: Vec::new(),
            };
            self.retrieve_items(&mut line)?;
            item_count += line.items.len();
            if item_count > self.limits.max_items {
                return Err("unsupported_beam_structure_item_budget".into());
            }
            lines.push(line);
        }
        self.lines = lines;
        self.computed = true;
        Ok(Some(total / count as f64))
    }
    pub fn compare_slopes(&self) -> f64 {
        let mut difference: f64 = 0.0;
        let mut previous: Option<f64> = None;
        for l in &self.lines {
            if l.median[2] - l.median[0] > self.params.max_hook_width {
                let current = slope(l.median);
                if let Some(last) = previous {
                    difference = difference.max((current - last).abs());
                }
                previous = Some(current);
            }
        }
        difference
    }
    pub fn adjust_sides(&mut self) {
        let left = self.bounds[0] as f64;
        let right = (self.bounds[0] + self.bounds[2] - 1) as f64;
        for l in &mut self.lines {
            if l.median[0] - left < self.params.min_beam_width_low {
                let p = intersection_at_x(l.median, left);
                l.median[0] = p[0];
                l.median[1] = p[1];
            }
            if right - l.median[2] < self.params.min_beam_width_low {
                let p = intersection_at_x(l.median, right);
                l.median[2] = p[0];
                l.median[3] = p[1];
            }
        }
    }
    /// Despite its original name, extends *all* lines if at least three exist.
    pub fn extend_middle_lines(&mut self) {
        if self.lines.len() < 3 {
            return;
        }
        let mut left = f64::MAX;
        let mut right = f64::from_bits(1);
        for l in &self.lines {
            left = left.min(l.median[0]);
            right = right.max(l.median[2]);
        }
        for l in &mut self.lines {
            l.median = slice(l.median, left, right);
        }
    }
    /// Original stuck-beam split always produces two empty-item lines, even
    /// when rounded mean thickness suggests more than two. Items are not rebuilt.
    pub fn split_lines(&mut self) {
        if self.lines.len() != 1 {
            return;
        }
        let count = (self.weight as f64 / self.bounds[2] as f64 / self.params.typical_height)
            .round_ties_even() as usize;
        if count <= self.lines.len() {
            return;
        }
        let old = &self.lines[0];
        let gutter = old.height - 2.0 * self.params.typical_height;
        if gutter < 0.0 {
            return;
        }
        let height = (old.height - gutter) / 2.0;
        let offset = (height + gutter) / 2.0;
        let a = old.median;
        let mut b = a;
        let mut c = a;
        b[1] -= offset;
        b[3] -= offset;
        c[1] += offset;
        c[3] += offset;
        self.lines = vec![
            BeamLine {
                median: b,
                height,
                items: Vec::new(),
            },
            BeamLine {
                median: c,
                height,
                items: Vec::new(),
            },
        ];
    }
    pub fn compute_jitter(&self, line: &BeamLine, side: VerticalSide) -> Result<f64, String> {
        if !line.median.iter().all(|v| v.is_finite()) || line.median[0] >= line.median[2] {
            return Err("invalid_beam_structure_jitter_line".into());
        }
        let dx = self.params.corner_margin.round_ties_even();
        let left = (line.median[0] + dx).round_ties_even();
        let right = (line.median[2] - dx).round_ties_even();
        let mut fit = BasicLine::default();
        for s in &self.sections {
            let x = (s.bounds[0] + s.bounds[2] / 2) as f64;
            let y = y_at_x(line.median, x).round_ties_even();
            if !s.contains(x, y) {
                continue;
            }
            for (i, [start, len]) in s.runs.iter().enumerate() {
                let x = (s.first_pos + i as i32) as f64;
                if x >= left && x <= right {
                    fit.include(
                        x,
                        if side == VerticalSide::Top {
                            *start
                        } else {
                            start + len - 1
                        } as f64,
                    );
                }
            }
        }
        Ok(fit.mean_distance()? / self.bounds[2] as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn rectangle(width: usize, height: usize) -> GlyphPixels {
        GlyphPixels {
            left: 100,
            top: 200,
            width,
            height,
            orientation: GlyphOrientation::Vertical,
            runs: (0..width)
                .map(|position| audiveris_staff_removal::Run {
                    position,
                    start: 0,
                    length: height,
                })
                .collect(),
        }
    }
    fn params() -> ItemParameters {
        ItemParameters::new(16, 8.0, false).unwrap()
    }
    fn compare(a: &Value, b: &Value, path: &str) {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 1e-8,
                "{path}: {a} != {b}"
            ),
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{i}]"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (key, a) in a {
                    compare(a, &b[key], &format!("{path}.{key}"));
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn direct_original_method_controls() {
        let controls: Value = serde_json::from_str(include_str!(
            "../fixtures/beam-structure-method-controls.json"
        ))
        .unwrap();
        let cases = controls["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 27);
        for expected in cases {
            let case = &expected["case"];
            let name = case["name"].as_str().unwrap();
            assert_eq!(
                expected["scope"],
                "injected_line_geometry_methods_only_computeLines_not_called"
            );
            let mut p = json!(params());
            if let Some(overrides) = case["parameter_overrides"].as_object() {
                for (k, v) in overrides {
                    p[k] = v.clone();
                }
            }
            let glyph = rectangle(120, case["glyph"]["bounds"][3].as_u64().unwrap() as usize);
            let mut s = BeamStructure::new(&glyph, serde_json::from_value(p).unwrap()).unwrap();
            s.lines = serde_json::from_value(
                case["initial_lines"]
                    .clone()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|l| {
                        let mut l = l.clone();
                        l["items"] = json!([]);
                        l
                    })
                    .collect::<Vec<_>>()
                    .into(),
            )
            .unwrap();
            compare(&json!(s.width()), &expected["width"], name);
            compare(
                &json!(s.compare_slopes()),
                &expected["slope_difference"],
                name,
            );
            s.adjust_sides();
            compare(&json!(s.lines()), &expected["after_adjust_sides"], name);
            s.extend_middle_lines();
            compare(&json!(s.lines()), &expected["after_extend_middle"], name);
            s.split_lines();
            compare(&json!(s.lines()), &expected["after_split_lines"], name);
            let jitter: Vec<Value> = s
                .lines()
                .iter()
                .map(|l| {
                    match (
                        s.compute_jitter(l, VerticalSide::Top),
                        s.compute_jitter(l, VerticalSide::Bottom),
                    ) {
                        (Ok(top), Ok(bottom)) => json!({"top":top,"bottom":bottom}),
                        (Err(error), _) | (_, Err(error)) => {
                            assert_eq!(error, "undefined_beam_structure_line");
                            json!({"exception":"UndefinedLineException"})
                        }
                    }
                })
                .collect();
            compare(&json!(jitter), &expected["jitters"], name);
        }
    }
    #[test]
    fn rejects_bad_parameters_without_mutation() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.0] {
            assert!(ItemParameters::new(16, value, false).is_err());
        }
        assert!(ItemParameters::new(0, 8.0, false).is_err());
        assert!(ItemParameters::new(usize::MAX, 8.0, false).is_err());
        let g = rectangle(32, 8);
        for i in 0..10 {
            for bad in [f64::NAN, f64::INFINITY, -1.0, MAX_PIXELS as f64 + 1.0] {
                let mut p = params();
                let fields = [
                    &mut p.min_beam_width_low,
                    &mut p.min_beam_width_high,
                    &mut p.min_hook_width_low,
                    &mut p.min_hook_width_high,
                    &mut p.max_hook_width,
                    &mut p.min_height_low,
                    &mut p.typical_height,
                    &mut p.max_height_high,
                    &mut p.corner_margin,
                    &mut p.max_item_x_gap,
                ];
                *fields.into_iter().nth(i).unwrap() = bad;
                assert!(BeamStructure::new(&g, p).is_err());
            }
        }
        let mut p = params();
        p.core_section_width = 1;
        assert!(BeamStructure::new(&g, p).is_err());
        p = params();
        p.min_beam_width_high = p.min_beam_width_low - 1.0;
        assert!(BeamStructure::new(&g, p).is_err());
        p = params();
        p.min_hook_width_high = p.min_hook_width_low - 1.0;
        assert!(BeamStructure::new(&g, p).is_err());
        p = params();
        p.max_hook_width = p.min_hook_width_high - 1.0;
        assert!(BeamStructure::new(&g, p).is_err());
        p = params();
        p.max_height_high = p.typical_height - 1.0;
        assert!(BeamStructure::new(&g, p).is_err());
    }
    #[test]
    fn rejects_invalid_glyphs_and_budgets() {
        let original = rectangle(32, 8);
        let mut g = original.clone();
        g.orientation = GlyphOrientation::Horizontal;
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.width = 0;
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.runs.clear();
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.runs[0].length = 0;
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.runs[0].length = usize::MAX;
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.runs[1] = g.runs[0];
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.runs[0].position = 32;
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.left = -1;
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.top = i32::MAX;
        assert!(BeamStructure::new(&g, params()).is_err());
        g = original.clone();
        g.width = usize::MAX;
        assert!(BeamStructure::new(&g, params()).is_err());
        assert!(
            BeamStructure::with_limits(
                &original,
                params(),
                Limits {
                    max_work: 1,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        for limits in [
            Limits {
                max_sections: 0,
                ..Limits::default()
            },
            Limits {
                max_border_lines: 0,
                ..Limits::default()
            },
            Limits {
                max_work: 0,
                ..Limits::default()
            },
            Limits {
                max_items: 0,
                ..Limits::default()
            },
        ] {
            assert!(BeamStructure::with_limits(&original, params(), limits).is_err());
        }
        g = rectangle(32, 24);
        g.runs = (0..32)
            .flat_map(|position| {
                [
                    audiveris_staff_removal::Run {
                        position,
                        start: 0,
                        length: 8,
                    },
                    audiveris_staff_removal::Run {
                        position,
                        start: 16,
                        length: 8,
                    },
                ]
            })
            .collect();
        assert_eq!(
            BeamStructure::with_limits(
                &g,
                params(),
                Limits {
                    max_sections: 1,
                    ..Limits::default()
                }
            )
            .unwrap_err(),
            "unsupported_beam_structure_section_budget"
        );
        let mut s = BeamStructure::with_limits(
            &g,
            params(),
            Limits {
                max_border_lines: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(
            s.compute_lines().unwrap_err(),
            "unsupported_beam_structure_border_budget"
        );
        assert!(s.lines().is_empty());
        let mut s = BeamStructure::with_limits(
            &g,
            params(),
            Limits {
                max_items: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(
            s.compute_lines().unwrap_err(),
            "unsupported_beam_structure_item_budget"
        );
        assert!(s.lines().is_empty());
    }
    #[test]
    fn empty_and_one_shot_contracts() {
        let mut s = BeamStructure::new(&rectangle(1, 8), params()).unwrap();
        assert_eq!(s.width(), None);
        assert_eq!(s.compute_lines().unwrap(), None);
        assert!(s.lines().is_empty());
        assert_eq!(
            s.compute_lines().unwrap_err(),
            "beam_structure_already_computed"
        );
        s.adjust_sides();
        s.extend_middle_lines();
        s.split_lines();
        assert!(s.lines().is_empty());
        let mut s = BeamStructure::new(&rectangle(32, 8), params()).unwrap();
        assert!(s.compute_lines().unwrap().is_some());
        let original = s.lines.clone();
        assert_eq!(
            s.compute_lines().unwrap_err(),
            "beam_structure_already_computed"
        );
        assert_eq!(s.lines, original);
        for median in [
            [0.0, 0.0, 0.0, 1.0],
            [f64::NAN, 0.0, 2.0, 1.0],
            [2.0, 0.0, 1.0, 0.0],
        ] {
            assert_eq!(
                s.compute_jitter(
                    &BeamLine {
                        median,
                        height: 8.0,
                        items: vec![]
                    },
                    VerticalSide::Top
                )
                .unwrap_err(),
                "invalid_beam_structure_jitter_line"
            );
        }
    }
}

/// Original Glyph.computeLine / BasicLine.toCenterLine geometry. Pixel traversal
/// follows each canonical run from its last pixel back to its start.
#[derive(Clone, Debug, Serialize)]
pub struct GlyphGeometry {
    pub mean_thickness: f64,
    pub center_line: Option<[f64; 4]>,
}
fn glyph_fit(glyph: &GlyphPixels) -> Result<BasicLine, String> {
    beam_dispatch::centroid(glyph)?;
    if glyph.left < 0
        || glyph.top < 0
        || glyph.width > sections::MAX_DIMENSION
        || glyph.height > sections::MAX_DIMENSION
        || glyph.runs.len() > sections::MAX_RUNS
        || (glyph.left as usize)
            .checked_add(glyph.width)
            .is_none_or(|v| v > sections::MAX_DIMENSION)
        || (glyph.top as usize)
            .checked_add(glyph.height)
            .is_none_or(|v| v > sections::MAX_DIMENSION)
    {
        return Err("unsupported_beam_glyph_geometry_bounds".into());
    }
    Ok(glyph_fit_pixels(glyph))
}
fn glyph_fit_pixels(glyph: &GlyphPixels) -> BasicLine {
    let mut line = BasicLine::default();
    for run in &glyph.runs {
        for delta in (0..run.length).rev() {
            let along = (run.start + delta) as f64;
            let across = run.position as f64;
            let (x, y) = if glyph.orientation == GlyphOrientation::Vertical {
                (across, along)
            } else {
                (along, across)
            };
            line.include(glyph.left as f64 + x, glyph.top as f64 + y);
        }
    }
    line
}

/// Original Glyph.getLine, before the distinct pixel-center conversion.
pub fn glyph_stem_line(glyph: &GlyphPixels) -> Result<Option<[f64; 4]>, String> {
    let line = glyph_fit(glyph)?;
    if line.n == 1 {
        return Ok(None);
    }
    line.segment().map(Some)
}

/// Original Glyph.getMeanDistance, using the same absolute raw moments as getLine.
pub fn glyph_mean_distance(glyph: &GlyphPixels) -> Result<f64, String> {
    glyph_fit(glyph)?.mean_distance()
}

pub fn glyph_geometry(glyph: &GlyphPixels) -> Result<GlyphGeometry, String> {
    let line = glyph_fit(glyph)?;
    geometry_from_fit(glyph, line)
}
/// Original absolute-coordinate Glyph fit for explicitly selected stem parts.
/// Negative origins can occur at a page boundary. Keep beam callers' existing
/// nonnegative bounds contract, dimensional/run budgets and pixel order intact.
pub fn glyph_geometry_signed(glyph: &GlyphPixels) -> Result<GlyphGeometry, String> {
    beam_dispatch::centroid(glyph)?;
    let bounds = [
        glyph.left as i64,
        glyph.top as i64,
        glyph.left as i64 + glyph.width as i64,
        glyph.top as i64 + glyph.height as i64,
    ];
    if glyph.width > sections::MAX_DIMENSION
        || glyph.height > sections::MAX_DIMENSION
        || glyph.runs.len() > sections::MAX_RUNS
        || bounds
            .iter()
            .any(|v| v.unsigned_abs() > sections::MAX_DIMENSION as u64)
    {
        return Err("unsupported_signed_glyph_geometry_bounds".into());
    }
    geometry_from_fit(glyph, glyph_fit_pixels(glyph))
}
fn geometry_from_fit(glyph: &GlyphPixels, line: BasicLine) -> Result<GlyphGeometry, String> {
    let mean_thickness = line.n as f64 / glyph.width as f64;
    if line.n == 1 {
        return Ok(GlyphGeometry {
            mean_thickness,
            center_line: None,
        });
    }
    let (a, b, c, vertical) = line.coefficients()?;
    let center_line = if vertical {
        [
            (-b * line.ymin - c) / a + 0.5,
            line.ymin,
            (-b * line.ymax - c) / a + 0.5,
            line.ymax + 1.0,
        ]
    } else {
        [
            line.xmin,
            (-a * line.xmin - c) / b + 0.5,
            line.xmax + 1.0,
            (-a * line.xmax - c) / b + 0.5,
        ]
    };
    Ok(GlyphGeometry {
        mean_thickness,
        center_line: Some(center_line),
    })
}
