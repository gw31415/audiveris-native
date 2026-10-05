// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original getGlyphsUntil/getLastGapBefore and declared endpoint to owned cache.
//! Registry IDs already encode plain Glyph physical equality including orientation.
//! Automatic endpoint selection and page cleaner/epilog remain separate.
use crate::{
    GrayImage,
    beam_dispatch::Registry,
    stem_cache_sig::{Creation, Retriever},
    stem_checks::Parameters,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub glyph_id: Option<usize>,
    pub gap: bool,
}
fn index(items: &[Item], last: isize) -> Result<(), String> {
    if items.len() > 4096 || last < -1 || last >= items.len() as isize {
        return Err("invalid or unsupported item prefix index".into());
    }
    Ok(())
}
/// Inclusive prefix; contribution/kind do not exclude a non-null glyph.
/// First physical occurrence order survives duplicate canonical registry IDs.
pub fn select_until(
    registry: &Registry,
    items: &[Item],
    last: isize,
) -> Result<Vec<usize>, String> {
    index(items, last)?;
    // Validate IDs against the declared physical registry before any downstream mutation, including
    // unused later items. Invalid native IDs are an explicit refusal boundary.
    for i in items {
        if let Some(id) = i.glyph_id {
            registry.get(id)?;
        }
    }
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for i in items.iter().take((last + 1) as usize) {
        if let Some(id) = i.glyph_id
            && seen.insert(id)
        {
            selected.push(id);
        }
    }
    Ok(selected)
}
/// Excludes the indexed item. Index equal to item count searches all items.
pub fn last_gap_before(items: &[Item], before: usize) -> Result<Option<usize>, String> {
    if items.len() > 4096 || before > items.len() {
        return Err("invalid or unsupported gap prefix index".into());
    }
    Ok((0..before).rev().find(|&i| items[i].gap))
}
/// Prefix selection feeds the existing physical registration/grade/cache path.
/// Grade rejection retains the registered glyph as in original createStem.
/// SIG registration/explicit head connection use the existing owned APIs after
/// this returns a stem; selection alone does not register a SIG vertex.
pub fn create_until(
    retriever: &mut Retriever,
    registry: &mut Registry,
    items: &[Item],
    last: isize,
    image: &GrayImage,
    parameters: &Parameters,
) -> Result<Creation, String> {
    let ids = select_until(registry, items, last)?;
    let parts = ids
        .iter()
        .map(|&id| registry.get(id).map(|e| e.glyph.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    retriever.create_selected(registry, &parts, image, parameters)
}
