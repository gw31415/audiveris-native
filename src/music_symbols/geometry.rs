// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{beam_dispatch, grid_ir::SystemGeometry, stem_image_owned::OwnedItems};
use audiveris_staff_removal::GlyphPixels;
use std::sync::Arc;
pub(super) struct SystemGlyphs<'a> {
    pub(super) system: Arc<SystemGeometry>,
    pub(super) glyphs: Vec<&'a GlyphPixels>,
}
pub(super) fn dispatch<'a>(
    owner: &OwnedItems,
    glyphs: &'a [GlyphPixels],
) -> Result<Vec<SystemGlyphs<'a>>, String> {
    let mut grouped = owner
        .systems
        .iter()
        .map(|s| SystemGlyphs {
            system: s.clone(),
            glyphs: Vec::new(),
        })
        .collect::<Vec<_>>();
    for glyph in glyphs {
        let point = beam_dispatch::centroid(glyph)?.map(|v| v as f64);
        for group in &mut grouped {
            if group.system.area.contains(point)? {
                group.glyphs.push(glyph);
            }
        }
    }
    Ok(grouped)
}
pub(super) fn closest(
    owner: &OwnedItems,
    system: &SystemGeometry,
    point: [f64; 2],
) -> Result<Option<Arc<crate::grid_ir::StaffGeometry>>, String> {
    let areas = system
        .staffs
        .iter()
        .map(|staff| {
            let index = owner
                .staffs
                .iter()
                .position(|held| Arc::ptr_eq(held, staff))
                .ok_or("music_symbol_actual_System_Staff_required")?;
            owner
                .staff_areas
                .get(index)
                .cloned()
                .ok_or_else(|| "music_symbol_actual_Staff_area_required".to_string())
        })
        .collect::<Result<Vec<_>, String>>()?;
    crate::grid_staff_neighborhood::closest(&system.staffs, &areas, system.id, point)
}
