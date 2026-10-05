// SPDX-License-Identifier: AGPL-3.0-or-later
// Adapter for isolated masks only. Does not segment page images or remove staff lines.
use crate::{GrayImage, sections::GlyphCandidate};
pub use audiveris_glyph_features::{Geometry, Orientation};
fn extract(g: Geometry, pixels: &[u8]) -> Result<[f64; 110], Box<dyn std::error::Error>> {
    if pixels.iter().any(|&p| p != 0 && p != 255) {
        return Err("glyph image must be binary (0/255); explicit binarization required".into());
    }
    let bitmap: Vec<_> = pixels.iter().map(|p| *p == 0).collect();
    Ok(audiveris_glyph_features::from_bitmap(g, &bitmap)?)
}
pub fn from_image(
    image: &GrayImage,
    interline: u32,
    orientation: Orientation,
    left: i32,
    top: i32,
) -> Result<[f64; 110], Box<dyn std::error::Error>> {
    extract(
        Geometry {
            width: image.width,
            height: image.height,
            left,
            top,
            interline,
            orientation,
        },
        &image.pixels,
    )
}
/// Caller must group/choose a candidate. A raw section alone is not necessarily a musical symbol.
pub fn from_candidate(
    candidate: &GlyphCandidate,
    interline: u32,
    orientation: Orientation,
) -> Result<[f64; 110], Box<dyn std::error::Error>> {
    extract(
        Geometry {
            width: candidate.bounds.width,
            height: candidate.bounds.height,
            left: i32::try_from(candidate.bounds.x)?,
            top: i32::try_from(candidate.bounds.y)?,
            interline,
            orientation,
        },
        &candidate.pixels,
    )
}
