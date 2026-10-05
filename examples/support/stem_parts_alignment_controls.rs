// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch,
    stem_parts_alignment::{self, Parameters, Part},
};
use serde_json::{Value, json};
pub fn observe(f: &Value) -> Value {
    let mut cases = Vec::new();
    for c in f["cases"].as_array().unwrap() {
        let stump = beam_dispatch::glyph_from_json(&c["stump"]).unwrap();
        let mut glyphs = vec![stump];
        for p in c["parts"].as_array().unwrap() {
            assert_eq!(p["id"].as_u64().unwrap() as usize, glyphs.len());
            glyphs.push(beam_dispatch::glyph_from_json(&p["glyph"]).unwrap());
        }
        let parts = c["input_order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| {
                let id = id.as_u64().unwrap() as usize;
                Part {
                    id: id + 1,
                    glyph: &glyphs[id],
                }
            })
            .collect::<Vec<_>>();
        let before = glyphs
            .iter()
            .map(beam_dispatch::glyph_json)
            .collect::<Vec<_>>();
        let parameters = Parameters::new(
            c["direction"].as_i64().unwrap() as i8,
            c["max_alignment_dx"].as_f64().unwrap(),
            c["max_alignment_dy"].as_f64().unwrap(),
            c["slope"].as_f64().unwrap(),
        )
        .unwrap();
        let result = stem_parts_alignment::filter(
            &parts,
            Some(Part {
                id: 1,
                glyph: &glyphs[0],
            }),
            &parameters,
        )
        .unwrap();
        assert_eq!(
            before,
            glyphs
                .iter()
                .map(beam_dispatch::glyph_json)
                .collect::<Vec<_>>()
        );
        let mut row = c.clone();
        row["retained_order"] = json!(
            result
                .retained_order
                .iter()
                .map(|id| id - 1)
                .collect::<Vec<_>>()
        );
        row["removed_order"] = json!(
            result
                .removed_order
                .iter()
                .map(|id| id - 1)
                .collect::<Vec<_>>()
        );
        cases.push(row);
    }
    json!({"cases":cases})
}
