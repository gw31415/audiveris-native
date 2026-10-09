// SPDX-License-Identifier: AGPL-3.0-or-later
//! Raw BINARY checks and native gap-point extraction from retained pixels.
//! A result remains a candidate: actual obstacle owners and Head links are not checked.
use super::fit::Pixel;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawSupportError {
    InvalidImageDimensions,
    NonBinaryPixel { index: usize, value: u8 },
    InvalidInterline,
    EmptyPoints,
    EndpointOutsideImage { point: Pixel },
    EndpointNotInCandidate { point: Pixel },
    MissingRun { point_index: usize, point: Pixel },
    CoordinateOverflow,
}
impl std::fmt::Display for RawSupportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidImageDimensions => f.write_str("InvalidImageDimensions"),
            Self::NonBinaryPixel { index, value } => {
                write!(f, "NonBinaryPixel {{ index: {index}, value: {value} }}")
            }
            Self::InvalidInterline => f.write_str("InvalidInterline"),
            Self::EmptyPoints => f.write_str("EmptyPoints"),
            Self::EndpointOutsideImage { point } => {
                write!(f, "EndpointOutsideImage {{ point: {point:?} }}")
            }
            Self::EndpointNotInCandidate { point } => {
                write!(f, "EndpointNotInCandidate {{ point: {point:?} }}")
            }
            Self::MissingRun { point_index, point } => {
                write!(
                    f,
                    "MissingRun {{ point_index: {point_index}, point: {point:?} }}"
                )
            }
            Self::CoordinateOverflow => f.write_str("CoordinateOverflow"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BinaryImage<'a> {
    pub width: usize,
    pub height: usize,
    pixels: &'a [u8],
}
impl<'a> BinaryImage<'a> {
    pub fn new(width: usize, height: usize, pixels: &'a [u8]) -> Result<Self, RawSupportError> {
        if width == 0
            || height == 0
            || width > i32::MAX as usize
            || height > i32::MAX as usize
            || width.checked_mul(height) != Some(pixels.len())
        {
            return Err(RawSupportError::InvalidImageDimensions);
        }
        if let Some((index, &value)) = pixels
            .iter()
            .enumerate()
            .find(|(_, v)| **v != 0 && **v != 255)
        {
            return Err(RawSupportError::NonBinaryPixel { index, value });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    fn contains(self, p: Pixel) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height
    }
    pub fn is_black(self, p: Pixel) -> bool {
        self.contains(p) && self.pixels[p.y as usize * self.width + p.x as usize] == 0
    }
}

#[derive(Clone, Debug)]
pub struct GapEvidence {
    #[cfg(test)]
    pub vector: Vec<u8>,
    #[cfg(test)]
    pub largest_hole: usize,
    #[cfg(test)]
    pub maximum_hole: usize,
    pub accepted_by_gap_only: bool,
    pub raw_black_points: Vec<Pixel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawVerticalRun {
    pub x: i32,
    pub start: i32,
    pub length: usize,
}
#[derive(Clone, Debug)]
pub struct SupportEvidence {
    pub glyph_bounds: [i32; 4],
    #[cfg(test)]
    pub required_run_count: usize,
    #[cfg(test)]
    pub accepted_run_count: usize,
    pub accepted_by_run_support_only: bool,
    #[cfg(test)]
    pub rows: Vec<(usize, Pixel, RawVerticalRun, bool, bool, bool)>,
    pub accepted_runs: Vec<RawVerticalRun>,
}
#[derive(Clone, Debug)]
pub struct PendingCandidate {
    #[cfg(test)]
    pub original_points: Vec<Pixel>,
    #[cfg(test)]
    pub added_actual_foreground_points: Vec<Pixel>,
    pub points: Vec<Pixel>,
    pub gap: GapEvidence,
    pub support: SupportEvidence,
}

fn validate_interline(interline: f64) -> Result<(), RawSupportError> {
    if !interline.is_finite() || interline <= 0.0 || interline > i32::MAX as f64 {
        Err(RawSupportError::InvalidInterline)
    } else {
        Ok(())
    }
}
fn coord(value: i64) -> Result<i32, RawSupportError> {
    i32::try_from(value).map_err(|_| RawSupportError::CoordinateOverflow)
}
fn pixel(x: i64, y: i64) -> Result<Pixel, RawSupportError> {
    Ok(Pixel {
        x: coord(x)?,
        y: coord(y)?,
    })
}

// Java Area line crossings include diagonal boundary vertices asymmetrically.
// Inclusive polygon edges or a rounded y-interval select different raw pixels.
pub fn area_contains(poly: &[Pixel; 4], point: Pixel) -> bool {
    let x = point.x as f64;
    let y = point.y as f64;
    let mut crossings = 0_i32;
    for index in 0..4 {
        let p0 = poly[index];
        let p1 = poly[(index + 1) % 4];
        let x0 = p0.x as f64;
        let y0 = p0.y as f64;
        let x1 = p1.x as f64;
        let y1 = p1.y as f64;
        if (y < y0 && y < y1) || (y >= y0 && y >= y1) || (x >= x0 && x >= x1) {
            continue;
        }
        if x < x0 && x < x1 {
            crossings += if y0 < y1 { 1 } else { -1 };
            continue;
        }
        let intercept = x0 + (y - y0) * (x1 - x0) / (y1 - y0);
        if x < intercept {
            crossings += if y0 < y1 { 1 } else { -1 };
        }
    }
    crossings != 0
}

pub fn gap_evidence(
    binary: &BinaryImage<'_>,
    a: Pixel,
    b: Pixel,
    interline: f64,
) -> Result<GapEvidence, RawSupportError> {
    validate_interline(interline)?;
    for point in [a, b] {
        if !binary.contains(point) {
            return Err(RawSupportError::EndpointOutsideImage { point });
        }
    }
    let (x1, y1, x2, y2) = (a.x as i64, a.y as i64, b.x as i64, b.y as i64);
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let horizontal = dx >= dy;
    let maximum_hole = (0.25 * interline).round_ties_even() as usize;
    if dx <= 1 && dy <= 1 {
        return Ok(GapEvidence {
            #[cfg(test)]
            vector: Vec::new(),
            #[cfg(test)]
            largest_hole: 0,
            #[cfg(test)]
            maximum_hole,
            accepted_by_gap_only: true,
            raw_black_points: Vec::new(),
        });
    }
    let poly = if horizontal {
        if x2 > x1 {
            [
                pixel(x1 + 1, y1 - 1)?,
                pixel(x2, y2 - 1)?,
                pixel(x2, y2 + 1)?,
                pixel(x1 + 1, y1 + 1)?,
            ]
        } else {
            [
                pixel(x1, y1 - 1)?,
                pixel(x2 + 1, y2 - 1)?,
                pixel(x2 + 1, y2 + 1)?,
                pixel(x1, y1 + 1)?,
            ]
        }
    } else if y2 > y1 {
        [
            pixel(x1 - 1, y1 + 1)?,
            pixel(x2 - 1, y2)?,
            pixel(x2 + 1, y2)?,
            pixel(x1 + 1, y1 + 1)?,
        ]
    } else {
        [
            pixel(x1 - 1, y1)?,
            pixel(x2 - 1, y2 + 1)?,
            pixel(x2 + 1, y2 + 1)?,
            pixel(x1 + 1, y1)?,
        ]
    };
    let xmin = poly.iter().map(|p| p.x).min().unwrap();
    let xmax = poly.iter().map(|p| p.x).max().unwrap();
    let ymin = poly.iter().map(|p| p.y).min().unwrap();
    let ymax = poly.iter().map(|p| p.y).max().unwrap();
    let mut vector = vec![0; (if horizontal { dx } else { dy } - 1) as usize];
    let mut raw_black_points = Vec::new();
    for x in xmin..xmax {
        for y in ymin..ymax {
            let p = Pixel { x, y };
            if area_contains(&poly, p) && x > 0 && y > 0 && binary.is_black(p) {
                let index = if horizontal { x - xmin } else { y - ymin } as usize;
                vector[index] = 1;
                raw_black_points.push(p);
            }
        }
    }
    let mut largest_hole = 0;
    let mut hole = 0;
    for &value in &vector {
        if value == 0 {
            hole += 1;
            largest_hole = largest_hole.max(hole);
        } else {
            hole = 0;
        }
    }
    Ok(GapEvidence {
        #[cfg(test)]
        vector,
        #[cfg(test)]
        largest_hole,
        #[cfg(test)]
        maximum_hole,
        accepted_by_gap_only: largest_hole <= maximum_hole,
        raw_black_points,
    })
}

fn close_to_curve(point: Pixel, points: &[Pixel], index: usize, maximum_squared: f64) -> bool {
    let close = |p: &Pixel| {
        let dx = p.x as f64 - point.x as f64;
        let dy = p.y as f64 - point.y as f64;
        dx * dx + dy * dy <= maximum_squared
    };
    points[index..].iter().any(close) || points[..index].iter().rev().any(close)
}

pub fn retrieve_support(
    binary: &BinaryImage<'_>,
    points: &[Pixel],
    interline: f64,
) -> Result<SupportEvidence, RawSupportError> {
    validate_interline(interline)?;
    if points.is_empty() {
        return Err(RawSupportError::EmptyPoints);
    }
    let xmin = points.iter().map(|p| p.x).min().unwrap();
    let xmax = points.iter().map(|p| p.x).max().unwrap();
    let ymin = points.iter().map(|p| p.y).min().unwrap();
    let ymax = points.iter().map(|p| p.y).max().unwrap();
    let width = coord(xmax as i64 - xmin as i64 + 1)?;
    let maximum_run_distance = 0.25 * interline;
    let maximum_squared = maximum_run_distance * maximum_run_distance;
    let margin = maximum_run_distance.ceil() as i64;
    let glyph_top = (ymin as i64 - margin).max(0);
    let glyph_height = ymax as i64 + 1 + margin - glyph_top;
    let required_run_count = (0.75 * width as f64).round_ties_even() as usize;
    #[cfg(test)]
    let mut rows = Vec::with_capacity(points.len());
    let mut accepted = BTreeSet::new();
    for (point_index, &point) in points.iter().enumerate() {
        if !binary.is_black(point) {
            return Err(RawSupportError::MissingRun { point_index, point });
        }
        let mut start = point.y;
        let mut stop = point.y;
        while start > 0
            && binary.is_black(Pixel {
                x: point.x,
                y: start - 1,
            })
        {
            start -= 1;
        }
        while stop < binary.height as i32 - 1
            && binary.is_black(Pixel {
                x: point.x,
                y: stop + 1,
            })
        {
            stop += 1;
        }
        let run = RawVerticalRun {
            x: point.x,
            start,
            length: (stop - start + 1) as usize,
        };
        let start_close = close_to_curve(
            Pixel {
                x: point.x,
                y: start,
            },
            points,
            point_index,
            maximum_squared,
        );
        let stop_close = run.length <= 1
            || close_to_curve(
                Pixel {
                    x: point.x,
                    y: stop,
                },
                points,
                point_index,
                maximum_squared,
            );
        let accepted_run = start_close && stop_close;
        if accepted_run {
            accepted.insert((point.x, start, stop));
        }
        #[cfg(test)]
        rows.push((
            point_index,
            point,
            run,
            start_close,
            stop_close,
            accepted_run,
        ));
    }
    let accepted_runs = accepted
        .into_iter()
        .map(|(x, start, stop)| RawVerticalRun {
            x,
            start,
            length: (stop - start + 1) as usize,
        })
        .collect::<Vec<_>>();
    Ok(SupportEvidence {
        glyph_bounds: [xmin, coord(glyph_top)?, width, coord(glyph_height)?],
        #[cfg(test)]
        required_run_count,
        #[cfg(test)]
        accepted_run_count: accepted_runs.len(),
        accepted_by_run_support_only: accepted_runs.len() >= required_run_count,
        #[cfg(test)]
        rows,
        accepted_runs,
    })
}

/// Native extraction experiment, not original Skeleton/getAllPoints behavior.
/// Every supplied point is retained, including duplicates; the returned sequence is x/y sorted.
pub fn refine_gap_foreground(
    binary: &BinaryImage<'_>,
    points: &[Pixel],
    a: Pixel,
    b: Pixel,
    interline: f64,
) -> Result<PendingCandidate, RawSupportError> {
    for point in [a, b] {
        if !points.contains(&point) {
            return Err(RawSupportError::EndpointNotInCandidate { point });
        }
    }
    let gap = gap_evidence(binary, a, b, interline)?;
    let existing_columns = points.iter().map(|p| p.x).collect::<BTreeSet<_>>();
    let mut columns: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    for p in &gap.raw_black_points {
        if p.x > a.x.min(b.x) && p.x < a.x.max(b.x) && !existing_columns.contains(&p.x) {
            columns.entry(p.x).or_default().push(p.y);
        }
    }
    let mut added_actual_foreground_points = Vec::new();
    for (x, ys) in columns {
        let sum = ys.iter().map(|&y| y as i64).sum::<i64>();
        let target = (sum as f64 / ys.len() as f64).round_ties_even() as i64;
        let y = *ys
            .iter()
            .min_by_key(|&&y| ((y as i64 - target).abs(), y))
            .unwrap();
        added_actual_foreground_points.push(Pixel { x, y });
    }
    let mut refined = points.to_vec();
    refined.extend(added_actual_foreground_points.iter().copied());
    refined.sort_by_key(|p| (p.x, p.y));
    let support = retrieve_support(binary, &refined, interline)?;
    Ok(PendingCandidate {
        #[cfg(test)]
        original_points: points.to_vec(),
        #[cfg(test)]
        added_actual_foreground_points,
        points: refined,
        gap,
        support,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(x: i32, y: i32) -> Pixel {
        Pixel { x, y }
    }
    #[test]
    fn area_boundaries_keep_left_top_and_exclude_right_bottom() {
        let p = [point(2, 2), point(6, 2), point(6, 4), point(2, 4)];
        assert!(area_contains(&p, point(2, 2)));
        assert!(area_contains(&p, point(5, 3)));
        assert!(!area_contains(&p, point(6, 3)));
        assert!(!area_contains(&p, point(3, 4)));
    }
    #[test]
    fn java_observed_diagonal_boundaries_keep_the_lower_left_vertex() {
        // Translated real pair corridors; an interval approximation chose different raw pixels.
        let equal_slope = [point(2, 2), point(6, 7), point(6, 9), point(2, 4)];
        assert!(!area_contains(&equal_slope, point(2, 2)));
        assert!(area_contains(&equal_slope, point(2, 3)));
        assert!(area_contains(&equal_slope, point(2, 4)));
        let short_diagonal = [point(2, 2), point(3, 3), point(3, 5), point(2, 4)];
        assert!(!area_contains(&short_diagonal, point(2, 2)));
        assert!(area_contains(&short_diagonal, point(2, 3)));
        assert!(area_contains(&short_diagonal, point(2, 4)));
        assert!(!area_contains(&short_diagonal, point(3, 4)));
    }
    #[test]
    fn source_gap_cutoff_accepts_three_empty_columns_and_rejects_four() {
        let pixels = vec![255; 100];
        let binary = BinaryImage::new(10, 10, &pixels).unwrap();
        let within = gap_evidence(&binary, point(2, 3), point(6, 3), 11.0).unwrap();
        let outside = gap_evidence(&binary, point(2, 3), point(7, 3), 11.0).unwrap();
        assert_eq!((within.largest_hole, within.maximum_hole), (3, 3));
        assert!(within.accepted_by_gap_only);
        assert_eq!((outside.largest_hole, outside.maximum_hole), (4, 3));
        assert!(!outside.accepted_by_gap_only);
    }
    #[test]
    fn source_run_distance_and_three_quarters_width_cutoffs_are_inclusive() {
        let mut pixels = vec![255; 100];
        pixels[3 * 10 + 3] = 0;
        pixels[4 * 10 + 3] = 0;
        let binary = BinaryImage::new(10, 10, &pixels).unwrap();
        assert!(
            retrieve_support(&binary, &[point(3, 3)], 4.0)
                .unwrap()
                .accepted_by_run_support_only
        );
        assert!(
            !retrieve_support(&binary, &[point(3, 3)], 3.999)
                .unwrap()
                .accepted_by_run_support_only
        );
        let points = [point(2, 3), point(3, 3), point(4, 3), point(5, 3)];
        for p in points {
            pixels[p.y as usize * 10 + p.x as usize] = 0;
        }
        for y in 0..7 {
            pixels[y * 10 + 5] = 0;
        }
        let binary = BinaryImage::new(10, 10, &pixels).unwrap();
        let threshold = retrieve_support(&binary, &points, 4.0).unwrap();
        assert_eq!(
            (threshold.accepted_run_count, threshold.required_run_count),
            (3, 3)
        );
        assert!(threshold.accepted_by_run_support_only);
        for y in 0..7 {
            pixels[y * 10 + 4] = 0;
        }
        let binary = BinaryImage::new(10, 10, &pixels).unwrap();
        let short = retrieve_support(&binary, &points, 4.0).unwrap();
        assert_eq!((short.accepted_run_count, short.required_run_count), (2, 3));
        assert!(!short.accepted_by_run_support_only);
    }
    #[test]
    fn duplicate_point_visits_do_not_inflate_run_support_and_white_is_an_error() {
        let mut pixels = vec![255; 100];
        pixels[4 * 10 + 3] = 0;
        pixels[5 * 10 + 3] = 0;
        let binary = BinaryImage::new(10, 10, &pixels).unwrap();
        let support =
            retrieve_support(&binary, &[point(3, 4), point(3, 5), point(3, 4)], 11.0).unwrap();
        assert_eq!(support.rows.len(), 3);
        assert_eq!(support.accepted_run_count, 1);
        assert_eq!(support.required_run_count, 1);
        assert_eq!(
            retrieve_support(&binary, &[point(4, 4)], 11.0).unwrap_err(),
            RawSupportError::MissingRun {
                point_index: 0,
                point: point(4, 4)
            }
        );
    }
    #[test]
    fn refinement_chooses_actual_ink_and_keeps_existing_points() {
        let mut pixels = vec![255; 100];
        for p in [point(2, 4), point(3, 3), point(3, 4), point(5, 4)] {
            pixels[p.y as usize * 10 + p.x as usize] = 0;
        }
        let binary = BinaryImage::new(10, 10, &pixels).unwrap();
        let original = [point(2, 4), point(5, 4)];
        let refined =
            refine_gap_foreground(&binary, &original, original[0], original[1], 11.0).unwrap();
        assert_eq!(refined.original_points, original);
        assert_eq!(refined.added_actual_foreground_points, [point(3, 4)]);
        assert_eq!(refined.gap.vector, [1, 0]);
        assert_eq!(refined.gap.largest_hole, 1);
        assert!(!refined.points.iter().any(|p| p.x == 4));
        assert!(refined.points.iter().all(|&p| binary.is_black(p)));
    }
}
