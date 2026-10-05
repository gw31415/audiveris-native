// SPDX-License-Identifier: AGPL-3.0-or-later
// StaffProjector computeProjection/computeLineThresholds and serif search
// prefilter. Refined StaffPeak acceptance and rest registration are pending.
use crate::GrayImage;
#[derive(Clone, Debug, PartialEq)]
pub struct Projection {
    pub values: Vec<i32>,
    pub derivative_threshold: i32,
    pub lines_threshold: i32,
    pub blank_threshold: i32,
    pub chunk_threshold: i32,
    pub bar_threshold: i32,
    pub serif_width: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Search {
    pub range: [i32; 2],
    pub threshold: i32,
    pub side_chunks: [i32; 2],
    pub bands: Vec<[i32; 2]>,
}
/// Integer top/bottom line ordinates are supplied for each image column.
/// Original projection includes top and excludes bottom before image clamping.
pub fn project(
    image: &GrayImage,
    rows: &[[i32; 2]],
    abscissas: [i32; 2],
    interline: usize,
    fore: usize,
    line_thickness: &[f64],
) -> Result<Projection, String> {
    if image.width == 0
        || image.height == 0
        || image
            .width
            .checked_mul(image.height)
            .is_none_or(|n| n > crate::MAX_PIXELS || n != image.pixels.len())
        || rows.len() != image.width
        || image.height > 32767
        || interline == 0
        || interline > 10000
        || fore == 0
        || fore > 1000
        || line_thickness.len() < 2
        || line_thickness.len() > 16
        || !line_thickness
            .iter()
            .all(|h| h.is_finite() && *h > 0. && *h <= 1000.)
        || abscissas[0] > abscissas[1]
        || abscissas.iter().any(|&x| i64::from(x).abs() > 10_000_000)
        || rows
            .iter()
            .any(|r| r[0] > r[1] || r.iter().any(|&y| i64::from(y).abs() > 10_000_000))
    {
        return Err("unsupported_serif_projection_geometry".into());
    }
    let margin = (15. * interline as f64).round_ties_even() as i32;
    let l = (abscissas[0] - margin).clamp(0, image.width as i32 - 1) as usize;
    let r = (abscissas[1] + margin).clamp(0, image.width as i32 - 1) as usize;
    if r - l < 5 {
        return Err("unsupported_serif_projection_derivatives".into());
    }
    let mut values = vec![0; image.width];
    let mut derivatives = Vec::with_capacity(r - l);
    for x in l..=r {
        let top = rows[x][0].clamp(0, image.height as i32 - 1) as usize;
        let bottom = (rows[x][1] - 1).clamp(0, image.height as i32 - 1) as usize;
        if top <= bottom {
            for y in top..=bottom {
                values[x] += i32::from(image.pixels[y * image.width + x] == 0);
            }
        }
        if x > l {
            derivatives.push((values[x] - values[x - 1]).abs());
        }
    }
    derivatives.sort();
    let elite = derivatives.iter().rev().take(5).sum::<i32>() as f64 / 5.;
    let n = line_thickness.len();
    let core = line_thickness.iter().sum::<f64>() * (n - 1) as f64 / n as f64;
    Ok(Projection {
        values,
        derivative_threshold: (elite * 0.3).round_ties_even() as i32,
        lines_threshold: core.round_ties_even() as i32,
        blank_threshold: (0.5 * core).floor() as i32,
        chunk_threshold: ((n - 1) * fore) as i32
            + (1.2 * interline as f64).round_ties_even() as i32,
        bar_threshold: (2.5 * interline as f64).round_ties_even() as i32,
        serif_width: (0.25 * interline as f64).round_ties_even() as i32,
    })
}
/// Inclusive threshold bands are candidate ranges, not accepted StaffPeaks.
/// This exposes raster positives without bypassing pending side/core/grade checks.
pub fn search(
    projection: &Projection,
    x: i32,
    left: bool,
    added_chunk: i32,
) -> Result<Search, String> {
    if projection.serif_width <= 0
        || projection.serif_width > 2500
        || projection.bar_threshold < 0
        || projection.bar_threshold > 25000
        || projection.lines_threshold < 0
        || projection.lines_threshold > 20000
        || i64::from(added_chunk).abs() > 20000
        || i64::from(x).abs() > 10_000_000
    {
        return Err("unsupported_serif_search_geometry".into());
    }
    let half = projection.bar_threshold / 2;
    let chunk = (half - 3 * projection.lines_threshold / 4).max(1);
    let adjusted = (chunk - added_chunk).max(1);
    let shift = (projection.serif_width as f64 * 0.5).ceil() as i32 / 2;
    let center = x + if left { shift } else { -shift };
    let range = [
        center - projection.serif_width,
        center + projection.serif_width,
    ];
    if range[0] < 0 || range[1] >= projection.values.len() as i32 {
        return Err("unsupported_serif_search_range".into());
    }
    let mut bands = Vec::new();
    let mut start = None;
    for i in range[0]..=range[1] {
        if projection.values[i as usize] >= half {
            start.get_or_insert(i);
        } else if let Some(a) = start.take() {
            bands.push([a, i - 1]);
        }
    }
    if let Some(a) = start {
        bands.push([a, range[1]]);
    }
    Ok(Search {
        range,
        threshold: half,
        side_chunks: if left {
            [chunk, adjusted]
        } else {
            [adjusted, chunk]
        },
        bands,
    })
}
