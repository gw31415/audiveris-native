// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_linked_tests::chord_key;
use super::tuplet_preparation_tests::{Prepared, control, state};
use super::*;
use crate::shared_sig::{Key, Phase};

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-tuplet-reuse/observed.json"
    ))
    .unwrap();
    let frames = d["frame_sets"].as_array().unwrap().clone();
    for case in d["cases"].as_array_mut().unwrap() {
        for s in case["scenarios"].as_array_mut().unwrap() {
            for f in ["before", "after"] {
                s[f] = frames[s[f].as_u64().unwrap() as usize].clone();
            }
        }
    }
    d
}
fn prepare(case: &Value) -> Prepared {
    let mut p = control(case);
    let spec = &case["declared_extra_group"];
    let snapshot = p.c.owner.owner.graph.snapshot();
    let next = |phase| {
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|n| serde_json::from_value::<Key>(n["origin"].clone()).ok())
            .filter(|k| k.system == 1 && k.phase == phase)
            .map(|k| k.local)
            .max()
            .unwrap_or(0)
            + 1
    };
    let group = Key {
        system: 1,
        phase: Phase::Group,
        local: next(Phase::Group),
    };
    let beam = Key {
        system: 1,
        phase: Phase::Beam,
        local: next(Phase::Beam),
    };
    for (key, data, label) in [
        (group, json!({"kind":"BeamGroupInter"}), "extra_group"),
        (
            beam,
            json!({"kind":"BeamInter","median":spec["median"],"height":spec["height"]}),
            "extra_beam",
        ),
    ] {
        let id = p.c.owner.owner.graph.vertex(key, data, false).unwrap();
        p.c.labels.insert(id, label.into());
    }
    p.c.owner
        .owner
        .graph
        .relations(
            1,
            Phase::Group,
            vec![(group, beam, json!({"kind":"Containment"}))],
        )
        .unwrap();
    for name in spec["chords"].as_array().unwrap() {
        let key = chord_key(&p.c, name.as_str().unwrap());
        let stem =
            p.c.owner
                .chords
                .iter()
                .find(|ch| ch.key == key)
                .unwrap()
                .stem
                .unwrap();
        p.c.owner
            .owner
            .graph
            .relations(
                1,
                Phase::Beam,
                vec![(beam, stem, json!({"kind":"BeamStemRelation"}))],
            )
            .unwrap();
        p.c.owner
            .owner
            .graph
            .invalidate_owned_head_chord_cache(key)
            .unwrap();
    }
    p
}
fn keys(p: &Prepared, names: &Value) -> Vec<Key> {
    names
        .as_array()
        .unwrap()
        .iter()
        .map(|name| {
            name.as_str().map(|s| chord_key(&p.c, s)).unwrap_or(Key {
                system: 1,
                phase: Phase::Chord,
                local: 999999,
            })
        })
        .collect()
}
fn names(p: &Prepared, keys: &[Key]) -> Value {
    json!(
        keys.iter()
            .map(|key| p.c.labels[&p.c.owner.owner.graph.id(*key).unwrap()].clone())
            .collect::<Vec<_>>()
    )
}
fn normalized(mut v: Value) -> Value {
    v["owned"]["relations"] = json!([]);
    v
}
#[test]
fn original_tuplet_reuse_keeps_first_actual_relation_manual_identity_and_single_pivot_groups() {
    for case in original()["cases"].as_array().unwrap() {
        let mut p = prepare(case);
        let physical = p.c.owner.chords.as_ptr();
        let slots = p.c.owner.measures[0].timing.slots.as_ptr();
        let tuplets = p.c.owner.measures[0].tuplets.as_ptr();
        let staffs = p.c.owner.owner.staffs.clone();
        let report = p.c.owner.owner.prepared.report.clone();
        for scenario in case["scenarios"].as_array().unwrap() {
            let spec = &scenario["spec"];
            let group = keys(&p, &spec["group"]);
            let before = state(&mut p);
            assert_eq!(
                before["state"],
                normalized(scenario["before"].clone()),
                "{} {} before",
                case["name"],
                spec["name"]
            );
            let graph = p.c.owner.owner.graph.snapshot();
            let mut winners = Vec::new();
            if spec["initial"].is_array() {
                let initial = keys(&p, &spec["initial"]);
                let plan =
                    p.c.owner
                        .extend_rhythm_tuplet_group(1, &group, &initial)
                        .unwrap();
                assert_eq!(names(&p, &plan.extended), scenario["extended"]);
                assert_eq!(
                    json!(plan.groups.iter().map(|g| names(&p, g)).collect::<Vec<_>>()),
                    scenario["groups"]
                );
                for g in plan.groups {
                    winners.push(p.c.owner.existing_rhythm_tuplet(1, &g).unwrap().unwrap());
                }
            } else if scenario["error"].is_string() {
                assert_eq!(
                    p.c.owner.existing_rhythm_tuplet(1, &group).unwrap_err(),
                    "music_actual_tuplet_chord_owner_required"
                );
            } else {
                winners.push(
                    p.c.owner
                        .existing_rhythm_tuplet(1, &group)
                        .unwrap()
                        .unwrap(),
                );
            }
            assert_eq!(names(&p, &winners), scenario["winners"]);
            assert_eq!(scenario["changes"], 0);
            assert_eq!(state(&mut p), before);
            assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
            assert_eq!(before["state"], normalized(scenario["after"].clone()));
            assert_eq!(p.c.owner.chords.as_ptr(), physical);
            assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(p.c.owner.measures[0].tuplets.as_ptr(), tuplets);
            assert_eq!(p.c.owner.owner.prepared.report, report);
            for (a, b) in staffs.iter().zip(&p.c.owner.owner.staffs) {
                assert!(Arc::ptr_eq(a, b));
                for (a, b) in a.lines.iter().zip(&b.lines) {
                    assert!(Arc::ptr_eq(a, b));
                }
            }
            for (i, key) in p.tuplets.iter().enumerate() {
                assert_eq!(
                    p.c.owner.owner.graph.tuplet_registration(*key)["manual"],
                    scenario["tuplets"][i]["manual"]
                );
            }
        }
    }
}
#[test]
fn late_tuplet_group_failure_rolls_back_warmed_caches_and_preserves_actual_owners_for_retry() {
    let d = original();
    let case = &d["cases"][0];
    let mut p = prepare(case);
    let seed = chord_key(&p.c, "chord3");
    let broken = chord_key(&p.c, "chord6");
    let index =
        p.c.owner
            .chords
            .iter()
            .position(|ch| ch.key == broken)
            .unwrap();
    let before = state(&mut p);
    let physical = p.c.owner.chords.as_ptr();
    let slots = p.c.owner.measures[0].timing.slots.as_ptr();
    let voices = projection(&p.c.owner.voices, &p.c.owner.measures);
    p.c.owner
        .owner
        .graph
        .invalidate_owned_head_chord_cache(broken)
        .unwrap();
    let saved = p.c.owner.chords[index].id;
    p.c.owner.chords[index].id = 999999;
    let graph = p.c.owner.owner.graph.snapshot();
    assert_eq!(
        p.c.owner
            .extend_rhythm_tuplet_group(1, &[seed, broken], &[])
            .unwrap_err(),
        "music_rest_actual_chord_required"
    );
    assert_eq!(p.c.owner.owner.graph.snapshot(), graph);
    assert_eq!(p.c.owner.chords.as_ptr(), physical);
    assert_eq!(p.c.owner.measures[0].timing.slots.as_ptr(), slots);
    assert_eq!(projection(&p.c.owner.voices, &p.c.owner.measures), voices);
    p.c.owner.chords[index].id = saved;
    let plan =
        p.c.owner
            .extend_rhythm_tuplet_group(1, &[broken, seed], &[])
            .unwrap();
    let expected = case["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["spec"]["name"] == "first_pivot_only")
        .unwrap();
    assert_eq!(names(&p, &plan.extended), expected["extended"]);
    assert_eq!(
        json!(plan.groups.iter().map(|g| names(&p, g)).collect::<Vec<_>>()),
        expected["groups"]
    );
    assert_eq!(state(&mut p), before);
}
