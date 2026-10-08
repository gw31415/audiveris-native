// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
fn staff(id: u64) -> Arc<StaffGeometry> {
    Arc::new(StaffGeometry {
        specific_interline: 11,
        id,
        system: 1,
        abscissae: [10, 500],
        lines: (0..5)
            .map(|i| {
                Arc::new(
                    crate::curves::Curve::interpolate(vec![
                        [10., id as f64 * 100. + i as f64 * 11.],
                        [500., id as f64 * 100. + i as f64 * 11.],
                    ])
                    .unwrap(),
                )
            })
            .collect(),
    })
}
fn part(staffs: Vec<Arc<StaffGeometry>>) -> Arc<PartGeometry> {
    Arc::new(PartGeometry {
        id: 1,
        system: 1,
        staffs,
        bounds: [10, 100, 490, 300],
    })
}
fn bar(graph: &mut Graph, staff: &Arc<StaffGeometry>, x: i64, local: usize) -> Key {
    let key = Key {
        system: 1,
        phase: crate::shared_sig::Phase::Grid,
        local,
    };
    graph.vertex(key,json!({"kind":"BarlineInter","staff_id":staff.id,"bounds":[x,staff.id*100,2,45],"shape":"THIN_BARLINE","grade":0.8,"manual":true,"frozen":true}),false).unwrap();
    key
}
#[test]
fn late_noncanonical_part_staff_preserves_physical_graph_and_manual_relations() {
    let st = staff(1);
    let alien = staff(1);
    let mut graph = Graph::default();
    let first = bar(&mut graph, &st, 200, 1);
    let last = bar(&mut graph, &st, 205, 2);
    graph
        .relations(
            1,
            crate::shared_sig::Phase::Grid,
            vec![(
                first,
                last,
                json!({"kind":"BarGroupRelation","manual":true}),
            )],
        )
        .unwrap();
    let before = graph.snapshot();
    assert!(
        build(
            &mut graph,
            &[st],
            &[part(vec![alien])],
            SheetSkew::new(0.).unwrap(),
            11
        )
        .is_err()
    );
    assert_eq!(before, graph.snapshot());
}
#[test]
fn back_to_back_left_boundary_shares_actual_partbarline_and_staff_refs() {
    let upper = staff(1);
    let lower = staff(2);
    let actual_part = part(vec![upper.clone(), lower.clone()]);
    let mut graph = Graph::default();
    let mut local = 1;
    for st in [&upper, &lower] {
        let mut group = Vec::new();
        for x in [10, 200, 205, 210, 499] {
            let key = bar(&mut graph, st, x, local);
            local += 1;
            if (200..=210).contains(&x) {
                group.push(key);
            }
        }
        for pair in group.windows(2) {
            graph
                .relations(
                    1,
                    crate::shared_sig::Phase::Grid,
                    vec![(
                        pair[0],
                        pair[1],
                        json!({"kind":"BarGroupRelation","manual":true}),
                    )],
                )
                .unwrap();
        }
    }
    let layout = build(
        &mut graph,
        &[upper.clone(), lower.clone()],
        std::slice::from_ref(&actual_part),
        SheetSkew::new(0.).unwrap(),
        11,
    )
    .unwrap();
    let p = &layout.parts[0];
    assert!(Arc::ptr_eq(&p.part, &actual_part));
    assert_eq!(p.measures.len(), 2);
    let first = &p.measures[0];
    let second = &p.measures[1];
    assert!(Arc::ptr_eq(
        first.right.as_ref().unwrap(),
        second.contained_left.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        second.contained_left.as_ref().unwrap(),
        second.effective_left.as_ref().unwrap()
    ));
    assert_eq!(first.abscissae(&upper).unwrap(), [11, 211]);
    assert_eq!(second.abscissae(&lower).unwrap(), [211, 500]);
    for m in &p.measures {
        assert!(Arc::ptr_eq(&m.part, &actual_part));
        assert!(layout.stacks.iter().any(|s| Arc::ptr_eq(&s.stack, &m.stack)
            && s.measures.iter().any(|actual| Arc::ptr_eq(actual, m))));
    }
    let substitute = staff(2);
    assert!(second.abscissae(&substitute).is_err());
}
fn source_bar(graph: &Graph, bar: &PartBarline) -> Value {
    json!(bar.bars.iter().map(|sb|json!({"staff":sb.group.staff.id,"members":sb.group.members.iter().map(|k| {
        let (n,_)=graph.retained_inter(*k).unwrap();json!({"bounds":n["bounds"],"shape":n["shape"],"grade":n["grade"],"frozen":n["frozen"]==true,"manual":n["manual"]==true})
    }).collect::<Vec<_>>()})).collect::<Vec<_>>())
}
pub(super) fn assert_real_layout(owner: &crate::stem_image_owned::OwnedItems) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-part-measures/image-observed.json"
    ))
    .unwrap();
    let mut graph = owner.graph.clone();
    let layout = build(
        &mut graph,
        &owner.staffs,
        &owner.parts,
        owner.sheet_skew,
        owner.interline,
    )
    .unwrap();
    assert_eq!(layout.stacks.len(), 4);
    for system in source["systems"].as_array().unwrap() {
        for expected in system["parts"].as_array().unwrap() {
            let p = layout
                .parts
                .iter()
                .find(|p| p.part.system == system["system"] && p.part.id == expected["part"])
                .unwrap();
            assert!(owner.parts.iter().any(|part| Arc::ptr_eq(&p.part, part)));
            assert_eq!(
                json!(p.part.staffs.iter().map(|s| s.id).collect::<Vec<_>>()),
                expected["staffs"]
            );
            assert_eq!(
                p.left
                    .as_ref()
                    .map(|b| source_bar(&graph, b))
                    .unwrap_or(Value::Null),
                expected["left_bar"]
            );
            assert_eq!(
                p.measures.len(),
                expected["measures"].as_array().unwrap().len()
            );
            for (index, (m, expected)) in p
                .measures
                .iter()
                .zip(expected["measures"].as_array().unwrap())
                .enumerate()
            {
                assert!(Arc::ptr_eq(&m.part, &p.part));
                assert_eq!(json!(m.stack.index), expected["stack_index"]);
                let stack = layout
                    .stacks
                    .iter()
                    .find(|s| Arc::ptr_eq(&s.stack, &m.stack))
                    .unwrap();
                assert!(stack.measures.iter().any(|actual| Arc::ptr_eq(actual, m)));
                assert_eq!(stack.measures.len(), 2);
                for x in expected["abscissae"].as_array().unwrap() {
                    let staff = p.part.staffs.iter().find(|s| s.id == x["staff"]).unwrap();
                    assert_eq!(
                        json!(m.abscissae(staff).unwrap()),
                        json!([x["left"], x["right"]])
                    );
                }
                for (actual, field) in [
                    (&m.contained_left, "contained_left"),
                    (&m.effective_left, "effective_left"),
                    (&m.right, "right_bar"),
                ] {
                    assert_eq!(
                        actual
                            .as_ref()
                            .map(|b| source_bar(&graph, b))
                            .unwrap_or(Value::Null),
                        expected[field]
                    );
                }
                assert_eq!(expected["same_effective_left_object"], true);
                assert_eq!(expected["same_Part_object"], true);
                assert_eq!(expected["same_Stack_member_object"], true);
                let effective = if index == 0 {
                    p.left.as_ref()
                } else {
                    p.measures[index - 1].right.as_ref()
                };
                assert!(Arc::ptr_eq(
                    m.effective_left.as_ref().unwrap(),
                    effective.unwrap()
                ));
            }
        }
    }
}
