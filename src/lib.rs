// Copyright © Audiveris 2026. All rights reserved.
// Rust adaptation and new I/O: 2026 audiveris-native contributors.
// SPDX-License-Identifier: AGPL-3.0-or-later
// Port of AdaptiveFilter/RandomFilter and RunsRetriever at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// This program is distributed WITHOUT ANY WARRANTY; see LICENSE.

pub const MAX_PIXELS: usize = 4_000_000;
#[derive(Debug)]
pub struct GrayImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}
impl GrayImage {
    pub fn new(width: usize, height: usize, pixels: Vec<u8>) -> Result<Self, String> {
        let n = width
            .checked_mul(height)
            .ok_or("image dimensions overflow")?;
        if n == 0 || n > MAX_PIXELS || n != pixels.len() {
            return Err("invalid dimensions/pixels (limit 4 million pixels)".into());
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    /// Decode grayscale bytes without introducing a new RGB-to-gray contract.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Self::from_pgm(bytes);
        }
        let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        decoder.set_limits(png::Limits { bytes: 32_000_000 });
        let info = decoder.read_header_info().map_err(|e| e.to_string())?;
        let (w, h) = (info.width as usize, info.height as usize);
        if w.checked_mul(h).is_none_or(|n| n == 0 || n > MAX_PIXELS) {
            return Err("invalid PNG dimensions".into());
        }
        if info.color_type != png::ColorType::Grayscale || info.bit_depth != png::BitDepth::Eight {
            return Err("this milestone accepts only 8-bit grayscale PNG, without alpha".into());
        }
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        if reader.info().trns.is_some() {
            return Err("PNG transparency (tRNS) is unsupported; require opaque grayscale".into());
        }
        if reader.info().animation_control.is_some() {
            return Err("animated PNG is unsupported".into());
        }
        let n = reader.output_buffer_size().ok_or("PNG buffer overflow")?;
        if n != w * h {
            return Err("unexpected PNG buffer size".into());
        }
        let mut pixels = vec![0; n];
        reader.next_frame(&mut pixels).map_err(|e| e.to_string())?;
        Self::new(w, h, pixels)
    }
    pub fn from_pgm(bytes: &[u8]) -> Result<Self, String> {
        let mut p = 0;
        fn token<'a>(b: &'a [u8], p: &mut usize) -> Result<&'a str, String> {
            loop {
                while *p < b.len() && b[*p].is_ascii_whitespace() {
                    *p += 1;
                }
                if b.get(*p) != Some(&b'#') {
                    break;
                }
                while *p < b.len() && b[*p] != b'\n' {
                    *p += 1;
                }
            }
            let start = *p;
            while *p < b.len() && !b[*p].is_ascii_whitespace() && b[*p] != b'#' {
                *p += 1;
            }
            if start == *p {
                return Err("truncated PGM".into());
            }
            std::str::from_utf8(&b[start..*p]).map_err(|_| "invalid PGM token".into())
        }
        let magic = token(bytes, &mut p)?;
        if magic != "P2" && magic != "P5" {
            return Err("only grayscale PGM P2/P5 is supported in this milestone".into());
        }
        let w: usize = token(bytes, &mut p)?.parse().map_err(|_| "invalid width")?;
        let h: usize = token(bytes, &mut p)?
            .parse()
            .map_err(|_| "invalid height")?;
        let max: usize = token(bytes, &mut p)?
            .parse()
            .map_err(|_| "invalid maximum")?;
        if max != 255 {
            return Err("PGM maximum must be 255 (no implicit rescaling)".into());
        }
        let n = w
            .checked_mul(h)
            .filter(|n| *n > 0 && *n <= MAX_PIXELS)
            .ok_or("invalid image size")?;
        let pixels = if magic == "P5" {
            if !bytes.get(p).is_some_and(u8::is_ascii_whitespace) {
                return Err("missing raster separator".into());
            }
            // Consume exactly one separator; raster may itself start with whitespace or '#'.
            p += 1;
            if bytes.len() - p != n {
                return Err("PGM raster size mismatch; P5 requires one separator byte".into());
            }
            bytes[p..].to_vec()
        } else {
            let mut pixels = Vec::with_capacity(n);
            for _ in 0..n {
                pixels.push(
                    token(bytes, &mut p)?
                        .parse::<u8>()
                        .map_err(|_| "invalid sample")?,
                );
            }
            if token(bytes, &mut p).is_ok() {
                return Err("extra PGM samples".into());
            }
            pixels
        };
        Self::new(w, h, pixels)
    }
    pub fn pgm(&self) -> Vec<u8> {
        let mut out = format!("P5\n{} {}\n255\n", self.width, self.height).into_bytes();
        out.extend_from_slice(&self.pixels);
        out
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub mean: f64,
    pub std_dev: f64,
    pub threshold: f64,
}
pub struct AdaptiveFilter<'a> {
    image: &'a GrayImage,
    sums: Vec<u64>,
    squares: Vec<u64>,
    stride: usize,
}
impl<'a> AdaptiveFilter<'a> {
    pub fn new(image: &'a GrayImage) -> Self {
        let stride = image.width + 1;
        let mut sums = vec![0; stride * (image.height + 1)];
        let mut squares = sums.clone();
        for y in 0..image.height {
            for x in 0..image.width {
                let v = u64::from(image.pixels[y * image.width + x]);
                let i = (y + 1) * stride + x + 1;
                sums[i] = v + sums[i - 1] + sums[i - stride] - sums[i - stride - 1];
                squares[i] = v * v + squares[i - 1] + squares[i - stride] - squares[i - stride - 1];
            }
        }
        Self {
            image,
            sums,
            squares,
            stride,
        }
    }
    pub fn context(&self, x: usize, y: usize) -> Context {
        assert!(x < self.image.width && y < self.image.height);
        let x1 = x.saturating_sub(18);
        let y1 = y.saturating_sub(18);
        let x2 = (x + 19).min(self.image.width);
        let y2 = (y + 19).min(self.image.height);
        let area = ((x2 - x1) * (y2 - y1)) as f64;
        let rect = |a: &[u64]| {
            (a[y2 * self.stride + x2] + a[y1 * self.stride + x1]
                - a[y1 * self.stride + x2]
                - a[y2 * self.stride + x1]) as f64
                / area
        };
        let mean = rect(&self.sums);
        // Preserve upstream abs, <=, and evaluation order (not a generic Sauvola formula).
        let std_dev = (rect(&self.squares) - mean * mean).abs().sqrt();
        Context {
            mean,
            std_dev,
            threshold: 0.7 * mean + 0.9 * std_dev,
        }
    }
    pub fn binary(&self) -> GrayImage {
        let pixels = self
            .image
            .pixels
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if f64::from(*p)
                    <= self
                        .context(i % self.image.width, i / self.image.width)
                        .threshold
                {
                    0
                } else {
                    255
                }
            })
            .collect();
        GrayImage {
            width: self.image.width,
            height: self.image.height,
            pixels,
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Run {
    pub start: usize,
    pub length: usize,
}
/// Equivalent to RunsRetriever.processPosition with an accept-all adapter, full-image ROI.
pub fn runs(image: &GrayImage, vertical: bool) -> Vec<Vec<Run>> {
    let (positions, coords) = if vertical {
        (image.width, image.height)
    } else {
        (image.height, image.width)
    };
    (0..positions)
        .map(|pos| {
            let mut runs = Vec::new();
            let mut start = None;
            for c in 0..=coords {
                let fore = c < coords
                    && image.pixels[if vertical {
                        c * image.width + pos
                    } else {
                        pos * image.width + c
                    }] == 0;
                if fore {
                    if start.is_none() {
                        start = Some(c);
                    }
                } else if let Some(s) = start.take() {
                    runs.push(Run {
                        start: s,
                        length: c - s,
                    });
                }
            }
            runs
        })
        .collect()
}

pub mod vertical;

/// One orientation/position at a time, with no heap allocation for runs.
pub fn position_runs(
    image: &GrayImage,
    vertical: bool,
    pos: usize,
) -> impl Iterator<Item = Run> + '_ {
    let coordinates = if vertical { image.height } else { image.width };
    let mut c = 0;
    let pixel = move |c: usize| {
        image.pixels[if vertical {
            c * image.width + pos
        } else {
            pos * image.width + c
        }]
    };
    std::iter::from_fn(move || {
        while c < coordinates && pixel(c) != 0 {
            c += 1;
        }
        if c == coordinates {
            return None;
        }
        let start = c;
        while c < coordinates && pixel(c) == 0 {
            c += 1;
        }
        Some(Run {
            start,
            length: c - start,
        })
    })
}
pub mod glyph;
pub mod scale;
pub mod sections;

pub mod filaments;

pub mod curves;
pub mod filament_merge;

pub mod clusters;

pub mod staff_projection;

pub mod bar_sticks;

pub mod line_completion;
pub mod page_glyphs;
pub mod system_graph;

pub mod bar_semantics;
pub mod braces;
pub mod grid_ir;

pub mod template_probes;

pub mod note_mask;

pub mod sig_context;

pub mod header_preparation;

pub mod clef_candidates;

pub mod clef_registration;

pub mod key_preparation;

pub mod key_peaks;

pub mod key_slices;

pub mod key_components;

pub mod key_clef;

pub mod clef_competition;

pub mod key_state;

pub mod time_preparation;

pub mod time_candidates;

pub mod time_registration;

pub mod key_column;

pub mod header_finalization;

pub mod time_column;

pub mod vertical_sticks;

pub mod stem_scale;

pub mod stem_checks;

pub mod stem_attachment;
mod stem_beam_finalizer;
pub mod stem_builder;
pub mod stem_cache_sig;
pub mod stem_cleaner;
pub mod stem_glyph;
pub mod stem_parts_alignment;
pub mod stem_parts_cli;
pub mod stem_parts_filter;
pub mod stem_parts_lookup;
pub mod stem_seeds;

pub mod beam_spots;

pub mod beam_preparation;

pub mod black_head_sizer;

pub mod beam_dispatch;

pub mod beam_prolog;

pub mod beam_structure;

pub mod beam_impacts;

pub mod beam_candidates;

pub mod beam_extension_geometry;

pub mod beam_extend_to_beam;
pub mod beam_extension_loop;
pub mod beam_extension_prefix;
pub mod beam_late_hooks;
pub mod beam_merge;
pub mod beam_side_lookup;

pub mod beam_extend_to_point;

pub mod beam_extend_to_stem;

pub mod beam_extend_to_spot;

pub mod beam_extend_in_parallel;

pub mod beam_groups;

pub mod multiple_rest_admission;

pub mod serif_raster;

pub mod serif_peaks;

pub mod multiple_rest_detection;

pub mod multiple_rest_registration;
pub mod serif_glyphs;

pub mod multiple_rest_pipeline;

pub mod shared_sig;

pub mod beams_pipeline;
pub mod native_rest_mapping;

pub mod system_area_ends;

pub mod system_area_geometry;

pub mod ledger_candidates;

pub mod ledger_checks;

pub mod ledger_beam_purge;
pub mod ledger_inputs;
pub mod ledger_post_analysis;
pub mod ledger_reference;
pub mod ledger_registration;

pub mod head_seed_search;
pub mod head_seed_veto;
pub mod head_staff_catalog;
pub mod head_stem_relation;

pub mod head_ledger_search;

pub mod head_range_search;

pub mod head_candidate_filter;

pub mod head_glyph_retrieval;

pub mod head_beam_context;
pub mod head_registration;

pub mod stem_lookup_area;

mod stem_beam_closer_limit;
pub mod stem_beam_lookup_area;

pub mod stem_chunk_registration;

pub mod stem_chunk_head_filter;

pub mod stem_chunks;

pub mod stem_chunks_cli;

pub mod stem_cache_attachment;

pub mod stem_length_map;

pub mod stem_item_events;
pub mod stem_items_cli;

pub mod stem_parts_cache;

pub mod stem_head_targets;

pub mod stem_head_neighborhood;

pub mod stem_head_reference_context;

pub mod stem_image_head_plan;

pub mod stem_image_seeds;

pub mod stem_image_sources;
pub mod stem_image_stumps;

pub mod stem_image_linkers;

pub mod stem_image_beam_targets;

pub mod stem_image_beam_generation;

pub mod stem_image_head_parts;

pub mod stem_image_head_chunks;

pub mod stem_image_head_items;

pub mod page_runtime;
pub mod stem_image_owned;

pub mod stem_image_head_expansion;

mod head_chord_geometry;
mod head_musical_pitch;
mod music_beams;
mod music_containers;
mod music_flags;
mod music_headers;
mod music_ledger_reduction;
mod music_links;
pub mod music_measures;
mod music_symbol_reduction;
pub mod music_symbols;
pub mod music_xml;
mod stem_chord_consistency;
mod stem_foundation_consistency;
mod stem_foundation_overlap;
pub mod stem_foundation_purge;
mod stem_head_foundation_plan;
pub mod stem_head_foundations;
mod stem_image_beam_builders;
pub mod stem_image_head_linking;
mod stem_overlap_ownership;
