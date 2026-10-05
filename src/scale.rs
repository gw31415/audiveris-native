// Copyright © Audiveris 2026. All rights reserved.
// Rust adaptation 2026-10-02; SPDX-License-Identifier: AGPL-3.0-or-later
// ScaleBuilder.HistoKeeper/computeBeamKeys, HiLoPeakFinder and IntegerFunction.
// Pinned upstream 9e1e55cd2746037d059345881c53e6a6754bffbd. No user overrides.
use crate::{GrayImage, position_runs};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub min: usize,
    pub main: usize,
    pub max: usize,
}
#[derive(Clone, Copy)]
struct DerPeak {
    min: usize,
    max: usize,
    finished: bool,
}

/// Same first-maximum/stable-tie/terminal-hilo behavior as Java (no trailing flush).
pub fn find_peaks(f: &[i32], min_derivative: i32) -> Vec<Range> {
    let mut hilos = Vec::new();
    let (mut hi, mut lo): (Option<DerPeak>, Option<DerPeak>) = (None, None);
    let make_hilo = |hi: DerPeak, lo: DerPeak| {
        let mut main = hi.min;
        for x in hi.min..=lo.max {
            if f[x] > f[main] {
                main = x;
            }
        }
        Range {
            min: hi.min,
            main,
            max: lo.max,
        }
    };
    for x in 1..f.len() {
        let der = f[x] - f[x - 1];
        if der >= min_derivative {
            if let Some(l) = lo.take() {
                hilos.push(make_hilo(hi.take().unwrap(), l));
            }
            if hi.is_none_or(|h| h.finished) {
                hi = Some(DerPeak {
                    min: x,
                    max: x,
                    finished: false,
                });
            } else {
                hi.as_mut().unwrap().max = x;
            }
        } else if der <= -min_derivative {
            if let Some(l) = lo.as_mut() {
                l.max = x;
            } else if hi.is_some() {
                lo = Some(DerPeak {
                    min: x,
                    max: x,
                    finished: false,
                });
            }
        } else if let Some(l) = lo.take() {
            hilos.push(make_hilo(hi.take().unwrap(), l));
        } else if let Some(h) = hi.as_mut() {
            if f[x] < 1 {
                hi = None;
            } else {
                h.finished = true;
            }
        }
    }
    let mut order: Vec<_> = (0..hilos.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(f[hilos[i].main]));
    let mut by_hilo = vec![None::<Range>; hilos.len()];
    let mut peaks = Vec::new();
    for i in order {
        let hilo = hilos[i];
        let mut pmin = hilo.min.saturating_sub(1).max(1);
        if i > 0 {
            pmin = pmin.max(by_hilo[i - 1].map_or(hilos[i - 1].max, |p| p.max) + 1);
        }
        let (mut lower, mut upper, mut total) = (hilo.main, hilo.main, f[hilo.main]);
        loop {
            let before = if lower == pmin { 0 } else { f[lower - 1] };
            let after = if upper == hilo.max { 0 } else { f[upper + 1] };
            let gain = before.max(after);
            if (gain as f64 / (total + gain) as f64) < 0.03 || total + gain == 0 {
                break;
            }
            if before > after {
                lower -= 1;
            } else {
                upper += 1;
            }
            total += gain;
        }
        let peak = Range {
            min: lower,
            main: hilo.main,
            max: upper,
        };
        by_hilo[i] = Some(peak);
        peaks.push(peak);
    }
    peaks.sort_by_key(|p| std::cmp::Reverse(f[p.main]));
    peaks
}

#[derive(Debug)]
pub struct Histograms {
    pub black: Vec<i32>,
    pub combo: Vec<i32>,
    pub max_black: usize,
    pub max_white: usize,
}
pub fn histograms(binary: &GrayImage) -> Histograms {
    let max_black = (binary.height as f64 * 0.08).round_ties_even() as usize;
    let max_white = (binary.height as f64 * 0.25).round_ties_even() as usize;
    let mut black = vec![0; max_black + 1];
    let mut combo = vec![0; max_black + max_white + 1];
    for x in 0..binary.width {
        let (mut y_last, mut last_black) = (0, 0);
        for run in position_runs(binary, true, x) {
            if run.length <= max_black {
                black[run.length] += 1;
            }
            if run.start > y_last {
                let white = run.start - y_last;
                if white <= max_white && last_black != 0 {
                    // Upstream does not test black<=maxBlack here; addValue ignores out-of-domain sums.
                    for n in [last_black + white, white + run.length] {
                        if let Some(v) = combo.get_mut(n) {
                            *v += 1;
                        }
                    }
                }
            }
            last_black = run.length;
            y_last = run.start + run.length;
        }
    }
    Histograms {
        black,
        combo,
        max_black,
        max_white,
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Beam {
    pub thickness: usize,
    pub extrapolated: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Scale {
    pub line: Range,
    pub interline: Range,
    pub small_interline: Option<Range>,
    pub beam: Beam,
    pub small_beam: Option<Beam>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Failure {
    NoRegularLines,
    TooFewBlackPixels,
    NoSignificantBlackLines,
    InterlineTooLow,
    InterlineTooHigh,
}
impl Failure {
    pub fn code(self) -> &'static str {
        match self {
            Self::NoRegularLines => "no_regular_lines",
            Self::TooFewBlackPixels => "too_few_black_pixels",
            Self::NoSignificantBlackLines => "no_significant_black_lines",
            Self::InterlineTooLow => "interline_too_low",
            Self::InterlineTooHigh => "interline_too_high",
        }
    }
}
#[derive(Debug)]
pub struct Analysis {
    pub histograms: Histograms,
    pub combo_peaks: Vec<Range>,
    pub black_peaks: Vec<Range>,
    pub result: Result<Scale, Failure>,
}
fn local_maxima(f: &[i32], min: i64, max: i64) -> Vec<usize> {
    let mut result = Vec::new();
    let mut prev = None;
    let mut growing = false;
    for x in min.max(0)..=max.min(f.len() as i64 - 1) {
        let x = x as usize;
        if let Some(p) = prev {
            if f[x] >= f[p] {
                growing = true;
            } else {
                if growing {
                    result.push(p);
                }
                growing = false;
            }
        }
        prev = Some(x);
    }
    result.sort_by_key(|&x| std::cmp::Reverse(f[x]));
    result
}
fn select(
    hist: &Histograms,
    width: usize,
    height: usize,
    combos: &mut Vec<Range>,
    blacks: &mut Vec<Range>,
) -> Result<Scale, Failure> {
    let mut combo = *combos.first().ok_or(Failure::NoRegularLines)?;
    let mut others = Vec::new();
    for &p in &combos[1..] {
        let ratio = combo.main.max(p.main) as f64 / combo.main.min(p.main) as f64;
        if ratio > 1.9 {
            continue;
        }
        if ratio < 1.2 {
            combo = Range {
                min: combo.min.min(p.min),
                main: (combo.main + p.main) / 2,
                max: combo.max.max(p.max),
            };
        } else {
            others.push(p);
        }
    }
    let combo2 = others.first().copied();
    // Java subList removals mutate the original finder list; keep its original first range.
    combos.truncate(1);
    combos.extend_from_slice(&others);
    let black_count: usize = hist
        .black
        .iter()
        .enumerate()
        .map(|(i, &n)| i * n as usize)
        .sum();
    if black_count as f64 / ((width * height) as f64) < 0.001 {
        return Err(Failure::TooFewBlackPixels);
    }
    *blacks = find_peaks(
        &hist.black,
        (hist.black.iter().sum::<i32>() as f64 * 0.025).round_ties_even() as i32,
    );
    if blacks.is_empty() {
        return Err(Failure::NoSignificantBlackLines);
    }
    let min_count = (hist.black[blacks[0].main] as f64 * 0.4).round_ties_even() as i32;
    if let Some(i) = (1..blacks.len()).find(|&i| hist.black[blacks[i].main] < min_count) {
        blacks.truncate(i);
    }
    let mut beam_key = None;
    let line = if blacks.len() > 1 {
        blacks.sort_by_key(|p| p.main);
        let (p0, p1) = (blacks[0], blacks[1]);
        if p1.main <= (combo.main as f64 * 0.5).round_ties_even() as usize {
            Range {
                min: p0.min,
                main: p1.main,
                max: p1.max,
            }
        } else {
            beam_key = Some(p1.main);
            p0
        }
    } else {
        blacks[0]
    };
    if combo.main < 11 {
        return Err(Failure::InterlineTooLow);
    }
    if combo.main > 100 {
        return Err(Failure::InterlineTooHigh);
    }
    let larger = combo2.map_or(combo.main, |p| p.main.max(combo.main));
    // Java uses signed ints: implausible dominant black runs can make this range inverted.
    let min_height = (line.max as i64).max((0.45 * larger as f64).round_ties_even() as i64);
    let max_height = (larger as i64 - (line.min / 2) as i64)
        .min((0.95 * larger as f64).round_ties_even() as i64);
    let mut second_beam = None;
    let guess = min_height + (0.25 * (max_height - min_height) as f64).round_ties_even() as i64;
    if beam_key.is_none() {
        let quorum = (hist.black.iter().sum::<i32>() as f64 * 0.01).round_ties_even() as i32;
        let peaks = local_maxima(&hist.black, min_height, max_height);
        if let Some(&peak) = peaks.first() {
            if (hist.black[peak] as f64 / quorum as f64) >= 1.0 || (peak as i64 - guess).abs() <= 2
            {
                beam_key = Some(peak);
            }
            if combo2.is_some()
                && peaks.len() > 1
                && hist.black[peaks[1]] as f64 / quorum as f64 >= 1.0
            {
                second_beam = Some(peaks[1]);
            }
        }
    }
    let beam = Beam {
        thickness: beam_key.unwrap_or(guess as usize),
        extrapolated: beam_key.is_none(),
    };
    let (beam, small_beam) = if let Some(second) = second_beam {
        (
            Beam {
                thickness: beam.thickness.max(second),
                extrapolated: false,
            },
            Some(Beam {
                thickness: beam.thickness.min(second),
                extrapolated: false,
            }),
        )
    } else {
        (beam, None)
    };
    let (interline, small_interline) = match combo2 {
        None => (combo, None),
        Some(p) if p.main < combo.main => (combo, Some(p)),
        Some(p) => (p, Some(combo)),
    };
    Ok(Scale {
        line,
        interline,
        small_interline,
        beam,
        small_beam,
    })
}
pub fn analyze(binary: &GrayImage) -> Analysis {
    let histograms = histograms(binary);
    let mut combo_peaks = find_peaks(
        &histograms.combo,
        (histograms.combo.iter().sum::<i32>() as f64 * 0.025).round_ties_even() as i32,
    );
    let mut black_peaks = Vec::new();
    let result = select(
        &histograms,
        binary.width,
        binary.height,
        &mut combo_peaks,
        &mut black_peaks,
    );
    Analysis {
        histograms,
        combo_peaks,
        black_peaks,
        result,
    }
}

impl Analysis {
    pub fn write_json(&self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        fn range(out: &mut impl std::io::Write, r: Option<Range>) -> std::io::Result<()> {
            match r {
                Some(r) => write!(out, "[{}, {}, {}]", r.min, r.main, r.max),
                None => write!(out, "null"),
            }
        }
        fn beam(out: &mut impl std::io::Write, b: Option<Beam>) -> std::io::Result<()> {
            match b {
                Some(b) => write!(
                    out,
                    "{{\"thickness\":{},\"extrapolated\":{}}}",
                    b.thickness, b.extrapolated
                ),
                None => write!(out, "null"),
            }
        }
        write!(
            out,
            "{{\"status\":\"{}\",\"error\":",
            if self.result.is_ok() { "ok" } else { "failed" }
        )?;
        match self.result {
            Ok(_) => write!(out, "null")?,
            Err(e) => write!(out, "\"{}\"", e.code())?,
        }
        write!(
            out,
            ",\"max_black\":{},\"max_white\":{}",
            self.histograms.max_black, self.histograms.max_white
        )?;
        for (name, data) in [
            ("black_histogram", &self.histograms.black),
            ("combo_histogram", &self.histograms.combo),
        ] {
            write!(out, ",\"{name}\":[")?;
            for (i, v) in data.iter().enumerate() {
                if i > 0 {
                    write!(out, ",")?;
                }
                write!(out, "{v}")?;
            }
            write!(out, "]")?;
        }
        for (name, peaks) in [
            ("combo_peaks", &self.combo_peaks),
            ("black_peaks", &self.black_peaks),
        ] {
            write!(out, ",\"{name}\":[")?;
            for (i, p) in peaks.iter().enumerate() {
                if i > 0 {
                    write!(out, ",")?;
                }
                range(out, Some(*p))?;
            }
            write!(out, "]")?;
        }
        write!(out, ",\"selected\":")?;
        if let Ok(s) = &self.result {
            write!(out, "{{\"line\":")?;
            range(out, Some(s.line))?;
            write!(out, ",\"interline\":")?;
            range(out, Some(s.interline))?;
            write!(out, ",\"small_interline\":")?;
            range(out, s.small_interline)?;
            write!(out, ",\"beam\":")?;
            beam(out, Some(s.beam))?;
            write!(out, ",\"small_beam\":")?;
            beam(out, s.small_beam)?;
            write!(out, "}}")?;
        } else {
            write!(out, "null")?;
        }
        write!(out, "}}")
    }
}
