// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete lookupChunks suffix on supplied raw glyphs and upstream context.
//! Registry IDs have one owner. Page discovery, cache/head/SIG and epilog are separate.
use crate::{
    beam_dispatch::Registry, stem_chunk_head_filter, stem_chunk_registration, stem_parts_alignment,
};
use audiveris_staff_removal::GlyphPixels;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Selected {
    pub registered_ids: Vec<usize>,
    pub retained_ids: Vec<usize>,
    pub new_count: usize,
    pub registry_count: usize,
}
/// Registers every raw glyph before filtering, including rejected glyphs.
/// Exclusion and alignment remove all aliases; stump removal removes one.
/// Stable ordinate/reverse-bottom order preserves ties. Refusal is atomic.
pub fn select(
    registry: &mut Registry,
    raw: &[GlyphPixels],
    existing: &[usize],
    stump: Option<usize>,
    head: Option<(&GlyphPixels, bool)>,
    parameters: &stem_parts_alignment::Parameters,
) -> Result<Selected, String> {
    let parameters = stem_parts_alignment::Parameters::new(
        parameters.direction,
        parameters.max_dx,
        parameters.max_dy,
        parameters.slope,
    )?;
    if let Some(id) = stump {
        registry.get(id)?;
    }
    // Stage the entire composition so a later head/alignment refusal cannot
    // leave only the registration prefix committed to the caller's registry.
    let mut staged = registry.clone();
    let registered = stem_chunk_registration::register_and_exclude(&mut staged, raw, existing)?;
    let ids = match head {
        Some((g, vip)) => {
            stem_chunk_head_filter::filter(&staged, &registered.retained_ids, g, vip)?.retained_ids
        }
        None => registered.retained_ids,
    };
    let parts = ids
        .iter()
        .map(|&id| {
            Ok(stem_parts_alignment::Part {
                id,
                glyph: &staged.get(id)?.glyph,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let stump_part = stump
        .map(|id| -> Result<stem_parts_alignment::Part<'_>, String> {
            Ok(stem_parts_alignment::Part {
                id,
                glyph: &staged.get(id)?.glyph,
            })
        })
        .transpose()?;
    let mut kept = stem_parts_alignment::filter(&parts, stump_part, &parameters)?.retained_order;
    if let Some(id) = stump
        && let Some(i) = kept.iter().position(|x| *x == id)
    {
        kept.remove(i);
    }
    kept.sort_by_key(|&id| {
        let g = &staged.get(id).expect("validated registry identity").glyph;
        if parameters.direction > 0 {
            g.top as i64
        } else {
            -(g.top as i64 + g.height as i64 - 1)
        }
    });
    let selected = Selected {
        registered_ids: registered.registered_ids,
        retained_ids: kept,
        new_count: registered.new_count,
        registry_count: registered.registry_count,
    };
    registry.commit_extension(staged)?;
    Ok(selected)
}
