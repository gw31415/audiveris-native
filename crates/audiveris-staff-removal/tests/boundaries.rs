use audiveris_staff_removal::*;
fn glyph(
    left: i32,
    top: i32,
    w: usize,
    h: usize,
    o: Orientation,
    r: &[(usize, usize, usize)],
) -> GlyphPixels {
    GlyphPixels {
        left,
        top,
        width: w,
        height: h,
        orientation: o,
        runs: r
            .iter()
            .map(|&(position, start, length)| Run {
                position,
                start,
                length,
            })
            .collect(),
    }
}
#[test]
fn membership_preserves_crossing_stem_and_notehead_and_duplicates_are_idempotent() {
    let mut b = vec![255; 49];
    for i in 0..7 {
        b[3 * 7 + i] = 0;
        b[i * 7 + 3] = 0;
    }
    for y in 2..=4 {
        for x in 2..=4 {
            b[y * 7 + x] = 0;
        }
    }
    let g = glyph(0, 3, 7, 1, Orientation::Horizontal, &[(0, 0, 2), (0, 5, 2)]);
    let page = BinaryPage::new(7, 7, &b).unwrap();
    let a = erase_staff(page, std::slice::from_ref(&g), Limits::default()).unwrap();
    let z = erase_staff(page, &[g.clone(), g], Limits::default()).unwrap();
    assert_eq!(a.removed_pixels, 4);
    assert_eq!(a.residual, z.residual);
    assert_eq!(a.removed_mask, z.removed_mask);
    for y in 0..7 {
        assert_eq!(a.residual[y * 7 + 3], 0);
    }
    for y in 2..=4 {
        for x in 2..=4 {
            assert_eq!(a.residual[y * 7 + x], 0);
        }
    }
    // Membership is authoritative: do not silently invent crossing restoration.
    let full = glyph(0, 3, 7, 1, Orientation::Horizontal, &[(0, 0, 7)]);
    assert_eq!(
        erase_staff(page, &[full], Limits::default())
            .unwrap()
            .residual[24],
        255
    );
}
#[test]
fn signed_origins_clip_both_orientations_and_white_pixels_are_not_removed() {
    let b = vec![0; 9];
    let p = BinaryPage::new(3, 3, &b).unwrap();
    let h = glyph(-2, -1, 4, 3, Orientation::Horizontal, &[(1, 0, 4)]);
    let v = glyph(-1, -2, 3, 4, Orientation::Vertical, &[(1, 0, 4)]);
    let far = glyph(
        i32::MAX,
        i32::MIN,
        3,
        1,
        Orientation::Horizontal,
        &[(0, 0, 3)],
    );
    let out = erase_staff(p, &[h, v, far], Limits::default()).unwrap();
    assert_eq!(out.removed_pixels, 3);
    assert_eq!(out.residual, vec![255, 255, 0, 255, 0, 0, 0, 0, 0]);
    let white = vec![255; 9];
    let g = glyph(0, 0, 3, 1, Orientation::Horizontal, &[(0, 0, 3)]);
    let out = erase_staff(
        BinaryPage::new(3, 3, &white).unwrap(),
        &[g],
        Limits::default(),
    )
    .unwrap();
    assert_eq!(out.removed_pixels, 0);
}
#[test]
fn empty_inputs_invalid_runs_nonbinary_and_resource_budgets() {
    assert_eq!(BinaryPage::new(0, 1, &[]).unwrap_err(), Error::Dimensions);
    assert_eq!(
        BinaryPage::new(usize::MAX, 2, &[]).unwrap_err(),
        Error::Dimensions
    );
    let b = vec![0; 9];
    let p = BinaryPage::new(3, 3, &b).unwrap();
    let empty = erase_staff(p, &[], Limits::default()).unwrap();
    assert_eq!(empty.residual, b);
    assert_eq!(empty.removed_pixels, 0);
    assert_eq!(
        erase_staff(
            BinaryPage::new(1, 1, &[128]).unwrap(),
            &[],
            Limits::default()
        )
        .unwrap_err(),
        Error::NotBinary
    );
    let g = glyph(0, 0, 3, 1, Orientation::Horizontal, &[(0, 0, 3)]);
    for bad in [
        glyph(0, 0, 3, 1, Orientation::Horizontal, &[(0, usize::MAX, 2)]),
        glyph(0, 0, 3, 1, Orientation::Horizontal, &[(1, 0, 1)]),
        glyph(0, 0, 3, 1, Orientation::Horizontal, &[(0, 0, 0)]),
    ] {
        assert_eq!(
            erase_staff(p, &[bad], Limits::default()).unwrap_err(),
            Error::RunBounds
        );
    }
    let bad = glyph(0, 0, 3, 1, Orientation::Horizontal, &[(0, 0, 1), (0, 1, 1)]);
    assert_eq!(
        erase_staff(p, &[bad], Limits::default()).unwrap_err(),
        Error::RunOrder
    );
    for limits in [
        Limits {
            max_pixels: 8,
            ..Limits::default()
        },
        Limits {
            max_dimension: 2,
            ..Limits::default()
        },
        Limits {
            max_glyphs: 0,
            ..Limits::default()
        },
        Limits {
            max_runs: 0,
            ..Limits::default()
        },
        Limits {
            max_run_pixels: 2,
            ..Limits::default()
        },
    ] {
        assert_eq!(
            erase_staff(p, std::slice::from_ref(&g), limits).unwrap_err(),
            Error::Budget
        );
    }
    assert_eq!(
        erase_staff(
            p,
            &[glyph(0, 0, 1, 1, Orientation::Horizontal, &[])],
            Limits::default()
        )
        .unwrap()
        .residual,
        b
    );
}
#[test]
fn four_connectivity_order_orientation_origin_empty_and_limits() {
    let diagonal = [0, 255, 255, 0];
    let p = BinaryPage::new(2, 2, &diagonal).unwrap();
    for o in [Orientation::Horizontal, Orientation::Vertical] {
        let gs = extract_glyphs(p, o, (-7, 11), Limits::default()).unwrap();
        assert_eq!(gs.len(), 2);
        assert_eq!((gs[0].left, gs[0].top), (-7, 11));
        assert_eq!((gs[1].left, gs[1].top), (-6, 12));
        assert_eq!(gs[0].orientation, o);
    }
    let white = [255; 4];
    assert!(
        extract_glyphs(
            BinaryPage::new(2, 2, &white).unwrap(),
            Orientation::Horizontal,
            (0, 0),
            Limits::default()
        )
        .unwrap()
        .is_empty()
    );
    let bridge = [0, 255, 0, 0, 0, 0];
    let g = extract_glyphs(
        BinaryPage::new(3, 2, &bridge).unwrap(),
        Orientation::Horizontal,
        (17, -9),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(g.len(), 1);
    assert_eq!(
        (g[0].left, g[0].top, g[0].width, g[0].height),
        (17, -9, 3, 2)
    );
    for limits in [
        Limits {
            max_runs: 1,
            ..Limits::default()
        },
        Limits {
            max_glyphs: 1,
            ..Limits::default()
        },
        Limits {
            max_run_pixels: 1,
            ..Limits::default()
        },
    ] {
        assert_eq!(
            extract_glyphs(p, Orientation::Horizontal, (0, 0), limits).unwrap_err(),
            Error::Budget
        );
    }
    assert_eq!(
        extract_glyphs(p, Orientation::Horizontal, (i32::MAX, 0), Limits::default()).unwrap_err(),
        Error::CoordinateOverflow
    );
}
#[test]
fn many_nonoverlapping_sequences_are_bounded_without_recursive_union() {
    let w = 256;
    let h = 256;
    let b: Vec<_> = (0..w * h)
        .map(|i| if (i % w + i / w) % 2 == 0 { 0 } else { 255 })
        .collect();
    let gs = extract_glyphs(
        BinaryPage::new(w, h, &b).unwrap(),
        Orientation::Horizontal,
        (0, 0),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(gs.len(), 32768);
    assert!(gs.iter().all(|g| g.width == 1 && g.height == 1));
}

#[test]
fn a4_300dpi_fits_default_budget_but_excess_page_and_origin_extent_do_not() {
    let data = vec![255; 2480 * 3508];
    let page = BinaryPage::new(2480, 3508, &data).unwrap();
    let result = erase_staff(page, &[], Limits::default()).unwrap();
    assert_eq!(result.residual.len(), 8_699_840);
    assert_eq!(result.removed_pixels, 0);
    drop(result);
    drop(data);
    let over = vec![255; 4000 * 4001];
    let page = BinaryPage::new(4000, 4001, &over).unwrap();
    assert_eq!(
        erase_staff(page, &[], Limits::default()).unwrap_err(),
        Error::Budget
    );
    assert_eq!(
        extract_glyphs(page, Orientation::Horizontal, (0, 0), Limits::default()).unwrap_err(),
        Error::Budget
    );
    assert_eq!(
        extract_glyphs(
            BinaryPage::new(2, 1, &[0, 0]).unwrap(),
            Orientation::Horizontal,
            (i32::MAX, 0),
            Limits::default()
        )
        .unwrap_err(),
        Error::CoordinateOverflow
    );
}
