// SPDX-License-Identifier: AGPL-3.0-or-later
//! StemBuilder.lookupChunks prefix: register every raw physical chunk before
//! removing existing canonical parts. Later filters, sorting and SIG writes
//! remain separate dependencies. Stable registry IDs represent glyph identity.
use crate::beam_dispatch::{self, Registry};
use audiveris_staff_removal::GlyphPixels;
use serde::Serialize;
use std::collections::BTreeSet;
#[derive(Debug, Serialize)]
pub struct Registered {
    pub registered_ids: Vec<usize>,
    pub retained_ids: Vec<usize>,
    pub new_count: usize,
    pub registry_count: usize,
}
/// Existing IDs must already belong to this registry. Register all raw glyphs
/// in input order, retaining aliases and rejected/excluded glyphs in the
/// registry. Remove every candidate occurrence of each existing canonical ID.
/// An invalid input/budget refusal leaves the original registry unchanged.
pub fn register_and_exclude(
    registry: &mut Registry,
    raw: &[GlyphPixels],
    existing: &[usize],
) -> Result<Registered, String> {
    if raw.len() > 4096 || existing.len() > 4096 {
        return Err("stem_chunk_registration_budget".into());
    }
    for &id in existing {
        registry.get(id)?;
    }
    let mut pixels = 0usize;
    for glyph in raw {
        beam_dispatch::centroid(glyph)?;
        for r in &glyph.runs {
            pixels = pixels
                .checked_add(r.length)
                .filter(|n| *n <= 8_000_000)
                .ok_or("stem_chunk_registration_pixel_budget")?;
        }
    }
    let before = registry.len();
    let mut staged = registry.clone();
    let mut registered_ids = Vec::with_capacity(raw.len());
    for glyph in raw {
        registered_ids.push(staged.register(glyph)?);
    }
    let excluded = existing.iter().copied().collect::<BTreeSet<_>>();
    let retained_ids = registered_ids
        .iter()
        .copied()
        .filter(|id| !excluded.contains(id))
        .collect();
    let result = Registered {
        registered_ids,
        retained_ids,
        new_count: staged.len() - before,
        registry_count: staged.len(),
    };
    registry.commit_extension(staged)?;
    Ok(result)
}
