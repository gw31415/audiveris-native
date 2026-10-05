// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    beam_dispatch,
    stem_parts_alignment::{Parameters as Alignment, Part},
    stem_parts_filter::{self, Kind, Linker, Parameters},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn observe(f: &Value) -> Value {
    let mut cases = Vec::new();
    for (index, c) in f["cases"].as_array().unwrap().iter().enumerate() {
        let glyphs = c["glyphs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| {
                (
                    g["id"].as_u64().unwrap() as usize,
                    beam_dispatch::glyph_from_json(&g["glyph"]).unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let part = |id: &Value| {
            let id = id.as_u64().unwrap() as usize;
            Part {
                id,
                glyph: &glyphs[&id],
            }
        };
        let linkers = c["linkers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| {
                let kind = match l["kind"].as_str().unwrap() {
                    "CLinker" => Kind::HeadCorner,
                    "VLinker" => Kind::BeamHalf,
                    "BLinker" => Kind::Beam,
                    _ => panic!("unsupported linker"),
                };
                assert_eq!(l["half"].as_bool().unwrap(), kind != Kind::Beam);
                let l = Linker {
                    id: l["id"].as_u64().unwrap() as usize,
                    kind,
                    stump: (!l["stump_id"].is_null()).then(|| part(&l["stump_id"])),
                    reference: serde_json::from_value(l["reference"].clone()).unwrap(),
                    beam_height: l["beam_height"].as_f64(),
                };
                (l.id, l)
            })
            .collect::<BTreeMap<_, _>>();
        let parts = c["part_order"]
            .as_array()
            .unwrap()
            .iter()
            .map(part)
            .collect::<Vec<_>>();
        let ls = c["linker_order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| linkers[&(id.as_u64().unwrap() as usize)])
            .collect::<Vec<_>>();
        let p = Parameters {
            alignment: Alignment::new(
                c["direction"].as_i64().unwrap() as i8,
                c["max_alignment_dx"].as_f64().unwrap(),
                c["max_alignment_dy"].as_f64().unwrap(),
                c["slope"].as_f64().unwrap(),
            )
            .unwrap(),
            y_range: serde_json::from_value(c["y_range"].clone()).unwrap(),
            start_is_beam: c["start_kind"] == "VLinker",
            head_box: serde_json::from_value(c["head_box"].clone()).unwrap(),
        };
        let before = glyphs
            .values()
            .map(beam_dispatch::glyph_json)
            .collect::<Vec<_>>();
        let result = stem_parts_filter::filter(
            &parts,
            (!c["start_stump_id"].is_null()).then(|| part(&c["start_stump_id"])),
            &ls,
            &p,
        )
        .unwrap_or_else(|e| panic!("original case{index}: {e}"));
        assert_eq!(
            before,
            glyphs
                .values()
                .map(beam_dispatch::glyph_json)
                .collect::<Vec<_>>()
        );
        let mut row = c.clone();
        let r = serde_json::to_value(result).unwrap();
        for field in [
            "retained_part_order",
            "retained_linker_order",
            "last_head_y",
            "items",
        ] {
            row[field] = r[field].clone();
        }
        cases.push(row);
    }
    json!({"cases":cases})
}
pub fn compare(f: &Value, n: &Value) -> f64 {
    let mut max = 0f64;
    for (index, (c, r)) in f["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["cases"].as_array().unwrap())
        .enumerate()
    {
        for field in [
            "retained_part_order",
            "retained_linker_order",
            "last_head_y",
        ] {
            assert_eq!(c[field], r[field], "case{index} {field}");
        }
        let a = c["items"].as_array().unwrap();
        let b = r["items"].as_array().unwrap();
        assert_eq!(a.len(), b.len(), "case{index} item count");
        for (j, (a, b)) in a.iter().zip(b).enumerate() {
            for field in ["kind", "contrib", "glyph_id", "linker_id"] {
                assert_eq!(a[field], b[field], "case{index} item{j} {field}");
            }
            for k in 0..4 {
                let d = (a["line"][k].as_f64().unwrap() - b["line"][k].as_f64().unwrap()).abs();
                max = max.max(d);
                assert!(
                    d <= 1e-10,
                    "case{index} item{j} line{k}: {} != {}",
                    a["line"][k],
                    b["line"][k]
                );
            }
        }
    }
    max
}
