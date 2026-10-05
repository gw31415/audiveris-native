// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, head_seed_search, head_staff_catalog::TemplateCatalog};
use audiveris_template_bank::Bank;
use audiveris_template_match::{Limits, Storage, Target, Weights, compute};
use serde_json::{Value, json};
#[test]
fn original_eight_default_cue_templates_geometry_anchors_keypoints_and_scores_match() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let f = root.join("fixtures/official-cue-default-heads/original-bank");
    let original: Value =
        serde_json::from_slice(&std::fs::read(f.join("assets.json")).unwrap()).unwrap();
    let bank = Bank::load_cue_default().unwrap();
    assert_eq!(bank.manifest_info().keys.len(), 8);
    for a in original["assets"].as_array().unwrap() {
        let asset = bank
            .get(
                a["shape"].as_str().unwrap(),
                a["point_size"].as_u64().unwrap() as u16,
            )
            .unwrap();
        assert_eq!(json!(asset.dimensions()), json!([a["width"], a["height"]]));
        for (bounds, key) in [
            (asset.fat_bounds(), "fat_bounds"),
            (asset.slim_bounds(), "slim_bounds"),
        ] {
            for (v, n) in bounds.iter().zip(a[key].as_array().unwrap()) {
                assert_eq!(*v as f64, n.as_f64().unwrap());
            }
        }
        assert_eq!(
            asset.template().points().len(),
            a["points"].as_array().unwrap().len()
        );
        for (p, q) in asset
            .template()
            .points()
            .iter()
            .zip(a["points"].as_array().unwrap())
        {
            assert_eq!(json!([p.x, p.y, p.distance]), *q);
        }
        for anchor in a["anchors"].as_array().unwrap() {
            let got = asset.anchor(anchor["name"].as_str().unwrap()).unwrap();
            assert_eq!(got.x, anchor["x"]);
            assert_eq!(got.y, anchor["y"]);
        }
        let (w, h) = asset.dimensions();
        let pixels = std::fs::read(f.join(a["file"].as_str().unwrap())).unwrap();
        let table = compute(
            w as usize,
            h as usize,
            &pixels,
            Target::Foreground,
            Storage::JavaShort,
            Limits::default(),
        )
        .unwrap();
        let score = asset
            .template()
            .evaluate(&table, 0, 0, None, Weights::default())
            .unwrap()
            .score;
        assert_eq!(score, a["scores"][0]["score"].as_f64().unwrap());
    }
    assert_eq!(Bank::load_builtin().unwrap().manifest_info().keys.len(), 4);
    assert_eq!(
        Bank::load_coverage("Bravura")
            .unwrap()
            .manifest_info()
            .keys
            .len(),
        16
    );
}
#[test]
fn default_cue_exact_bank_and_staff_preflight_refuse_unsupported_sizes_without_mutation() {
    let catalog = TemplateCatalog {
        family: "Bravura".into(),
        stored_point_size: 44,
    };
    catalog.validate().unwrap();
    assert_eq!(catalog.staff_point_size(7, 11).unwrap(), 28);
    assert_eq!(catalog.staff_point_size(11, 11).unwrap(), 44);
    for n in [0, 1, 6, 8, 10, 12, 129] {
        assert!(catalog.staff_point_size(n, 11).is_err());
    }
    for family in ["Leland", "FinaleJazz", "Petaluma"] {
        assert!(
            TemplateCatalog {
                family: family.into(),
                stored_point_size: 44
            }
            .validate()
            .is_err()
        );
    }
    let bank = catalog.bank().unwrap();
    for size in [0, 27, 29, 32, 40, 43, 45, 51, 64] {
        assert!(bank.get("NOTEHEAD_BLACK", size).is_err());
    }
    assert!(bank.get("NOTEHEAD_CROSS", 44).is_err());
    let image = GrayImage {
        width: 1,
        height: 1,
        pixels: vec![255],
    };
    let grid =
        json!({"staffs":[{"id":1,"specific_interline":8,"small":true,"lines":[{},{},{},{},{}]}]});
    let before = grid.clone();
    let error = head_seed_search::analyze_catalog(
        &image,
        &grid,
        &json!({}),
        &json!({}),
        &json!({"music_font":{"name":"Bravura","pointSize":44}}),
        3,
        11,
    )
    .unwrap_err();
    assert_eq!(error, "unsupported_head_staff_template_catalog");
    assert_eq!(grid, before);
}
