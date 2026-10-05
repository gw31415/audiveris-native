// SPDX-License-Identifier: AGPL-3.0-or-later
// Pinned StaffProjector HALF-mode processMultiRestSide peak acceptance.
use crate::{
    GrayImage,
    serif_raster::{self, Projection, Search},
};
#[derive(Clone, Copy, Debug)]
pub struct StaffGeometry<'a> {
    pub rows: &'a [[i32; 2]],
    pub middle: &'a [i32],
    pub abscissas: [i32; 2],
    pub interline: usize,
    pub fore: usize,
    pub line_thickness: &'a [f64],
}
#[derive(Clone, Debug, PartialEq)]
pub struct Peak {
    pub start: i32,
    pub stop: i32,
    pub top: i32,
    pub bottom: i32,
    pub impacts: [f64; 6],
    pub grade: f64,
}
pub struct Detector<'a> {
    image: &'a GrayImage,
    geometry: StaffGeometry<'a>,
    projection: Projection,
    work: usize,
}
impl<'a> Detector<'a> {
    pub fn new(image: &'a GrayImage, geometry: StaffGeometry<'a>) -> Result<Self, String> {
        let projection = serif_raster::project(
            image,
            geometry.rows,
            geometry.abscissas,
            geometry.interline,
            geometry.fore,
            geometry.line_thickness,
        )?;
        if geometry.middle.len() != image.width
            || geometry
                .rows
                .iter()
                .zip(geometry.middle)
                .any(|(r, &m)| r[0] < 0 || r[1] >= image.height as i32 || m < r[0] || m > r[1])
        {
            return Err("unsupported_serif_peak_staff_geometry".into());
        }
        Ok(Self {
            image,
            geometry,
            projection,
            work: 0,
        })
    }
    fn px(&self, v: f64) -> i32 {
        (v * self.geometry.interline as f64).round_ties_even() as i32
    }
    fn clamp(&self, x: i32) -> i32 {
        x.clamp(0, self.image.width as i32 - 1)
    }
    fn value(&self, x: i32) -> i32 {
        self.projection.values[x as usize]
    }
    fn der(&self, x: i32) -> i32 {
        if x == 0 {
            0
        } else {
            self.value(x) - self.value(x - 1)
        }
    }
    fn get_chunk(&self, x: i32, dir: i32) -> i32 {
        let a = x + dir;
        let b = a + dir * (self.px(0.15) - 1);
        if b < 0 || b >= self.image.width as i32 {
            return 0;
        }
        let mut result = i32::MAX;
        let mut x = a;
        while dir * (b - x) >= 0 {
            result = result.min(self.value(x));
            x += dir;
        }
        result
    }
    fn refine(
        &self,
        start: i32,
        stop: i32,
        dir: i32,
        min_derivative: i32,
        added_chunk: i32,
    ) -> Option<(i32, f64, f64)> {
        let mid = (start + stop) as f64 / 2.;
        let mut x = if dir > 0 { mid.ceil() } else { mid.floor() } as i32;
        let limit = self.clamp(if dir > 0 {
            stop + self.px(0.25)
        } else {
            start - self.px(0.25)
        });
        let (mut best, mut bx) = (0, None);
        while dir * (limit - x) >= 0 {
            let d = self.der(x);
            if dir * (best - d) > 0 {
                best = d;
                bx = Some(x);
            }
            x += dir;
        }
        let best = best.abs();
        let half = self.projection.bar_threshold / 2;
        if let Some(bx) = bx
            && best >= min_derivative
        {
            let x = bx - i32::from(dir > 0);
            let chunk = self.get_chunk(x, dir);
            let low = added_chunk + self.projection.lines_threshold;
            let high = added_chunk + self.projection.chunk_threshold;
            let chunk_grade = if chunk < low {
                1.
            } else if chunk > high {
                0.
            } else {
                (high - chunk) as f64 / (high - low) as f64
            };
            Some((x, best as f64 / (half - min_derivative) as f64, chunk_grade))
        } else {
            let border = if dir > 0 {
                self.image.width as i32 - 1
            } else {
                0
            };
            let v = self.value(border);
            if limit == border && v >= min_derivative {
                Some((border, v as f64 / (half - min_derivative) as f64, 1.))
            } else {
                None
            }
        }
    }
    fn take(&mut self, n: usize) -> Result<(), String> {
        self.work = self
            .work
            .checked_add(n)
            .filter(|n| *n <= 1_000_000_000)
            .ok_or("unsupported_serif_peak_work")?;
        Ok(())
    }
    fn core(&mut self, left: i32, right: i32, top: i32, bottom: i32) -> Result<(i32, f64), String> {
        if left < 0
            || right >= self.image.width as i32
            || left > right
            || top < 0
            || bottom >= self.image.height as i32
            || top > bottom
        {
            return Err("unsupported_serif_peak_core_geometry".into());
        }
        self.take((right - left + 1) as usize * (bottom - top + 1) as usize)?;
        let (mut gap, mut last_black, mut last_white, mut white) = (0, None, None, 0);
        for y in top..=bottom {
            let empty = (left..=right)
                .all(|x| self.image.pixels[y as usize * self.image.width + x as usize] != 0);
            if empty {
                white += 1;
                last_white = Some(y);
            } else {
                if let (Some(w), Some(b)) = (last_white, last_black) {
                    gap = gap.max(w - b);
                    last_white = None;
                }
                last_black = Some(y);
            }
        }
        Ok((gap, white as f64 / (bottom - top + 1) as f64))
    }
    fn peak(
        &mut self,
        start: i32,
        stop: i32,
        search: &Search,
        added_chunk: i32,
    ) -> Result<Option<Peak>, String> {
        let raw_start = start;
        let Some((start, sd, sc)) =
            self.refine(start, stop, -1, search.side_chunks[0], added_chunk)
        else {
            return Ok(None);
        };
        let Some((stop, ed, ec)) =
            self.refine(raw_start, stop, 1, search.side_chunks[1], added_chunk)
        else {
            return Ok(None);
        };
        self.finish_peak(start, stop, [(sd, sc), (ed, ec)], added_chunk)
    }
    fn finish_peak(
        &mut self,
        start: i32,
        stop: i32,
        side_impacts: [(f64, f64); 2],
        added_chunk: i32,
    ) -> Result<Option<Peak>, String> {
        let [(sd, sc), (ed, ec)] = side_impacts;
        if stop < start || stop - start + 1 > self.px(1.5) {
            return Ok(None);
        }
        let value = (start..=stop).map(|x| self.value(x)).max().unwrap();
        let mid = (start + stop) / 2;
        let [mut top, mut bottom] = self.geometry.rows[mid as usize];
        let dx = i32::from(stop - start < 2);
        let beyond = if added_chunk != 0 {
            Some(self.core(start - dx, stop + dx, top, bottom)?)
        } else {
            None
        };
        if added_chunk != 0 {
            let middle = self.geometry.middle[mid as usize];
            top += (middle - top) / 2;
            bottom -= (bottom - middle) / 2;
        }
        let (gap, _) = self.core(start - dx, stop + dx, top, bottom)?;
        let gap_threshold = self.px(0.6);
        if gap > gap_threshold || beyond.is_some_and(|(_, white)| white < 0.3) {
            return Ok(None);
        }
        let half = self.projection.bar_threshold / 2;
        let staff_height =
            self.geometry.interline as i32 * (self.geometry.line_thickness.len() as i32 - 1);
        let impacts = [
            (value - half) as f64 / (staff_height / 2 - half) as f64,
            1. - gap as f64 / gap_threshold as f64,
            sd,
            ed,
            sc,
            ec,
        ];
        let mut product = 1.;
        for (i, v) in impacts.iter().enumerate() {
            product *= v.clamp(0., 1.).powf(if i < 4 { 1. } else { 0.5 });
        }
        let grade = 0.8 * product.powf(1. / 5.);
        if grade >= 0.08 {
            Ok(Some(Peak {
                start,
                stop,
                top,
                bottom,
                impacts,
                grade,
            }))
        } else {
            Ok(None)
        }
    }
    fn browse(
        &mut self,
        start: i32,
        stop: i32,
        search: &Search,
        added_chunk: i32,
    ) -> Result<Vec<Peak>, String> {
        let mut peaks = Vec::new();
        let mut first = Some(start);
        let mut x = start;
        while x <= stop {
            let d = self.der(x);
            if d >= search.side_chunks[0] {
                let mut best = d;
                let mut n = x + 1;
                while n <= stop {
                    let d = self.der(n);
                    if d <= best {
                        break;
                    }
                    best = d;
                    x = n;
                    n += 1;
                }
                first = Some(x);
            } else if d <= -search.side_chunks[1] {
                let mut best = d;
                let mut n = x + 1;
                while n <= self.clamp(stop + 1) {
                    let d = self.der(n);
                    if d > best {
                        break;
                    }
                    best = d;
                    x = n;
                    n += 1;
                }
                if x == stop {
                    x = stop + 1;
                }
                if let Some(a) = first
                    && a < x
                {
                    if let Some(p) = self.peak(a, x - 1, search, added_chunk)? {
                        peaks.push(p);
                    }
                    first = None;
                }
            }
            x += 1;
        }
        if let Some(a) = first
            && let Some(p) = self.peak(a, stop, search, added_chunk)?
        {
            peaks.push(p);
        }
        Ok(peaks)
    }
    pub fn detect(&mut self, x: i32, left: bool, added_chunk: i32) -> Result<Vec<Peak>, String> {
        let search = serif_raster::search(&self.projection, x, left, added_chunk)?;
        self.take(
            (search.range[1] - search.range[0] + 1) as usize
                * (2 * self.px(0.25) + self.px(1.5) + 4) as usize,
        )?;
        let mut result = Vec::new();
        let mut first = None;
        let mut last = 0;
        let mut x = search.range[0];
        while x <= search.range[1] {
            if self.value(x) >= search.threshold {
                first.get_or_insert(x);
                last = x;
            } else if let Some(a) = first.take() {
                for p in self.browse(a, last, &search, added_chunk)? {
                    x = x.max(p.stop);
                    result.push(p);
                }
            }
            x += 1;
        }
        if let Some(a) = first
            && let Some(p) = self.peak(a, last, &search, added_chunk)?
        {
            result.push(p);
        }
        Ok(result)
    }
}
