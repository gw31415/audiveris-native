use audiveris_classifier::Model;
use audiveris_glyph_features::{Geometry, Orientation, Run, from_bitmap, from_runs};
use serde_json::{Value, json};
const ABS: f64 = 1e-12;
const REL: f64 = 1e-10;
#[test]
fn java_glyph_features_and_classifier_chain() {
    let golden: Value =
        serde_json::from_slice(include_bytes!("fixtures/java-glyph-golden.json")).unwrap();
    let model = Model::from_json(include_bytes!(
        "../../audiveris-classifier/models/audiveris-5.11.0.json"
    ))
    .unwrap();
    assert_eq!(golden["input_labels"], json!(model.input_labels()));
    let mut max = [0.0_f64; 5];
    let mut count = [0usize; 5];
    let cases = golden["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 96);
    for case in cases {
        let i = &case["input"];
        let num = |s: &str| i[s].as_u64().unwrap() as usize;
        let g = Geometry {
            width: num("width"),
            height: num("height"),
            left: num("left") as i32,
            top: num("top") as i32,
            interline: num("interline") as u32,
            orientation: if i["orientation"] == "HORIZONTAL" {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            },
        };
        let runs: Vec<Run> = i["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| Run {
                position: a[0].as_u64().unwrap() as usize,
                start: a[1].as_u64().unwrap() as usize,
                length: a[2].as_u64().unwrap() as usize,
            })
            .collect();
        let pixels: Vec<bool> = i["bitmap_rows"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|r| r.as_str().unwrap().bytes().map(|c| c == b'#'))
            .collect();
        let features = from_runs(g, &runs).unwrap();
        assert_eq!(features, from_bitmap(g, &pixels).unwrap());
        let evaluated = model.evaluate(&features).unwrap();
        let java_raw: Vec<f64> = serde_json::from_value(case["raw_features"].clone()).unwrap();
        let isolated = model.evaluate(&java_raw).unwrap();
        for (stage, (name, actual)) in [
            ("raw_features", features.as_slice()),
            ("normalized", evaluated.normalized.as_slice()),
            ("hidden", evaluated.hidden.as_slice()),
            ("scores", evaluated.scores.as_slice()),
            ("scores", isolated.scores.as_slice()),
        ]
        .iter()
        .enumerate()
        {
            let expected: Vec<f64> = serde_json::from_value(case[*name].clone()).unwrap();
            assert_eq!(actual.len(), expected.len());
            for (index, (&a, &b)) in actual.iter().zip(&expected).enumerate() {
                let err = (a - b).abs();
                assert!(
                    a.is_finite() && b.is_finite() && err <= ABS + REL * b.abs(),
                    "{} stage {stage} {name}[{index}]: actual {a:.17e} expected {b:.17e} diff {err:.17e}",
                    case["id"]
                );
                max[stage] = max[stage].max(err);
                count[stage] += 1;
            }
        }
    }
    let report = json!({"cases":cases.len(),"abs_tolerance":ABS,"rel_tolerance":REL,"stage_order":["raw_features","normalized_from_rust_features","hidden_from_rust_features","scores_from_rust_features","scores_from_java_features"],"max_absolute_error":max,"compared_values":count,"bitmap_run_exact_match":true});
    println!("{report}");
    if let Ok(path) = std::env::var("GLYPH_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
}
#[test]
fn invalid_inputs_are_explicit_errors() {
    let g = Geometry {
        width: 5,
        height: 5,
        left: 0,
        top: 0,
        interline: 8,
        orientation: Orientation::Horizontal,
    };
    assert!(from_runs(g, &[]).is_err());
    assert!(from_bitmap(g, &[false; 25]).is_err());
    assert!(from_bitmap(g, &[true; 24]).is_err());
    assert!(
        from_runs(
            g,
            &[
                Run {
                    position: 0,
                    start: 0,
                    length: 2
                },
                Run {
                    position: 0,
                    start: 2,
                    length: 1
                }
            ]
        )
        .is_err()
    );
    assert!(
        from_runs(
            g,
            &[Run {
                position: 0,
                start: 4,
                length: 2
            }]
        )
        .is_err()
    );
    assert!(from_bitmap(Geometry { interline: 0, ..g }, &[true; 25]).is_err());
    assert!(
        from_bitmap(
            Geometry {
                left: i32::MAX,
                ..g
            },
            &[true; 25]
        )
        .is_err()
    );
}
