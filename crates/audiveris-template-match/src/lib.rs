// SPDX-License-Identifier: AGPL-3.0-or-later
// Audiveris 5.11.0 numerical translations, 2026.
// Chamfer code originally by Xavier Philippeau; kernels Verwer/Borgefors/Thiel;
// Audiveris integration and Template by Hervé Bitteur. See NOTICE.
use std::fmt;
pub const UNKNOWN: i32 = -1;
pub const NORMALIZER: i32 = 3;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Storage {
    JavaShort,
    Integer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Foreground,
    Background,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_pixels: usize,
    pub max_dimension: usize,
    pub max_template_points: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_pixels: 16_000_000,
            max_dimension: 65_536,
            max_template_points: 1_000_000,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Dimensions,
    Budget,
    MaskLength,
    Allocation,
    CoordinateOverflow,
    NonFinite,
    InvalidWeight,
    NumericOverflow,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
#[derive(Debug)]
enum Values {
    Short(Vec<i16>),
    Integer(Vec<i32>),
}
#[derive(Debug)]
pub struct DistanceTable {
    width: usize,
    height: usize,
    values: Values,
    overflowed: bool,
}
impl DistanceTable {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn normalizer(&self) -> i32 {
        NORMALIZER
    }
    /// True if Java's signed-short narrowing actually wrapped a stored value.
    pub fn overflowed(&self) -> bool {
        self.overflowed
    }
    pub fn values(&self) -> impl Iterator<Item = i32> + '_ {
        (0..self.width * self.height).map(|i| self.value(i))
    }
    pub fn get(&self, x: usize, y: usize) -> Option<i32> {
        if x < self.width && y < self.height {
            Some(self.value(y * self.width + x))
        } else {
            None
        }
    }
    fn value(&self, i: usize) -> i32 {
        match &self.values {
            Values::Short(v) => v[i] as i32,
            Values::Integer(v) => v[i],
        }
    }
    fn set(&mut self, i: usize, v: i32) {
        match &mut self.values {
            Values::Short(a) => {
                self.overflowed |= v < i16::MIN as i32 || v > i16::MAX as i32;
                a[i] = v as i16;
            }
            Values::Integer(a) => a[i] = v,
        }
    }
    /// Mask neutralization occurs AFTER propagation; -1 is ignored by Template.
    pub fn neutralize(&mut self, mask: &[bool]) -> Result<(), Error> {
        if mask.len() != self.width * self.height {
            return Err(Error::MaskLength);
        }
        for (i, &ignored) in mask.iter().enumerate() {
            if ignored {
                self.set(i, UNKNOWN);
            }
        }
        Ok(())
    }
    fn allocate(
        width: usize,
        height: usize,
        storage: Storage,
        limits: Limits,
    ) -> Result<Self, Error> {
        let n = width
            .checked_mul(height)
            .filter(|&n| n > 0)
            .ok_or(Error::Dimensions)?;
        if width > limits.max_dimension || height > limits.max_dimension || n > limits.max_pixels {
            return Err(Error::Budget);
        }
        // Bounds coordinate arithmetic and default integer-distance propagation.
        if width > i32::MAX as usize / 8 || height > i32::MAX as usize / 8 {
            return Err(Error::CoordinateOverflow);
        }
        let values = match storage {
            Storage::JavaShort => {
                let mut v = Vec::new();
                v.try_reserve_exact(n).map_err(|_| Error::Allocation)?;
                v.resize(n, -1i16);
                Values::Short(v)
            }
            Storage::Integer => {
                let mut v = Vec::new();
                v.try_reserve_exact(n).map_err(|_| Error::Allocation)?;
                v.resize(n, -1i32);
                Values::Integer(v)
            }
        };
        Ok(Self {
            width,
            height,
            values,
            overflowed: false,
        })
    }
    fn test_and_set(&mut self, x: i64, y: i64, new_value: i32) {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return;
        }
        let i = y as usize * self.width + x as usize;
        let old = self.value(i);
        if old >= 0 && old < new_value {
            return;
        }
        self.set(i, new_value);
    }
    fn process(&mut self) -> Result<(), Error> {
        for backward in [false, true] {
            for row in 0..self.height {
                let y = if backward { self.height - 1 - row } else { row };
                for col in 0..self.width {
                    let x = if backward { self.width - 1 - col } else { col };
                    let v = self.value(y * self.width + x);
                    if v == UNKNOWN {
                        continue;
                    }
                    let s = if backward { -1 } else { 1 };
                    let x = x as i64;
                    let y = y as i64;
                    // DEFAULT_MASK: (1,0,3), (1,1,4), same update order as Java.
                    for (dx, dy, dt) in [(1, 0, 3), (1, 1, 4)] {
                        let nv = v.checked_add(dt).ok_or(Error::NumericOverflow)?;
                        self.test_and_set(x + s * dx, y + s * dy, nv);
                        if dy != 0 {
                            self.test_and_set(x - s * dx, y + s * dy, nv);
                        }
                        if dx != dy {
                            self.test_and_set(x + s * dy, y + s * dx, nv);
                            if dy != 0 {
                                self.test_and_set(x - s * dy, y + s * dx, nv);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
/// Exact default 3/4 chamfer. Java considers byte value 0 foreground, any other value background.
pub fn compute(
    width: usize,
    height: usize,
    pixels: &[u8],
    target: Target,
    storage: Storage,
    limits: Limits,
) -> Result<DistanceTable, Error> {
    if width.checked_mul(height) != Some(pixels.len()) {
        return Err(Error::Dimensions);
    }
    let mut table = DistanceTable::allocate(width, height, storage, limits)?;
    for (i, &p) in pixels.iter().enumerate() {
        if (p == 0) == (target == Target::Foreground) {
            table.set(i, 0);
        }
    }
    table.process()?;
    Ok(table)
}
/// Pure pixel part of DistancesBuilder: remove ignored pixels from foreground,
/// compute distances, then mark the same locations UNKNOWN. Mask selection is caller-owned.
pub fn compute_masked_foreground(
    width: usize,
    height: usize,
    pixels: &[u8],
    mask: &[bool],
    storage: Storage,
    limits: Limits,
) -> Result<DistanceTable, Error> {
    if pixels.len() != mask.len() {
        return Err(Error::MaskLength);
    }
    if width.checked_mul(height) != Some(pixels.len()) {
        return Err(Error::Dimensions);
    }
    let mut table = DistanceTable::allocate(width, height, storage, limits)?;
    for (i, &p) in pixels.iter().enumerate() {
        if p == 0 && !mask[i] {
            table.set(i, 0);
        }
    }
    table.process()?;
    table.neutralize(mask)?;
    Ok(table)
}

#[derive(Clone, Copy, Debug)]
pub struct KeyPoint {
    pub x: i32,
    pub y: i32,
    pub distance: f64,
}
#[derive(Clone, Copy, Debug)]
pub enum AnchorSide {
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy, Debug)]
pub struct AnchorOffset {
    pub x: f64,
    pub y: f64,
    pub side: AnchorSide,
}
impl AnchorOffset {
    /// Template.getOffset: Math.round(offset-.5 +/- .001), not ties-to-even.
    pub fn rounded(self) -> Result<(i32, i32), Error> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(Error::NonFinite);
        }
        let bias = match self.side {
            AnchorSide::Left => -0.001,
            AnchorSide::Center => 0.,
            AnchorSide::Right => 0.001,
        };
        let rx = ((self.x - 0.5 + bias) + 0.5).floor();
        let ry = ((self.y - 0.5) + 0.5).floor();
        if rx < i32::MIN as f64
            || rx > i32::MAX as f64
            || ry < i32::MIN as f64
            || ry > i32::MAX as f64
        {
            return Err(Error::CoordinateOverflow);
        }
        Ok((rx as i32, ry as i32))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Weights {
    pub foreground: f64,
    pub background: f64,
    pub hole: f64,
}
impl Default for Weights {
    fn default() -> Self {
        Self {
            foreground: 6.,
            background: 1.,
            hole: 4.,
        }
    }
}
#[derive(Debug)]
pub struct Template {
    points: Vec<KeyPoint>,
}
#[derive(Debug)]
pub struct Evaluation {
    pub score: f64,
    pub hole_white_ratio: f64,
    pub included_points: usize,
    pub expected_holes: usize,
}
impl Template {
    pub fn new(points: Vec<KeyPoint>, limits: Limits) -> Result<Self, Error> {
        if points.len() > limits.max_template_points {
            return Err(Error::Budget);
        }
        if points.iter().any(|p| !p.distance.is_finite()) {
            return Err(Error::NonFinite);
        }
        Ok(Self { points })
    }
    pub fn points(&self) -> &[KeyPoint] {
        &self.points
    }
    pub fn evaluate(
        &self,
        table: &DistanceTable,
        x: i32,
        y: i32,
        anchor: Option<AnchorOffset>,
        weights: Weights,
    ) -> Result<Evaluation, Error> {
        if [weights.foreground, weights.background, weights.hole]
            .iter()
            .any(|w| !w.is_finite() || *w < 0.)
        {
            return Err(Error::InvalidWeight);
        }
        let (ox, oy) = match anchor {
            Some(a) => a.rounded()?,
            None => (0, 0),
        };
        let x = x.checked_sub(ox).ok_or(Error::CoordinateOverflow)?;
        let y = y.checked_sub(oy).ok_or(Error::CoordinateOverflow)?;
        let (mut total, mut sum, mut count, mut holes, mut whites) =
            (0., 0., 0usize, 0usize, 0usize);
        for p in &self.points {
            let nx = x.checked_add(p.x).ok_or(Error::CoordinateOverflow)?;
            let ny = y.checked_add(p.y).ok_or(Error::CoordinateOverflow)?;
            if nx < 0 || ny < 0 {
                continue;
            }
            let Some(actual) = table.get(nx as usize, ny as usize) else {
                continue;
            };
            if actual == UNKNOWN {
                continue;
            }
            let weight = if p.distance == 0. {
                weights.foreground
            } else if p.distance > 0. {
                weights.background
            } else {
                weights.hole
            };
            let mismatch = (actual == 0) != (p.distance == 0.);
            total += weight * if mismatch { 1. } else { 0. };
            sum += weight;
            count += 1;
            if p.distance < 0. {
                holes += 1;
                if actual != 0 {
                    whites += 1;
                }
            }
        }
        if !sum.is_finite() || !total.is_finite() {
            return Err(Error::NumericOverflow);
        }
        Ok(Evaluation {
            score: if sum == 0. { f64::MAX } else { total / sum },
            hole_white_ratio: if holes == 0 {
                0.
            } else {
                whites as f64 / holes as f64
            },
            included_points: count,
            expected_holes: holes,
        })
    }
}
