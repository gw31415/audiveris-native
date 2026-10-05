// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "support/stem_cache_head_controls.rs"]
mod controls;
fn main() {
    let p = std::env::args().nth(1).expect("original controls path");
    let o = serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
    println!("{}", controls::observe(&o));
}
