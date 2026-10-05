// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch,
    stem_parts_lookup::{self, Parameters},
    vertical_sticks::SeedSection,
};
use serde_json::{Value, json};

/// Controls decode observed original geometry; native Head/Beam area derivation
/// and the page CLI are intentionally outside this observer's contract.
pub fn observe(f: &Value) -> Value {
    let sections = |name: &str| {
        f[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let section: SeedSection = serde_json::from_value(s.clone()).unwrap();
                assert_eq!(
                    json!(stem_parts_lookup::section_centroid(&section).unwrap()),
                    s["centroid"]
                );
                section
            })
            .collect::<Vec<_>>()
    };
    let v = sections("vertical_sections");
    let h = sections("horizontal_sections");
    let mut cases = Vec::new();
    for (index, c) in f["cases"].as_array().unwrap().iter().enumerate() {
        let mut points = Vec::new();
        let mut closed = false;
        for segment in c["area_path"].as_array().unwrap() {
            assert!(!closed);
            match segment["type"].as_i64().unwrap() {
                0 => {
                    assert!(points.is_empty());
                    points.push(serde_json::from_value(segment["coordinates"].clone()).unwrap());
                }
                1 => {
                    assert!(!points.is_empty());
                    points.push(serde_json::from_value(segment["coordinates"].clone()).unwrap());
                }
                4 => {
                    closed = true;
                }
                _ => panic!("uncertified curve or multipath lookup area"),
            }
        }
        let stump =
            (!c["stump"].is_null()).then(|| beam_dispatch::glyph_from_json(&c["stump"]).unwrap());
        let p = Parameters {
            direction: c["direction"].as_i64().unwrap() as i8,
            slope: c["slope"].as_f64().unwrap(),
            max_stem_thickness: c["max_stem_thickness"].as_i64().unwrap(),
            max_line_section_dx: c["max_line_section_dx"].as_f64().unwrap(),
            last_head_y: c["last_head_y"].as_f64(),
            theoretical_line: serde_json::from_value(c["theoretical_line"].clone()).unwrap(),
        };
        let n = stem_parts_lookup::collect(&v, &h, &points, stump.as_ref(), &p)
            .unwrap_or_else(|e| panic!("case{index}: {e}"));
        assert_eq!(
            json!(n.vertical_ids),
            c["selected_vertical_ids"],
            "case{index} vertical"
        );
        assert_eq!(
            json!(n.horizontal_ids),
            c["selected_horizontal_ids"],
            "case{index} horizontal"
        );
        assert_eq!(
            json!(n.raw_glyphs),
            c["raw_sticks_min_core0"],
            "case{index} glyphs"
        );
        assert_eq!(
            c["raw_sticks_min_core0"], c["raw_sticks_min_core1"],
            "case{index} certified positive-run minCore equivalence"
        );
        cases.push(json!({"selected_vertical_ids":n.vertical_ids,"selected_horizontal_ids":n.horizontal_ids,"raw_sticks":n.raw_glyphs}));
    }
    json!({"scope":"Native section selection/raw assembly; observed lookup areas supplied, zero skew, not complete page STEMS", "cases":cases})
}
