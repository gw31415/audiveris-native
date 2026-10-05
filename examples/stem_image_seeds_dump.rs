// SPDX-License-Identifier: AGPL-3.0-or-later
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let image = audiveris_native::GrayImage::decode(&std::fs::read(&a[1]).unwrap()).unwrap();
    let page = serde_json::from_slice(&std::fs::read(&a[2]).unwrap()).unwrap();
    let report = audiveris_native::stem_image_seeds::analyze(&image, &page).unwrap();
    std::fs::write(&a[3], serde_json::to_vec(&report).unwrap()).unwrap();
}
