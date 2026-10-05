// SPDX-License-Identifier: AGPL-3.0-or-later
use serde_json::Value;
use std::collections::BTreeMap;
fn key(v: &Value) -> String {
    serde_json::to_string(&serde_json::json!([
        v["head_bounds"],
        v["shape"],
        v["side"],
        v["vertical"]
    ]))
    .unwrap()
}
fn beam(v: &Value) -> Value {
    if v.is_null() {
        Value::Null
    } else {
        serde_json::json!({"median":v["median"],"height":v["height"]})
    }
}
pub fn compare(o: &Value, n: &Value) {
    assert_eq!(o["native_outputs_used"], false);
    assert_eq!(
        o["original_phase"],
        "inspectStems_before_linkStems_and_finalizeStems"
    );
    assert_eq!(o["corners"], n["corner_count"]);
    assert_eq!(n["corner_count"], 1408);
    assert_eq!(
        o["selected_sibling_member_occurrences"],
        n["selected_sibling_member_occurrences"]
    );
    assert_eq!(n["selected_sibling_member_occurrences"], 339);
    let mut count = 0;
    for (a, b) in o["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(n["systems"].as_array().unwrap())
    {
        assert_eq!(a["system"], b["system"]);
        assert_eq!(a["max_beam_side_dx"], b["max_beam_side_dx"]);
        let expected = a["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| (key(c), c))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(expected.len(), b["cases"].as_array().unwrap().len());
        for c in b["cases"].as_array().unwrap() {
            let e = expected[&key(&c["head"])];
            for f in ["intersection", "void_side_veto"] {
                assert_eq!(e[f], c[f], "{} {f}", key(&c["head"]));
            }
            assert_eq!(beam(&e["target_beam"]), beam(&c["target_beam"]));
            assert_eq!(
                e["selected_sibling_members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(beam)
                    .collect::<Vec<_>>(),
                c["selected_sibling_members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(beam)
                    .collect::<Vec<_>>(),
                "{}",
                key(&c["head"])
            );
            count += 1;
        }
    }
    assert_eq!(count, 1408);
    assert_eq!(o["controls"].as_array().unwrap().len(), 13);
    assert_eq!(
        n["head_origin_beam_sibling_member_selection_complete"],
        true
    );
    for f in [
        "beam_sibling_linker_selection_complete",
        "final_stem_glyph_selection_complete",
        "automatic_raster_pipeline_complete",
        "page_cache_sig_complete",
        "epilog_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(n[f], false);
    }
}
