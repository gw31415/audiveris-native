// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected-part StemBuilder.createStem boundary; automatic parts/links/epilog
//! remain separate. The returned cached stem has no SIG identity yet.
use crate::{
    GrayImage,
    beam_dispatch::{self, Registry},
    stem_checks, stem_glyph,
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
/// Original StemInter.getMinGrade: minimum-grade ratio0.1 * intrinsic ratio0.8.
/// It is distinct from StemChecker's SEED minimum threshold0.2.
pub const MIN_GRADE: f64 = 0.1 * 0.8;
const MAX_CACHED_STEMS: usize = 32768;
#[derive(Clone, Debug)]
pub struct CreatedStem {
    pub glyph_id: usize,
    pub record: Value,
}
#[derive(Clone, Debug)]
pub struct Creation {
    pub glyph_id: usize,
    pub cache_hit: bool,
    pub stem: Option<Arc<CreatedStem>>,
}
#[derive(Debug)]
pub struct Cache {
    system_id: u64,
    stems: BTreeMap<usize, Arc<CreatedStem>>,
}
impl Cache {
    pub fn new(system_id: u64) -> Result<Self, String> {
        if system_id == 0 {
            return Err("invalid_stem_builder_system".into());
        }
        Ok(Self {
            system_id,
            stems: BTreeMap::new(),
        })
    }
    pub fn system_id(&self) -> u64 {
        self.system_id
    }
    pub fn len(&self) -> usize {
        self.stems.len()
    }
    pub fn is_empty(&self) -> bool {
        self.stems.is_empty()
    }
    /// Validated parts are registered before cache/grade rejection. A singleton
    /// retains its run orientation; compounds use the original vertical union.
    /// Cache hits return the same object without regrading, even if the valid
    /// supplied image/profile changed. New head profiles0..3; artificial beam-side4 pending.
    /// A cache belongs to one system and physical registry. Reusing it with a
    /// conflicting registry ID refuses rather than returning another glyph.
    /// This method never adds a SIG vertex or HeadStemRelation.
    pub fn create_selected(
        &mut self,
        registry: &mut Registry,
        parts: &[GlyphPixels],
        image: &GrayImage,
        parameters: &stem_checks::Parameters,
    ) -> Result<Creation, String> {
        let parameters = stem_checks::Parameters::new(
            parameters.interline,
            parameters.max_stem,
            parameters.global_slope,
            parameters.profile,
        )?;
        if image.width == 0
            || image.height == 0
            || image
                .width
                .checked_mul(image.height)
                .is_none_or(|n| n > crate::MAX_PIXELS || n != image.pixels.len())
        {
            return Err("invalid_stem_builder_image".into());
        }
        let glyph = if let [single] = parts {
            beam_dispatch::centroid(single)?;
            if single.runs.iter().map(|r| r.length).sum::<usize>() > stem_glyph::MAX_PART_PIXELS {
                return Err("stem_glyph_pixel_budget".into());
            }
            single.clone()
        } else {
            stem_glyph::assemble(parts)?
        };
        let glyph_id = registry.register(&glyph)?;
        if let Some(stem) = self.stems.get(&glyph_id) {
            if stem.record["glyph"] != beam_dispatch::glyph_json(&glyph) {
                return Err("stem_builder_cache_registry_mismatch".into());
            }
            return Ok(Creation {
                glyph_id,
                cache_hit: true,
                stem: Some(Arc::clone(stem)),
            });
        }
        let checked = stem_checks::check_glyph(image, &glyph, &parameters)?;
        if checked.grade < MIN_GRADE {
            return Ok(Creation {
                glyph_id,
                cache_hit: false,
                stem: None,
            });
        }
        if self.stems.len() >= MAX_CACHED_STEMS {
            return Err("unsupported_stem_builder_cache_budget".into());
        }
        let geometry = stem_glyph::geometry(&glyph)?;
        let impacts = json!({"grade":checked.grade,"intrinsic_ratio":0.8,"checks":checked.checks.iter().map(|c|json!({"name":c.name,"weight":c.weight,"impact":c.impact})).collect::<Vec<_>>()});
        let record = json!({"id":0,"in_sig":false,"grade":checked.grade,"bounds":geometry.bounds,"median":geometry.median,"width":geometry.width,"glyph":beam_dispatch::glyph_json(&glyph),"impacts":impacts});
        let stem = Arc::new(CreatedStem { glyph_id, record });
        self.stems.insert(glyph_id, Arc::clone(&stem));
        Ok(Creation {
            glyph_id,
            cache_hit: false,
            stem: Some(stem),
        })
    }
}
