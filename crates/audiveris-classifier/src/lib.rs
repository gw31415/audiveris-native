// SPDX-License-Identifier: AGPL-3.0-or-later
// Derived from Audiveris NeuralNetwork.java, BasicClassifier.java and PoorManAlgebra.java.
// Copyright © Audiveris 2026. All rights reserved.
// Rust translation and validation added 2026-10-02. See NOTICE and LICENSE.

//! Native inference on an already extracted 110-element Audiveris feature vector.
//! This crate does not extract glyphs or implement the full OMR pipeline.
//! Labels and model provenance must match the pinned Audiveris 5.11.0 schema.

mod labels;

use serde::{Deserialize, Serialize};
use std::fmt;

pub const INPUTS: usize = 110;
pub const HIDDEN: usize = 149;
pub const OUTPUTS: usize = 149;
pub const SOURCE_COMMIT: &str = "9e1e55cd2746037d059345881c53e6a6754bffbd";
pub const SOURCE_ZIP_SHA256: &str =
    "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e";
/// Bounds JSON allocation before decoding an untrusted model.
pub const MAX_MODEL_BYTES: usize = 2_000_000;

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    InvalidModel(String),
    InvalidFeatures(String),
    NumericalOverflow(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "invalid model JSON: {e}"),
            Self::InvalidModel(e) => write!(f, "invalid model: {e}"),
            Self::InvalidFeatures(e) => write!(f, "invalid features: {e}"),
            Self::NumericalOverflow(stage) => write!(f, "non-finite arithmetic at {stage}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelFile {
    source_commit: String,
    source_zip_sha256: String,
    format: String,
    input_labels: Vec<String>,
    output_labels: Vec<String>,
    means: Vec<f64>,
    stds: Vec<f64>,
    hidden_weights: Vec<Vec<f64>>,
    output_weights: Vec<Vec<f64>>,
}

/// Validated, immutable model. The provenance fields identify the expected
/// source; they are not a cryptographic authentication of supplied weight data.
pub struct Model {
    file: ModelFile,
}

/// Values are in the model's original order; scores are independent sigmoids,
/// not a softmax probability distribution. Exposed intermediate arrays support
/// differential testing of ports.
#[derive(Debug, Serialize)]
pub struct Evaluation {
    pub normalized: Vec<f64>,
    pub hidden: Vec<f64>,
    pub scores: Vec<f64>,
}

impl Model {
    /// Decode and validate the JSON exported from the pinned XML model.
    /// Non-finite values, non-positive standard deviations, incorrect labels,
    /// incorrect dimensions and wrong provenance are rejected.
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_MODEL_BYTES {
            return Err(Error::InvalidModel("JSON exceeds size limit".into()));
        }
        let file: ModelFile = serde_json::from_slice(bytes).map_err(Error::Json)?;
        Self::validate(file)
    }

    fn validate(file: ModelFile) -> Result<Self, Error> {
        if file.source_commit != SOURCE_COMMIT || file.source_zip_sha256 != SOURCE_ZIP_SHA256 {
            return Err(Error::InvalidModel(
                "provenance does not match 5.11.0".into(),
            ));
        }
        if file.format.is_empty() {
            return Err(Error::InvalidModel("missing format description".into()));
        }
        if file.input_labels != labels::INPUT_LABELS {
            return Err(Error::InvalidModel(
                "input label order does not match 5.11.0".into(),
            ));
        }
        if file.output_labels != labels::OUTPUT_LABELS {
            return Err(Error::InvalidModel(
                "output label order does not match 5.11.0".into(),
            ));
        }
        validate_vector("means", &file.means, INPUTS)?;
        validate_vector("stds", &file.stds, INPUTS)?;
        if file.stds.iter().any(|v| *v <= 0.0) {
            return Err(Error::InvalidModel(
                "standard deviations must be positive".into(),
            ));
        }
        validate_matrix("hidden_weights", &file.hidden_weights, HIDDEN, INPUTS + 1)?;
        validate_matrix("output_weights", &file.output_weights, OUTPUTS, HIDDEN + 1)?;
        Ok(Self { file })
    }

    pub fn input_labels(&self) -> &[String] {
        &self.file.input_labels
    }

    pub fn output_labels(&self) -> &[String] {
        &self.file.output_labels
    }

    /// Evaluate raw features using f64 subtraction/division and the original
    /// reverse-index accumulation order. No BLAS, FMA, JNI or Java subprocess.
    /// Inputs/intermediate sums outside the finite f64 domain return an error.
    /// A finite sigmoid argument whose exponential overflows still saturates
    /// to 0, exactly as in the original Java formula.
    pub fn evaluate(&self, raw: &[f64]) -> Result<Evaluation, Error> {
        if raw.len() != INPUTS {
            return Err(Error::InvalidFeatures(format!(
                "expected {INPUTS} values, got {}",
                raw.len()
            )));
        }
        if raw.iter().any(|v| !v.is_finite()) {
            return Err(Error::InvalidFeatures("all values must be finite".into()));
        }
        let normalized: Vec<f64> = raw
            .iter()
            .zip(&self.file.means)
            .zip(&self.file.stds)
            .map(|((&value, &mean), &std)| (value - mean) / std)
            .collect();
        if normalized.iter().any(|v| !v.is_finite()) {
            return Err(Error::NumericalOverflow("normalization"));
        }
        let hidden = forward(&normalized, &self.file.hidden_weights, "hidden layer")?;
        let scores = forward(&hidden, &self.file.output_weights, "output layer")?;
        Ok(Evaluation {
            normalized,
            hidden,
            scores,
        })
    }
}

fn validate_vector(name: &str, data: &[f64], size: usize) -> Result<(), Error> {
    if data.len() != size {
        return Err(Error::InvalidModel(format!(
            "{name}: expected {size} values, got {}",
            data.len()
        )));
    }
    if data.iter().any(|v| !v.is_finite()) {
        return Err(Error::InvalidModel(format!("{name}: non-finite value")));
    }
    Ok(())
}

fn validate_matrix(
    name: &str,
    data: &[Vec<f64>],
    rows: usize,
    columns: usize,
) -> Result<(), Error> {
    if data.len() != rows {
        return Err(Error::InvalidModel(format!(
            "{name}: expected {rows} rows, got {}",
            data.len()
        )));
    }
    for row in data {
        validate_vector(name, row, columns)?;
    }
    Ok(())
}

fn forward(input: &[f64], weights: &[Vec<f64>], stage: &'static str) -> Result<Vec<f64>, Error> {
    let mut output = vec![0.0; weights.len()];
    for index in (0..weights.len()).rev() {
        let row = &weights[index];
        let mut sum = 0.0;
        for i in (0..input.len()).rev() {
            // Preserve Java's multiply then add, with bias added last.
            sum += row[i + 1] * input[i];
        }
        sum += row[0];
        if !sum.is_finite() {
            return Err(Error::NumericalOverflow(stage));
        }
        output[index] = 1.0 / (1.0 + (-sum).exp());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    const MODEL: &[u8] = include_bytes!("../models/audiveris-5.11.0.json");

    #[test]
    fn rejects_nonfinite_model_values_even_before_json_encoding() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for field in 0..4 {
                let mut file: ModelFile = serde_json::from_slice(MODEL).unwrap();
                match field {
                    0 => file.means[0] = value,
                    1 => file.stds[0] = value,
                    2 => file.hidden_weights[0][0] = value,
                    _ => file.output_weights[0][0] = value,
                }
                assert!(matches!(Model::validate(file), Err(Error::InvalidModel(_))));
            }
        }
    }
}
