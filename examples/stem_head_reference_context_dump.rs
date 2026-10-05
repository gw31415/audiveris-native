// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "support/stem_head_reference_context_controls.rs"]
mod controls;
fn main() {
    let mut args = std::env::args().skip(1);
    let o = serde_json::from_slice(&std::fs::read(args.next().unwrap()).unwrap()).unwrap();
    let page = serde_json::from_slice(&std::fs::read(args.next().unwrap()).unwrap()).unwrap();
    println!("{}", controls::observe(&o, &page));
}
