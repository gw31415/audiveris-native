// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    Run,
    sections::{Orientation, Section, SectionId},
    serif_glyphs,
    serif_peaks::Peak,
};
use serde_json::Value;
#[test]
fn original_filament_acceptance_and_exact_glyph_pixels() {
    let v: Value =
        serde_json::from_str(include_str!("../fixtures/official-serif-filaments.json")).unwrap();
    assert_eq!(v["native_outputs_used"], false);
    let mut accepted = 0;
    for (i, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let input = &c["input"];
        let b = &input["core"];
        let peak = Peak {
            start: b[0].as_i64().unwrap() as i32,
            stop: (b[0].as_i64().unwrap() + b[2].as_i64().unwrap() - 1) as i32,
            top: b[1].as_i64().unwrap() as i32,
            bottom: (b[1].as_i64().unwrap() + b[3].as_i64().unwrap() - 1) as i32,
            impacts: [1.; 6],
            grade: 0.8,
        };
        let sections = input["sections"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(id, s)| {
                let runs = s["runs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| Run {
                        start: r[0].as_u64().unwrap() as usize,
                        length: r[1].as_u64().unwrap() as usize,
                    })
                    .collect::<Vec<_>>();
                Section {
                    id: SectionId(id as u32 + 1),
                    orientation: if s["vertical"].as_bool().unwrap() {
                        Orientation::Vertical
                    } else {
                        Orientation::Horizontal
                    },
                    first_pos: s["first_pos"].as_u64().unwrap() as usize,
                    weight: runs.iter().map(|r| r.length).sum(),
                    runs,
                }
            })
            .collect::<Vec<_>>();
        let actual = serif_glyphs::build(
            &peak,
            &sections,
            input["interline"].as_u64().unwrap_or(16) as usize,
        )
        .unwrap();
        assert_eq!(
            actual.is_some(),
            c["accepted"].as_bool().unwrap(),
            "case {i}"
        );
        if let Some(g) = actual {
            accepted += 1;
            assert_eq!(
                serde_json::json!([g.left, g.top, g.width as i32, g.height as i32]),
                c["bounds"],
                "case {i}"
            );
            assert_eq!(c["orientation"], "VERTICAL");
            let mut pixels = Vec::new();
            for r in &g.runs {
                for y in r.start..r.start + r.length {
                    pixels.push([g.left + r.position as i32, g.top + y as i32]);
                }
            }
            pixels.sort_by_key(|p| (p[1], p[0]));
            assert_eq!(serde_json::json!(pixels), c["pixels"], "case {i}");
        }
    }
    assert_eq!(accepted, 32);
}
