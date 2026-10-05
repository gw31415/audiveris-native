// Copyright © Audiveris 2026. Original author: Hervé Bitteur.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact staff glyph erasure, then four-connected GlyphFactory extraction.
//! Geometry/section membership selection belongs to the upstream Staff/GRID stage.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub position: usize,
    pub start: usize,
    pub length: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphPixels {
    pub left: i32,
    pub top: i32,
    pub width: usize,
    pub height: usize,
    pub orientation: Orientation,
    /// Canonical sorted, nonadjacent foreground runs in local coordinates.
    pub runs: Vec<Run>,
}
/// Native resource budgets; not claimed to be upstream Audiveris limits.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_pixels: usize,
    pub max_dimension: usize,
    pub max_glyphs: usize,
    pub max_runs: usize,
    pub max_run_pixels: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_pixels: 16_000_000,
            max_dimension: 65_536,
            max_glyphs: 100_000,
            max_runs: 1_000_000,
            max_run_pixels: 64_000_000,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BinaryPage<'a> {
    width: usize,
    height: usize,
    pixels: &'a [u8],
}
impl<'a> BinaryPage<'a> {
    pub fn new(width: usize, height: usize, pixels: &'a [u8]) -> Result<Self, Error> {
        if width == 0 || height == 0 || width.checked_mul(height) != Some(pixels.len()) {
            return Err(Error::Dimensions);
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    pub fn width(self) -> usize {
        self.width
    }
    pub fn height(self) -> usize {
        self.height
    }
    pub fn pixels(self) -> &'a [u8] {
        self.pixels
    }
    fn validate(self, limits: Limits) -> Result<(), Error> {
        if self.width > limits.max_dimension
            || self.height > limits.max_dimension
            || self.pixels.len() > limits.max_pixels
        {
            return Err(Error::Budget);
        }
        if self.pixels.iter().any(|&p| p != 0 && p != 255) {
            return Err(Error::NotBinary);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Dimensions,
    NotBinary,
    Budget,
    RunBounds,
    RunOrder,
    CoordinateOverflow,
    Allocation,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
#[derive(Debug)]
pub struct EraseResult {
    pub residual: Vec<u8>,
    /// 0 for black pixels actually changed to white; 255 elsewhere.
    pub removed_mask: Vec<u8>,
    pub removed_pixels: usize,
}
fn byte_buffer(n: usize, value: u8) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    out.try_reserve_exact(n).map_err(|_| Error::Allocation)?;
    out.resize(n, value);
    Ok(out)
}
fn validate_glyph(g: &GlyphPixels, limits: Limits) -> Result<(usize, usize), Error> {
    if g.runs.len() > limits.max_runs {
        return Err(Error::Budget);
    }
    if g.width == 0 || g.height == 0 {
        return Err(Error::Dimensions);
    }
    if g.width > limits.max_dimension || g.height > limits.max_dimension {
        return Err(Error::Budget);
    }
    // Keep every coordinate conversion safe even with caller-supplied budgets.
    if g.width > i32::MAX as usize || g.height > i32::MAX as usize {
        return Err(Error::CoordinateOverflow);
    }
    let (seq, extent) = match g.orientation {
        Orientation::Horizontal => (g.height, g.width),
        Orientation::Vertical => (g.width, g.height),
    };
    let mut prev: Option<Run> = None;
    let mut mass = 0usize;
    for &r in &g.runs {
        if r.position >= seq
            || r.length == 0
            || r.start.checked_add(r.length).is_none_or(|e| e > extent)
        {
            return Err(Error::RunBounds);
        }
        if let Some(p) = prev
            && (r.position < p.position
                || r.position == p.position && r.start <= p.start + p.length)
        {
            return Err(Error::RunOrder);
        }
        mass = mass.checked_add(r.length).ok_or(Error::Budget)?;
        prev = Some(r);
        if mass > limits.max_run_pixels {
            return Err(Error::Budget);
        }
    }
    Ok((g.runs.len(), mass))
}
/// Picture.buildNoStaffBuffer + RunTable.render: paint exact glyph pixels white.
/// Signed origins are clipped to the image as by Graphics2D; masks may overlap.
/// All validation occurs before allocation/mutation. Input pixels are borrowed.
pub fn erase_staff(
    page: BinaryPage<'_>,
    glyphs: &[GlyphPixels],
    limits: Limits,
) -> Result<EraseResult, Error> {
    page.validate(limits)?;
    if glyphs.len() > limits.max_glyphs {
        return Err(Error::Budget);
    }
    let (mut runs, mut mass) = (0usize, 0usize);
    for g in glyphs {
        let (n, m) = validate_glyph(g, limits)?;
        runs = runs.checked_add(n).ok_or(Error::Budget)?;
        mass = mass.checked_add(m).ok_or(Error::Budget)?;
        if runs > limits.max_runs || mass > limits.max_run_pixels {
            return Err(Error::Budget);
        }
    }
    let mut result = EraseResult {
        residual: byte_buffer(page.pixels.len(), 255)?,
        removed_mask: byte_buffer(page.pixels.len(), 255)?,
        removed_pixels: 0,
    };
    result.residual.copy_from_slice(page.pixels);
    for g in glyphs {
        for r in &g.runs {
            let (fixed, begin, limit, extent) = match g.orientation {
                Orientation::Horizontal => (
                    g.top as i64 + r.position as i64,
                    g.left as i64 + r.start as i64,
                    page.height,
                    page.width,
                ),
                Orientation::Vertical => (
                    g.left as i64 + r.position as i64,
                    g.top as i64 + r.start as i64,
                    page.width,
                    page.height,
                ),
            };
            if fixed < 0 || fixed >= limit as i64 {
                continue;
            }
            let end = (begin + r.length as i64).min(extent as i64);
            let start = begin.max(0);
            for c in start..end {
                let i = match g.orientation {
                    Orientation::Horizontal => fixed as usize * page.width + c as usize,
                    Orientation::Vertical => c as usize * page.width + fixed as usize,
                };
                if result.residual[i] == 0 {
                    result.residual[i] = 255;
                    result.removed_mask[i] = 0;
                    result.removed_pixels += 1;
                }
            }
        }
    }
    Ok(result)
}

fn push<T>(v: &mut Vec<T>, value: T) -> Result<(), Error> {
    v.try_reserve(1).map_err(|_| Error::Allocation)?;
    v.push(value);
    Ok(())
}
fn ancestor(parents: &mut [usize], mut id: usize) -> usize {
    while parents[id] != id {
        parents[id] = parents[parents[id]];
        id = parents[id];
    }
    id
}
#[derive(Clone, Copy)]
struct Marked {
    run: Run,
    mark: usize,
}
/// GlyphFactory.scanTable/createLut/buildAllGlyphs/buildGlyph.
/// Strict overlap between consecutive sequences gives FOUR-connectivity.
/// Output order follows the smallest ancestral mark (first scan occurrence),
/// preserving the source orientation and canonical glyph-local foreground runs.
pub fn extract_glyphs(
    page: BinaryPage<'_>,
    orientation: Orientation,
    origin: (i32, i32),
    limits: Limits,
) -> Result<Vec<GlyphPixels>, Error> {
    page.validate(limits)?;
    if page.width > i32::MAX as usize || page.height > i32::MAX as usize {
        return Err(Error::CoordinateOverflow);
    }
    let (sequences, extent) = match orientation {
        Orientation::Horizontal => (page.height, page.width),
        Orientation::Vertical => (page.width, page.height),
    };
    let mut all: Vec<Marked> = Vec::new();
    let mut parents = vec![0usize];
    let (mut prev_begin, mut prev_end, mut mass) = (0usize, 0usize, 0usize);
    for position in 0..sequences {
        let begin = all.len();
        let mut coord = 0usize;
        let mut active = prev_begin;
        while coord < extent {
            let index = |c: usize| match orientation {
                Orientation::Horizontal => position * page.width + c,
                Orientation::Vertical => c * page.width + position,
            };
            if page.pixels[index(coord)] != 0 {
                coord += 1;
                continue;
            }
            let start = coord;
            while coord < extent && page.pixels[index(coord)] == 0 {
                coord += 1;
            }
            let run = Run {
                position,
                start,
                length: coord - start,
            };
            if all.len() >= limits.max_runs {
                return Err(Error::Budget);
            }
            mass = mass.checked_add(run.length).ok_or(Error::Budget)?;
            if mass > limits.max_run_pixels {
                return Err(Error::Budget);
            }
            // Equivalent to Java pIdxActive, skipping permanently preceding
            // intervals even when there is no overlap (linear checkerboard scan).
            while active < prev_end && all[active].run.start + all[active].run.length <= start {
                active += 1;
            }
            let mut mark = 0;
            for prev in &all[active..prev_end] {
                if prev.run.start >= coord {
                    break;
                }
                if mark == 0 {
                    mark = prev.mark;
                } else {
                    let a = ancestor(&mut parents, mark);
                    let b = ancestor(&mut parents, prev.mark);
                    parents[a.max(b)] = a.min(b);
                }
            }
            if mark == 0 {
                mark = parents.len();
                push(&mut parents, mark)?;
            }
            push(&mut all, Marked { run, mark })?;
        }
        prev_begin = begin;
        prev_end = all.len();
    }
    let mut mapping = Vec::new();
    mapping
        .try_reserve_exact(parents.len())
        .map_err(|_| Error::Allocation)?;
    mapping.resize(parents.len(), usize::MAX);
    // Temporary bounds are [sequence_min, sequence_max, coordinate_min, coordinate_max_exclusive].
    let mut bounds: Vec<[usize; 4]> = Vec::new();
    let mut glyphs: Vec<GlyphPixels> = Vec::new();
    for marked in all {
        let root = ancestor(&mut parents, marked.mark);
        let run = marked.run;
        if mapping[root] == usize::MAX {
            if glyphs.len() >= limits.max_glyphs {
                return Err(Error::Budget);
            }
            mapping[root] = glyphs.len();
            push(
                &mut glyphs,
                GlyphPixels {
                    left: 0,
                    top: 0,
                    width: 0,
                    height: 0,
                    orientation,
                    runs: Vec::new(),
                },
            )?;
            push(
                &mut bounds,
                [
                    run.position,
                    run.position,
                    run.start,
                    run.start + run.length,
                ],
            )?;
        }
        let i = mapping[root];
        let b = &mut bounds[i];
        b[1] = run.position;
        b[2] = b[2].min(run.start);
        b[3] = b[3].max(run.start + run.length);
        push(&mut glyphs[i].runs, run)?;
    }
    for (g, b) in glyphs.iter_mut().zip(bounds) {
        let (x, y, width, height) = match orientation {
            Orientation::Horizontal => (b[2], b[0], b[3] - b[2], b[1] - b[0] + 1),
            Orientation::Vertical => (b[0], b[2], b[1] - b[0] + 1, b[3] - b[2]),
        };
        g.left = origin
            .0
            .checked_add(x as i32)
            .ok_or(Error::CoordinateOverflow)?;
        g.top = origin
            .1
            .checked_add(y as i32)
            .ok_or(Error::CoordinateOverflow)?;
        g.left
            .checked_add((width - 1) as i32)
            .ok_or(Error::CoordinateOverflow)?;
        g.top
            .checked_add((height - 1) as i32)
            .ok_or(Error::CoordinateOverflow)?;
        g.width = width;
        g.height = height;
        for r in &mut g.runs {
            r.position -= b[0];
            r.start -= b[2];
        }
    }
    Ok(glyphs)
}
