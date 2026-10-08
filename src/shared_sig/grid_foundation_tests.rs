// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/official-real-grid-weak/observed.json"
    ))
    .unwrap()
}
fn graph_for(case: &Value) -> Graph {
    let mut graph = Graph::default();
    let mut labels = BTreeMap::new();
    for (index, n) in case["nodes"].as_array().unwrap().iter().enumerate() {
        let key = Key {
            system: 1,
            phase: Phase::Grid,
            local: index + 1,
        };
        let id = graph.vertex(key, n.clone(), false).unwrap();
        labels.insert(n["label"].as_str().unwrap(), (key, id));
    }
    graph
        .relations(
            1,
            Phase::Grid,
            vec![(
                labels["brace"].0,
                labels["connector"].0,
                json!({"kind":"NoExclusion","manual":true}),
            )],
        )
        .unwrap();
    graph
}
#[test]
fn original_automatic_grid_weak_boundary_frozen_order_and_retained_body_relation_contracts() {
    for case in fixture()["cases"].as_array().unwrap() {
        let graph = graph_for(case);
        let before = graph.snapshot();
        crate::stem_head_foundation_plan::validate_chord_inputs(&before).unwrap();
        let check = crate::stem_head_foundation_plan::prepare_stem_checks(&before).unwrap();
        let mut checked = graph.clone();
        checked.apply_head_foundation_plan(&check).unwrap();
        assert_eq!(checked.snapshot(), before);
        let (staged, removed) = graph.prepare_weak_foundations().unwrap();
        assert_eq!(graph.snapshot(), before);
        assert_eq!(
            json!(
                removed
                    .iter()
                    .map(|id| graph.vertices[*id - 1].data["label"].clone())
                    .collect::<Vec<_>>()
            ),
            case["removed_order"]
        );
        let after = staged.snapshot();
        assert_eq!(
            json!(
                after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n["label"].clone())
                    .collect::<Vec<_>>()
            ),
            case["live"]
        );
        for (old, new) in graph.vertices.iter().zip(&staged.vertices) {
            assert_eq!(old.key, new.key);
            assert_eq!(old.data, new.data);
            let removed = if new.data["kind"] == "BraceInter" {
                case["brace_removed"].as_bool().unwrap()
            } else {
                case["connector_removed"].as_bool().unwrap()
            };
            assert_eq!(new.removed, removed);
        }
        assert_eq!(
            !staged.edges[0].removed,
            case["relation_registered"].as_bool().unwrap()
        );
        assert_eq!(staged.edges[0].data, graph.edges[0].data);
        assert_eq!(
            staged.edges[0].data["manual"],
            case["relation_manual_retained"]
        );
        let (repeated, again) = staged.prepare_weak_foundations().unwrap();
        assert_eq!(
            again.len() as u64,
            case["repeat_removed_count"].as_u64().unwrap()
        );
        assert_eq!(repeated.snapshot(), after);
    }
}
#[test]
fn protected_manual_brace_and_late_grid_origin_failure_restore_prefix_and_relation_bodies() {
    let fixture = fixture();
    let mut graph = graph_for(&fixture["cases"][7]);
    let brace = graph
        .vertices
        .iter()
        .position(|v| v.data["kind"] == "BraceInter")
        .unwrap();
    graph.vertices[brace].data["manual"] = json!(true);
    let before = graph.snapshot();
    let bodies = graph
        .vertices
        .iter()
        .map(|v| v.data.clone())
        .collect::<Vec<_>>();
    let edges = graph
        .edges
        .iter()
        .map(|e| e.data.clone())
        .collect::<Vec<_>>();
    assert!(
        matches!(graph.prepare_weak_foundations(), Err(reason) if reason.contains("protected_GRID_member"))
    );
    assert_eq!(graph.snapshot(), before);
    assert_eq!(
        graph
            .vertices
            .iter()
            .map(|v| v.data.clone())
            .collect::<Vec<_>>(),
        bodies
    );
    assert_eq!(
        graph
            .edges
            .iter()
            .map(|e| e.data.clone())
            .collect::<Vec<_>>(),
        edges
    );
    graph.vertices[brace].data["manual"] = json!(false);
    let (staged, removed) = graph.prepare_weak_foundations().unwrap();
    assert_eq!(removed.len(), 2);
    assert_eq!(staged.snapshot()["inters"], json!([]));
    let original = graph.snapshot();
    let mut final_state = original.clone();
    final_state["inters"] = json!([]);
    final_state["relations"] = json!([]);
    let invalid = crate::stem_head_foundation_plan::Plan {
        events: vec![
            crate::stem_head_foundation_plan::Event::RemoveVertex(1),
            crate::stem_head_foundation_plan::Event::RemoveVertex(99999),
        ],
        stages: json!({}),
        final_state,
    };
    assert!(graph.apply_head_foundation_plan(&invalid).is_err());
    assert_eq!(graph.snapshot(), original);
    let mut wrong_origin = original;
    wrong_origin["inters"][1]["origin"]["phase"] = json!("SYMBOL");
    assert!(crate::stem_head_foundation_plan::validate_chord_inputs(&wrong_origin).is_err());
}

#[test]
fn automatic_actual_real_image_brace_retirement_keeps_part_staff_curve_and_physical_owners() {
    use std::sync::Arc;
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-grid/laura-gray.png"
    ))
    .unwrap();
    let mut page = crate::page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    let held_parts = page.parts.clone();
    let held_staffs = page.staffs.clone();
    let part_storage = page.parts.as_ptr();
    let staff_storage = page.staffs.as_ptr();
    let part_bounds = page.parts.iter().map(|p| p.bounds).collect::<Vec<_>>();
    let physical = page.registry.as_ref().unwrap().snapshot();
    page.graph.contextualize_head_stems().unwrap();
    let index = page
        .graph
        .vertices
        .iter()
        .position(|v| !v.removed && v.data["kind"] == "BraceInter")
        .unwrap();
    let key = page.graph.vertices[index].key;
    // Declared below-threshold condition on an actual detected image Brace.
    page.graph.vertices[index].data["contextual_grade"] = json!(0.499);
    let brace_body = page.graph.vertices[index].data.clone();
    let (staged, removed) = page.graph.prepare_weak_foundations().unwrap();
    assert!(removed.contains(&(index + 1)));
    page.graph = staged;
    assert!(page.graph.vertices[index].removed);
    assert_eq!(page.graph.id(key), Some(index + 1));
    assert_eq!(page.graph.vertices[index].data, brace_body);
    assert_eq!(page.parts.as_ptr(), part_storage);
    assert_eq!(page.staffs.as_ptr(), staff_storage);
    assert_eq!(
        page.parts.iter().map(|p| p.bounds).collect::<Vec<_>>(),
        part_bounds
    );
    assert_eq!(page.registry.as_ref().unwrap().snapshot(), physical);
    for (now, held) in page.parts.iter().zip(&held_parts) {
        assert!(Arc::ptr_eq(now, held));
        for (staff, old) in now.staffs.iter().zip(&held.staffs) {
            assert!(Arc::ptr_eq(staff, old));
            for (curve, old) in staff.lines.iter().zip(&old.lines) {
                assert!(Arc::ptr_eq(curve, old));
            }
        }
    }
    for (now, held) in page.staffs.iter().zip(&held_staffs) {
        assert!(Arc::ptr_eq(now, held));
    }
    assert!(
        fixture()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["same_original_part_and_staff_members"] == true)
    );
}
