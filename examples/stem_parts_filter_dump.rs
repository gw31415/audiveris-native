// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "support/stem_parts_filter_controls.rs"]
mod controls;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    assert!(!std::path::Path::new(&args[2]).exists());
    let f: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let n = controls::observe(&f);
    let max = controls::compare(&f, &n);
    eprintln!("Maximum center-line coordinate difference: {max}");
    std::fs::write(&args[2], serde_json::to_vec_pretty(&n).unwrap()).unwrap();
}
