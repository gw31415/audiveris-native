// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
//! StemsRetriever.Finalizer and StemsStep's ordered per-system continuation.
//! Runs only after all systems have linked; complete STEMS remains pending.
use crate::{
    beam_dispatch::{self, Registry},
    curves::Curve,
    shared_sig::{Graph, Key},
    system_area_geometry,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
struct System {
    id: u64,
    above: Vec<u64>,
    below: Vec<u64>,
    first: Curve,
    last: Curve,
}
fn geometry(
    grid: &Value,
    width: usize,
    height: usize,
    interline: usize,
) -> Result<Vec<System>, String> {
    let layout = system_area_geometry::build(grid, width, height, interline)?;
    let staffs = grid["staffs"]
        .as_array()
        .ok_or("beam finalizer staffs required")?;
    let mut curves = BTreeMap::new();
    let mut budget = 0;
    for s in staffs {
        let id = s["id"].as_u64().ok_or("beam finalizer staff ID required")?;
        curves.insert(
            id,
            (
                beam_dispatch::curve(&s["lines"][0], &mut budget)?,
                beam_dispatch::curve(&s["lines"][4], &mut budget)?,
            ),
        );
    }
    grid["systems"]
        .as_array()
        .unwrap()
        .iter()
        .zip(layout.system_neighbors)
        .map(|(s, n)| {
            let ids = s["staff_ids"].as_array().unwrap();
            Ok(System {
                id: s["id"].as_u64().unwrap(),
                above: serde_json::from_value(n["north"].clone())
                    .map_err(|_| "beam finalizer neighbors required")?,
                below: serde_json::from_value(n["south"].clone())
                    .map_err(|_| "beam finalizer neighbors required")?,
                first: curves[&ids[0].as_u64().unwrap()].0.clone(),
                last: curves[&ids.last().unwrap().as_u64().unwrap()].1.clone(),
            })
        })
        .collect()
}
fn physical_id(n: &Value, registry: &Registry) -> Result<Option<usize>, String> {
    if n["glyph_id"].is_null() {
        return Ok(None);
    }
    let id = n["glyph_id"]
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or("beam finalizer physical glyph ID required")?;
    if beam_dispatch::glyph_json(&registry.get(id)?.glyph) != n["glyph"] {
        return Err("beam finalizer canonical physical glyph required".into());
    }
    Ok(Some(id))
}
fn has_stems(g: &Value, b: &Value) -> bool {
    g["relations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["kind"] == "BeamStemRelation" && e["source"] == b["id"])
}
fn cross_beam<'a>(
    g: &'a Value,
    b: &Value,
    sys: &System,
    registry: &Registry,
) -> Result<Option<&'a Value>, String> {
    let Some(glyph) = physical_id(b, registry)? else {
        return Ok(None);
    };
    let bounds: [i32; 4] = serde_json::from_value(b["bounds"].clone())
        .map_err(|_| "beam finalizer actual bounds required")?;
    if bounds[2] <= 0 || bounds[3] <= 0 {
        return Err("beam finalizer actual bounds required".into());
    }
    // java.awt.Rectangle integer center; no floating median-center substitution.
    let x = i64::from(bounds[0]) + i64::from(bounds[2]) / 2;
    let y = i64::from(bounds[1]) + i64::from(bounds[3]) / 2;
    for (neighbors, outside) in [
        (&sys.above, (y as f64) < sys.first.y(x as f64)),
        (&sys.below, (y as f64) > sys.last.y(x as f64)),
    ] {
        if !outside {
            continue;
        }
        for id in neighbors {
            // Original searches BeamInter.class only, returns the first physical
            // match even if it is orphaned. A hook is not a BeamInter subclass.
            for n in g["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["kind"] == "BeamInter" && n["origin"]["system"] == *id)
            {
                if physical_id(n, registry)? == Some(glyph) {
                    return Ok(Some(n));
                }
            }
        }
    }
    Ok(None)
}
pub(crate) fn finalize(
    graph: &mut Graph,
    registry: &Registry,
    grid: &Value,
    width: usize,
    height: usize,
    interline: usize,
) -> Result<Value, String> {
    let systems = geometry(grid, width, height, interline)?;
    let before = graph.snapshot();
    for b in before["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter")))
    {
        if !systems.iter().any(|s| b["origin"]["system"] == s.id) {
            return Err("beam finalizer live system required".into());
        }
        physical_id(b, registry)?;
    }
    let mut next = graph.clone();
    let mut removed = Vec::new();
    let mut supports = Vec::new();
    let mut reports = Vec::new();
    for sys in systems {
        let snapshot = next.snapshot();
        let mut system_removed = Vec::new();
        let beams = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| {
                matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                    && n["origin"]["system"] == sys.id
            })
            .map(|n| n["id"].as_u64().unwrap())
            .collect::<Vec<_>>();
        for id in beams {
            let live = next.snapshot();
            let b = live["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == id)
                .ok_or("beam finalizer live beam required")?;
            if has_stems(&live, b) {
                continue;
            }
            if let Some(other) = cross_beam(&live, b, &sys, registry)?
                && has_stems(&live, other)
            {
                let event = json!({"beam":id,"system":sys.id,"counterpart":other["id"],"counterpart_system":other["origin"]["system"],"glyph_id":b["glyph_id"]});
                let key: Key = serde_json::from_value(b["origin"].clone())
                    .map_err(|_| "beam finalizer beam origin required")?;
                next.remove_music_inter(key)?;
                system_removed.push(event.clone());
                removed.push(event);
            }
        }
        let boost = crate::music_beams::boost_system_sides(&mut next, sys.id)?;
        supports.extend(
            boost["beam_side_supports"]
                .as_array()
                .unwrap()
                .iter()
                .cloned(),
        );
        reports.push(json!({"system":sys.id,"removed_cross_system_orphan_beams":system_removed,"contextualized_after_finalizer":true}));
    }
    *graph = next;
    Ok(
        json!({"removed_cross_system_orphan_beams":removed,"beam_side_supports":supports,"systems":reports,"scope":"original ordered all-systems StemsRetriever.Finalizer orphan purge/beam-side support and per-system StemsStep contextualization; no whole STEMS or positive full-image cross-system orphan parity claim","stems_complete":false,"epilog_complete":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GrayImage, shared_sig::Phase};
    #[test]
    fn original_two_system_orphan_conditions_use_owned_glyphs_neighbors_and_first_match() {
        let input = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-limited/quarter-half-break.png"
        ))
        .unwrap();
        let page = crate::page_runtime::analyze(&input, "page-glyphs", None, None).unwrap();
        assert!(page.success);
        let grid = &page.report["grid_ir"];
        let geo = geometry(grid, input.width, input.height, 11).unwrap();
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-cross-system/controls.json"
        ))
        .unwrap();
        for c in oracle["cases"].as_array().unwrap() {
            let current = c["current_system"].as_u64().unwrap();
            let other = c["other_system"].as_u64().unwrap();
            let sys = geo.iter().find(|s| s.id == current).unwrap();
            let x = c["current"]["center"][0].as_f64().unwrap();
            assert_eq!(sys.first.y(x), c["top_y_at_center"].as_f64().unwrap());
            assert_eq!(sys.last.y(x), c["bottom_y_at_center"].as_f64().unwrap());
            assert_eq!(
                json!(if current == 1 { &sys.below } else { &sys.above }),
                c["vertical_neighbors"]
            );
            let mut graph = Graph::default();
            let mut registry = Registry::default();
            let mut beam_keys = Vec::new();
            for (name, system, local) in [
                ("current", current, 1),
                ("other", other, 1),
                ("extra", other, 2),
            ] {
                let data = &c[name];
                if data.is_null() {
                    continue;
                }
                let glyph = beam_dispatch::glyph_from_json(&data["glyph"]).unwrap();
                let gid = registry.register(&glyph).unwrap();
                let key = Key {
                    system,
                    phase: Phase::Beam,
                    local,
                };
                let mut b = data.clone();
                b["glyph_id"] = json!(gid);
                b["glyph_id_namespace"] = json!("BEAMS");
                graph.vertex(key, b, false).unwrap();
                beam_keys.push(key);
                for link in data["stem_links"].as_array().unwrap() {
                    let sk = Key {
                        system,
                        phase: Phase::Stem,
                        local: 1,
                    };
                    if graph.id(sk).is_none() {
                        graph.vertex(sk, link["stem"].clone(), false).unwrap();
                    }
                    graph
                        .relations(
                            system,
                            Phase::Beam,
                            vec![(key, sk, link["relation"].clone())],
                        )
                        .unwrap();
                }
            }
            assert_eq!(
                graph.snapshot()["inters"][0]["glyph_id"]
                    == graph.snapshot()["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["origin"] == json!(beam_keys[1]))
                        .unwrap()["glyph_id"],
                c["same_physical_glyph"].as_bool().unwrap()
            );
            let before = graph.snapshot();
            assert!(
                finalize(
                    &mut graph,
                    &Registry::default(),
                    grid,
                    input.width,
                    input.height,
                    11
                )
                .is_err()
            );
            assert_eq!(graph.snapshot(), before);
            let report =
                finalize(&mut graph, &registry, grid, input.width, input.height, 11).unwrap();
            let live = graph.snapshot();
            for (i, field) in [(0, "current_removed"), (1, "other_removed")] {
                let id = graph.id(beam_keys[i]).unwrap();
                let removed = !live["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|n| n["id"] == id);
                assert_eq!(
                    removed,
                    c[field].as_bool().unwrap(),
                    "{} {field}",
                    c["name"]
                );
            }
            assert_eq!(
                report["removed_cross_system_orphan_beams"]
                    .as_array()
                    .unwrap()
                    .len(),
                usize::from(c["current_removed"].as_bool().unwrap())
            );
            for old in before["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["kind"] == "StemInter")
            {
                let n = live["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == old["id"])
                    .unwrap();
                for field in ["glyph", "bounds", "median", "grade", "origin"] {
                    assert_eq!(n[field], old[field]);
                }
            }
        }
    }
}
