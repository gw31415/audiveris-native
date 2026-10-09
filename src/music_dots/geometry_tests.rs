// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::{music_measures::OwnedMusic, music_symbols};
use sha2::{Digest, Sha256};
pub(crate) fn assert_actual_geometry(music: &OwnedMusic, cleaned: &crate::GrayImage) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-dot-geometry/image-observed.json"
    ))
    .unwrap();
    let owner = &music.owner;
    let before = owner.graph.snapshot();
    let glyphs = audiveris_staff_removal::extract_glyphs(
        audiveris_staff_removal::BinaryPage::new(cleaned.width, cleaned.height, &cleaned.pixels)
            .unwrap(),
        audiveris_staff_removal::Orientation::Vertical,
        (0, 0),
        audiveris_staff_removal::Limits::default(),
    )
    .unwrap();
    let mut count = 0;
    let mut accepted = 0;
    let mut controls = 0;
    let mut ledger_links = 0;
    for group in music_symbols::geometry::dispatch(owner, &glyphs).unwrap() {
        let expected = source["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["system"] == group.system.id)
            .unwrap();
        let fine = music_symbols::actual_fine_boxes(&owner.graph, group.system.id, owner.interline)
            .unwrap();
        let parts = group.glyphs.into_iter().cloned().collect::<Vec<_>>();
        let candidates =
            music_symbols::symbol_candidates(&parts, owner.interline, group.system.left, &fine)
                .unwrap();
        let actual = candidates
            .iter()
            .map(|(g, _)| {
                let bytes = serde_json::to_vec(&crate::beam_dispatch::glyph_json(g)).unwrap();
                (format!("{:x}", Sha256::digest(bytes)), g)
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for row in expected["trials"].as_array().unwrap() {
            let glyph = actual.get(row["glyph_sha256"].as_str().unwrap()).unwrap();
            let bounds: [i64; 4] = [
                glyph.left as i64,
                glyph.top as i64,
                glyph.width as i64,
                glyph.height as i64,
            ];
            assert_eq!(json!(bounds), row["bounds"]);
            let staff = geometry::closest(owner, group.system.id, bounds).unwrap();
            assert_eq!(json!(staff.id), row["closest_staff"]);
            let result = geometry::distant(owner, &staff, bounds).unwrap();
            assert_eq!(json!(result), row["dot_distance_accepted"], "{bounds:?}");
            accepted += usize::from(result);
            count += 1;
        }
        for row in expected["ledger_controls"].as_array().unwrap() {
            let bounds: [i64; 4] = serde_json::from_value(row["glyph"]["bounds"].clone()).unwrap();
            let staff = geometry::closest(owner, group.system.id, bounds).unwrap();
            assert_eq!(json!(staff.id), row["closest_staff"]);
            let point = [
                bounds[0] as f64 + bounds[2] as f64 / 2.,
                bounds[1] as f64 + bounds[3] as f64 / 2.,
            ];
            let ledger = geometry::concrete_ledger(owner, &staff, point).unwrap();
            let bounds = ledger
                .map(|key| owner.graph.retained_inter(key).unwrap().0["bounds"].clone())
                .unwrap_or(Value::Null);
            assert_eq!(bounds, row["concrete_ledger_bounds"]);
            let b: [i64; 4] = serde_json::from_value(row["glyph"]["bounds"].clone()).unwrap();
            assert_eq!(
                json!(geometry::distant(owner, &staff, b).unwrap()),
                row["dot_distance_accepted"]
            );
            ledger_links += usize::from(ledger.is_some());
            controls += 1;
        }
    }
    assert_eq!(
        (count, accepted, controls, ledger_links),
        (788, 736, 45, 45)
    );
    let membership: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-dot-shared-stacks/image-observed.json"
    ))
    .unwrap();
    let mut dispatches = 0;
    for sys in membership["systems"].as_array().unwrap() {
        for row in sys["controls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| !r["staff"].is_null())
        {
            let staff = owner
                .staffs
                .iter()
                .find(|s| Some(s.id) == row["staff"].as_u64())
                .unwrap();
            let index = geometry::measure_at(
                music,
                staff,
                serde_json::from_value(row["point"].clone()).unwrap(),
            )
            .unwrap();
            let measure = &music.measures[index];
            assert_eq!(
                json!([{"part":measure.part,"stack_index":measure.index}]),
                row["added"]["members"]
            );
            dispatches += 1;
        }
    }
    assert_eq!(dispatches, 12);
    assert!(geometry::closest(owner, owner.systems[0].id, [-100, -100, 2, 2]).is_err());
    assert_eq!(owner.graph.snapshot(), before);
}
