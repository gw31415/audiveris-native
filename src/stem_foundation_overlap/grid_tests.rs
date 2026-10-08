// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::shared_sig::Phase;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/official-real-grid-overlap/observed.json"
    ))
    .unwrap()
}
#[test]
fn original_grid_connector_brace_glyph_area_bounds_support_and_relation_identity_contracts() {
    for case in fixture()["cases"].as_array().unwrap() {
        let mut graph = Graph::default();
        let mut geometry = BTreeMap::new();
        let mut keys = Vec::new();
        for (index, n) in case["nodes"].as_array().unwrap().iter().enumerate() {
            let k = Key {
                system: 1,
                phase: if n["kind"] == "HeadInter" {
                    Phase::Head
                } else {
                    Phase::Grid
                },
                local: index + 1,
            };
            let id = graph.vertex(k, n.clone(), false).unwrap();
            let glyph = (!n["glyph"].is_null())
                .then(|| beam_dispatch::glyph_from_json(&n["glyph"]).unwrap());
            let g = Geometry::new(n, glyph.as_ref()).unwrap();
            assert_eq!(json!(g.core.map(|v| v as i32)), n["core_bounds"]);
            geometry.insert(id as u64, g);
            keys.push(k);
        }
        assert_eq!(
            json!([
                geometry[&1].overlaps(&geometry[&2]).unwrap(),
                geometry[&2].overlaps(&geometry[&1]).unwrap()
            ]),
            case["directional_overlaps"],
            "{}",
            case["name"]
        );
        if let Some(kind) = case["support_kind"].as_str() {
            graph
                .relations(
                    1,
                    Phase::Grid,
                    vec![(keys[0], keys[1], json!({"kind":kind,"manual":true}))],
                )
                .unwrap();
        }
        let before = graph.snapshot();
        assert_eq!(
            Ownership::new(&before).unwrap().no_support(1, 2).unwrap(),
            case["no_support"].as_bool().unwrap()
        );
        discover_graph(&mut graph, &geometry).unwrap();
        let snapshot = graph.snapshot();
        let exclusions=snapshot["relations"].as_array().unwrap().iter().filter(|e|e["kind"]=="Exclusion").map(|e|json!({"source":if e["source"]==1 {"a"} else {"b"},"target":if e["target"]==1 {"a"} else {"b"},"cause":e["cause"]})).collect::<Vec<_>>();
        assert_eq!(json!(exclusions), case["exclusions"], "{}", case["name"]);
        for relation in before["relations"].as_array().unwrap() {
            assert!(snapshot["relations"].as_array().unwrap().contains(relation));
        }
        discover_graph(&mut graph, &geometry).unwrap();
        assert_eq!(snapshot, graph.snapshot());
        assert_eq!(case["repeat_relation_identity_retained"], true);
        let unchanged = graph.snapshot();
        assert!(discover_graph(&mut graph, &BTreeMap::new()).is_err());
        assert_eq!(graph.snapshot(), unchanged);
    }
}
#[test]
fn real_image_original_grid_brace_physical_glyph_and_connector_area_geometry_are_retained() {
    let image = crate::GrayImage::decode(include_bytes!(
        "../../fixtures/official-grid/laura-gray.png"
    ))
    .unwrap();
    let page = crate::page_runtime::analyze(&image, "page-glyphs", None, None).unwrap();
    let snapshot = page.graph.snapshot();
    for original in fixture()["actual"].as_array().unwrap() {
        let matching = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| {
                n["origin"]["system"] == original["system"]
                    && n["kind"] == original["kind"]
                    && n["bounds"] == original["bounds"]
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "{} {}",
            original["kind"],
            original["bounds"]
        );
        let n = matching[0];
        for field in ["shape", "median", "width"] {
            assert_eq!(n[field], original[field]);
        }
        let glyph = if n["glyph_id"].is_null() {
            None
        } else {
            let registered = page.report["grid_ir"]["glyphs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["id"] == n["glyph_id"])
                .unwrap();
            let g=beam_dispatch::glyph_from_json(&json!({"bounds":[registered["left"],registered["top"],registered["width"],registered["height"]],"orientation":registered["orientation"],"runs":registered["runs"].as_array().unwrap().iter().map(|r|json!([r["position"],r["start"],r["length"]])).collect::<Vec<_>>()})).unwrap();
            assert_eq!(
                g,
                beam_dispatch::glyph_from_json(&original["glyph"]).unwrap()
            );
            Some(g)
        };
        let geometry = Geometry::new(n, glyph.as_ref()).unwrap();
        assert_eq!(
            json!(geometry.core.map(|v| v as i32)),
            original["core_bounds"]
        );
        assert_eq!(
            geometry.area.is_some(),
            original["kind"] == "BarConnectorInter"
        );
    }
    assert_eq!(page.graph.snapshot(), snapshot);
}
