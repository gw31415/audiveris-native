// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit-position template evaluation, not a notehead detector.
use crate::{GrayImage, MAX_PIXELS};
use audiveris_template_bank::Bank;
use audiveris_template_match::{
    DistanceTable, Limits, Storage, Weights, compute_masked_foreground,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const MAX_REQUEST_BYTES: usize = 1_048_576;
pub const MAX_PROBES: usize = 4096;
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub shape: String,
    pub point_size: u16,
    pub x: i32,
    pub y: i32,
    pub anchor: Option<String>,
}

/// Binary page and optional externally supplied ignore mask (0 ignored, 255 kept).
/// The mask is whitened before distance propagation and neutralized after it.
/// This does not infer the staff/ledger/seed mask selected by DistancesBuilder.
pub fn distances(binary: &GrayImage, mask: Option<&GrayImage>) -> Result<DistanceTable, String> {
    if binary.width.checked_mul(binary.height) != Some(binary.pixels.len())
        || binary.pixels.is_empty()
        || binary.pixels.len() > MAX_PIXELS
        || binary.pixels.iter().any(|&v| v != 0 && v != 255)
    {
        return Err("invalid binary page or pixel budget".into());
    }
    let ignored = match mask {
        Some(m) => {
            if m.width != binary.width
                || m.height != binary.height
                || m.pixels.len() != binary.pixels.len()
                || m.pixels.iter().any(|&v| v != 0 && v != 255)
            {
                return Err("ignore mask must be binary and match page dimensions".into());
            }
            m.pixels.iter().map(|&v| v == 0).collect::<Vec<_>>()
        }
        None => vec![false; binary.pixels.len()],
    };
    compute_masked_foreground(
        binary.width,
        binary.height,
        &binary.pixels,
        &ignored,
        Storage::JavaShort,
        Limits {
            max_pixels: MAX_PIXELS,
            ..Limits::default()
        },
    )
    .map_err(|e| e.to_string())
}

pub fn evaluate(
    binary: &GrayImage,
    mask: Option<&GrayImage>,
    request: &[u8],
) -> Result<Value, String> {
    if request.len() > MAX_REQUEST_BYTES {
        return Err("probe request exceeds 1 MiB".into());
    }
    let probes: Vec<Probe> = serde_json::from_slice(request).map_err(|e| e.to_string())?;
    if probes.is_empty() || probes.len() > MAX_PROBES {
        return Err("expected 1..4096 probes".into());
    }
    let bank = Bank::load_builtin().map_err(|e| e.to_string())?;
    // Validate coverage/anchors before allocating the page distance table.
    for p in &probes {
        let asset = bank
            .get(&p.shape, p.point_size)
            .map_err(|e| e.to_string())?;
        if let Some(a) = &p.anchor {
            asset.anchor(a).map_err(|e| e.to_string())?;
        }
    }
    let table = distances(binary, mask)?;
    let mut digest = Sha256::new();
    for value in table.values() {
        digest.update((value as i16).to_be_bytes());
    }
    let distance_hash = format!("{:x}", digest.finalize());
    let binary_hash = format!("{:x}", Sha256::digest(&binary.pixels));
    let mut results = Vec::with_capacity(probes.len());
    for p in probes {
        let asset = bank
            .get(&p.shape, p.point_size)
            .map_err(|e| e.to_string())?;
        let anchor = p
            .anchor
            .as_deref()
            .map(|a| asset.anchor(a))
            .transpose()
            .map_err(|e| e.to_string())?;
        let e = asset
            .template()
            .evaluate(&table, p.x, p.y, anchor, Weights::default())
            .map_err(|e| e.to_string())?;
        results.push(
            json!({"probe":p,"score":e.score,"hole_white_ratio":e.hole_white_ratio,
            "included_points":e.included_points,"expected_holes":e.expected_holes}),
        );
    }
    Ok(
        json!({"schema":"audiveris-native.template-probes.v1", "upstream_commit":"9e1e55cd2746037d059345881c53e6a6754bffbd",
        "width":binary.width,"height":binary.height,"binary_sha256":binary_hash,"distance_sha256_be_i16":distance_hash,"font":"Bravura","storage":"JavaShort",
        "distance_overflowed":table.overflowed(),"mask_source":if mask.is_some(){"explicit_binary_mask"}else{"none"},
        "grid_complete":false,"note_recognition_complete":false,"automatic_mask_selection":false,
        "results":results}),
    )
}
