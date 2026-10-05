// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_classifier::{Error, MAX_MODEL_BYTES, Model};
use serde_json::{Value, json};

const DATA: &[u8] = include_bytes!("../models/audiveris-5.11.0.json");

#[test]
fn rejects_malformed_or_nonfinite_json() {
    for input in [b"null".as_slice(), b"{}", b"{", b"NaN", b"[Infinity]"] {
        assert!(Model::from_json(input).is_err());
    }
    let text = std::str::from_utf8(DATA)
        .unwrap()
        .replacen("\"means\":[", "\"means\":[1e999,", 1);
    assert!(matches!(
        Model::from_json(text.as_bytes()),
        Err(Error::Json(_))
    ));
    assert!(matches!(
        Model::from_json(&vec![b' '; MAX_MODEL_BYTES + 1]),
        Err(Error::InvalidModel(_))
    ));
}

#[test]
fn rejects_bad_shapes_labels_norms_and_provenance() {
    let original: Value = serde_json::from_slice(DATA).unwrap();
    let mut cases = Vec::new();
    for field in [
        "means",
        "stds",
        "hidden_weights",
        "output_weights",
        "input_labels",
        "output_labels",
    ] {
        let mut v = original.clone();
        v[field].as_array_mut().unwrap().pop();
        cases.push(v);
    }
    for field in ["hidden_weights", "output_weights"] {
        let mut v = original.clone();
        v[field][0].as_array_mut().unwrap().pop();
        cases.push(v);
        let mut v = original.clone();
        v[field][0].as_array_mut().unwrap().push(json!(0));
        cases.push(v);
    }
    for field in ["input_labels", "output_labels"] {
        let mut v = original.clone();
        v[field].as_array_mut().unwrap().swap(0, 1);
        cases.push(v);
        let mut v = original.clone();
        v[field][0] = v[field][1].clone();
        cases.push(v);
    }
    for bad in [0.0, -0.0, -1.0] {
        let mut v = original.clone();
        v["stds"][0] = json!(bad);
        cases.push(v);
    }
    for field in ["source_commit", "source_zip_sha256"] {
        let mut v = original.clone();
        v[field] = json!("wrong");
        cases.push(v);
    }
    let mut unknown = original.clone();
    unknown["unknown_field"] = json!(true);
    cases.push(unknown);
    for (index, bad) in cases.iter().enumerate() {
        assert!(
            Model::from_json(&serde_json::to_vec(bad).unwrap()).is_err(),
            "bad model {index} accepted"
        );
    }
}

#[test]
fn rejects_invalid_feature_vectors_and_arithmetic_overflow() {
    let model = Model::from_json(DATA).unwrap();
    for count in [0, 109, 111] {
        assert!(matches!(
            model.evaluate(&vec![0.0; count]),
            Err(Error::InvalidFeatures(_))
        ));
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut features = vec![0.0; 110];
        features[55] = bad;
        assert!(matches!(
            model.evaluate(&features),
            Err(Error::InvalidFeatures(_))
        ));
    }
    assert!(matches!(
        model.evaluate(&[f64::MAX; 110]),
        Err(Error::NumericalOverflow(_))
    ));
}

#[test]
fn cli_emits_ordered_scores_and_rejects_bad_input() {
    let binary = env!("CARGO_BIN_EXE_audiveris-classify");
    let output = std::process::Command::new(binary)
        .args([
            "models/audiveris-5.11.0.json",
            "tests/fixtures/mean-features.json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout).unwrap();
    let model = Model::from_json(DATA).unwrap();
    let raw: Vec<f64> =
        serde_json::from_slice(include_bytes!("fixtures/mean-features.json")).unwrap();
    assert_eq!(parsed["class_labels"], json!(model.output_labels()));
    assert_eq!(
        parsed["scores"],
        json!(model.evaluate(&raw).unwrap().scores)
    );
    let bad = std::process::Command::new(binary)
        .args(["models/audiveris-5.11.0.json", "Cargo.toml"])
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2));
    assert!(bad.stdout.is_empty());
}
