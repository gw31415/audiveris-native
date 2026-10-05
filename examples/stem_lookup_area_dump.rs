// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "support/stem_lookup_area_controls.rs"]
mod controls;
fn main() {
    let path = std::env::args().nth(1).expect("original controls path");
    let original = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    println!("{}", controls::observe(&original));
}
