// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_template_bank::{Bank, Error};
#[test]
fn exact_family_size_coverage_and_legacy_bank_are_separate() {
    for family in ["Bravura", "Leland", "FinaleJazz"] {
        let bank = Bank::load_coverage(family).unwrap();
        assert_eq!(bank.manifest_info().font_family, family);
        assert_eq!(bank.manifest_info().keys.len(), 16);
        for size in [32, 40, 51, 64] {
            for shape in ["BREVE", "WHOLE_NOTE", "NOTEHEAD_VOID", "NOTEHEAD_BLACK"] {
                assert_eq!(bank.get(shape, size).unwrap().key().point_size, size);
            }
        }
        for size in [0, 31, 33, 50, 52, 65, u16::MAX] {
            assert!(matches!(
                bank.get("NOTEHEAD_BLACK", size),
                Err(Error::UnsupportedTemplate)
            ));
        }
        assert!(bank.get("NOTEHEAD_CROSS", 51).is_err());
    }
    let legacy = Bank::load_builtin().unwrap();
    assert_eq!(legacy.manifest_info().keys.len(), 4);
    assert!(legacy.get("NOTEHEAD_BLACK", 32).is_err());
    assert_eq!(legacy.point_size_for_width(15.).unwrap(), 51);
}
#[test]
fn unsupported_font_names_do_not_fall_back_or_change_staff_rounding() {
    for family in ["bravura", "Bravura ", "Petaluma", "JazzPerc", ""] {
        assert!(matches!(
            Bank::load_coverage(family),
            Err(Error::UnsupportedTemplate)
        ));
    }
    assert_eq!(Bank::staff_point_size(7., 11., 51).unwrap(), 32);
    assert_eq!(Bank::staff_point_size(8.5, 11., 51).unwrap(), 39);
    assert_eq!(Bank::staff_point_size(2.5, 4., 64).unwrap(), 40);
    assert_eq!(Bank::staff_point_size(1., 2., 51).unwrap(), 26);
    assert_eq!(Bank::staff_point_size(3., 2., 51).unwrap(), 76);
    assert!(Bank::staff_point_size(0., 11., 51).is_err());
}
