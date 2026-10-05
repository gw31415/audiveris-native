// Copyright © Audiveris 2026. All rights reserved.
// Rust adaptation 2026-10-02; SPDX-License-Identifier: AGPL-3.0-or-later
// AdaptiveFilter.Tile + VerticalFilter.MyTile, pinned upstream 9e1e55cd.
use crate::{Context, GrayImage};

/// Actual two-u64-tile storage, not including input/output buffers or allocator overhead.
/// min(width, 38) avoids allocating 38 columns for a one-column image.
pub fn scratch_bytes(width: usize, height: usize) -> usize {
    16 * width.min(38) * height
}

/// Column-major traversal preserves the Java circular integral tile contract.
/// Callback exposes the exact classification context without retaining all contexts.
pub fn binarize(image: &GrayImage, mut probe: impl FnMut(usize, usize, Context)) -> GrayImage {
    let h = image.height;
    let columns = image.width.min(38);
    let mut sums = vec![0_u64; columns * h];
    let mut squares = vec![0_u64; columns * h];
    let mut next_column = 0;
    let mut pixels = vec![255; image.pixels.len()];
    for x in 0..image.width {
        let right = (x + 18).min(image.width - 1);
        while next_column <= right {
            let cx = next_column;
            let col = cx % columns;
            let prev = (cx + columns - 1) % columns;
            let (mut top, mut top_sq, mut top_left, mut top_left_sq) = (0, 0, 0, 0);
            for y in 0..h {
                let left = sums[prev * h + y];
                let left_sq = squares[prev * h + y];
                let v = u64::from(image.pixels[y * image.width + cx]);
                top = v + left + top - top_left;
                top_sq = v * v + left_sq + top_sq - top_left_sq;
                sums[col * h + y] = top;
                squares[col * h + y] = top_sq;
                top_left = left;
                top_left_sq = left_sq;
            }
            next_column += 1;
        }
        let left = x.checked_sub(19);
        for y in 0..h {
            let above = y.checked_sub(19);
            let bottom = (y + 18).min(h - 1);
            let area =
                (right + 1 - left.map_or(0, |v| v + 1)) * (bottom + 1 - above.map_or(0, |v| v + 1));
            let mean_of = |a: &[u64]| {
                let ul = match (left, above) {
                    (Some(l), Some(t)) => a[l % columns * h + t],
                    _ => 0,
                };
                let up = above.map_or(0, |t| a[right % columns * h + t]);
                let le = left.map_or(0, |l| a[l % columns * h + bottom]);
                let br = a[right % columns * h + bottom];
                (ul + br - up - le) as f64 / area as f64
            };
            let mean = mean_of(&sums);
            let std_dev = (mean_of(&squares) - mean * mean).abs().sqrt();
            let context = Context {
                mean,
                std_dev,
                threshold: 0.7 * mean + 0.9 * std_dev,
            };
            if f64::from(image.pixels[y * image.width + x]) <= context.threshold {
                pixels[y * image.width + x] = 0;
            }
            probe(x, y, context);
        }
    }
    GrayImage {
        width: image.width,
        height: image.height,
        pixels,
    }
}
