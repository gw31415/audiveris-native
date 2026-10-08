// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::shared_sig::Phase;
fn staff(id: u64) -> Arc<StaffGeometry> {
    Arc::new(StaffGeometry {
        id,
        system: 1,
        abscissae: [0, 500],
        lines: (0..5)
            .map(|i| {
                Arc::new(
                    crate::curves::Curve::interpolate(vec![
                        [0., i as f64 * 11.],
                        [500., i as f64 * 11.],
                    ])
                    .unwrap(),
                )
            })
            .collect(),
    })
}
fn member(graph: &Graph, key: Key) -> Value {
    let (n, _) = graph.retained_inter(key).unwrap();
    json!({"bounds":n["bounds"],"shape":n["shape"],"grade":n["grade"],"frozen":n["frozen"]==true,"manual":n["manual"]==true})
}
fn row(graph: &Graph, group: &Group) -> Value {
    json!({"members":group.members.iter().map(|k|member(graph,*k)).collect::<Vec<_>>(),"center":group.center,"deskew_x":group.deskew_x})
}
#[test]
fn original_constructor_styles_average_grade_frozen_manual_and_actual_member_references_match() {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-barline-groups/constructors.json"
    ))
    .unwrap();
    for case in source["cases"].as_array().unwrap() {
        let mut g = Graph::default();
        let staff = staff(2);
        let mut members = Vec::new();
        for (i, n) in case["members"].as_array().unwrap().iter().enumerate() {
            let k = Key {
                system: 1,
                phase: Phase::Grid,
                local: i + 1,
            };
            let mut n = n.clone();
            n["kind"] = json!("BarlineInter");
            n["staff_id"] = json!(2);
            g.vertex(k, n, false).unwrap();
            members.push(k);
        }
        for pair in members.windows(2) {
            g.relations(
                1,
                Phase::Grid,
                vec![(
                    pair[0],
                    pair[1],
                    json!({"kind":"BarGroupRelation","manual":true}),
                )],
            )
            .unwrap();
        }
        let before = g.snapshot();
        let groups = groups(&g, staff.clone(), SheetSkew::new(0.).unwrap()).unwrap();
        assert_eq!(groups.len(), 1);
        let group = &groups[0];
        assert!(Arc::ptr_eq(&group.staff, &staff));
        assert_eq!(group.members, members);
        let (key, _) = group.register(&mut g, 11).unwrap();
        let (n, _) = g.retained_inter(key).unwrap();
        for field in [
            "bounds",
            "style",
            "grade",
            "frozen",
            "manual",
            "back_to_back",
            "left_x",
            "right_x",
        ] {
            assert_eq!(n[field], case[field], "{}", field);
        }
        assert_eq!(
            group
                .members
                .iter()
                .map(|k| member(&g, *k))
                .collect::<Vec<_>>(),
            case["members"].as_array().unwrap().clone()
        );
        for old in before["relations"].as_array().unwrap() {
            assert_eq!(
                g.snapshot()["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["id"] == old["id"])
                    .unwrap(),
                old
            );
        }
    }
}
#[test]
fn late_bad_containment_endpoint_preserves_graph_and_existing_manual_group_relation() {
    let staff = staff(2);
    let mut g = Graph::default();
    let key = Key {
        system: 1,
        phase: Phase::Grid,
        local: 1,
    };
    g.vertex(key,json!({"kind":"BarlineInter","staff_id":2,"shape":"THIN_BARLINE","bounds":[100,100,2,40],"grade":0.8,"manual":true}),false).unwrap();
    let second = Key {
        system: 1,
        phase: Phase::Grid,
        local: 2,
    };
    g.vertex(second,json!({"kind":"BarlineInter","staff_id":2,"shape":"THIN_BARLINE","bounds":[105,100,2,40],"grade":0.8,"manual":true}),false).unwrap();
    g.relations(
        1,
        Phase::Grid,
        vec![(
            key,
            second,
            json!({"kind":"BarGroupRelation","manual":true}),
        )],
    )
    .unwrap();
    let mut group = groups(&g, staff, SheetSkew::new(0.).unwrap())
        .unwrap()
        .remove(0);
    group.members.push(Key {
        system: 1,
        phase: Phase::Grid,
        local: 999,
    });
    let before = g.snapshot();
    assert!(group.register(&mut g, 11).is_err());
    assert_eq!(g.snapshot(), before);
}
#[test]
fn real_foundation_bar_groups_and_construction_match_original_before_multistaff_measure_ownership()
{
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-barline-groups/image-observed.json"
    ))
    .unwrap();
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-grid/laura-gray.png"
    ))
    .unwrap();
    let owner = crate::stem_image_owned::prepare_items(&image).unwrap();
    let owner = crate::music_links::prepare_beam_linking(owner).unwrap();
    let owner = crate::stem_image_head_linking::continue_process(owner).unwrap();
    let owner = crate::stem_foundation_consistency::continue_reduction(owner).unwrap();
    let mut owner = crate::music_links::finalize_stems(owner).unwrap();
    let mut graph = owner.graph.clone();
    for system in source["systems"].as_array().unwrap() {
        let sid = system["system"].as_u64().unwrap();
        let staffs = owner
            .staffs
            .iter()
            .filter(|s| s.system == sid)
            .cloned()
            .collect::<Vec<_>>();
        let mut actual = Vec::new();
        for expected in system["groups"].as_array().unwrap() {
            let staff = staffs
                .iter()
                .find(|s| s.id == expected["staff"].as_u64().unwrap())
                .unwrap();
            let groups = groups(&graph, staff.clone(), owner.sheet_skew).unwrap();
            assert_eq!(groups.len(), expected["groups"].as_array().unwrap().len());
            let offset = expected["groups"][0]["deskew_x"].as_f64().unwrap() - groups[0].deskew_x;
            for (g, expected) in groups.iter().zip(expected["groups"].as_array().unwrap()) {
                let mut observed = row(&graph, g);
                observed["deskew_x"] = json!(g.deskew_x + offset);
                assert_eq!(observed["members"], expected["members"]);
                assert_eq!(observed["center"], expected["center"]);
                assert!(
                    (observed["deskew_x"].as_f64().unwrap()
                        - expected["deskew_x"].as_f64().unwrap())
                    .abs()
                        < 1e-9
                );
                assert!(Arc::ptr_eq(&g.staff, staff));
            }
            let bars = system["staff_bars"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["staff"] == expected["staff"])
                .unwrap();
            for (g, expected) in groups.iter().zip(bars["bars"].as_array().unwrap()) {
                let (key, _) = g.register(&mut graph, 11).unwrap();
                let (n, _) = graph.retained_inter(key).unwrap();
                for field in [
                    "bounds",
                    "style",
                    "grade",
                    "frozen",
                    "manual",
                    "back_to_back",
                ] {
                    assert_eq!(
                        n[field], expected[field],
                        "system {} Staff {} field {}",
                        sid, staff.id, field
                    );
                }
                assert_eq!(
                    g.members
                        .iter()
                        .map(|k| member(&graph, *k))
                        .collect::<Vec<_>>(),
                    expected["members"].as_array().unwrap().clone()
                );
            }
            actual.extend(groups);
        }
        complete_columns(&actual, &staffs, 11).unwrap();
    }

    crate::music_measure_layout::assert_real_layout(&owner);
    let parts = owner.parts.clone();
    owner.upstream_report["grid_ir"]["systems"] = json!([]);
    owner.upstream_report["grid_ir"]["staffs"] = json!([]);
    let music = crate::music_measures::continue_measures(owner).unwrap();
    crate::music_measures::assert_real_chords(&music);
    assert_eq!(
        parts.iter().map(|p| p.staffs.len()).collect::<Vec<_>>(),
        vec![1, 2, 1, 2]
    );
}
