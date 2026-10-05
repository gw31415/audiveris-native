// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, Run, scale,
    sections::{self, JunctionPolicy, Orientation, RunTable},
    vertical,
};
#[test]
fn section_factory_partitions_all_pixels_and_masks_preserve_origins() {
    let image = GrayImage::decode(include_bytes!("../fixtures/sections/crossings.pgm")).unwrap();
    for o in [Orientation::Horizontal, Orientation::Vertical] {
        let table = RunTable::from_binary(&image, o, |_| true).unwrap();
        let sections = sections::build(&table, JunctionPolicy::Ratio(1.25)).unwrap();
        let mut rebuilt = vec![255; image.pixels.len()];
        for s in &sections {
            let glyph = s.as_glyph_candidate();
            assert_eq!(glyph.pixels.iter().filter(|p| **p == 0).count(), s.weight);
            for y in 0..glyph.bounds.height {
                for x in 0..glyph.bounds.width {
                    if glyph.pixels[y * glyph.bounds.width + x] == 0 {
                        let i = (glyph.bounds.y + y) * image.width + glyph.bounds.x + x;
                        assert_eq!(rebuilt[i], 255);
                        rebuilt[i] = 0;
                    }
                }
            }
        }
        assert_eq!(rebuilt, image.pixels);
    }
}
#[test]
fn split_merge_empty_gap_and_integer_mean_boundaries() {
    let table = RunTable {
        orientation: Orientation::Horizontal,
        width: 30,
        height: 5,
        sequences: vec![
            vec![Run {
                start: 0,
                length: 10,
            }],
            vec![Run {
                start: 0,
                length: 12,
            }],
            vec![Run {
                start: 0,
                length: 13,
            }],
            vec![],
            vec![Run {
                start: 0,
                length: 10,
            }],
        ],
    };
    let s = sections::build(&table, JunctionPolicy::Ratio(1.25)).unwrap();
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].weight, 35);
    assert_eq!(s[0].mean_run_length(), 11);
    assert_eq!(s[1].first_pos, 4);
    let split = RunTable {
        orientation: Orientation::Horizontal,
        width: 20,
        height: 3,
        sequences: vec![
            vec![Run {
                start: 0,
                length: 10,
            }],
            vec![
                Run {
                    start: 0,
                    length: 4,
                },
                Run {
                    start: 6,
                    length: 4,
                },
            ],
            vec![Run {
                start: 0,
                length: 10,
            }],
        ],
    };
    let s = sections::build(&split, JunctionPolicy::Ratio(1.25)).unwrap();
    assert_eq!(s.len(), 4);
    assert!(s.iter().all(|s| s.runs.len() == 1));
}
#[test]
fn grid_preparation_is_not_staff_detection() {
    let image = GrayImage::decode(include_bytes!("../fixtures/sections/straight.pgm")).unwrap();
    let binary = vertical::binarize(&image, |_, _, _| {});
    let scale = scale::analyze(&binary).result.unwrap();
    let prep = sections::prepare(&binary, &scale).unwrap();
    assert_eq!(prep.horizontal.len(), 5);
    assert_eq!(prep.vertical.len(), 0);
    assert_eq!(prep.min_vertical_length, 3);
    assert_eq!(prep.min_horizontal_length, 4);
}
