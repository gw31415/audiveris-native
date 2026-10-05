// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only actual-raster cue CLEF observer; it does not complete HEADERS.
use audiveris_native::{
    GrayImage, bar_sticks, clef_candidates, clef_registration, clusters, filament_merge, grid_ir,
    header_preparation, line_completion, page_glyphs, scale, sections, staff_projection,
    system_graph, vertical,
};
use serde_json::{Value, json};
fn observe(path: &std::path::Path) -> Value {
    let image = GrayImage::decode(&std::fs::read(path).unwrap()).unwrap();
    let b = vertical::binarize(&image, |_, _, _| {});
    let sc = scale::analyze(&b).result.unwrap();
    let prep = sections::prepare(&b, &sc).unwrap();
    let f = filament_merge::analyze(b.width, &sc, &prep.horizontal);
    let st = clusters::analyze_two_pass(b.width, &sc, &prep.horizontal, &f);
    let p = staff_projection::analyze(&b, &sc, &prep.horizontal, &st);
    let sticks = bar_sticks::analyze(&b, &sc, &prep, &p);
    let slope = f["line_filtering"]["global_slope"].as_f64().unwrap();
    let sys = system_graph::analyze(&b, &sc, &st, &sticks, slope);
    let comp = line_completion::analyze(&b, &sc, &prep, &st, &p, &sticks, &sys, slope);
    let (residual, _) = page_glyphs::extract(&b, &comp, sc.interline.main as u32, None).unwrap();
    let ir = grid_ir::build(&residual, &sc, &prep, &comp, &sys, &sticks).unwrap();
    let headers = header_preparation::from_grid(&ir).unwrap();
    let candidates = clef_candidates::analyze(&residual, &ir, &headers, sc.interline.main).unwrap();
    let registered =
        clef_registration::analyze(&candidates, &ir, &headers, sc.interline.main).unwrap();
    json!({"grid_ir":ir,"headers":headers,"clef_candidates":candidates,"clef_registration":registered})
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: cue_clef_dump INPUT.png NEW_OUTPUT.json"
    );
    assert!(!std::path::Path::new(&args[2]).exists());
    let out = observe(std::path::Path::new(&args[1]));
    std::fs::write(&args[2], serde_json::to_vec_pretty(&out).unwrap()).unwrap();
}
