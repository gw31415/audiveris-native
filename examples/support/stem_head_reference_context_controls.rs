// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    head_stem_relation::{self, HeadSide, VerticalSide},
    stem_head_reference_context::{self, Head},
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
pub fn observe(o: &Value, page: &Value) -> Value {
    let mut codec = Vec::new();
    for c in o["XML_adapter_controls"].as_array().unwrap() {
        let raw = c["raw"].as_f64();
        let xml = stem_head_reference_context::to_xml(raw).unwrap();
        assert_eq!(json!(xml), c["xml"]);
        let restored = stem_head_reference_context::restored_dx([raw, None]).unwrap()[0];
        assert_eq!(json!(restored), c["restored"]);
        let bits = restored.map(|v| format!("{:x}", v.to_bits()));
        assert_eq!(json!(bits), c["restored_bits"]);
        codec.push(json!({"xml":xml,"restored":restored,"restored_bits":bits}));
    }
    let scale = &page["head_registration"]["head_seed_scale"];
    let mut dx = Vec::new();
    for c in o["loaded_HEADS_scale_before"].as_array().unwrap() {
        let native = scale
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["shape"] == c["shape"] && n["side"] == c["side"])
            .unwrap();
        let restored =
            stem_head_reference_context::restored_dx([native["dx"].as_f64(), None]).unwrap()[0];
        assert_eq!(json!(restored), c["dx"]);
        dx.push(json!({"shape":c["shape"],"side":c["side"],"dx":restored}));
    }
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let mut refs = Vec::new();
    let mut transient_differences = 0;
    for (i, c) in o["head_references"].as_array().unwrap().iter().enumerate() {
        let sys = page["head_registration"]["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["system_id"] == c["system"])
            .unwrap();
        let heads = sys["heads"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| {
                h["removed"] != true && h["bounds"] == c["bounds"] && h["shape"] == c["shape"]
            })
            .collect::<Vec<_>>();
        assert_eq!(heads.len(), 1, "ref{i} native physical head match");
        let bounds = serde_json::from_value(heads[0]["bounds"].clone()).unwrap();
        let shape = c["shape"].as_str().unwrap();
        let raw = std::array::from_fn(|j| {
            scale
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["shape"] == shape && n["side"] == if j == 0 { "LEFT" } else { "RIGHT" })
                .unwrap()["dx"]
                .as_f64()
        });
        let side = if c["side"] == "LEFT" {
            HeadSide::Left
        } else {
            HeadSide::Right
        };
        let vertical = if c["vertical"] == "TOP" {
            VerticalSide::Top
        } else {
            VerticalSide::Bottom
        };
        let h = Head {
            bank: &bank,
            point_size: 51,
            shape,
            bounds,
            side,
            vertical,
        };
        let point = stem_head_reference_context::reference(&h, raw).unwrap();
        assert_eq!(json!(point), c["point"], "ref{i}");
        let transient =
            head_stem_relation::stem_reference_point(&bank, 51, shape, bounds, side, vertical, raw)
                .unwrap();
        if transient != point {
            transient_differences += 1;
        }
        refs.push(json!({"point":point,"transient_point":transient}));
    }
    json!({"XML_adapter_controls":codec,"loaded_context_DX":dx,"head_references":refs,"transient_reference_differences":transient_differences,"explicit_XML_context":true,"automatic_raster_target_pipeline_complete":false,"page_cache_SIG_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false})
}
