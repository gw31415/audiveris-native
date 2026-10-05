// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, stem_image_linkers};
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let im = GrayImage::decode(&std::fs::read(&a[1]).unwrap()).unwrap();
    let p = serde_json::from_slice(&std::fs::read(&a[2]).unwrap()).unwrap();
    println!("{}", stem_image_linkers::analyze(&im, &p).unwrap());
}
