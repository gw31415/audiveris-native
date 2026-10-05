// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::time_registration::{builtin_metrics, symbol_bounds};
#[test]
fn generic_font_bank_covers_all_scales_and_matches_original_combo_controls() {
    let entries = builtin_metrics().unwrap();
    let mut keys = std::collections::BTreeSet::new();
    let mut combos = 0;
    for m in &entries {
        let shape = m["shape"].as_str().unwrap();
        let il = m["interline"].as_u64().unwrap() as usize;
        assert!(keys.insert((shape, il)));
        let bounds = symbol_bounds(shape, il, [1000, 1000, 100, 100], &entries).unwrap();
        if m["bounds_method"] == "numden_dimension_integer_half" {
            assert_eq!(serde_json::json!(bounds), m["original_bounds_control"]);
            combos += 1;
        }
    }
    assert_eq!(keys.len(), 2688);
    assert_eq!(combos, 9 * 128);
}
#[test]
fn malformed_public_geometry_returns_error() {
    let entries = builtin_metrics().unwrap();
    assert!(symbol_bounds("COMMON_TIME", 0, [0, 0, 1, 1], &entries).is_err());
    assert!(symbol_bounds("COMMON_TIME", 10, [0, 0, 0, 1], &entries).is_err());
    assert!(symbol_bounds("G_CLEF", 10, [0, 0, 1, 1], &entries).is_err());
    assert!(symbol_bounds("COMMON_TIME", 10, [0, 0, 1, 1], &[]).is_err());
}
