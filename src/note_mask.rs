// Copyright © Audiveris 2026; SPDX-License-Identifier: AGPL-3.0-or-later
//! DistancesBuilder staff pass at GRID, before LEDGERS and STEM_SEEDS exist.
use crate::{GrayImage, curves::Curve, template_probes};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// Select actual line-glyph pixels PLUS the full persistent StaffLine curve,
/// including crossings. No component-derived guess, note positions, or probes.
/// Only the ordinary non-tablature GRID subset is represented by this IR.
pub fn from_grid(binary: &GrayImage, grid: &Value) -> Result<(GrayImage, Value), String> {
    let mut mask = GrayImage::new(binary.width, binary.height, vec![255; binary.pixels.len()])?;
    let glyphs = grid["glyphs"].as_array().ok_or("GRID glyphs unavailable")?;
    let mut count_lines = 0usize;
    let mut paint = |x: i64, y: i64| -> Result<(), String> {
        if x < 0 || y < 0 || x as usize >= binary.width || y as usize >= binary.height {
            return Err("staff mask pixel outside page".into());
        }
        mask.pixels[y as usize * binary.width + x as usize] = 0;
        Ok(())
    };
    for staff in grid["staffs"].as_array().ok_or("GRID staffs unavailable")? {
        for line in staff["lines"]
            .as_array()
            .ok_or("GRID staff lines unavailable")?
        {
            let glyph = glyphs
                .iter()
                .find(|g| g["id"] == line["glyph_id"])
                .ok_or("staff glyph reference missing")?;
            let left = glyph["left"].as_i64().ok_or("invalid glyph origin")?;
            let top = glyph["top"].as_i64().ok_or("invalid glyph origin")?;
            let mut weight = 0usize;
            for r in glyph["runs"].as_array().ok_or("invalid glyph runs")? {
                let pos = r["position"].as_i64().ok_or("invalid glyph position")?;
                let start = r["start"].as_i64().ok_or("invalid glyph run")?;
                let len = r["length"].as_i64().ok_or("invalid glyph length")?;
                weight += len as usize;
                for v in start..start + len {
                    match glyph["orientation"].as_str() {
                        Some("HORIZONTAL") => paint(left + v, top + pos)?,
                        Some("VERTICAL") => paint(left + pos, top + v)?,
                        _ => return Err("invalid glyph orientation".into()),
                    }
                }
            }
            let length = glyph["width"].as_f64().ok_or("invalid glyph width")?;
            if length <= 0. {
                return Err("empty line glyph".into());
            }
            let half_line = 0.5 * weight as f64 / length;
            let points = line["points"]
                .as_array()
                .ok_or("invalid StaffLine points")?
                .iter()
                .map(|p| {
                    Ok([
                        p[0].as_f64().ok_or("invalid StaffLine x")?,
                        p[1].as_f64().ok_or("invalid StaffLine y")?,
                    ])
                })
                .collect::<Result<Vec<_>, String>>()?;
            let curve = Curve::interpolate(points).ok_or("invalid StaffLine spline")?;
            let a = curve.points[0];
            let b = *curve.points.last().unwrap();
            for x in a[0].floor() as i64..=b[0].ceil() as i64 {
                let xf = x as f64;
                let y = if xf < a[0] || xf > b[0] {
                    a[1] + (b[1] - a[1]) / (b[0] - a[0]) * (xf - a[0])
                } else {
                    curve.y(xf)
                };
                for y in (y - half_line).round_ties_even() as i64
                    ..=(y + half_line).round_ties_even() as i64
                {
                    paint(x, y)?;
                }
            }
            count_lines += 1;
        }
    }
    let table = template_probes::distances(binary, Some(&mask))?;
    let mut digest = Sha256::new();
    for v in table.values() {
        digest.update((v as i16).to_be_bytes());
    }
    let report = json!({"schema":"audiveris-native.grid-staff-mask.v1","source_stage":"GRID",
        "staff_line_count":count_lines,"masked_pixel_count":mask.pixels.iter().filter(|&&v|v==0).count(),
        "mask_sha256":format!("{:x}",Sha256::digest(&mask.pixels)),"distance_sha256_be_i16":format!("{:x}",digest.finalize()),
        "distance_overflowed":table.overflowed(),"automatic_staff_mask":true,"heads_mask_complete":false,
        "note_recognition_complete":false,"pending":["ledger recognition and mask selection","vertical stem seeds and mask selection","head width calibration and candidate search"]});
    Ok((mask, report))
}
