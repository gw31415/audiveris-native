// SPDX-License-Identifier: AGPL-3.0-or-later
// Experimental native pipeline; residual components are not assembled musical symbols.
use crate::GrayImage;
use audiveris_staff_removal::{BinaryPage, GlyphPixels, Limits, Orientation, Run};
use serde_json::{Value, json};
pub fn extract(
    binary: &GrayImage,
    completion: &Value,
    interline: u32,
    model: Option<&audiveris_classifier::Model>,
) -> Result<(GrayImage, Value), Box<dyn std::error::Error>> {
    let values = completion["staff_line_glyphs"]
        .as_array()
        .ok_or("line completion unavailable")?;
    let glyphs: Vec<_> = values
        .iter()
        .map(|g| GlyphPixels {
            left: g["left"].as_i64().unwrap() as i32,
            top: g["top"].as_i64().unwrap() as i32,
            width: g["width"].as_u64().unwrap() as usize,
            height: g["height"].as_u64().unwrap() as usize,
            orientation: Orientation::Horizontal,
            runs: g["runs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| Run {
                    position: r["position"].as_u64().unwrap() as usize,
                    start: r["start"].as_u64().unwrap() as usize,
                    length: r["length"].as_u64().unwrap() as usize,
                })
                .collect(),
        })
        .collect();
    let erased = audiveris_staff_removal::erase_staff(
        BinaryPage::new(binary.width, binary.height, &binary.pixels)?,
        &glyphs,
        Limits::default(),
    )?;
    let residual = GrayImage::new(binary.width, binary.height, erased.residual)?;
    let components = audiveris_staff_removal::extract_glyphs(
        BinaryPage::new(residual.width, residual.height, &residual.pixels)?,
        Orientation::Vertical,
        (0, 0),
        Limits::default(),
    )?;
    let mut out = vec![];
    for g in components {
        let runs: Vec<_> = g
            .runs
            .iter()
            .map(|r| audiveris_glyph_features::Run {
                position: r.position,
                start: r.start,
                length: r.length,
            })
            .collect();
        let features = audiveris_glyph_features::from_runs(
            audiveris_glyph_features::Geometry {
                width: g.width,
                height: g.height,
                left: g.left,
                top: g.top,
                interline,
                orientation: audiveris_glyph_features::Orientation::Vertical,
            },
            &runs,
        )?;
        let scores = model
            .map(|m| m.evaluate(&features))
            .transpose()?
            .map(|v| v.scores);
        out.push(json!({"left":g.left,"top":g.top,"width":g.width,"height":g.height,"orientation":"VERTICAL","weight":g.runs.iter().map(|r|r.length).sum::<usize>(),"runs":g.runs.iter().map(|r|json!({"position":r.position,"start":r.start,"length":r.length})).collect::<Vec<_>>(),"features":features.as_slice(),"scores":scores}));
    }
    Ok((
        residual,
        json!({"status":"experimental_page_components","grid_complete":false,"removed_pixels":erased.removed_pixels,"component_count":out.len(),"components":out,"class_labels":model.map(|m|m.output_labels()),"scope":"raw four-connected residual components; no musical grouping, contextual classification, rhythm or MusicXML"}),
    ))
}
