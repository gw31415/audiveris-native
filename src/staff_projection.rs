// Copyright © Audiveris 2026. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: AGPL-3.0-or-later
// StaffProjector five-line process, with separate sheet and staff interlines.
use crate::{GrayImage, curves::Curve, scale::Scale, sections::Section};
use serde_json::{Value, json};
struct Projector<'a> {
    image: &'a GrayImage,
    first: Curve,
    last: Curve,
    values: Vec<i32>,
    il: i32,
    barline_height: i32,
    bar_threshold: i32,
    gap_threshold: i32,
    derivative: i32,
    lines: i32,
    chunk: i32,
}
impl Projector<'_> {
    fn px(&self, v: f64) -> i32 {
        (self.il as f64 * v).round_ties_even() as i32
    }
    fn clamp(&self, x: i32) -> i32 {
        x.clamp(0, self.image.width as i32 - 1)
    }
    fn der(&self, x: i32) -> i32 {
        if x == 0 {
            0
        } else {
            self.values[x as usize] - self.values[x as usize - 1]
        }
    }
    fn y(&self, curve: &Curve, x: i32) -> i32 {
        curve.y(x as f64).round_ties_even() as i32
    }
    fn get_chunk(&self, x: i32, dir: i32) -> i32 {
        let x1 = x + dir;
        let x2 = x1 + dir * (self.px(0.15) - 1);
        if x2 < 0 || x2 >= self.image.width as i32 {
            return 0;
        }
        let mut result = i32::MAX;
        let mut x = x1;
        while dir * (x2 - x) >= 0 {
            result = result.min(self.values[x as usize]);
            x += dir;
        }
        result
    }
    fn refine(&self, start: i32, stop: i32, dir: i32) -> Option<(i32, f64, f64)> {
        let mid = (start + stop) as f64 / 2.;
        let x1 = if dir > 0 { mid.ceil() } else { mid.floor() } as i32;
        let x2 = self.clamp(if dir > 0 {
            stop + self.px(0.25)
        } else {
            start - self.px(0.25)
        });
        let mut best = 0;
        let mut bx = None;
        let mut x = x1;
        while dir * (x2 - x) >= 0 {
            let d = self.der(x);
            if dir * (best - d) > 0 {
                best = d;
                bx = Some(x);
            }
            x += dir;
        }
        let best = best.abs();
        let minbar = self.bar_threshold;
        if let Some(bx) = bx
            && best >= self.derivative
        {
            let x = bx - i32::from(dir > 0);
            let chunk = self.get_chunk(x, dir);
            let ci = if chunk < self.lines {
                1.
            } else if chunk > self.chunk {
                0.
            } else {
                (self.chunk - chunk) as f64 / (self.chunk - self.lines) as f64
            };
            Some((x, best as f64 / (minbar - self.derivative) as f64, ci))
        } else {
            let border = if dir > 0 {
                self.image.width as i32 - 1
            } else {
                0
            };
            let v = self.values[border as usize];
            if x2 == border && v >= self.derivative {
                Some((border, v as f64 / (minbar - self.derivative) as f64, 1.))
            } else {
                None
            }
        }
    }
    fn peak(&self, raw_start: i32, raw_stop: i32) -> Option<([i32; 4], f64)> {
        let (start, sd, sc) = self.refine(raw_start, raw_stop, -1)?;
        let (stop, ed, ec) = self.refine(raw_start, raw_stop, 1)?;
        if stop - start + 1 > self.px(1.5) {
            return None;
        }
        let value = (start..=stop)
            .map(|x| self.values[x as usize])
            .max()
            .unwrap_or(0);
        let xm = (start + stop) / 2;
        let top = self.y(&self.first, xm);
        let bottom = self.y(&self.last, xm);
        let dx = i32::from(stop - start < 2);
        // AreaUtil.verticalCore: leading/trailing empty rows do not count as interior gaps.
        let mut gap = 0;
        let mut last_black = None;
        let mut last_white = None;
        for y in top..=bottom {
            let empty = (start - dx..=stop + dx).all(|x| {
                x < 0
                    || y < 0
                    || x >= self.image.width as i32
                    || y >= self.image.height as i32
                    || self.image.pixels[y as usize * self.image.width + x as usize] != 0
            });
            if empty {
                last_white = Some(y);
                continue;
            }
            if let (Some(w), Some(b)) = (last_white, last_black) {
                gap = gap.max(w - b);
                last_white = None;
            }
            last_black = Some(y);
        }
        if gap > self.gap_threshold {
            return None;
        }
        let min = self.bar_threshold;
        let core = (value - min) as f64 / (self.barline_height - min) as f64;
        let gi = 1. - gap as f64 / self.gap_threshold as f64;
        let impacts = [core, gi, sd, ed, sc, ec];
        let mut product = 1.;
        for (i, v) in impacts.into_iter().enumerate() {
            product *= v.clamp(0., 1.).powf(if i < 4 { 1. } else { 0.5 });
        }
        let grade = 0.8 * product.powf(1. / 5.);
        if grade >= 0.8 * 0.1 {
            Some(([start, stop, top, bottom], grade))
        } else {
            None
        }
    }
    fn browse(&self, start: i32, stop: i32) -> Vec<([i32; 4], f64)> {
        let mut result = Vec::new();
        let mut current = Some(start);
        let mut x = start;
        while x <= stop {
            let der = self.der(x);
            if der >= self.derivative {
                let mut max = der;
                for xx in x + 1..=stop {
                    let d = self.der(xx);
                    if d > max {
                        max = d;
                        x = xx;
                    } else {
                        break;
                    }
                }
                current = Some(x);
            } else if der <= -self.derivative {
                let mut min = der;
                for xx in x + 1..=self.clamp(stop + 1) {
                    let d = self.der(xx);
                    if d <= min {
                        min = d;
                        x = xx;
                    } else {
                        break;
                    }
                }
                if x == stop {
                    x = stop + 1;
                }
                if let Some(s) = current
                    && s < x
                {
                    if let Some(p) = self.peak(s, x - 1) {
                        result.push(p);
                    }
                    current = None;
                }
            }
            x += 1;
        }
        if let Some(s) = current
            && let Some(p) = self.peak(s, stop)
        {
            result.push(p);
        }
        result
    }
    fn peaks(&self, start: i32, stop: i32) -> Vec<([i32; 4], f64)> {
        let mut result = Vec::new();
        let mut first = None;
        let mut last = 0;
        let mut x = start;
        while x <= stop {
            if self.values[x as usize] >= self.bar_threshold {
                if first.is_none() {
                    first = Some(x);
                }
                last = x;
            } else if let Some(s) = first {
                for p in self.browse(s, last) {
                    x = x.max(p.0[1]);
                    result.push(p);
                }
                first = None;
            }
            x += 1;
        }
        if let Some(s) = first
            && let Some(p) = self.peak(s, last)
        {
            result.push(p);
        }
        result
    }
}
fn curve(line: &Value) -> Curve {
    Curve::interpolate(
        line["points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
            .collect(),
    )
    .unwrap()
}
pub fn analyze(image: &GrayImage, sc: &Scale, sections: &[Section], staffs: &Value) -> Value {
    if staffs["status"] != "initial_staffs" {
        return json!({"status":"blocked","error":"initial_staffs_unavailable"});
    }
    let mut out = Vec::new();
    for staff in staffs["staffs"].as_array().unwrap() {
        let lines = staff["lines"].as_array().unwrap();
        let mut thickness = Vec::new();
        for line in lines {
            let ids: Vec<_> = line["section_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u32)
                .collect();
            let mut selected: Vec<_> = sections.iter().filter(|s| ids.contains(&s.id.0)).collect();
            selected.sort_by_key(|s| {
                let b = s.bounds();
                (b.x, b.y, s.id.0)
            });
            let first = selected[0].bounds().x as i64;
            let mut last = first - 1;
            let mut holes = 0;
            let mut weight = 0;
            for s in selected {
                let b = s.bounds();
                let start = b.x as i64;
                if last < start {
                    holes += start - last;
                }
                last = last.max((b.x + b.width - 1) as i64);
                weight += s.weight;
            }
            // First section initializes getTrueLength; the initial gap above is not a hole.
            holes -= 1;
            let length = last - first + 1 - holes;
            thickness.push(weight as f64 / length as f64);
        }
        let cumul = thickness.iter().sum::<f64>() * 0.8;
        let specific = staff["interline"]
            .as_u64()
            .unwrap_or(sc.interline.main as u64) as usize;
        if !(1..=128).contains(&specific)
            || !(specific == sc.interline.main
                || sc
                    .small_interline
                    .as_ref()
                    .is_some_and(|s| s.main == specific))
            || (specific != sc.interline.main && staff["small"] != true)
        {
            return json!({"status":"unsupported","error":"unsupported_staff_projection_specific_interline"});
        }
        let mut p = Projector {
            image,
            first: curve(&lines[0]),
            last: curve(lines.last().unwrap()),
            values: vec![0; image.width],
            il: sc.interline.main as i32,
            barline_height: (4 * specific) as i32,
            bar_threshold: (specific as f64 * 2.5).round_ties_even() as i32,
            gap_threshold: (specific as f64 * 0.6).round_ties_even() as i32,
            derivative: 0,
            lines: cumul.round_ties_even() as i32,
            chunk: (4 * sc.line.main) as i32 + (specific as f64 * 1.2).round_ties_even() as i32,
        };
        let left = staff["left"].as_f64().unwrap().round_ties_even() as i32;
        let right = staff["right"].as_f64().unwrap().round_ties_even() as i32;
        let xmin = p.clamp(left - p.px(15.));
        let xmax = p.clamp(right + p.px(15.));
        let mut ders = Vec::new();
        for x in xmin..=xmax {
            let top = p.y(&p.first, x).clamp(0, image.height as i32 - 1);
            let bottom = (p.y(&p.last, x) - 1).clamp(0, image.height as i32 - 1);
            p.values[x as usize] = (top..=bottom)
                .filter(|&y| image.pixels[y as usize * image.width + x as usize] == 0)
                .count() as i32;
            if x > xmin {
                ders.push(p.der(x).abs());
            }
        }
        if ders.len() < 5 {
            return json!({"status":"failed","error":"projection_derivative_window_too_short"});
        }
        ders.sort();
        p.derivative =
            (ders.iter().rev().take(5).sum::<i32>() as f64 / 5. * 0.3).round_ties_even() as i32;
        let blank = (cumul * 0.5).floor() as i32;
        let mut blanks = Vec::new();
        let mut first = None;
        for x in 0..image.width {
            if p.values[x] <= blank {
                if first.is_none() {
                    first = Some(x);
                }
            } else if let Some(s) = first {
                blanks.push([s as i32, x as i32 - 1]);
                first = None;
            }
        }
        if let Some(s) = first {
            blanks.push([s as i32, image.width as i32 - 1]);
        }
        let mut ending = serde_json::Map::new();
        let mut a = 0;
        let mut b = image.width as i32 - 1;
        for (dir, start, key) in [(-1, left, "LEFT"), (1, right, "RIGHT")] {
            let mut bs = blanks.clone();
            if dir < 0 {
                bs.reverse();
            }
            if let Some(v) = bs
                .iter()
                .find(|v| dir * ((v[0] + v[1]) / 2 - start) > 0 && v[1] - v[0] + 1 >= p.px(2.))
            {
                ending.insert(key.into(), json!(v));
                if dir < 0 {
                    a = v[1];
                } else {
                    b = v[0];
                }
            }
        }
        let peaks = p.peaks(a, b);
        out.push(json!({"staff_id":staff["id"],"left":left,"right":right,"line_thicknesses":thickness,"projection":p.values,"derivative_threshold":p.derivative,"blank_threshold":blank,"lines_threshold":p.lines,"chunk_threshold":p.chunk,"blanks":blanks,"ending_blanks":ending,"peaks":peaks.iter().map(|p|p.0).collect::<Vec<_>>(),"peak_grades":peaks.iter().map(|p|p.1).collect::<Vec<_>>()}));
    }
    json!({"status":"bar_candidates","grid_complete":false,"staff_projections":out})
}
