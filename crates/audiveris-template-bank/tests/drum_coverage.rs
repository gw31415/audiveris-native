// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_template_bank::Bank;
#[test]
fn exact_drum_catalogs_cover_108_templates_without_changing_historical_banks() {
    let shapes = [
        "BREVE",
        "WHOLE_NOTE",
        "NOTEHEAD_VOID",
        "NOTEHEAD_BLACK",
        "NOTEHEAD_BLACK_SMALL",
        "NOTEHEAD_CROSS",
        "NOTEHEAD_DIAMOND_FILLED",
        "NOTEHEAD_TRIANGLE_DOWN_FILLED",
        "NOTEHEAD_CIRCLE_X",
    ];
    for family in ["Bravura", "Leland", "FinaleJazz"] {
        let bank = Bank::load_drum_coverage(family).unwrap();
        assert_eq!(bank.manifest_info().keys.len(), 36);
        assert!(bank.manifest_info().bank_id.contains("heads-drum-coverage"));
        for size in [32, 40, 51, 64] {
            for shape in shapes {
                let asset = bank.get(shape, size).unwrap();
                assert_eq!(asset.key().shape, shape);
                assert_eq!(asset.key().point_size, size);
            }
        }
        let ordinary = Bank::load_coverage(family).unwrap();
        for size in [32, 40, 51, 64] {
            for shape in ["BREVE", "WHOLE_NOTE", "NOTEHEAD_VOID", "NOTEHEAD_BLACK"] {
                let a = bank.get(shape, size).unwrap();
                let b = ordinary.get(shape, size).unwrap();
                assert_eq!(a.dimensions(), b.dimensions());
                assert_eq!(a.slim_bounds(), b.slim_bounds());
                assert_eq!(a.fat_bounds(), b.fat_bounds());
                assert_eq!(
                    a.template()
                        .points()
                        .iter()
                        .map(|p| (p.x, p.y, p.distance))
                        .collect::<Vec<_>>(),
                    b.template()
                        .points()
                        .iter()
                        .map(|p| (p.x, p.y, p.distance))
                        .collect::<Vec<_>>()
                );
            }
        }
        assert!(bank.get("WHOLE_NOTE_SMALL", 51).is_err());
        assert!(bank.get("NOTEHEAD_CROSS", 33).is_err());
        assert_eq!(
            Bank::load_coverage(family)
                .unwrap()
                .manifest_info()
                .keys
                .len(),
            16
        );
    }
    assert_eq!(Bank::load_builtin().unwrap().manifest_info().keys.len(), 4);
    assert!(
        Bank::load_builtin()
            .unwrap()
            .get("NOTEHEAD_CROSS", 51)
            .is_err()
    );
    assert!(Bank::load_drum_coverage("Petaluma").is_err());
}
