// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Pure BeamsBuilder.mergeOf port at 9e1e55cd2746037d059345881c53e6a6754bffbd.
// No SIG insertion/removal, glyph registration, gap policy, or orchestration.
use crate::{
    GrayImage,
    beam_impacts::{Evaluator, Impacts},
    beam_structure::{BeamItem, ItemParameters},
};

#[derive(Clone, Debug, PartialEq)]
pub struct MergeInput {
    pub item: BeamItem,
    pub distance_impact: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MergeCandidate {
    pub item: BeamItem,
    pub is_small: bool,
    pub impacts: Impacts,
}

/// Return the original merge candidate or rejection, without mutating inputs.
///
/// Supported medians increase in x and obey the existing item geometry bounds;
/// heights are positive and distance impacts finite. Parameter selection uses
/// the FIRST input's height, not its class or the merged height; ties use standard.
/// Gap/slope eligibility belongs to the caller and is not added to this method.
pub fn merge_of(
    image: &GrayImage,
    first: &MergeInput,
    second: &MergeInput,
    interline: usize,
    standard_height: f64,
    small_height: Option<f64>,
) -> Result<Option<MergeCandidate>, String> {
    for input in [first, second] {
        if input
            .item
            .median
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 8_000_000.)
            || input.item.median[0] >= input.item.median[2]
            || !input.item.height.is_finite()
            || input.item.height <= 0.
            || input.item.height > crate::MAX_PIXELS as f64
            || !input.distance_impact.is_finite()
        {
            return Err("unsupported_beam_merge_geometry".into());
        }
    }
    let standard = ItemParameters::new(interline, standard_height, false)?;
    let small = small_height
        .map(|h| ItemParameters::new(interline, h, true))
        .transpose()?;
    let params = match small {
        Some(small)
            if (first.item.height - standard.typical_height).abs()
                > (first.item.height - small.typical_height).abs() =>
        {
            small
        }
        _ => standard,
    };
    let a = first.item.median;
    let b = second.item.median;
    let aw = a[2] - a[0];
    let bw = b[2] - b[0];
    let item = BeamItem {
        median: if a[0] < b[0] {
            [a[0], a[1], b[2], b[3]]
        } else {
            [b[0], b[1], a[2], a[3]]
        },
        height: (first.item.height * aw + second.item.height * bw) / (aw + bw),
    };
    let distance = (first.distance_impact + second.distance_impact) / 2.0;
    let evaluation = Evaluator::new(image, interline)?
        .compute_beam_impacts(&item, true, true, distance, &params)?;
    Ok(evaluation
        .impacts
        .filter(Impacts::accepted_as_beam)
        .map(|impacts| MergeCandidate {
            item,
            is_small: params.is_small,
            impacts,
        }))
}
