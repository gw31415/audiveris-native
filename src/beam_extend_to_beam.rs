// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// BeamsBuilder.extendToBeam at 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::{GrayImage, beam_extension_geometry::middle_area, beam_structure::BeamItem};

#[derive(Clone, Debug, PartialEq)]
pub struct GapGate {
    pub gap: f64,
    pub minimum_gap: i32,
    pub counts: Option<(usize, usize)>, // original AreaMask total, black
    pub accepted: bool,
}

/// Side lookup supplies alignment eligibility. This method adds no slope test.
/// Counts include the full area; only black pixels are clipped to the image.
pub fn gap_gate(
    image: &GrayImage,
    first: &BeamItem,
    second: &BeamItem,
    interline: usize,
) -> Result<GapGate, String> {
    // Validate both inputs even when the original skips sampling a short gap.
    let area = middle_area(first.median, first.height, second.median, second.height)?;
    if interline == 0 || interline > crate::MAX_PIXELS {
        return Err("unsupported_beam_extension_interline".into());
    }
    let a = first.median;
    let b = second.median;
    let gap = if a[0] < b[0] {
        b[0] - a[2]
    } else {
        a[0] - b[2]
    };
    let minimum_gap = (0.2 * interline as f64).round_ties_even() as i32;
    let mut result = GapGate {
        gap,
        minimum_gap,
        counts: None,
        accepted: true,
    };
    if gap >= minimum_gap as f64 {
        let bounds = area.bounds();
        let cells = (bounds[2] as usize)
            .checked_mul(bounds[3] as usize)
            .ok_or("beam_extension_mask_budget_exceeded")?;
        if cells > 100_000_000 {
            return Err("beam_extension_mask_budget_exceeded".into());
        }
        let (mut total, mut black) = (0, 0);
        for y in bounds[1]..bounds[1] + bounds[3] {
            for x in bounds[0]..bounds[0] + bounds[2] {
                if area.contains([x as f64, y as f64])? {
                    total += 1;
                    if x >= 0
                        && y >= 0
                        && (x as usize) < image.width
                        && (y as usize) < image.height
                        && image.pixels[y as usize * image.width + x as usize] == 0
                    {
                        black += 1;
                    }
                }
            }
        }
        result.counts = Some((total, black));
        // Original dcmpg rejects NaN as well as a fraction below 0.6.
        result.accepted = black as f64 / total as f64 >= 0.6;
    }
    Ok(result)
}

#[derive(Clone, Copy, Debug)]
pub struct ExtensionScale {
    pub interline: usize,
    pub standard_height: f64,
    pub small_height: Option<f64>,
}

impl crate::beam_candidates::SystemCandidates {
    /// Original gap gate, merge evaluation, register/append, then remove both.
    /// Rejection preserves candidates, exclusions and the shared glyph registry.
    pub fn extend_to_beam(
        &mut self,
        image: &GrayImage,
        registry: &mut crate::beam_dispatch::Registry,
        first: usize,
        second: usize,
        scale: ExtensionScale,
    ) -> Result<Option<usize>, String> {
        if first == second {
            return Err("invalid_beam_extension_endpoints".into());
        }
        let input = |id| {
            self.active(id)
                .map(|v| crate::beam_merge::MergeInput {
                    item: v.item.clone(),
                    distance_impact: v.impacts.values[5],
                })
                .ok_or_else(|| "invalid_beam_extension_endpoints".to_string())
        };
        let a = input(first)?;
        let b = input(second)?;
        if !gap_gate(image, &a.item, &b.item, scale.interline)?.accepted {
            return Ok(None);
        }
        let candidate = crate::beam_merge::merge_of(
            image,
            &a,
            &b,
            scale.interline,
            scale.standard_height,
            scale.small_height,
        )?;
        self.apply_merge(image, registry, first, second, candidate.as_ref())
    }
}
