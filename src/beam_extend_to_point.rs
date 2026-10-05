// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Original extendToPoint boundary, pinned 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::{
    GrayImage,
    beam_extend_to_beam::ExtensionScale,
    beam_extension_geometry::{HorizontalSide, side_area_of},
    beam_impacts::{Evaluator, Impacts},
    beam_merge::MergeCandidate,
    beam_structure::{BeamItem, ItemParameters},
};
#[derive(Clone, Debug, PartialEq)]
pub struct PointGate {
    pub gap: f64,
    pub counts: Option<(usize, usize)>,
    pub accepted: bool,
}
pub fn point_gate(
    image: &GrayImage,
    item: &BeamItem,
    side: HorizontalSide,
    point: [f64; 2],
) -> Result<PointGate, String> {
    if point.iter().any(|v| !v.is_finite() || v.abs() > 8_000_000.) {
        return Err("unsupported_beam_extension_point".into());
    }
    // Validate the item even on the short-gap path.
    side_area_of(item.median, item.height, side, 0., 0., 0.)?;
    let gap = match side {
        HorizontalSide::Left => item.median[0] - point[0],
        HorizontalSide::Right => point[0] - item.median[2],
    };
    if gap <= 1. {
        return Ok(PointGate {
            gap,
            counts: None,
            accepted: false,
        });
    }
    let area = side_area_of(item.median, item.height, side, 0., gap, 0.)?;
    let b = area.bounds();
    if (b[2] as usize)
        .checked_mul(b[3] as usize)
        .is_none_or(|n| n > 100_000_000)
    {
        return Err("beam_extension_mask_budget_exceeded".into());
    }
    let (mut total, mut black) = (0, 0);
    for y in b[1]..b[1] + b[3] {
        for x in b[0]..b[0] + b[2] {
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
    Ok(PointGate {
        gap,
        counts: Some((total, black)),
        // Original dcmpg/ifge permits NaN when the sampled area is empty.
        accepted: total == 0 || black as f64 / total as f64 >= 0.6,
    })
}
impl crate::beam_candidates::SystemCandidates {
    /// Register and append a graded replacement before removing the original.
    /// This is the point boundary only; stem selection and loop scheduling are separate.
    pub fn extend_to_point(
        &mut self,
        image: &GrayImage,
        registry: &mut crate::beam_dispatch::Registry,
        target: usize,
        side: HorizontalSide,
        point: [f64; 2],
        scale: ExtensionScale,
    ) -> Result<Option<usize>, String> {
        let old = self.active(target).ok_or("invalid_beam_extension_target")?;
        let standard = ItemParameters::new(scale.interline, scale.standard_height, false)?;
        let small = scale
            .small_height
            .map(|h| ItemParameters::new(scale.interline, h, true))
            .transpose()?;
        let params = match small {
            Some(s)
                if (old.item.height - standard.typical_height).abs()
                    > (old.item.height - s.typical_height).abs() =>
            {
                s
            }
            _ => standard,
        };
        if !point_gate(image, &old.item, side, point)?.accepted {
            return Ok(None);
        }
        let m = old.item.median;
        let item = BeamItem {
            median: match side {
                HorizontalSide::Left => [point[0], point[1], m[2], m[3]],
                HorizontalSide::Right => [m[0], m[1], point[0], point[1]],
            },
            height: old.item.height,
        };
        let evaluation = Evaluator::new(image, scale.interline)?.compute_beam_impacts(
            &item,
            true,
            true,
            old.impacts.values[5],
            &params,
        )?;
        let Some(impacts) = evaluation.impacts.filter(Impacts::accepted_as_beam) else {
            return Ok(None);
        };
        let candidate = MergeCandidate {
            item,
            is_small: params.is_small,
            impacts,
        };
        let appended = self.register_merge(image, registry, &candidate)?;
        self.remove(target)?;
        Ok(Some(appended))
    }
}
