// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only GRID observer; HEADERS/HEADS/STEMS are deliberately not invoked.
use audiveris_native::{
    GrayImage, bar_sticks, clusters, filament_merge, grid_ir, line_completion, note_mask,
    page_glyphs, scale, sections, staff_projection, system_graph, vertical,
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: cargo run --example grid_cue_dump -- INPUT.png OUTPUT.json"
    );
    let path = &args[1];
    assert!(
        !std::path::Path::new(&args[2]).exists(),
        "output must not exist"
    );
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
    let mut out = serde_json::json!({"staffs":st,"projection":p,"sticks":sticks,"systems":sys,"completion":comp});
    match page_glyphs::extract(&b, &comp, sc.interline.main as u32, None) {
        Ok((residual, _)) => match grid_ir::build(&residual, &sc, &prep, &comp, &sys, &sticks) {
            Ok(ir) => {
                match note_mask::from_grid(&b, &ir) {
                    Ok((_, mask)) => out["mask"] = mask,
                    Err(e) => out["mask_error"] = e.to_string().into(),
                };
                out["grid_ir"] = ir
            }
            Err(e) => out["grid_ir_error"] = e.to_string().into(),
        },
        Err(e) => out["extract_error"] = e.to_string().into(),
    };
    let dest = &args[2];
    std::fs::write(dest, serde_json::to_string_pretty(&out).unwrap()).unwrap();
}
