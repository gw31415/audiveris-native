//! Sheet scale (Audiveris ScaleBuilder): staff line thickness and interline from
//! vertical run histograms, beam thickness from a secondary black peak, stem
//! width from horizontal runs.

use crate::binary::Bitmap;

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Scale {
    /// Most frequent staff line thickness.
    pub line: i32,
    /// Maximum acceptable line thickness.
    pub max_line: i32,
    /// Distance between two staff lines (center to center).
    pub interline: i32,
    /// Beam thickness.
    pub beam: i32,
    /// Typical stem width.
    pub stem: i32,
    /// Maximum stem width.
    pub max_stem: i32,
}

impl Scale {
    pub fn frac(&self, f: f64) -> f64 {
        f * self.interline as f64
    }
    pub fn px(&self, f: f64) -> i32 {
        (f * self.interline as f64).round() as i32
    }
}

fn peak(hist: &[u64], lo: usize, hi: usize) -> Option<usize> {
    (lo..hi.min(hist.len()))
        .max_by_key(|&i| hist[i])
        .filter(|&i| hist[i] > 0)
}

/// Extent of a peak: contiguous range around `p` where counts stay >= ratio * hist[p].
fn peak_range(hist: &[u64], p: usize, ratio: f64) -> (usize, usize) {
    let lim = (hist[p] as f64 * ratio) as u64;
    let mut a = p;
    while a > 1 && hist[a - 1] >= lim {
        a -= 1;
    }
    let mut b = p;
    while b + 1 < hist.len() && hist[b + 1] >= lim {
        b += 1;
    }
    (a, b)
}

pub fn compute(img: &Bitmap) -> Result<Scale, String> {
    let (w, h) = (img.width, img.height);
    let max = h.max(2) + 1;
    let mut black = vec![0u64; max];
    let mut combo = vec![0u64; 2 * max];
    for x in 0..w {
        // Sequence of (is_black, length) runs along this column.
        let mut runs: Vec<(bool, usize)> = Vec::new();
        let mut y = 0;
        while y < h {
            let b = img.data[y * w + x];
            let s = y;
            while y < h && img.data[y * w + x] == b {
                y += 1;
            }
            runs.push((b, y - s));
        }
        for (i, &(b, len)) in runs.iter().enumerate() {
            if b {
                black[len] += 1;
            } else if i > 0 && i + 1 < runs.len() {
                // B1 + W and W + B2 combos
                combo[runs[i - 1].1 + len] += 1;
                combo[len + runs[i + 1].1] += 1;
            }
        }
    }
    let line = peak(&black, 1, max).ok_or("no black runs: empty image?")?;
    let (_, line_hi) = peak_range(&black, line, 0.1);
    let interline = peak(&combo, 3 * line + 2, 2 * max).ok_or("no regularly spaced lines")?;
    // Beam thickness: vertical runs of long horizontal structures (beams),
    // within [line, interline].
    let hr = img.horizontal_run_lengths();
    let mut beam_hist = vec![0u64; interline + 1];
    for x in 0..w {
        let mut y = 0;
        while y < h {
            if img.data[y * w + x] {
                let s = y;
                while y < h && img.data[y * w + x] {
                    y += 1;
                }
                let len = y - s;
                let mid = (s + y) / 2;
                if len > line_hi && len <= interline && hr[mid * w + x] as usize >= 2 * interline {
                    beam_hist[len] += 1;
                }
            } else {
                y += 1;
            }
        }
    }
    let lo = ((0.4 * interline as f64).round() as usize).max(line_hi.max(line + 1) + 2);
    let hi = (0.75 * interline as f64).round() as usize;
    let beam = peak(&beam_hist, lo, hi + 1)
        .filter(|&p| beam_hist[p] as f64 >= 5.0 * interline as f64)
        .unwrap_or((0.5 * interline as f64).round() as usize);
    // Stem width: horizontal runs between 1 and interline/2.
    let mut horiz = vec![0u64; w + 1];
    for y in 0..h {
        let mut x = 0;
        while x < w {
            if img.data[y * w + x] {
                let s = x;
                while x < w && img.data[y * w + x] {
                    x += 1;
                }
                horiz[x - s] += 1;
            } else {
                x += 1;
            }
        }
    }
    let stem = peak(&horiz, 1, (interline / 2).max(2)).unwrap_or(line.max(2));
    let (_, stem_hi) = peak_range(&horiz, stem, 0.25);
    if interline < 6 {
        return Err(format!(
            "interline {interline} too small; image resolution too low"
        ));
    }
    Ok(Scale {
        line: line as i32,
        max_line: (line_hi as i32).max(line as i32 + 1),
        interline: interline as i32,
        beam: beam as i32,
        stem: stem as i32,
        max_stem: (stem_hi as i32).max(stem as i32 + 1),
    })
}
