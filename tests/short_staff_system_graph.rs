// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, bar_sticks, clusters, filament_merge, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::Value;
#[test]
fn native_rasters_match_original_short_staff_peak_graph() {
    let f: Value = serde_json::from_str(include_str!(
        "../fixtures/official-grid/short-staff-system-controls.json"
    ))
    .unwrap();
    assert_eq!(f["native_outputs_used"], false);
    for c in f["cases"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("fixtures/official-grid/{name}.png"));
        let image = GrayImage::decode(&std::fs::read(source).unwrap()).unwrap();
        let binary = vertical::binarize(&image, |_, _, _| {});
        let sc = scale::analyze(&binary).result.unwrap();
        let prep = sections::prepare(&binary, &sc).unwrap();
        let factory = filament_merge::analyze(image.width, &sc, &prep.horizontal);
        let st = clusters::analyze(&sc, &prep.horizontal, &factory);
        let projection = staff_projection::analyze(&binary, &sc, &prep.horizontal, &st);
        let sticks = bar_sticks::analyze(&binary, &sc, &prep, &projection);
        let slope = factory["line_filtering"]["global_slope"].as_f64().unwrap();
        let a = system_graph::analyze(&binary, &sc, &st, &sticks, slope);
        assert_eq!(
            st["staffs"].as_array().unwrap().len(),
            c["projection"].as_array().unwrap().len()
        );
        for (n, o) in st["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["projection"].as_array().unwrap())
        {
            assert_eq!(n["id"], o["staff_id"]);
            assert_eq!(n["left"].as_f64(), o["left"].as_f64());
            assert_eq!(n["right"].as_f64(), o["right"].as_f64());
            for (l, r) in n["lines"]
                .as_array()
                .unwrap()
                .iter()
                .zip(o["lines"].as_array().unwrap())
            {
                assert_eq!(l["points"], r["points"]);
            }
        }
        assert_eq!(a["status"], "initial_systems", "{name}");
        assert_eq!(a["stages"], c["graph"]["stages"], "{name}");
        assert_eq!(a["system_tops"], c["graph"]["system_tops"], "{name}");
        let shorts = st["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["short"] == true)
            .count();
        assert_eq!(shorts, c["short_staff_count"].as_u64().unwrap() as usize);
    }
}
