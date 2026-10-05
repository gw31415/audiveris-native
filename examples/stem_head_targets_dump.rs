// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "support/stem_head_targets_controls.rs"]
mod controls;
fn main() {
    let p = std::env::args().nth(1).unwrap();
    let v = serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
    println!("{}", controls::observe(&v));
}
