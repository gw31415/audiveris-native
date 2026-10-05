// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "support/stem_parts_lookup_controls.rs"]
mod controls;
fn main() {
    let f = serde_json::from_str(include_str!(
        "../fixtures/official-stem-parts-lookup/controls.json"
    ))
    .unwrap();
    let n = controls::observe(&f);
    println!("{}", serde_json::to_string_pretty(&n).unwrap());
}
