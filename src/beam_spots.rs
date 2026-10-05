// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Native SpotsBuilder / Picture filtering / MorphoProcessor pixel boundary.
// Pinned upstream: 9e1e55cd2746037d059345881c53e6a6754bffbd.
// Translation checked against the official 5.11.0 JAR's local bytecode.
// No JVM, subprocess, expected fixture, or network access is used at runtime.
use crate::{GrayImage, MAX_PIXELS, position_runs};
use audiveris_staff_removal::{BinaryPage, GlyphPixels, Limits, Orientation};
use serde::Serialize;

pub const BEAM_THRESHOLD: u8 = 140;
pub const HEAD_THRESHOLD: u8 = 170;
/// Additional native computational bounds, not claimed upstream restrictions.
pub const MAX_MORPH_RADIUS: f32 = 64.0;
pub const MAX_MORPH_PROBES: usize = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct HeaderRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Parameters {
    pub max_stem: usize,
    pub effective_beam: f64,
}

/// Pixel/glyph preparation only. Head sizing, glyph indexing and dispatch,
/// BeamStructure, BeamInter creation and MultipleRestsBuilder remain downstream.
#[derive(Debug)]
pub struct PreparedSpots {
    pub parameters: Parameters,
    pub filtered_horizontal: GrayImage,
    pub median: GrayImage,
    pub gaussian: GrayImage,
    pub header_erased: GrayImage,
    pub closed: GrayImage,
    pub binarized: GrayImage,
    pub head_spots: GrayImage,
    /// Original GlyphFactory vertical scan order, before BlackHeadSizer tagging.
    pub glyphs: Vec<GlyphPixels>,
}

fn validate(image: &GrayImage) -> Result<(), String> {
    if image.width == 0
        || image.height == 0
        || image
            .width
            .checked_mul(image.height)
            .is_none_or(|n| n > MAX_PIXELS || n != image.pixels.len())
    {
        return Err("invalid_beam_spot_image".into());
    }
    Ok(())
}
fn copy(image: &GrayImage) -> GrayImage {
    GrayImage {
        width: image.width,
        height: image.height,
        pixels: image.pixels.clone(),
    }
}
fn blank(image: &GrayImage, value: u8) -> GrayImage {
    GrayImage {
        width: image.width,
        height: image.height,
        pixels: vec![value; image.pixels.len()],
    }
}

/// RunTableFactory.LengthFilter is inclusive: retain a horizontal foreground run
/// iff length >= maxStem. NO_STAFF must already be binary.
pub fn filter_horizontal(image: &GrayImage, max_stem: usize) -> Result<GrayImage, String> {
    validate(image)?;
    if max_stem == 0 || max_stem > MAX_PIXELS {
        return Err("invalid_beam_spot_max_stem".into());
    }
    if image.pixels.iter().any(|&v| v != 0 && v != 255) {
        return Err("invalid_beam_spot_binary_input".into());
    }
    let mut out = blank(image, 255);
    for y in 0..image.height {
        for run in position_runs(image, false, y) {
            if run.length >= max_stem {
                out.pixels[y * image.width + run.start..y * image.width + run.start + run.length]
                    .fill(0);
            }
        }
    }
    Ok(out)
}

/// Picture's default MedianGrayFilter radius is 1. The original shrinks the
/// square radius uniformly near *any* edge, so boundary pixels are unchanged.
pub fn median_filter(image: &GrayImage) -> Result<GrayImage, String> {
    validate(image)?;
    let mut out = copy(image);
    if image.width < 3 || image.height < 3 {
        return Ok(out);
    }
    for y in 1..image.height - 1 {
        for x in 1..image.width - 1 {
            let mut values = [0u8; 9];
            for dy in 0..3 {
                values[dy * 3..dy * 3 + 3].copy_from_slice(
                    &image.pixels
                        [(y + dy - 1) * image.width + x - 1..(y + dy - 1) * image.width + x + 2],
                );
            }
            values.sort_unstable();
            out.pixels[y * image.width + x] = values[4];
        }
    }
    Ok(out)
}

/// GaussianGrayFilter.makeKernel(1): sigma remains 1 and each intermediate
/// kernel/accumulator operation is Java float. Each pass rounds to bytes before
/// the next pass; this is not a single floating-point two-dimensional blur.
pub fn gaussian_kernel() -> [f32; 3] {
    let side = (-0.5_f64).exp() as f32;
    let total = (side + 1.0_f32) + side;
    [side / total, 1.0_f32 / total, side / total]
}
pub fn gaussian_filter(image: &GrayImage) -> Result<GrayImage, String> {
    validate(image)?;
    let kernel = gaussian_kernel();
    fn transpose(src: &[u8], dst: &mut [u8], width: usize, height: usize, kernel: [f32; 3]) {
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0_f32;
                for (k, weight) in kernel.iter().enumerate() {
                    let xx = (x as isize + k as isize - 1).clamp(0, width as isize - 1) as usize;
                    // Explicitly separate multiplication and addition, matching
                    // strict Java fmul/fadd (no fused multiply-add).
                    let term = *weight * src[y * width + xx] as f32;
                    sum += term;
                }
                dst[x * height + y] = ((sum as f64 + 0.5) as i32).clamp(0, 255) as u8;
            }
        }
    }
    let mut work = vec![0; image.pixels.len()];
    let mut out = blank(image, 0);
    transpose(&image.pixels, &mut work, image.width, image.height, kernel);
    transpose(&work, &mut out.pixels, image.height, image.width, kernel);
    Ok(out)
}

/// SpotsBuilder.eraseHeaderAreas uses ByteProcessor.setRoi/fill, which actually
/// changes bytes. Inclusive geometry is supplied by the GRID/header adapter.
/// Negative/off-page edges clip exactly as ImageProcessor.setRoi does for a
/// normal positive-sized rectangle. Reversed/overflowing rectangles are rejected.
pub fn erase_headers(image: &GrayImage, rectangles: &[HeaderRect]) -> Result<GrayImage, String> {
    validate(image)?;
    if rectangles.len() > 1024 {
        return Err("unsupported_beam_header_count".into());
    }
    let mut out = copy(image);
    for r in rectangles {
        let w = i64::from(r.right) - i64::from(r.left) + 1;
        let h = i64::from(r.bottom) - i64::from(r.top) + 1;
        if w <= 0
            || h <= 0
            || w > i32::MAX as i64
            || h > i32::MAX as i64
            || r.right == i32::MAX
            || r.bottom == i32::MAX
        {
            return Err("invalid_beam_header_rectangle".into());
        }
        let x0 = (r.left as i64).clamp(0, image.width as i64) as usize;
        let x1 = (r.right as i64 + 1).clamp(0, image.width as i64) as usize;
        let y0 = (r.top as i64).clamp(0, image.height as i64) as usize;
        let y1 = (r.bottom as i64 + 1).clamp(0, image.height as i64) as usize;
        for y in y0..y1 {
            out.pixels[y * image.width + x0..y * image.width + x1].fill(255);
        }
    }
    Ok(out)
}

/// StructureElement(0, 1, radius, [0,0]). Its support uses the strict test
/// x*x+y*y < radius*radius+1; this differs from rounding to an integer disk.
/// Each mask member's grayscale height is 255. MorphoProcessor.close's
/// +255/-255 modulo-byte operations cancel, leaving maximum then minimum.
pub fn circular_offsets(radius: f32) -> Result<Vec<[i32; 2]>, String> {
    if !radius.is_finite() || radius < -0.5 || radius > MAX_MORPH_RADIUS {
        return Err("unsupported_beam_morph_radius".into());
    }
    let reach = (radius as f64 + 1.5) as i32;
    let r2 = radius as f64 * radius as f64 + 1.0;
    let mut offsets = Vec::new();
    for y in -reach..=reach {
        for x in -reach..=reach {
            if ((x * x + y * y) as f64) < r2 {
                offsets.push([x, y]);
            }
        }
    }
    Ok(offsets)
}

/// Exact grayscale result of original close. Its delayed output schedule only
/// reads already-computed first-pass pixels, so two complete passes are equivalent
/// even when the structuring element is larger than the image. Exterior values
/// are 0 in the maximum pass and 255 in the minimum pass, not clamped edges.
pub fn close_with_radius(image: &GrayImage, radius: f32) -> Result<GrayImage, String> {
    validate(image)?;
    let offsets = circular_offsets(radius)?;
    if image
        .pixels
        .len()
        .checked_mul(offsets.len())
        .and_then(|v| v.checked_mul(2))
        .is_none_or(|n| n > MAX_MORPH_PROBES)
    {
        return Err("unsupported_beam_morph_work_budget".into());
    }
    let mut work = blank(image, 0);
    let mut out = blank(image, 255);
    extremum(
        &image.pixels,
        &mut work.pixels,
        image.width,
        image.height,
        &offsets,
        false,
    );
    extremum(
        &work.pixels,
        &mut out.pixels,
        image.width,
        image.height,
        &offsets,
        true,
    );
    Ok(out)
}
fn extremum(
    src: &[u8],
    dst: &mut [u8],
    width: usize,
    height: usize,
    offsets: &[[i32; 2]],
    minimum: bool,
) {
    for y in 0..height {
        for x in 0..width {
            let mut value = if minimum { 255 } else { 0 };
            for [dx, dy] in offsets {
                let xx = x as i64 + *dx as i64;
                let yy = y as i64 + *dy as i64;
                // Outside contributes the neutral element for this operation.
                if xx >= 0 && xx < width as i64 && yy >= 0 && yy < height as i64 {
                    let p = src[yy as usize * width + xx as usize];
                    value = if minimum { value.min(p) } else { value.max(p) };
                }
            }
            dst[y * width + x] = value;
        }
    }
}
pub fn close(image: &GrayImage, effective_beam: f64) -> Result<GrayImage, String> {
    if !effective_beam.is_finite() || effective_beam <= 0.0 || effective_beam > 128.0 {
        return Err("unsupported_beam_spot_thickness".into());
    }
    let radius = ((effective_beam * 0.8 - 1.0) as f32) / 2.0_f32;
    close_with_radius(image, radius)
}

/// ImageJ ByteProcessor.threshold uses > level for white; equality stays black.
pub fn threshold(image: &GrayImage, level: u8) -> Result<GrayImage, String> {
    validate(image)?;
    let mut out = copy(image);
    out.pixels
        .iter_mut()
        .for_each(|v| *v = if *v > level { 255 } else { 0 });
    Ok(out)
}
pub fn extract_spots(binary: &GrayImage) -> Result<Vec<GlyphPixels>, String> {
    validate(binary)?;
    audiveris_staff_removal::extract_glyphs(
        BinaryPage::new(binary.width, binary.height, &binary.pixels).map_err(|e| e.to_string())?,
        Orientation::Vertical,
        (0, 0),
        Limits {
            max_pixels: MAX_PIXELS,
            max_dimension: MAX_PIXELS,
            max_run_pixels: MAX_PIXELS,
            max_runs: 1_000_000,
            max_glyphs: 100_000,
        },
    )
    .map_err(|e| format!("beam_spot_glyphs:{e}"))
}

pub fn prepare(
    no_staff: &GrayImage,
    parameters: Parameters,
    headers: &[HeaderRect],
) -> Result<PreparedSpots, String> {
    let filtered_horizontal = filter_horizontal(no_staff, parameters.max_stem)?;
    let median = median_filter(&filtered_horizontal)?;
    let gaussian = gaussian_filter(&median)?;
    let header_erased = erase_headers(&gaussian, headers)?;
    let closed = close(&header_erased, parameters.effective_beam)?;
    let head_spots = threshold(&closed, HEAD_THRESHOLD)?;
    let binarized = threshold(&closed, BEAM_THRESHOLD)?;
    let glyphs = extract_spots(&binarized)?;
    Ok(PreparedSpots {
        parameters,
        filtered_horizontal,
        median,
        gaussian,
        header_erased,
        closed,
        binarized,
        head_spots,
        glyphs,
    })
}
