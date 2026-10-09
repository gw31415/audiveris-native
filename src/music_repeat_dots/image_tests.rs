// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(crate) fn assert_actual_lookup(music: &OwnedMusic) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-repeat-dot-lookup/image-observed.json"
    ))
    .unwrap();
    let before = music.owner.graph.snapshot();
    let (mut candidates, mut declared, mut links, mut allowed) = (0, 0, 0, 0);
    for sys in source["systems"].as_array().unwrap() {
        let system = sys["system"].as_u64().unwrap();
        assert_eq!(sys["actual_instant_repeat_count"], 0);
        for (name, controls) in [
            ("candidates", &sys["candidates"]),
            ("declared_bar_neighbors", &sys["declared_bar_neighbors"]),
        ] {
            for row in controls.as_array().unwrap() {
                let b: [i64; 4] = serde_json::from_value(row["bounds"].clone()).unwrap();
                let found =
                    lookup(music, system, b, row["profile"].as_u64().unwrap() as u32).unwrap();
                assert_eq!(found.staff.as_ref().map(|s| s.id), row["staff"].as_u64());
                assert!(
                    (found.pitch.unwrap() - row["estimated_pitch"].as_f64().unwrap()).abs() < 1e-10
                );
                assert_eq!(found.pitch_allowed, row["pitch_allowed"]);
                let actual=found.bar.as_ref().map(|(k,r)|json!({"bar_bounds":music.owner.graph.retained_inter(*k).unwrap().0["bounds"],"grade":r["grade"],"dx":r["dx"],"dy":r["dy"],"source_ratio":r["source_ratio"],"target_ratio":r["target_ratio"]})).unwrap_or(Value::Null);
                if actual.is_null() {
                    assert!(row["bar_lookup"].is_null());
                } else {
                    assert_eq!(actual["bar_bounds"], row["bar_lookup"]["bar_bounds"]);
                    for field in ["grade", "dx", "dy", "source_ratio", "target_ratio"] {
                        assert!(
                            (actual[field].as_f64().unwrap()
                                - row["bar_lookup"][field].as_f64().unwrap())
                            .abs()
                                < 1e-10,
                            "{field} {row}"
                        );
                    }
                }
                if name == "candidates" {
                    candidates += 1;
                    instant_check(music, system, b).unwrap();
                } else {
                    declared += 1;
                    links += usize::from(found.bar.is_some());
                    allowed += usize::from(found.pitch_allowed);
                }
            }
        }
    }
    assert_eq!((candidates, declared, links), (97, 168, 168));
    assert!(allowed > 0);
    assert_eq!(music.owner.graph.snapshot(), before);
}
