use audiveris_template_match::*;
use serde_json::Value;
fn ints(v: &Value) -> Vec<i32> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_i64().unwrap() as i32)
        .collect()
}
fn mask(v: &Value) -> Vec<bool> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_bool().unwrap())
        .collect()
}
fn size(v: &Value, n: &str) -> usize {
    v[n].as_u64().unwrap() as usize
}
fn anchor(v: &Value) -> AnchorOffset {
    AnchorOffset {
        x: v["x"].as_f64().unwrap(),
        y: v["y"].as_f64().unwrap(),
        side: match v["side"].as_str().unwrap() {
            "LEFT" => AnchorSide::Left,
            "RIGHT" => AnchorSide::Right,
            _ => AnchorSide::Center,
        },
    }
}
#[test]
fn official_distance_arrays() {
    let root: Value = serde_json::from_str(include_str!("../fixtures/java-golden.json")).unwrap();
    for c in root["distance_cases"].as_array().unwrap() {
        let pixels: Vec<u8> = ints(&c["pixels"]).iter().map(|p| *p as u8).collect();
        let m = mask(&c["mask"]);
        let storage = if c["storage"] == "Short" {
            Storage::JavaShort
        } else {
            Storage::Integer
        };
        let table = if c["target"] == "Fore" {
            compute_masked_foreground(
                size(c, "width"),
                size(c, "height"),
                &pixels,
                &m,
                storage,
                Limits::default(),
            )
            .unwrap()
        } else {
            let whitened: Vec<u8> = pixels
                .iter()
                .zip(&m)
                .map(|(&p, &m)| if m { 255 } else { p })
                .collect();
            let mut t = compute(
                size(c, "width"),
                size(c, "height"),
                &whitened,
                Target::Background,
                storage,
                Limits::default(),
            )
            .unwrap();
            t.neutralize(&m).unwrap();
            t
        };
        assert_eq!(
            table.values().collect::<Vec<_>>(),
            ints(&c["values"]),
            "mode={} storage={} target={}",
            c["mode"],
            c["storage"],
            c["target"]
        );
        assert_eq!(table.normalizer(), 3);
        if c["mode"] == 6 && c["target"] == "Fore" {
            assert_eq!(table.overflowed(), storage == Storage::JavaShort);
        }
    }
}
#[test]
fn official_font_templates_scores_and_anchors() {
    let root: Value = serde_json::from_str(include_str!("../fixtures/java-golden.json")).unwrap();
    let input = &root["score_input"];
    let pixels: Vec<u8> = ints(&input["pixels"]).iter().map(|p| *p as u8).collect();
    let table = compute_masked_foreground(
        size(input, "width"),
        size(input, "height"),
        &pixels,
        &mask(&input["mask"]),
        Storage::JavaShort,
        Limits::default(),
    )
    .unwrap();
    for t in root["templates"].as_array().unwrap() {
        let template = Template::new(
            t["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| KeyPoint {
                    x: p[0].as_i64().unwrap() as i32,
                    y: p[1].as_i64().unwrap() as i32,
                    distance: p[2].as_f64().unwrap(),
                })
                .collect(),
            Limits::default(),
        )
        .unwrap();
        for a in t["anchors"].as_array().unwrap() {
            let r = ints(&a["rounded"]);
            assert_eq!(anchor(a).rounded().unwrap(), (r[0], r[1]));
        }
        for s in t["scores"].as_array().unwrap() {
            let a = if s["anchor"].is_null() {
                None
            } else {
                Some(anchor(
                    t["anchors"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|a| a["name"] == s["anchor"])
                        .unwrap(),
                ))
            };
            let result = template
                .evaluate(
                    &table,
                    s["x"].as_i64().unwrap() as i32,
                    s["y"].as_i64().unwrap() as i32,
                    a,
                    Weights::default(),
                )
                .unwrap();
            assert_eq!(
                result.score,
                s["score"].as_f64().unwrap(),
                "shape={} score={s}",
                t["shape"]
            );
            assert_eq!(
                result.hole_white_ratio,
                s["hole"].as_f64().unwrap(),
                "shape={} score={s}",
                t["shape"]
            );
        }
    }
}
#[test]
fn boundaries_and_rejected_inputs() {
    let l = Limits::default();
    assert!(matches!(
        compute(0, 0, &[], Target::Foreground, Storage::Integer, l),
        Err(Error::Dimensions)
    ));
    assert!(matches!(
        compute(2, 1, &[0], Target::Foreground, Storage::Integer, l),
        Err(Error::Dimensions)
    ));
    assert!(matches!(
        compute(
            2,
            1,
            &[0, 0],
            Target::Foreground,
            Storage::Integer,
            Limits { max_pixels: 1, ..l }
        ),
        Err(Error::Budget)
    ));
    assert!(matches!(
        compute_masked_foreground(1, 1, &[0], &[], Storage::Integer, l),
        Err(Error::MaskLength)
    ));
    let unknown = compute(1, 1, &[255], Target::Foreground, Storage::JavaShort, l).unwrap();
    assert_eq!(unknown.get(0, 0), Some(-1));
    assert_eq!(unknown.get(1, 0), None);
    let points = vec![KeyPoint {
        x: 0,
        y: 0,
        distance: -1.,
    }];
    let template = Template::new(points, l).unwrap();
    let score = template
        .evaluate(&unknown, 0, 0, None, Weights::default())
        .unwrap();
    assert_eq!(score.score, f64::MAX);
    assert_eq!(score.hole_white_ratio, 0.);
    let black = compute(1, 1, &[0], Target::Foreground, Storage::Integer, l).unwrap();
    let empty = Template::new(vec![], l)
        .unwrap()
        .evaluate(&black, 0, 0, None, Weights::default())
        .unwrap();
    assert_eq!(empty.score, f64::MAX);
    assert!(matches!(
        Template::new(
            vec![KeyPoint {
                x: 0,
                y: 0,
                distance: f64::NAN
            }],
            l
        ),
        Err(Error::NonFinite)
    ));
    assert!(matches!(
        template.evaluate(
            &black,
            0,
            0,
            None,
            Weights {
                foreground: -1.,
                ..Weights::default()
            }
        ),
        Err(Error::InvalidWeight)
    ));
    assert!(matches!(
        AnchorOffset {
            x: f64::INFINITY,
            y: 0.,
            side: AnchorSide::Center
        }
        .rounded(),
        Err(Error::NonFinite)
    ));
    assert!(matches!(
        template.evaluate(
            &black,
            i32::MIN,
            0,
            Some(AnchorOffset {
                x: 1.,
                y: 0.,
                side: AnchorSide::Center
            }),
            Weights::default()
        ),
        Err(Error::CoordinateOverflow)
    ));
    for (side, x) in [
        (AnchorSide::Left, -1),
        (AnchorSide::Center, 0),
        (AnchorSide::Right, 0),
    ] {
        assert_eq!(
            AnchorOffset {
                x: 0.,
                y: -0.5,
                side
            }
            .rounded()
            .unwrap(),
            (x, -1)
        );
    }
}
