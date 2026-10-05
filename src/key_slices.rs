// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// KeyBuilder.ShapeBuilder through allocateSlices, before retrieveComponents.
// Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::{
    GrayImage,
    curves::Curve,
    key_peaks::Peak,
    key_preparation::{Projection, StaffInput, project_rectangle},
};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Shape {
    Flat,
    Sharp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Range {
    pub browse_start: i64,
    pub browse_stop: i64,
    pub start: Option<i64>,
    pub stop: Option<i64>,
    pub valid: bool,
}
impl Range {
    fn end(&self) -> i64 {
        self.stop.unwrap_or(self.browse_stop)
    }
    fn shrink(&mut self, stop: i64) {
        // A null explicit stop does not implicitly clamp to browse_stop.
        if self.stop.is_none_or(|old| stop <= old) {
            self.stop = Some(stop);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KeyPeak {
    pub min: i64,
    pub main: i64,
    pub max: i64,
    pub height: i32,
    pub area: i32,
}
impl KeyPeak {
    fn center(&self) -> f64 {
        (self.min + self.max) as f64 * 0.5
    }
    fn width(&self) -> i64 {
        self.max - self.min + 1
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Checkpoint {
    pub phase: &'static str,
    pub range: Range,
    pub peaks: Vec<KeyPeak>,
    pub signature: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ShapePreparation {
    pub shape: Shape,
    pub range: Range,
    pub peaks: Vec<KeyPeak>,
    /// Geometric hypothesis only. This is not a recognized key signature.
    pub signature: i32,
    pub starts: Vec<i64>,
    pub slices: Vec<[i64; 4]>,
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Preparation {
    pub staff_id: u64,
    pub stage: &'static str,
    pub shapes: Vec<ShapePreparation>,
    pub key_recognition_complete: bool,
    pub headers_complete: bool,
}

struct Params {
    max_space: i32,
    min_peak: i32,
    max_peak: i32,
    max_width: i64,
    first_peak_offset: i64,
    first_space: i64,
    inner_space: i64,
    inner_gap: i64,
    core_length: usize,
    max_peak_dx: i64,
    flat_heading: i64,
    sharp_light_dx: f64,
    flat_light_dx: f64,
    sharp_delta: f64,
    flat_delta: f64,
    flat_trail: [i64; 3],
    sharp_trail: [i64; 3],
}
impl Params {
    fn new(sheet: usize, staff: usize) -> Result<Self, String> {
        if !(1..=128).contains(&sheet) || !(1..=128).contains(&staff) {
            return Err("unsupported_key_slice_interline".into());
        }
        let large = |r: f64| (r * sheet as f64).round_ties_even() as i64;
        let specific = |r: f64| (r * staff as f64).round_ties_even() as i64;
        Ok(Self {
            max_space: specific(0.4) as i32,
            min_peak: specific(1.25) as i32,
            max_peak: specific(4.25) as i32,
            max_width: specific(0.45),
            first_peak_offset: large(2.3),
            first_space: large(2.1),
            inner_space: large(0.7),
            inner_gap: large(1.35),
            core_length: specific(2.0) as usize,
            max_peak_dx: large(1.5),
            flat_heading: specific(0.2),
            sharp_light_dx: sheet as f64 * 0.25,
            flat_light_dx: sheet as f64 * 0.5,
            sharp_delta: sheet as f64 * 0.7,
            flat_delta: sheet as f64 * 0.45,
            flat_trail: [specific(0.7), specific(1.0), specific(1.3)],
            sharp_trail: [specific(0.2), specific(0.3), specific(0.5)],
        })
    }
}

struct State<'a> {
    image: &'a GrayImage,
    staff: &'a StaffInput,
    projection: &'a Projection,
    params: &'a Params,
    shape: Shape,
    range: Range,
    peaks: Vec<KeyPeak>,
    checkpoints: Vec<Checkpoint>,
}
impl State<'_> {
    fn value(&self, x: i64) -> Result<i32, String> {
        x.checked_sub(self.projection.projection_start)
            .and_then(|x| usize::try_from(x).ok())
            .and_then(|x| self.projection.values.get(x).copied())
            .ok_or("unsupported_key_slice_projection_domain".into())
    }
    fn area(&self, a: i64, b: i64) -> Result<i32, String> {
        let mut area = 0;
        for x in a..=b {
            area += (self.value(x)? - self.params.min_peak).max(0);
        }
        Ok(area)
    }
    fn checkpoint(&mut self, phase: &'static str, signature: Option<i32>) {
        self.checkpoints.push(Checkpoint {
            phase,
            range: self.range.clone(),
            peaks: self.peaks.clone(),
            signature,
        });
    }
    fn check_space(&mut self, start: i64, stop: i64) -> bool {
        if self.range.start.is_none() {
            if stop - start + 1 > self.params.first_space {
                return false;
            }
            self.range.start = Some(stop + 1);
        } else if self.peaks.is_empty() {
            self.range.start = Some(stop + 1);
        } else if stop - start + 1 > self.params.inner_space {
            self.range.shrink(start - 1);
            return false;
        }
        true
    }
    fn stem_like(&self, peak: &KeyPeak) -> Result<bool, String> {
        let margin = i64::from(peak.width() <= 2);
        let left = (peak.min - margin).max(0);
        let right = (peak.max + 1 + margin).min(self.image.width as i64);
        let top = self.projection.browse_rect[1].max(0);
        let bottom = (self.projection.browse_rect[1] + self.projection.browse_rect[3])
            .min(self.image.height as i64);
        if right <= left || bottom - top < self.params.core_length as i64 {
            return Err("unsupported_key_stem_window_height".into());
        }
        let black = (top..bottom)
            .map(|y| {
                (left..right)
                    .any(|x| self.image.pixels[y as usize * self.image.width + x as usize] == 0)
            })
            .collect::<Vec<_>>();
        let n = self.params.core_length;
        let quorum = (n as f64 * 0.75).round_ties_even() as usize;
        let mut count = black[..n].iter().filter(|&&v| v).count();
        if count >= quorum {
            return Ok(true);
        }
        for y in 1..=black.len() - n {
            count -= usize::from(black[y - 1]);
            count += usize::from(black[y + n - 1]);
            if count >= quorum {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn create_peak(&mut self, p: &Peak) -> Result<bool, String> {
        let peak = KeyPeak {
            min: p.min as i64,
            main: p.main as i64,
            max: p.max as i64,
            height: self.value(p.main as i64)?,
            area: self.area(p.min as i64, p.max as i64)?,
        };
        if peak.height > self.params.max_peak || peak.width() > self.params.max_width {
            self.range.shrink(peak.min - 1);
            return Ok(false);
        }
        if !self.stem_like(&peak)? {
            return Ok(true);
        }
        let invalid = if let Some(last) = self.peaks.last() {
            peak.center() - last.center() > self.params.max_peak_dx as f64
        } else {
            peak.min - self.range.browse_start > self.params.first_peak_offset
        };
        if invalid {
            self.range.shrink(peak.min - 1);
            return Ok(false);
        }
        if self.peaks.is_empty() && self.range.start.is_none() {
            self.range.start = Some(self.range.browse_start);
        }
        self.peaks.push(peak);
        Ok(true)
    }
    fn browse(&mut self, hilos: &[Peak]) -> Result<(), String> {
        if hilos.is_empty() {
            return Ok(());
        }
        let mut index = 0;
        let mut current = hilos.first();
        let mut space: Option<(i64, i64)> = None;
        let mut x = self.range.browse_start;
        while x <= self.range.browse_stop {
            if self.value(x)? > self.params.max_space {
                if let Some((a, b)) = space.take()
                    && !self.check_space(a, b)
                {
                    return Ok(());
                }
                if let Some(hl) = current
                    && x >= hl.min as i64
                {
                    if !self.create_peak(hl)? {
                        return Ok(());
                    }
                    x = hl.max as i64;
                    index += 1;
                    current = hilos.get(index).filter(|next| {
                        next.min as i64 - hl.max as i64 - 1 <= self.params.inner_gap
                    });
                }
            } else {
                space = Some((space.map_or(x, |(a, _)| a), x));
            }
            x += 1;
        }
        if let Some((a, b)) = space {
            self.check_space(a, b);
        }
        Ok(())
    }
    fn refine_start(&mut self) -> Result<(), String> {
        let Some(first) = self.peaks.first() else {
            return Ok(());
        };
        let a = self.range.start.ok_or("missing_key_slice_range_start")?;
        let b = first.min - 1;
        if b < a {
            self.range.start = Some(b + 1);
            return Ok(());
        }
        let line_y = |idx: usize| -> Result<i64, String> {
            let y = Curve::interpolate(self.staff.lines[idx].clone())
                .ok_or("invalid_key_slice_curve")?
                .y(a as f64);
            if !y.is_finite() || y.abs() > 4_000_000. {
                return Err("unsupported_key_slice_ordinate".into());
            }
            Ok(y.round_ties_even() as i64)
        };
        let top = line_y(0)? - self.staff.specific_interline as i64;
        let bottom = line_y(4)?;
        let p = project_rectangle(self.image, a, [a, top, b - a + 1, bottom - top + 1])?;
        let mut start = b + 1;
        for x in a..=b {
            let value = p
                .values
                .get((x - p.projection_start) as usize)
                .ok_or("unsupported_key_refine_start_domain")?;
            if *value > self.params.max_space {
                start = x;
                break;
            }
        }
        self.range.start = Some(start);
        Ok(())
    }
    fn merge(&mut self) -> Result<(), String> {
        let mut i = 1;
        while i < self.peaks.len() {
            let a = self.peaks[i - 1].clone();
            let b = self.peaks[i].clone();
            if b.min - a.max <= 1 {
                let mut main = a.min;
                for x in a.min..=b.max {
                    if self.value(x)? > self.value(main)? {
                        main = x;
                    }
                }
                self.peaks[i - 1] = KeyPeak {
                    min: a.min,
                    main,
                    max: b.max,
                    height: a.height.max(b.height),
                    area: self.area(a.min, b.max)?,
                };
                self.peaks.remove(i);
            } else {
                i += 1;
            }
        }
        Ok(())
    }
    fn purge_light(&mut self) {
        if self.peaks.len() <= 1 {
            return;
        }
        let mut sorted = self.peaks.clone();
        sorted.sort_by_key(|p| p.area);
        let total: i32 = sorted.iter().skip(1).map(|p| p.area).sum();
        let quorum = 0.5 * total as f64 / (sorted.len() - 1) as f64;
        let min_dx = match self.shape {
            Shape::Flat => self.params.flat_light_dx,
            Shape::Sharp => self.params.sharp_light_dx,
        };
        let mut i = 0;
        while i < sorted.len() && (sorted[i].area as f64) < quorum {
            let center = sorted[i].center();
            let dx = sorted
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, p)| (p.center() - center).abs())
                .fold(f64::MAX, f64::min);
            if dx < min_dx {
                sorted.remove(i);
            } else {
                i += 1;
            }
        }
        self.peaks.retain(|p| sorted.contains(p));
    }
    fn infer(&mut self) -> Result<i32, String> {
        let mut count = self.peaks.len();
        if count == 0 {
            return Ok(0);
        }
        let first = &self.peaks[0];
        let last = self.peaks[count - 1].clone();
        let heading = first.min - self.range.start.ok_or("missing_key_slice_range_start")?;
        if count > 1 {
            let mean = (last.center() - first.center()) / (count - 1) as f64;
            let mut sum = 0.;
            let mut shorts = 0;
            for w in self.peaks.windows(2) {
                let interval = w[1].center() - w[0].center();
                if interval <= mean {
                    sum += interval;
                    shorts += 1;
                }
            }
            if shorts == 0 {
                return Err("unsupported_key_peak_interval_order".into());
            }
            let short = sum / shorts as f64;
            if short < self.params.flat_delta {
                if self.shape != Shape::Sharp {
                    return Ok(0);
                }
            } else if short > self.params.sharp_delta {
                if self.shape != Shape::Flat {
                    return Ok(0);
                }
            } else if heading > self.params.flat_heading && self.shape != Shape::Sharp {
                return Ok(0);
            }
            if self.shape == Shape::Sharp {
                if !count.is_multiple_of(2) {
                    self.range.shrink(last.min - 1);
                    self.peaks.pop();
                    count -= 1;
                }
                Ok(if count / 2 > 7 { 0 } else { (count / 2) as i32 })
            } else {
                Ok(if count > 7 { 0 } else { -(count as i32) })
            }
        } else if heading <= self.params.flat_heading {
            Ok(if self.shape == Shape::Flat { -1 } else { 0 })
        } else {
            self.range.shrink(last.min - 1);
            self.peaks.clear();
            Ok(0)
        }
    }
    fn refine_signature(&mut self, mut signature: i32) -> Result<i32, String> {
        if signature == 0 {
            return Ok(0);
        }
        let last = self.peaks.last().ok_or("missing_key_slice_peak")?.clone();
        let trail = self.range.end() - last.min + 1;
        if signature < 0 && trail < self.params.flat_trail[0] {
            self.peaks.pop();
            self.range.shrink(last.min - 1);
            signature += 1;
        } else if signature > 0 && trail < self.params.sharp_trail[0] {
            self.peaks.pop();
            let last = self.peaks.pop().ok_or("missing_key_sharp_pair")?;
            self.range.shrink(last.min - 1);
            signature -= 1;
        }
        if signature.abs() > 2 {
            let count = signature.unsigned_abs() as usize;
            let ipc = if signature < 0 { 1 } else { 2 };
            let dxs = (ipc..self.peaks.len())
                .step_by(ipc)
                .map(|i| self.peaks[i].center() - self.peaks[i - 1].center())
                .collect::<Vec<_>>();
            if dxs.len() != count - 1 {
                return Err("unsupported_key_peak_count".into());
            }
            let total: f64 = dxs.iter().sum();
            let max = dxs.iter().copied().fold(f64::from_bits(1), f64::max);
            let weight = (count - 2) as f64;
            let mean = (total - max) / weight;
            let cutoff = (weight * mean + 3.5 * self.params.max_peak_dx as f64) / (weight + 3.5);
            for (i, dx) in dxs.iter().enumerate() {
                if *dx > cutoff {
                    let ip = ipc * (i + 1);
                    self.range.shrink(self.peaks[ip].min - 1);
                    self.peaks.truncate(ip);
                    return Ok(if signature < 0 {
                        -((i + 1) as i32)
                    } else {
                        (i + 1) as i32
                    });
                }
            }
        }
        Ok(signature)
    }
    fn refine_stop(&mut self, typical: i64, max_trail: i64) -> Result<(), String> {
        let last = self.peaks.last().ok_or("missing_key_slice_peak")?;
        let a = last.min + typical - 1;
        let b = (self.projection.projection_start + self.projection.values.len() as i64 - 1)
            .min(last.min + max_trail);
        let mut minimum = i32::MAX;
        let mut stop = None;
        for x in a..=b {
            let count = self.value(x)?;
            if count < minimum {
                minimum = count;
                stop = Some(x - 1);
            }
        }
        if let Some(stop) = stop {
            self.range.shrink(stop);
        }
        Ok(())
    }
    fn starts(&mut self, signature: i32) -> Result<Vec<i64>, String> {
        let mut starts = Vec::new();
        if signature > 0 {
            starts.push(self.range.start.ok_or("missing_key_slice_range_start")?);
            for i in (2..self.peaks.len()).step_by(2) {
                starts
                    .push(((self.peaks[i].min + self.peaks[i - 1].max) as f64 * 0.5).ceil() as i64);
            }
            self.refine_stop(self.params.sharp_trail[1], self.params.sharp_trail[2])?;
        } else if signature < 0 {
            let start = self.range.start.ok_or("missing_key_slice_range_start")?;
            if self.peaks[0].min - start <= self.params.flat_heading {
                starts.push(start);
                starts.extend(self.peaks.iter().skip(1).map(|p| p.min));
                self.refine_stop(self.params.flat_trail[1], self.params.flat_trail[2])?;
            }
        }
        Ok(starts)
    }
}

/// Prepare FLAT then SHARP geometric slice hypotheses without inserting any SIG inter.
pub fn prepare_staff(
    image: &GrayImage,
    sheet_interline: usize,
    staff: &StaffInput,
    projection: &Projection,
    hilos: &[Peak],
) -> Result<Preparation, String> {
    let params = Params::new(sheet_interline, staff.specific_interline)?;
    if staff.lines.len() != 5
        || staff
            .lines
            .iter()
            .any(|p| p.len() > 4096 || Curve::interpolate(p.clone()).is_none())
    {
        return Err("unsupported_key_slice_staff".into());
    }
    if image
        .width
        .checked_mul(image.height)
        .is_none_or(|n| n == 0 || n > crate::MAX_PIXELS || n != image.pixels.len())
        || image.pixels.iter().any(|&v| v != 0 && v != 255)
    {
        return Err("unsupported_key_slice_image".into());
    }
    if projection.staff_id != staff.staff_id
        || projection.values.is_empty()
        || projection.values.len() > 8192
        || projection.values.iter().any(|v| !(0..=32767).contains(v))
        || !(0..=4_000_000).contains(&projection.projection_start)
        || projection.browse_start < projection.projection_start
        || projection.browse_stop >= projection.projection_start + projection.values.len() as i64
        || projection.browse_start > projection.browse_stop
        || projection.projection_start + projection.values.len() as i64 > image.width as i64
        || hilos.len() > 128
        || projection
            .browse_rect
            .iter()
            .any(|v| !(-4_000_000..=4_000_000).contains(v))
        || projection.browse_rect[3] <= 0
        || projection.browse_rect[0] != projection.browse_start
        || projection.browse_rect[2] != projection.browse_stop - projection.browse_start + 1
    {
        return Err("unsupported_key_slice_projection".into());
    }
    if hilos.iter().any(|p| {
        p.min > p.main
            || p.main > p.max
            || p.min < projection.browse_start as usize
            || p.max > projection.browse_stop as usize
    }) || hilos
        .windows(2)
        .any(|p| p[0].main >= p[1].main || p[0].max >= p[1].min)
    {
        return Err("unsupported_key_slice_hilo_order".into());
    }
    let mut shapes = Vec::new();
    // EnumMap follows Shape declaration order, even though Java inserts SHARP first.
    for shape in [Shape::Flat, Shape::Sharp] {
        let mut state = State {
            image,
            staff,
            projection,
            params: &params,
            shape,
            range: Range {
                browse_start: projection.browse_start,
                browse_stop: projection.browse_stop,
                start: None,
                stop: None,
                valid: false,
            },
            peaks: Vec::new(),
            checkpoints: Vec::new(),
        };
        state.browse(hilos)?;
        state.checkpoint("browse_area", None);
        state.refine_start()?;
        state.checkpoint("refine_area_start", None);
        state.merge()?;
        state.checkpoint("merge_peaks", None);
        state.purge_light();
        state.checkpoint("purge_light_peaks", None);
        let signature = state.infer()?;
        state.checkpoint("infer_signature", Some(signature));
        let signature = state.refine_signature(signature)?;
        state.checkpoint("refine_signature", Some(signature));
        let starts = state.starts(signature)?;
        state.checkpoint("compute_starts", Some(signature));
        let mut slices = Vec::new();
        for (i, &start) in starts.iter().enumerate() {
            let stop = starts.get(i + 1).map_or(state.range.end(), |x| x - 1);
            if stop < start {
                return Err("unsupported_negative_key_slice_width".into());
            }
            slices.push([
                start,
                projection.browse_rect[1],
                stop - start + 1,
                projection.browse_rect[3],
            ]);
        }
        shapes.push(ShapePreparation {
            shape,
            range: state.range,
            peaks: state.peaks,
            signature,
            starts,
            slices,
            checkpoints: state.checkpoints,
        });
    }
    Ok(Preparation {
        staff_id: staff.staff_id,
        stage: "HEADERS.key_slice_preparation",
        shapes,
        key_recognition_complete: false,
        headers_complete: false,
    })
}

/// End-to-end native slice preparation, stopping before alteration classification.
pub fn analyze(
    image: &GrayImage,
    grid: &Value,
    registration: &Value,
    sheet_interline: usize,
) -> Result<Value, String> {
    let inputs = crate::key_preparation::staff_inputs(grid, registration, sheet_interline)?;
    let mut staffs = Vec::with_capacity(inputs.len());
    for input in inputs {
        let projection = crate::key_preparation::prepare_staff(image, sheet_interline, &input)?;
        let hilos = crate::key_peaks::retrieve(&projection, input.specific_interline)?;
        staffs.push(prepare_staff(
            image,
            sheet_interline,
            &input,
            &projection,
            &hilos,
        )?);
    }
    Ok(
        json!({"stage":"HEADERS.key_slice_preparation","staffs":staffs,"key_recognition_complete":false,"headers_complete":false,
        "pending":["alteration glyph extraction and classification","hard-slice fallback and clef/key consistency","cross-staff key selection and finalization"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inputs() -> (GrayImage, StaffInput, Projection, Params) {
        let image = GrayImage::new(120, 120, vec![255; 14_400]).unwrap();
        let staff = StaffInput {
            staff_id: 1,
            header_start: 0,
            header_stop: 9,
            clef_stop: Some(9),
            specific_interline: 10,
            lines: (0..5)
                .map(|i| vec![[0., 30. + 10. * i as f64], [119., 30. + 10. * i as f64]])
                .collect(),
            barlines: vec![],
        };
        let projection = Projection {
            staff_id: 1,
            measure_start: 0,
            browse_start: 10,
            browse_stop: 100,
            browse_rect: [10, 10, 91, 71],
            safe_rect: [10, 10, 91, 71],
            projection_start: 0,
            values: vec![0; 101],
            stopping_barline: None,
        };
        (image, staff, projection, Params::new(10, 10).unwrap())
    }
    fn state<'a>(
        image: &'a GrayImage,
        staff: &'a StaffInput,
        projection: &'a Projection,
        params: &'a Params,
    ) -> State<'a> {
        State {
            image,
            staff,
            projection,
            params,
            shape: Shape::Sharp,
            range: Range {
                browse_start: 10,
                browse_stop: 100,
                start: Some(20),
                stop: None,
                valid: false,
            },
            peaks: vec![],
            checkpoints: vec![],
        }
    }
    fn peak(x: i64, area: i32) -> KeyPeak {
        KeyPeak {
            min: x,
            main: x,
            max: x,
            height: 30,
            area,
        }
    }
    #[test]
    fn explicit_stop_preserves_null_semantics() {
        let mut range = Range {
            browse_start: 0,
            browse_stop: 100,
            start: None,
            stop: None,
            valid: false,
        };
        range.shrink(120);
        assert_eq!(range.end(), 120);
        range.shrink(121);
        assert_eq!(range.end(), 120);
        range.shrink(99);
        assert_eq!(range.end(), 99);
    }
    #[test]
    fn stem_row_quorum_is_inclusive_and_uses_grown_narrow_peak() {
        let (mut image, staff, projection, params) = inputs();
        for y in 10..25 {
            image.pixels[y * 120 + 19] = 0;
        }
        assert!(
            state(&image, &staff, &projection, &params)
                .stem_like(&peak(20, 1))
                .unwrap()
        );
        image.pixels[24 * 120 + 19] = 255;
        assert!(
            !state(&image, &staff, &projection, &params)
                .stem_like(&peak(20, 1))
                .unwrap()
        );
    }
    #[test]
    fn light_peak_distance_uses_strict_cutoff() {
        let (image, staff, projection, params) = inputs();
        let mut s = state(&image, &staff, &projection, &params);
        s.peaks = vec![peak(20, 1), peak(22, 10), peak(45, 10)];
        s.purge_light();
        assert_eq!(s.peaks.len(), 2);
        let mut boundary = peak(22, 10);
        boundary.max = 23;
        s.peaks = vec![peak(20, 1), boundary, peak(45, 10)];
        s.purge_light();
        assert_eq!(s.peaks.len(), 3);
    }
    #[test]
    fn odd_sharp_peak_is_removed_and_interval_outlier_truncates_flat_items() {
        let (image, staff, projection, params) = inputs();
        let mut s = state(&image, &staff, &projection, &params);
        s.peaks = vec![peak(20, 10), peak(24, 10), peak(36, 10)];
        assert_eq!(s.infer().unwrap(), 1);
        assert_eq!(s.peaks.len(), 2);
        assert_eq!(s.range.stop, Some(35));
        s.shape = Shape::Flat;
        s.range.stop = None;
        s.peaks = vec![peak(20, 10), peak(30, 10), peak(45, 10)];
        assert_eq!(s.refine_signature(-3).unwrap(), -2);
        assert_eq!(s.peaks.len(), 2);
        assert_eq!(s.range.stop, Some(44));
    }
    #[test]
    fn refine_stop_keeps_first_minimum_and_empty_hilos_preserve_nullable_range() {
        let (image, staff, mut projection, params) = inputs();
        projection.values[29..=33].copy_from_slice(&[4, 3, 2, 2, 5]);
        let mut s = state(&image, &staff, &projection, &params);
        s.peaks.push(peak(20, 10));
        s.refine_stop(10, 13).unwrap();
        assert_eq!(s.range.stop, Some(30));
        let empty = prepare_staff(&image, 10, &staff, &projection, &[]).unwrap();
        assert_eq!(empty.shapes[0].shape, Shape::Flat);
        assert_eq!(empty.shapes[1].shape, Shape::Sharp);
        assert!(
            empty
                .shapes
                .iter()
                .all(|s| s.range.start.is_none() && s.range.stop.is_none() && s.slices.is_empty())
        );
        assert!(!empty.key_recognition_complete);
    }
    #[test]
    fn malformed_or_overlapping_hilos_are_refused() {
        let (image, staff, projection, _) = inputs();
        assert!(
            prepare_staff(
                &image,
                10,
                &staff,
                &projection,
                &[Peak {
                    min: 20,
                    main: 19,
                    max: 21
                }]
            )
            .is_err()
        );
        assert!(
            prepare_staff(
                &image,
                10,
                &staff,
                &projection,
                &[
                    Peak {
                        min: 20,
                        main: 21,
                        max: 24
                    },
                    Peak {
                        min: 24,
                        main: 25,
                        max: 26
                    }
                ]
            )
            .is_err()
        );
        assert!(prepare_staff(&image, 0, &staff, &projection, &[]).is_err());
    }
}
