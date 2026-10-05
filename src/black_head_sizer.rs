// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
//! Native BlackHeadSizer from pinned Audiveris 5.11.0 / 9e1e55cd2746037d059345881c53e6a6754bffbd.
//! Closing creates new glyphs. It never changes the passed beam spot list. The
//! caller owns system membership, canonical registration and group union.
use crate::{GrayImage, MAX_PIXELS, beam_spots};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde::Serialize;
use std::sync::OnceLock;

/// Explicit native coverage bound, not an upstream restriction.
pub const MAX_INTERLINE: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Parameters {
    pub min_width: usize,
    pub max_width: usize,
    pub min_height: usize,
    pub max_height: usize,
    pub min_weight: usize,
    pub max_weight: usize,
    pub min_stack_height: usize,
    pub max_stack_height: usize,
    pub min_stack_weight: usize,
    pub max_stack_weight: usize,
    pub diameter: f64,
    pub binarization_threshold: u8,
    pub singles_quorum: usize,
}
impl Parameters {
    pub fn from_interline(interline: usize) -> Result<Self, String> {
        if !(1..=MAX_INTERLINE).contains(&interline) {
            return Err("unsupported_black_head_interline".into());
        }
        let i = interline as f64;
        let length = |fraction: f64| (i * fraction).round_ties_even() as usize;
        // Scale.InterlineScale.toPixels(AreaFraction) multiplies interline^2
        // in integer arithmetic before multiplying by the double fraction.
        let area =
            |fraction: f64| ((interline * interline) as f64 * fraction).round_ties_even() as usize;
        Ok(Self {
            min_width: length(1.0),
            max_width: length(2.0),
            min_height: length(0.9),
            max_height: length(1.5),
            min_weight: area(0.8),
            max_weight: area(1.5),
            min_stack_height: length(2.0),
            max_stack_height: length(4.0),
            min_stack_weight: area(2.0),
            max_stack_weight: area(4.0),
            diameter: i * 0.9,
            binarization_threshold: 110,
            singles_quorum: 20,
        })
    }
    fn validate(&self) -> Result<(), String> {
        if !self.diameter.is_finite()
            || self.diameter < 0.0
            || self.diameter > MAX_INTERLINE as f64 * 0.9
            || self.min_width > self.max_width
            || self.min_height > self.max_height
            || self.max_height > self.max_stack_height
            || self.min_stack_height > self.max_stack_height
            || self.min_weight > self.max_weight
            || self.max_weight > self.max_stack_weight
            || self.min_stack_weight > self.max_stack_weight
            || self.max_width > MAX_PIXELS
            || self.max_stack_height > MAX_PIXELS
            || self.max_stack_weight > MAX_PIXELS
            || self.singles_quorum < 2
        {
            return Err("invalid_black_head_parameters".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum HeadGroup {
    #[serde(rename = "BLACK_HEAD_SPOT")]
    Single,
    #[serde(rename = "BLACK_STACK_SPOT")]
    Stack,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BlackHeadScale {
    pub width_mean: f64,
    pub width_std: f64,
    pub height_mean: f64,
    pub height_std: f64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MusicFontScale {
    pub name: String,
    pub point_size: i32,
}
#[derive(Debug)]
pub struct ClosedCandidate {
    pub source_index: usize,
    pub glyph: GlyphPixels,
    /// Canonical identity returned by the caller, absent outside eligible systems.
    pub registered_id: Option<usize>,
    /// Registration deliberately occurs before this second size check.
    pub accepted_after_close: bool,
    pub group: Option<HeadGroup>,
}
#[derive(Debug)]
pub struct SizingResult {
    pub parameters: Parameters,
    /// Every exactly-one-component closing, including post-close rejects.
    pub closed_candidates: Vec<ClosedCandidate>,
    /// Indexes into closed_candidates. Singles are stably width-sorted only when
    /// quorum is met, exactly as measureSingles mutates the local single list.
    pub singles: Vec<usize>,
    pub stacks: Vec<usize>,
    pub measured_core: Vec<usize>,
    pub black_head_scale: Option<BlackHeadScale>,
    pub music_font_scale: Option<MusicFontScale>,
    /// A missing font metric does not erase a successfully measured head scale.
    pub music_font_unsupported: Option<String>,
}

fn validate_glyph(g: &GlyphPixels) -> Result<usize, String> {
    if g.width == 0
        || g.height == 0
        || g.width.checked_mul(g.height).is_none_or(|n| n > MAX_PIXELS)
        || g.runs.is_empty()
        || g.runs.len() > 1_000_000
        || i64::from(g.left) + g.width as i64 - 1 > i32::MAX as i64
        || i64::from(g.top) + g.height as i64 - 1 > i32::MAX as i64
    {
        return Err("invalid_black_head_glyph".into());
    }
    let (seq, extent) = match g.orientation {
        Orientation::Vertical => (g.width, g.height),
        Orientation::Horizontal => (g.height, g.width),
    };
    let mut previous: Option<Run> = None;
    let mut weight = 0;
    for &r in &g.runs {
        if r.position >= seq
            || r.length == 0
            || r.start.checked_add(r.length).is_none_or(|v| v > extent)
            || previous.is_some_and(|p| {
                r.position < p.position
                    || (r.position == p.position && r.start <= p.start + p.length)
            })
        {
            return Err("invalid_black_head_glyph_runs".into());
        }
        weight += r.length;
        previous = Some(r);
    }
    Ok(weight)
}
fn check_dimensions(g: &GlyphPixels, weight: usize, p: &Parameters) -> bool {
    (p.min_width..=p.max_width).contains(&g.width)
        && (p.min_weight..=p.max_stack_weight).contains(&weight)
        && (p.min_height..=p.max_stack_height).contains(&g.height)
}
pub fn check_spot(g: &GlyphPixels, p: &Parameters) -> Result<bool, String> {
    p.validate()?;
    Ok(check_dimensions(g, validate_glyph(g)?, p))
}

/// Glyph.getBuffer -> original morphology -> threshold -> vertical GlyphFactory.
/// The local glyph buffer has no extra padding. Zero/multiple resulting glyphs
/// produce None, even when a largest component could otherwise be selected.
pub fn close_black_head(g: &GlyphPixels, p: &Parameters) -> Result<Option<GlyphPixels>, String> {
    p.validate()?;
    validate_glyph(g)?;
    let mut image = GrayImage::new(g.width, g.height, vec![255; g.width * g.height])?;
    for run in &g.runs {
        for c in run.start..run.start + run.length {
            let (x, y) = match g.orientation {
                Orientation::Vertical => (run.position, c),
                Orientation::Horizontal => (c, run.position),
            };
            image.pixels[y * g.width + x] = 0;
        }
    }
    let radius = ((p.diameter - 1.0) as f32) / 2.0_f32;
    let closed = beam_spots::close_with_radius(&image, radius)?;
    let binary = beam_spots::threshold(&closed, p.binarization_threshold)?;
    let mut glyphs = beam_spots::extract_spots(&binary)?;
    if glyphs.len() != 1 {
        return Ok(None);
    }
    let mut out = glyphs.pop().unwrap();
    out.left = g
        .left
        .checked_add(out.left)
        .ok_or("black_head_coordinate_overflow")?;
    out.top = g
        .top
        .checked_add(out.top)
        .ok_or("black_head_coordinate_overflow")?;
    Ok(Some(out))
}

/// Population uses the sample standard deviation with this precise sequence of
/// double operations, rather than reducing the expression algebraically.
fn population(values: impl Iterator<Item = usize>) -> (f64, f64) {
    let (mut n, mut s, mut s2) = (0usize, 0.0, 0.0);
    for value in values {
        let v = value as f64;
        n += 1;
        s += v;
        s2 += v * v;
    }
    let nf = n as f64;
    let variance = if n == 1 {
        0.0
    } else {
        nf * ((s2 - s * s / nf) / nf).max(0.0) / (nf - 1.0)
    };
    (s / nf, variance.sqrt())
}

/// The embedded table contains generic original TextLayout widths for every
/// integer Bravura size 0..1024, derived from the pinned OTF, never image results.
pub fn music_font_scale(width: f64, family: &str) -> Result<MusicFontScale, String> {
    if family != "Bravura" {
        return Err("unsupported_black_head_music_family".into());
    }
    if !width.is_finite() || width <= 0.0 || width > 256.0 {
        return Err("unsupported_black_head_font_width".into());
    }
    static WIDTHS: OnceLock<Vec<f64>> = OnceLock::new();
    let widths = WIDTHS.get_or_init(|| {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../assets/black-head-font-metrics.json"))
                .expect("checked-in Bravura metrics");
        v["widths"]
            .as_array()
            .expect("width array")
            .iter()
            .map(|x| x.as_f64().expect("finite font width"))
            .collect()
    });
    let metric = |size: i32| {
        widths
            .get(size as usize)
            .copied()
            .ok_or_else(|| "unsupported_black_head_font_size".to_string())
    };
    let v1 = (width * 3.3).round_ties_even() as i32;
    let w1 = metric(v1)?;
    let dv = (width * 0.25).round_ties_even() as i32;
    let v2 = if w1 < width { v1 + dv } else { v1 - dv };
    let w2 = metric(v2)?;
    let dw = w2 - w1;
    let point_size = if dw.abs() < 0.01 {
        v1
    } else {
        (v1 as f64 + (v2 - v1) as f64 * ((width - w1) / (w2 - w1))).round_ties_even() as i32
    };
    Ok(MusicFontScale {
        name: family.to_string(),
        point_size,
    })
}

#[derive(Debug)]
pub struct SingleMeasurement {
    pub sorted_indices: Vec<usize>,
    pub core_indices: Vec<usize>,
    pub black_head_scale: Option<BlackHeadScale>,
    pub music_font_scale: Option<MusicFontScale>,
    pub music_font_unsupported: Option<String>,
}
/// Original measureSingles boundary, including no-op below quorum. None means
/// leave a pre-existing sheet scale unchanged; it does not mean erase that scale.
/// Width ties retain input order, and height uses the same width-selected core.
pub fn measure_singles(
    singles: &[&GlyphPixels],
    quorum: usize,
    family: &str,
) -> Result<SingleMeasurement, String> {
    if quorum < 2 || singles.len() > 100_000 {
        return Err("invalid_black_head_measurement".into());
    }
    let run_count = singles
        .iter()
        .try_fold(0usize, |n, g| n.checked_add(g.runs.len()));
    if run_count.is_none_or(|n| n > 1_000_000) {
        return Err("unsupported_black_head_measurement_runs".into());
    }
    for glyph in singles {
        validate_glyph(glyph)?;
    }
    let mut out = SingleMeasurement {
        sorted_indices: (0..singles.len()).collect(),
        core_indices: vec![],
        black_head_scale: None,
        music_font_scale: None,
        music_font_unsupported: None,
    };
    if singles.len() < quorum {
        return Ok(out);
    }
    out.sorted_indices.sort_by_key(|&i| singles[i].width);
    let n = singles.len();
    out.core_indices = out.sorted_indices[n / 4..3 * n / 4].to_vec();
    let (width_mean, width_std) = population(out.core_indices.iter().map(|&i| singles[i].width));
    let (height_mean, height_std) = population(out.core_indices.iter().map(|&i| singles[i].height));
    out.black_head_scale = Some(BlackHeadScale {
        width_mean,
        width_std,
        height_mean,
        height_std,
    });
    match music_font_scale(width_mean, family) {
        Ok(scale) => out.music_font_scale = Some(scale),
        Err(reason) => out.music_font_unsupported = Some(reason),
    }
    Ok(out)
}

/// Default original processing with a caller-supplied registration boundary.
/// Registration can be skipped (None) without preventing tagging/measurement.
/// Callers union each returned group onto the registered canonical glyph, while
/// keeping the original passed spot list's order and references unchanged.
pub fn process(
    spots: &[GlyphPixels],
    parameters: Parameters,
    music_family: &str,
    mut register: impl FnMut(&GlyphPixels) -> Result<Option<usize>, String>,
) -> Result<SizingResult, String> {
    parameters.validate()?;
    if spots.len() > 100_000 {
        return Err("unsupported_black_head_spot_count".into());
    }
    // Validate all caller data before registration mutates its external registry.
    let run_count = spots
        .iter()
        .try_fold(0usize, |n, g| n.checked_add(g.runs.len()));
    if run_count.is_none_or(|n| n > 1_000_000) {
        return Err("unsupported_black_head_total_runs".into());
    }
    let weights: Vec<_> = spots.iter().map(validate_glyph).collect::<Result<_, _>>()?;
    let radius = ((parameters.diameter - 1.0) as f32) / 2.0_f32;
    let support = beam_spots::circular_offsets(radius)?.len();
    let work = spots
        .iter()
        .zip(&weights)
        .filter(|(g, w)| check_dimensions(g, **w, &parameters))
        .try_fold(0usize, |total, (g, _)| {
            g.width
                .checked_mul(g.height)
                .and_then(|n| n.checked_mul(support))
                .and_then(|n| n.checked_mul(2))
                .and_then(|n| total.checked_add(n))
        });
    if work.is_none_or(|n| n > beam_spots::MAX_MORPH_PROBES) {
        return Err("unsupported_black_head_total_morph_work".into());
    }
    let mut out = SizingResult {
        parameters,
        closed_candidates: vec![],
        singles: vec![],
        stacks: vec![],
        measured_core: vec![],
        black_head_scale: None,
        music_font_scale: None,
        music_font_unsupported: None,
    };
    for (source_index, (spot, weight)) in spots.iter().zip(weights).enumerate() {
        if !check_dimensions(spot, weight, &parameters) {
            continue;
        }
        let Some(glyph) = close_black_head(spot, &parameters)? else {
            continue;
        };
        let registered_id = register(&glyph)?;
        let weight = glyph.runs.iter().map(|r| r.length).sum();
        let accepted_after_close = check_dimensions(&glyph, weight, &parameters);
        let group = if !accepted_after_close {
            None
        } else if glyph.height <= parameters.max_height && weight <= parameters.max_weight {
            Some(HeadGroup::Single)
        } else if glyph.height >= parameters.min_stack_height
            && weight >= parameters.min_stack_weight
        {
            Some(HeadGroup::Stack)
        } else {
            None
        };
        let index = out.closed_candidates.len();
        if group == Some(HeadGroup::Single) {
            out.singles.push(index);
        }
        if group == Some(HeadGroup::Stack) {
            out.stacks.push(index);
        }
        out.closed_candidates.push(ClosedCandidate {
            source_index,
            glyph,
            registered_id,
            accepted_after_close,
            group,
        });
    }
    let singles: Vec<_> = out
        .singles
        .iter()
        .map(|&i| &out.closed_candidates[i].glyph)
        .collect();
    let measured = measure_singles(&singles, parameters.singles_quorum, music_family)?;
    out.measured_core = measured
        .core_indices
        .iter()
        .map(|&i| out.singles[i])
        .collect();
    out.singles = measured
        .sorted_indices
        .iter()
        .map(|&i| out.singles[i])
        .collect();
    out.black_head_scale = measured.black_head_scale;
    out.music_font_scale = measured.music_font_scale;
    out.music_font_unsupported = measured.music_font_unsupported;
    Ok(out)
}
