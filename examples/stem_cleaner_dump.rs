// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "support/stem_cleaner_controls.rs"]
mod controls;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    assert!(!std::path::Path::new(&args[2]).exists());
    let f: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(&controls::observe(&f)).unwrap(),
    )
    .unwrap();
}
