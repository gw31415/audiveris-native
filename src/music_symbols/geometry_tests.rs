// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::sync::Arc;
pub(crate) fn assert_real_symbol_geometry(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-geometry/image-observed.json"
    ))
    .unwrap();
    let owner = &music.owner;
    let before = owner.graph.snapshot();
    let mut queries = 0;
    let mut virtual_curves = 0;
    let mut centroids = 0;
    for expected in source["systems"].as_array().unwrap() {
        let system = owner
            .systems
            .iter()
            .find(|s| Some(s.id) == expected["system"].as_u64())
            .unwrap();
        assert_eq!(serde_json::json!(system.left), expected["system_left"]);
        assert!(
            system
                .staffs
                .iter()
                .all(|s| owner.staffs.iter().any(|held| Arc::ptr_eq(s, held)))
        );
        for row in expected["virtual_ledgers"].as_array().unwrap() {
            let map = owner
                .ledger_memberships
                .maps
                .iter()
                .find(|m| m.staff.id == row["staff"].as_u64().unwrap())
                .unwrap();
            assert_eq!(
                map.virtual_lines.len(),
                row["virtual_ledger_curves"].as_u64().unwrap() as usize
            );
            virtual_curves += map.virtual_lines.len();
        }
        for row in expected["symbol_centroids"].as_array().unwrap() {
            let point: [f64; 2] = serde_json::from_value(row["centroid"].clone()).unwrap();
            let eligible = owner
                .systems
                .iter()
                .filter(|s| s.area.contains(point).unwrap())
                .map(|s| s.id)
                .collect::<Vec<_>>();
            assert_eq!(serde_json::json!(eligible), row["systems"]);
            centroids += 1;
        }
        for row in expected["queries"]
            .as_array()
            .unwrap()
            .iter()
            .chain(expected["lookup_samples"].as_array().unwrap())
        {
            let point = serde_json::from_value(row["point"].clone()).unwrap();
            let selected = geometry::closest(owner, system, point).unwrap();
            assert_eq!(
                selected.as_ref().map(|s| s.id),
                row["closest_staff"].as_u64(),
                "system {} point {:?}",
                system.id,
                point
            );
            if let Some(staff) = selected {
                let pitch = owner.ledger_memberships.pitch_at(&staff, point).unwrap();
                assert!(
                    (pitch - row["pitch"].as_f64().unwrap()).abs() < 1e-7,
                    "system {} Staff {} point {:?}: {} vs {}",
                    system.id,
                    staff.id,
                    point,
                    pitch,
                    row["pitch"]
                );
            }
            queries += 1;
        }
    }
    assert_eq!((queries, centroids, virtual_curves), (796, 543, 12));
    let neighbors: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-real-beam-staff-neighborhood/observed.json"
    ))
    .unwrap();
    for system in neighbors["systems"].as_array().unwrap() {
        for case in system["cases"].as_array().unwrap() {
            let selected = crate::grid_staff_neighborhood::around(
                &owner.staffs,
                &owner.staff_areas,
                &owner.parts,
                system["system"].as_u64().unwrap(),
                serde_json::from_value(case["point"].clone()).unwrap(),
            )
            .unwrap();
            assert_eq!(
                json!(selected.staffs.iter().map(|s| s.id).collect::<Vec<_>>()),
                case["staff_ids"]
            );
        }
    }
    let cleaned = filter(music).unwrap();
    std::fs::write(
        "/tmp/real-head-cleaner-margin-native-cleaner.pixels",
        &cleaned.pixels,
    )
    .unwrap();
    let original: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-cleaner/corrected-observed.json"
    ))
    .unwrap();
    let metadata = json!({"catalog":owner.catalog,"dilation":head_dilation(owner.catalog.stored_point_size),"pixel_sha256":format!("{:x}",Sha256::digest(&cleaned.pixels))});
    std::fs::write(
        "/tmp/real-head-cleaner-margin-native-cleaner.json",
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();
    assert_eq!(metadata["pixel_sha256"], original["pixel_sha256"]);
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-head-cleaner-templates/image-observed.json"
    ))
    .unwrap();
    let bank = owner.catalog.bank().unwrap();
    for row in source["templates"].as_array().unwrap() {
        assert_eq!(json!(owner.catalog.family), row["family"]);
        assert_eq!(json!(owner.catalog.stored_point_size), row["point_size"]);
        let asset = bank
            .get(
                row["shape"].as_str().unwrap(),
                owner.catalog.stored_point_size,
            )
            .unwrap();
        assert_eq!(json!(asset.dimensions().0), row["width"]);
        assert_eq!(json!(asset.dimensions().1), row["height"]);
        assert_eq!(
            json!(
                asset
                    .template()
                    .points()
                    .iter()
                    .filter(|p| p.distance == 0.)
                    .map(|p| [p.x, p.y])
                    .collect::<Vec<_>>()
            ),
            row["foreground_points"]
        );
    }
    super::clusters_tests::assert_actual_image_clusters(music, &cleaned);
    crate::music_alters::assert_actual_image_factory(music, &cleaned);
    crate::shared_sig::Graph::assert_actual_image_dot_stacks(music);
    assert_eq!(owner.graph.snapshot(), before);
}
