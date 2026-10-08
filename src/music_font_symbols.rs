// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pinned FontSymbol dimensions, independent of calibrated head size.
//! Resource lookup is bounded; general native font layout remains unfinished.
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TupletShape {
    Three,
    Six,
}
impl TupletShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Three => "TUPLET_THREE",
            Self::Six => "TUPLET_SIX",
        }
    }
    pub(crate) fn inferred(size: usize) -> Self {
        if (size as f64 / 3.0).round_ties_even() as i32 == 1 {
            Self::Three
        } else {
            Self::Six
        }
    }
}
type Dimensions = BTreeMap<String, Vec<[i64; 4]>>;
static DIMENSIONS: OnceLock<Result<Dimensions, String>> = OnceLock::new();
fn dimensions() -> Result<&'static Dimensions, String> {
    DIMENSIONS
        .get_or_init(|| {
            let bytes = include_bytes!("../assets/tuplet-font-dimensions/dimensions.json");
            if format!("{:x}", Sha256::digest(bytes))
                != "8b2efa4a3534ecef24d298a30e128a64b9baa4aefacdba93c9e652386d333805"
            {
                return Err("music_tuplet_font_resource_integrity_required".into());
            }
            let rows: Dimensions =
                serde_json::from_slice(bytes).map_err(|_| "music_tuplet_font_resource_required")?;
            if rows.len() != 6
                || rows.values().any(|rows| {
                    rows.len() != 128 || rows.iter().flatten().any(|n| *n < 0 || *n > 1024)
                })
            {
                return Err("music_tuplet_font_resource_required".into());
            }
            Ok(rows)
        })
        .as_ref()
        .map_err(Clone::clone)
}
/// An immutable font resource, supplied explicitly until actual SheetStub
/// family ownership is ported; never infer it from a musical report.
pub struct TupletFontSymbols {
    family: String,
    interline: usize,
    dimensions: [i64; 4],
}
impl TupletFontSymbols {
    pub fn load(family: &str, interline: usize) -> Result<Self, String> {
        let rows = dimensions()?
            .get(family)
            .ok_or("music_tuplet_font_family_pending")?;
        let row = interline
            .checked_sub(1)
            .and_then(|i| rows.get(i))
            .ok_or("music_tuplet_font_interline_pending")?;
        Ok(Self {
            family: family.into(),
            interline,
            dimensions: *row,
        })
    }
    pub fn family(&self) -> &str {
        &self.family
    }
    pub fn interline(&self) -> usize {
        self.interline
    }
    pub fn dimension(&self, shape: TupletShape) -> [i64; 2] {
        match shape {
            TupletShape::Three => [self.dimensions[0], self.dimensions[1]],
            TupletShape::Six => [self.dimensions[2], self.dimensions[3]],
        }
    }
}
