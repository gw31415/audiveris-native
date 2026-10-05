// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Rust translation of BeamItem/BeamsBuilder.compute{Beam,Hook,}Impacts,
// AreaMask and AbstractBeamInter.Impacts at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Item method boundary only: no builder orchestration, SIG mutation, or oracle lookup.
use crate::{
    GrayImage, MAX_PIXELS,
    beam_structure::{BeamItem, ItemParameters},
};
use audiveris_area::LinearArea;
use serde::{Deserialize, Serialize};

pub const IMPACT_WEIGHTS: [f64; 6] = [0.5, 1.0, 1.0, 2.0, 2.0, 2.0];
pub const INTRINSIC_RATIO: f64 = 0.8;
// Original inherits AbstractInter.getMinGrade => Grades.intrinsicRatio * 0.1.
pub const MIN_BEAM_GRADE: f64 = INTRINSIC_RATIO * 0.1;
pub const MIN_HOOK_GRADE: f64 = MIN_BEAM_GRADE;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub belt_margin_dx: usize,
    pub belt_margin_dy: usize,
    pub min_core_black_ratio: f64,
    pub max_belt_black_ratio: f64,
}
impl Parameters {
    pub fn new(interline: usize) -> Result<Self, String> {
        if interline == 0 || interline > crate::sections::MAX_DIMENSION {
            return Err("invalid_beam_impacts_interline".into());
        }
        Ok(Self {
            belt_margin_dx: (0.25 * interline as f64).round_ties_even() as usize,
            belt_margin_dy: (0.15 * interline as f64).round_ties_even() as usize,
            min_core_black_ratio: 0.7,
            max_belt_black_ratio: 0.4,
        })
    }
    fn validate(self) -> Result<(), String> {
        if self.belt_margin_dx > crate::sections::MAX_DIMENSION
            || self.belt_margin_dy > crate::sections::MAX_DIMENSION
            || !self.min_core_black_ratio.is_finite()
            || !(0.0..1.0).contains(&self.min_core_black_ratio)
            || !self.max_belt_black_ratio.is_finite()
            || !(0.0..=1.0).contains(&self.max_belt_black_ratio)
            || self.max_belt_black_ratio == 0.0
        {
            return Err("invalid_beam_impacts_parameters".into());
        }
        Ok(())
    }
}
/// Native execution bounds, not restrictions attributed to original Audiveris.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Sum of core and normalized belt bounding-rectangle cells sampled per item.
    pub max_mask_cells: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_mask_cells: 8_000_000,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelCounts {
    pub total: usize,
    pub black: usize,
}
impl PixelCounts {
    /// Empty masks produce NaN in Java. The original method keeps impacts,
    /// even if a later zero impact makes the aggregate grade finite zero.
    /// Both outcomes fail the separate acceptance gate.
    pub fn black_ratio(self) -> Option<f64> {
        (self.total != 0).then(|| self.black as f64 / self.total as f64)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Impacts {
    /// wdth, minH, maxH, core, belt, jit; all clamped by GradeUtil.clamp.
    pub values: [f64; 6],
    pub grade: f64,
    /// True for original empty-mask 0/0; JSON numeric nulls then mean NaN,
    /// not an absent method result. Grade acceptance is always false.
    pub non_finite: bool,
}
impl Impacts {
    pub fn new(values: [f64; 6]) -> Result<Self, String> {
        if values.iter().any(|v| !v.is_finite()) {
            return Err("nonfinite_beam_impacts".into());
        }
        Ok(Self::from_original_values(values))
    }
    fn from_original_values(values: [f64; 6]) -> Self {
        let values = values.map(|v| v.clamp(0.0, 1.0));
        let mut product: f64 = 1.0;
        let mut total_weight = 0.0;
        for (v, weight) in values.into_iter().zip(IMPACT_WEIGHTS) {
            total_weight += weight;
            if v == 0.0 {
                product = 0.0;
            } else if weight != 0.0 {
                product *= v.powf(weight);
            }
        }
        let grade = INTRINSIC_RATIO * product.powf(1.0 / total_weight);
        Self {
            non_finite: !grade.is_finite() || values.iter().any(|v| !v.is_finite()),
            values,
            grade,
        }
    }
    pub fn accepted_as_beam(&self) -> bool {
        self.grade >= MIN_BEAM_GRADE
    }
    pub fn accepted_as_hook(&self) -> bool {
        self.grade >= MIN_HOOK_GRADE
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    Width,
    MinHeight,
    MaxHeight,
    CoreRatio,
    BeltRatio,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Evaluation {
    pub width: i32,
    pub core: PixelCounts,
    pub belt: PixelCounts,
    pub rejection: Option<Rejection>,
    /// Some impacts may still fail the separately evaluated grade threshold.
    pub impacts: Option<Impacts>,
}
#[derive(Clone, Debug)]
struct Polygon {
    area: LinearArea,
    bounds: [i32; 4],
}
impl Polygon {
    fn parallelogram(median: [f64; 4], height: f64) -> Result<Self, String> {
        let [x1, y1, x2, y2] = median;
        let h = height / 2.0;
        let points = [[x1, y1 - h], [x2, y2 - h], [x2, y2 + h], [x1, y1 + h]];
        if points
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 16_000_000.0)
        {
            return Err("invalid_beam_item_geometry".into());
        }
        let area = LinearArea::polygon(&points)?;
        let bounds = area.bounds();
        Ok(Self { area, bounds })
    }
    fn cells(&self) -> Result<usize, String> {
        (self.bounds[2] as usize)
            .checked_mul(self.bounds[3] as usize)
            .ok_or_else(|| "beam_mask_budget_exceeded".into())
    }
}
/// Original BeamItem core plus the expanded outer area. Belt containment is
/// expanded area minus core, including the original half-open Area boundaries.
/// Subtraction normalizes/clips the line edges with OpenJDK arithmetic; simple
/// boolean combination of the two containment queries is not equivalent.
#[derive(Clone, Debug)]
pub struct ItemAreas {
    core: Polygon,
    outer: Polygon,
    belt: Polygon,
}
impl ItemAreas {
    pub fn new(
        item: &BeamItem,
        dx: usize,
        top_dy: usize,
        bottom_dy: usize,
    ) -> Result<Self, String> {
        if item
            .median
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 8_000_000.0)
            || item.median[2] <= item.median[0]
            || !item.height.is_finite()
            || item.height <= 0.0
            || item.height > MAX_PIXELS as f64
            || [dx, top_dy, bottom_dy]
                .iter()
                .any(|v| *v > crate::sections::MAX_DIMENSION)
        {
            return Err("invalid_beam_item_geometry".into());
        }
        let core = Polygon::parallelogram(item.median, item.height)?;
        let mut left = intersection_at_x(item.median, item.median[0] - dx as f64);
        let mut right = intersection_at_x(item.median, item.median[2] + dx as f64);
        let shift = (bottom_dy as f64 - top_dy as f64) / 2.0;
        left[1] += shift;
        right[1] += shift;
        let outer = Polygon::parallelogram(
            [left[0], left[1], right[0], right[1]],
            item.height + top_dy as f64 + bottom_dy as f64,
        )?;
        let belt_area = outer.area.subtract(&core.area)?;
        let belt = Polygon {
            bounds: belt_area.bounds(),
            area: belt_area,
        };
        Ok(Self { core, outer, belt })
    }
    pub fn core_bounds(&self) -> [i32; 4] {
        self.core.bounds
    }
    pub fn outer_bounds(&self) -> [i32; 4] {
        self.outer.bounds
    }
    pub fn belt_bounds(&self) -> [i32; 4] {
        self.belt.bounds
    }
    pub fn core_contains(&self, point: [f64; 2]) -> Result<bool, String> {
        self.core.area.contains(point)
    }
    pub fn belt_contains(&self, point: [f64; 2]) -> Result<bool, String> {
        self.belt.area.contains(point)
    }
}
// Preserve LineUtil.intersectionAtX's general line-intersection arithmetic,
// including the synthetic vertical line ending at y=1000. Algebraic slope
// substitution changes one-ULP boundary membership on sloped integer rasters.
#[allow(clippy::eq_op)] // Keep the original eight-coordinate intersection operation order.
fn intersection_at_x([x1, y1, x2, y2]: [f64; 4], x: f64) -> [f64; 2] {
    let den = (x1 - x2) * (0.0 - 1000.0) - (y1 - y2) * (x - x);
    let cross1 = x1 * y2 - y1 * x2;
    let cross2 = x * 1000.0 - 0.0 * x;
    [
        (cross1 * (x - x) - (x1 - x2) * cross2) / den,
        (cross1 * (0.0 - 1000.0) - (y1 - y2) * cross2) / den,
    ]
}
pub struct Evaluator<'a> {
    no_staff: &'a GrayImage,
    params: Parameters,
    limits: Limits,
}
impl<'a> Evaluator<'a> {
    pub fn new(no_staff: &'a GrayImage, interline: usize) -> Result<Self, String> {
        Self::with_parameters(no_staff, Parameters::new(interline)?, Limits::default())
    }
    pub fn with_parameters(
        no_staff: &'a GrayImage,
        params: Parameters,
        limits: Limits,
    ) -> Result<Self, String> {
        params.validate()?;
        if no_staff.width == 0
            || no_staff.height == 0
            || no_staff
                .width
                .checked_mul(no_staff.height)
                .is_none_or(|n| n > MAX_PIXELS || n != no_staff.pixels.len())
        {
            return Err("invalid_beam_no_staff_raster".into());
        }
        if limits.max_mask_cells == 0 {
            return Err("invalid_beam_impacts_limits".into());
        }
        Ok(Self {
            no_staff,
            params,
            limits,
        })
    }
    pub fn compute_beam_impacts(
        &self,
        item: &BeamItem,
        top: bool,
        bottom: bool,
        distance_impact: f64,
        p: &ItemParameters,
    ) -> Result<Evaluation, String> {
        self.compute(item, top, bottom, distance_impact, p, false)
    }
    pub fn compute_hook_impacts(
        &self,
        item: &BeamItem,
        distance_impact: f64,
        p: &ItemParameters,
    ) -> Result<Evaluation, String> {
        self.compute(item, true, true, distance_impact, p, true)
    }
    pub fn sample(&self, areas: &ItemAreas) -> Result<(PixelCounts, PixelCounts), String> {
        let cells = areas
            .core
            .cells()?
            .checked_add(areas.belt.cells()?)
            .ok_or("beam_mask_budget_exceeded")?;
        if cells > self.limits.max_mask_cells {
            return Err("beam_mask_budget_exceeded".into());
        }
        let mut counts = [PixelCounts { total: 0, black: 0 }; 2];
        for (which, b) in [areas.core_bounds(), areas.belt_bounds()]
            .into_iter()
            .enumerate()
        {
            for y in b[1]..b[1] + b[3] {
                for x in b[0]..b[0] + b[2] {
                    let point = [x as f64, y as f64];
                    let inside = if which == 0 {
                        areas.core_contains(point)?
                    } else {
                        areas.belt_contains(point)?
                    };
                    if inside {
                        counts[which].total += 1;
                        if x >= 0
                            && y >= 0
                            && (x as usize) < self.no_staff.width
                            && (y as usize) < self.no_staff.height
                            && self.no_staff.pixels[y as usize * self.no_staff.width + x as usize]
                                == 0
                        {
                            counts[which].black += 1;
                        }
                    }
                }
            }
        }
        Ok((counts[0], counts[1]))
    }
    fn compute(
        &self,
        item: &BeamItem,
        top: bool,
        bottom: bool,
        distance_impact: f64,
        p: &ItemParameters,
        hook: bool,
    ) -> Result<Evaluation, String> {
        let (low, high) = if hook {
            (p.min_hook_width_low, p.min_hook_width_high)
        } else {
            (p.min_beam_width_low, p.min_beam_width_high)
        };
        if [
            low,
            high,
            p.min_height_low,
            p.typical_height,
            p.max_height_high,
            distance_impact,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || low < 0.0
            || high <= low
            || p.min_height_low < 0.0
            || p.typical_height <= p.min_height_low
            || p.max_height_high <= p.typical_height
        {
            return Err("invalid_beam_impacts_item_parameters".into());
        }
        let areas = ItemAreas::new(
            item,
            self.params.belt_margin_dx,
            if top { self.params.belt_margin_dy } else { 0 },
            if bottom {
                self.params.belt_margin_dy
            } else {
                0
            },
        )?;
        let (core, belt) = self.sample(&areas)?;
        let width = (item.median[2] - item.median[0] + 1.0).round_ties_even() as i32;
        let core_ratio = core.black_ratio();
        let belt_ratio = belt.black_ratio();
        let rejection = if (width as f64) < low {
            Some(Rejection::Width)
        } else if item.height < p.min_height_low {
            Some(Rejection::MinHeight)
        } else if item.height > p.max_height_high {
            Some(Rejection::MaxHeight)
        } else if core_ratio.is_some_and(|r| r < self.params.min_core_black_ratio) {
            Some(Rejection::CoreRatio)
        } else if belt_ratio.is_some_and(|r| r > self.params.max_belt_black_ratio) {
            Some(Rejection::BeltRatio)
        } else {
            None
        };
        let impacts = if rejection.is_some() {
            None
        } else {
            Some(Impacts::from_original_values([
                (width as f64 - low) / (high - low),
                (item.height - p.min_height_low) / (p.typical_height - p.min_height_low),
                (p.max_height_high - item.height) / (p.max_height_high - p.typical_height),
                (core_ratio.unwrap_or(f64::NAN) - self.params.min_core_black_ratio)
                    / (1.0 - self.params.min_core_black_ratio),
                1.0 - belt_ratio.unwrap_or(f64::NAN) / self.params.max_belt_black_ratio,
                distance_impact,
            ]))
        };
        Ok(Evaluation {
            width,
            core,
            belt,
            rejection,
            impacts,
        })
    }
}
