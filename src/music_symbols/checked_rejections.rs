// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compared ShapeChecker failures; surviving physical factories remain guarded.
use crate::grid_ir::StaffGeometry;
use audiveris_staff_removal::GlyphPixels;

pub(super) fn staff_gap(glyph: &GlyphPixels, staff: &StaffGeometry) -> Result<f64, String> {
    let x = glyph.left as f64 + glyph.width as f64 / 2.;
    let top = staff
        .lines
        .first()
        .ok_or("music_shape_check_actual_Staff_lines_required")?
        .y(x);
    let bottom = staff.lines.last().unwrap().y(x);
    if !top.is_finite() || !bottom.is_finite() || bottom < top {
        return Err("music_shape_check_actual_Staff_lines_required".into());
    }
    let glyph_top = glyph.top as f64;
    let glyph_bottom = glyph_top + glyph.height as f64;
    if glyph_top.max(top) <= glyph_bottom.min(bottom) {
        return Ok(0.);
    }
    Ok((top - glyph_top)
        .abs()
        .min((top - glyph_bottom).abs())
        .min((bottom - glyph_top).abs())
        .min((bottom - glyph_bottom).abs()))
}
pub(super) fn rejection(
    shape: &str,
    glyph: &GlyphPixels,
    staff: &StaffGeometry,
    main_interline: usize,
    estimated_pitch: Option<f64>,
) -> Result<Option<&'static str>, String> {
    match shape {
        "G_CLEF" | "F_CLEF" | "F_CLEF_SMALL" | "C_CLEF" => {
            Ok((!estimated_pitch.is_some_and(|p| p.abs() < 4.)).then_some("NotWithinStaffHeight"))
        }
        "CODA" | "DA_CAPO" | "DAL_SEGNO" => {
            if estimated_pitch.is_none_or(|p| p >= -4.) {
                return Ok(Some("AboveStaff"));
            }
            Ok((staff_gap(glyph, staff)? <= 0.).then_some("AboveStaff"))
        }
        "REPEAT_TWO_BARS" => Ok(match estimated_pitch {
            None => Some("MeasureRepeats:tablature"),
            Some(p) if p.abs() > 1. => Some("MeasureRepeats:pitch"),
            Some(_) => None,
        }),
        "ACCENT" => Ok(
            (staff_gap(glyph, staff)? > (main_interline as f64 * 8.).round_ties_even())
                .then_some("StaffGap"),
        ),
        _ => Ok(None),
    }
}
