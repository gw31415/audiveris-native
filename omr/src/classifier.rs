//! Glyph classifier: Audiveris 5.11 BasicClassifier (110 inputs, 149 hidden,
//! 149 sigmoid outputs) applied to MixGlyphDescriptor features.

use std::sync::OnceLock;

use crate::binary::Component;
use crate::features::{self, Geometry, Orientation};
use crate::labels::OUTPUT_LABELS;

#[derive(serde::Deserialize)]
struct ModelFile {
    means: Vec<f64>,
    stds: Vec<f64>,
    hidden_weights: Vec<Vec<f64>>,
    output_weights: Vec<Vec<f64>>,
}

static MODEL: OnceLock<ModelFile> = OnceLock::new();

fn model() -> &'static ModelFile {
    MODEL.get_or_init(|| serde_json::from_slice(include_bytes!("../assets/classifier.json")).expect("classifier model"))
}

fn forward(input: &[f64], weights: &[Vec<f64>]) -> Vec<f64> {
    weights
        .iter()
        .map(|row| {
            let mut sum = 0.0;
            for i in (0..input.len()).rev() {
                sum += row[i + 1] * input[i];
            }
            sum += row[0];
            1.0 / (1.0 + (-sum).exp())
        })
        .collect()
}

/// Ranked (shape, grade) evaluations of a glyph, best first.
pub fn classify(glyph: &Component, interline: i32) -> Vec<(&'static str, f64)> {
    let b = glyph.bounds;
    let geo = Geometry {
        width: b.w as usize,
        height: b.h as usize,
        left: b.x,
        top: b.y,
        interline: interline as u32,
        orientation: Orientation::Vertical,
    };
    let Ok(raw) = features::from_bitmap(geo, &glyph.mask()) else {
        return Vec::new();
    };
    let m = model();
    let norm: Vec<f64> = raw.iter().zip(&m.means).zip(&m.stds).map(|((v, mu), s)| (v - mu) / s).collect();
    let hidden = forward(&norm, &m.hidden_weights);
    let scores = forward(&hidden, &m.output_weights);
    let mut out: Vec<(&'static str, f64)> = OUTPUT_LABELS.iter().copied().zip(scores).collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    out
}
