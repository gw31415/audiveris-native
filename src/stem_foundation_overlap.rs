// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original SigReducer.detectOverlaps on the actual ordinary foundation owner.
//! Ordinary/special head size contacts; mirror/ensemble/text/slur remain pending.
use crate::{
    beam_dispatch,
    shared_sig::{Graph, Key},
    stem_image_owned::OwnedItems,
};
use audiveris_area::LinearArea;
use audiveris_staff_removal::{GlyphPixels, Orientation};
use serde_json::{Value, json};
use std::collections::BTreeMap;

type Rect = [f64; 4];
fn bounds(n: &Value) -> Result<Rect, String> {
    let b: [i32; 4] = serde_json::from_value(n["bounds"].clone())
        .map_err(|_| "foundation_overlap_bounds_required")?;
    if b.iter().any(|x| x.unsigned_abs() > 4_000_000) || b[2] <= 0 || b[3] <= 0 {
        return Err("invalid_foundation_overlap_bounds".into());
    }
    Ok(b.map(f64::from))
}
fn rect_intersects(a: Rect, b: Rect) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
fn iou(a: Rect, b: Rect) -> f64 {
    let w = (a[0] + a[2]).min(b[0] + b[2]) - a[0].max(b[0]);
    let h = (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1]);
    if w <= 0. || h <= 0. {
        return 0.;
    }
    let common = w * h;
    common / (a[2] * a[3] + b[2] * b[3] - common)
}
fn key(n: &Value) -> Result<Key, String> {
    serde_json::from_value(n["origin"].clone())
        .map_err(|_| "foundation_overlap_origin_required".into())
}
fn beam(n: &Value) -> bool {
    matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
}
fn run_rects(g: &GlyphPixels) -> Vec<Rect> {
    g.runs
        .iter()
        .map(|r| match g.orientation {
            Orientation::Vertical => [
                f64::from(g.left) + r.position as f64,
                f64::from(g.top) + r.start as f64,
                1.,
                r.length as f64,
            ],
            Orientation::Horizontal => [
                f64::from(g.left) + r.start as f64,
                f64::from(g.top) + r.position as f64,
                r.length as f64,
                1.,
            ],
        })
        .collect()
}
struct Geometry {
    bounds: Rect,
    core: Rect,
    runs: Option<Vec<Rect>>,
    area: Option<LinearArea>,
    implicit: bool,
}
impl Geometry {
    fn new(n: &Value, glyph: Option<&GlyphPixels>) -> Result<Self, String> {
        let b = bounds(n)?;
        let core = if n["kind"] == "HeadInter" {
            // HeadInter.shrink (0.5/0.5) -> Rectangle2D.getBounds floor/ceil.
            let x = (b[0] + b[2] / 4.).floor();
            let y = (b[1] + b[3] / 4.).floor();
            [
                x,
                y,
                (b[0] + 3. * b[2] / 4.).ceil() - x,
                (b[1] + 3. * b[3] / 4.).ceil() - y,
            ]
        } else {
            b
        };
        let area = if n["kind"] == "HeadInter" {
            None
        } else {
            let m: [f64; 4] = serde_json::from_value(n["median"].clone())
                .map_err(|_| "foundation_overlap_median_required")?;
            let thickness = n[if beam(n) { "height" } else { "width" }]
                .as_f64()
                .filter(|v| v.is_finite() && *v > 0. && *v <= 4096.)
                .ok_or("foundation_overlap_thickness_required")?;
            if m.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.)
                || (beam(n) && m[2] <= m[0])
                || (!beam(n) && m[3] <= m[1])
            {
                return Err("invalid_foundation_overlap_median".into());
            }
            let d = thickness / 2.;
            let points = if beam(n) {
                [
                    [m[0], m[1] - d],
                    [m[2], m[3] - d],
                    [m[2], m[3] + d],
                    [m[0], m[1] + d],
                ]
            } else {
                [
                    [m[0] - d, m[1]],
                    [m[0] + d, m[1]],
                    [m[2] + d, m[3]],
                    [m[2] - d, m[3]],
                ]
            };
            Some(LinearArea::polygon(&points)?)
        };
        Ok(Self {
            bounds: b,
            core,
            runs: glyph.map(run_rects),
            area,
            implicit: n["implicit"] == true || n["is_implicit"] == true,
        })
    }
    // AbstractInter.overlaps, preserving branch priority and run cells.
    fn overlaps(&self, other: &Self) -> Result<bool, String> {
        if self.implicit || other.implicit || !rect_intersects(self.core, other.core) {
            return Ok(false);
        }
        if let (Some(a), Some(b)) = (&self.runs, &other.runs) {
            return Ok(a.iter().any(|r| b.iter().any(|s| rect_intersects(*r, *s))));
        }
        if let Some(area) = &self.area {
            if let Some(other_area) = &other.area {
                return area.intersects_area(other_area);
            }
            if let Some(runs) = &other.runs {
                for r in runs {
                    if area.intersects_rect(*r)? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            return area.intersects_rect(other.bounds);
        }
        if let Some(runs) = &self.runs {
            return Ok(runs.iter().any(|r| rect_intersects(*r, other.bounds)));
        }
        if let Some(runs) = &other.runs {
            return Ok(runs.iter().any(|r| rect_intersects(*r, self.bounds)));
        }
        Ok(true)
    }
}
fn candidates(snapshot: &Value) -> Result<Vec<&Value>, String> {
    let mut nodes = Vec::new();
    for n in snapshot["inters"]
        .as_array()
        .ok_or("foundation_overlap_nodes_required")?
    {
        if n["removed"] == true
            || n["implicit"] == true
            || n["is_implicit"] == true
            || matches!(n["kind"].as_str(), Some("BeamGroupInter" | "LedgerInter"))
        {
            continue;
        }
        if !matches!(
            n["kind"].as_str(),
            Some("HeadInter" | "StemInter" | "BeamInter" | "BeamHookInter" | "BarlineInter")
        ) {
            return Err("foundation_overlap_class_dependency_pending".into());
        }
        if n["kind"] == "HeadInter"
            && !n["shape"]
                .as_str()
                .is_some_and(|s| crate::head_range_search::SUPPORTED_SHAPES.contains(&s))
        {
            return Err("foundation_overlap_special_head_dependency_pending".into());
        }
        if !n["mirror_id"].is_null() || !n["mirror"].is_null() || !n["member_ids"].is_null() {
            return Err("foundation_overlap_mirror_ensemble_pending".into());
        }
        key(n)?;
        bounds(n)?;
        nodes.push(n);
    }
    nodes.sort_by(|a, b| a["bounds"][0].as_i64().cmp(&b["bounds"][0].as_i64()));
    Ok(nodes)
}
fn discover_graph(graph: &mut Graph, geometry: &BTreeMap<u64, Geometry>) -> Result<Value, String> {
    let snapshot = graph.snapshot();
    let nodes = candidates(&snapshot)?;
    let mut staged = graph.clone();
    let mut inserted = Vec::new();
    for (i, a) in nodes.iter().enumerate() {
        for b in nodes.iter().skip(i + 1) {
            // Each original reducer runs on one system. Beam/BeamHook derives
            // AbstractBeamInter and is an accepted compatible overlap.
            if a["origin"]["system"] != b["origin"]["system"] || (beam(a) && beam(b)) {
                continue;
            }
            let ga = geometry
                .get(&a["id"].as_u64().ok_or("foundation_overlap_id_required")?)
                .ok_or("foundation_overlap_geometry_required")?;
            let gb = geometry
                .get(&b["id"].as_u64().ok_or("foundation_overlap_id_required")?)
                .ok_or("foundation_overlap_geometry_required")?;
            if iou(ga.bounds, gb.bounds) < 0.05 {
                if gb.bounds[0] > ga.bounds[0] + ga.bounds[2] {
                    // Interleaved systems require per-system scans: a farther
                    // other-system candidate never ends this system's search.
                    break;
                }
                continue;
            }
            let heads = a["kind"] == "HeadInter" && b["kind"] == "HeadInter";
            let overlaps = if heads {
                crate::head_registration::overlaps(a, b)?
                    && crate::head_registration::overlaps(b, a)?
            } else {
                ga.overlaps(gb)? && gb.overlaps(ga)?
            };
            // Original excludeOverlap exempts standard-size head/stem contacts.
            let standard_head_stem = [(a, b), (b, a)].iter().any(|(head, stem)| {
                head["kind"] == "HeadInter"
                    && stem["kind"] == "StemInter"
                    && !crate::head_registration::is_small_head(head["shape"].as_str().unwrap())
            });
            if overlaps
                && !standard_head_stem
                && staged.insert_foundation_contact(
                    key(a)?,
                    key(b)?,
                    "Exclusion",
                    Some("OVERLAP"),
                )?
            {
                inserted.push(json!([a["id"], b["id"]]));
            }
        }
    }
    *graph = staged;
    Ok(
        json!({"inserted_exclusions":inserted,"ordinary_detectOverlaps_connected":true,
        "special_head_size_contacts_connected":true,
        "scope":"actual glyph/core/line-area geometry, >=0.05 IOU, both directional overlaps, original small-head size exemption and support refusal before chord prolog; full-class/mirror/ensemble/header/frozen contracts pending","full_general_overlap_complete":false}),
    )
}

/// Resolve actual retained registries; never rasterize the Inter rectangle.
pub(crate) fn discover(owned: &mut OwnedItems) -> Result<Value, String> {
    let snapshot = owned.graph.snapshot();
    let nodes = candidates(&snapshot)?;
    let mut geometry = BTreeMap::new();
    let mut grid_ids = Vec::new();
    for n in nodes {
        let glyph = if n["glyph_id_namespace"] == "GRID" {
            let k = key(n)?;
            let grid = &owned.upstream_report["grid_ir"];
            let source = grid["bar_semantics"]["systems"]
                .as_array()
                .ok_or("foundation_overlap_owned_grid_required")?
                .iter()
                .find(|s| s["id"] == k.system)
                .and_then(|s| s["inters"].as_array())
                .and_then(|ns| ns.iter().find(|v| v["id"] == k.local))
                .ok_or("foundation_overlap_grid_origin_required")?;
            if [
                "kind",
                "bounds",
                "median",
                "width",
                "glyph_id",
                "source_peak",
            ]
            .iter()
            .any(|f| n[*f] != source[*f])
            {
                return Err("stale_foundation_overlap_grid_geometry".into());
            }
            let g = grid["glyphs"]
                .as_array()
                .ok_or("foundation_overlap_grid_registry_required")?
                .iter()
                .find(|v| v["id"] == n["glyph_id"])
                .ok_or("foundation_overlap_grid_glyph_required")?;
            let mut runs = Vec::new();
            for run in g["runs"]
                .as_array()
                .ok_or("foundation_overlap_grid_runs_required")?
            {
                runs.push(json!([run["position"], run["start"], run["length"]]));
            }
            let canonical = json!({"bounds":[g["left"],g["top"],g["width"],g["height"]],"orientation":g["orientation"],"runs":runs});
            grid_ids.push(n["glyph_id"].clone());
            Some(beam_dispatch::glyph_from_json(&canonical)?)
        } else if !n["glyph"].is_null() {
            let g = beam_dispatch::glyph_from_json(&n["glyph"])?;
            let id = n["glyph_id"]
                .as_u64()
                .and_then(|v| usize::try_from(v).ok())
                .ok_or("foundation_overlap_owned_glyph_id_required")?;
            if owned.prepared.registry.get(id)?.glyph != g {
                return Err("stale_foundation_overlap_physical_glyph".into());
            }
            if n["kind"] == "StemInter" {
                let k = key(n)?;
                owned
                    .caches
                    .get(&k.system)
                    .ok_or("foundation_overlap_cache_required")?
                    .registered_stem(&owned.graph, k.local)?;
            }
            Some(g)
        } else {
            None
        };
        geometry.insert(
            n["id"].as_u64().ok_or("foundation_overlap_id_required")?,
            Geometry::new(n, glyph.as_ref())?,
        );
    }
    let mut result = discover_graph(&mut owned.graph, &geometry)?;
    result["resolved_existing_GRID_glyph_ids"] = json!(grid_ids);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GrayImage, shared_sig::Phase};
    use std::sync::Arc;

    #[test]
    fn original_ordinary_overlap_geometry_contacts_and_exemptions() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-foundation-overlap/controls.json"
        ))
        .unwrap();
        let special: Value = serde_json::from_str(include_str!(
            "../fixtures/official-foundation-special-head-overlap/controls.json"
        ))
        .unwrap();
        for c in oracle["cases"]
            .as_array()
            .unwrap()
            .iter()
            .chain(special["cases"].as_array().unwrap())
        {
            if let Some(small) = c["shape_is_small_head"].as_bool() {
                assert_eq!(
                    crate::head_registration::is_small_head(
                        c["nodes"][0]["shape"].as_str().unwrap()
                    ),
                    small,
                    "{}",
                    c["name"]
                );
            }
            let mut graph = Graph::default();
            let mut geometry = BTreeMap::new();
            let mut keys = BTreeMap::new();
            let mut labels = BTreeMap::new();
            for (i, n) in c["nodes"].as_array().unwrap().iter().enumerate() {
                let k = Key {
                    system: 1,
                    phase: match n["kind"].as_str().unwrap() {
                        "HeadInter" => Phase::Head,
                        "StemInter" => Phase::Stem,
                        "BarlineInter" => Phase::Grid,
                        _ => Phase::Beam,
                    },
                    local: i + 1,
                };
                let mut data = n.clone();
                data.as_object_mut().unwrap().remove("core_bounds");
                // The GRID grade was already contextualized upstream. Its
                // overlap exclusion is not a new Support contribution.
                if n["kind"] == "BarlineInter" {
                    data["contextual_grade"] = n["grade"].clone();
                }
                let id = graph.vertex(k, data, false).unwrap();
                let label = n["label"].as_str().unwrap().to_owned();
                keys.insert(label.clone(), k);
                labels.insert(id, label);
                let g = if n["glyph"].is_null() {
                    None
                } else {
                    Some(beam_dispatch::glyph_from_json(&n["glyph"]).unwrap())
                };
                let geo = Geometry::new(n, g.as_ref()).unwrap();
                assert_eq!(
                    json!(geo.core.map(|v| v as i32)),
                    n["core_bounds"],
                    "{}",
                    c["name"]
                );
                geometry.insert(id as u64, geo);
            }
            let a = &geometry[&1];
            let b = &geometry[&2];
            assert_eq!(
                json!([a.overlaps(b).unwrap(), b.overlaps(a).unwrap()]),
                c["directional_overlaps"],
                "{}",
                c["name"]
            );
            for support in c["support"].as_array().unwrap() {
                graph
                    .relations(
                        1,
                        Phase::Beam,
                        vec![(
                            keys[support["source"].as_str().unwrap()],
                            keys[support["target"].as_str().unwrap()],
                            json!({"kind":support["kind"],"grade":support["grade"],
                                "source_ratio":support["source_ratio"],"target_ratio":support["target_ratio"]}),
                        )],
                    )
                    .unwrap();
            }
            let compare_context = |graph: &Graph, expected: &Value| {
                let snapshot = graph.snapshot();
                for (n, e) in snapshot["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(expected.as_array().unwrap())
                {
                    assert!(
                        (n["contextual_grade"].as_f64().unwrap() - e.as_f64().unwrap()).abs()
                            < 1e-12,
                        "{}",
                        c["name"]
                    );
                }
            };
            graph.contextualize_head_stems().unwrap();
            compare_context(&graph, &c["contextual_grades_before"]);
            discover_graph(&mut graph, &geometry).unwrap();
            graph.contextualize_head_stems().unwrap();
            compare_context(&graph, &c["contextual_grades_after"]);
            let snapshot = graph.snapshot();
            let actual: Vec<_> = snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "Exclusion")
                .map(|e| {
                    json!({"source":labels[&(e["source"].as_u64().unwrap() as usize)],
                    "target":labels[&(e["target"].as_u64().unwrap() as usize)],"cause":e["cause"]})
                })
                .collect();
            assert_eq!(json!(actual), c["exclusions"], "{}", c["name"]);
            assert_eq!(
                discover_graph(&mut graph, &geometry).unwrap()["inserted_exclusions"],
                json!([])
            );
            assert_eq!(snapshot, graph.snapshot());
            assert_eq!(c["repeat_is_idempotent"], true);
        }
        let mut empty = Graph::default();
        assert_eq!(
            discover_graph(&mut empty, &BTreeMap::new()).unwrap()["inserted_exclusions"],
            json!([])
        );
        // Original overlapMinIou is inclusive: positive box overlap alone is
        // still insufficient; actual glyph/core/area branches were checked above.
        assert_eq!(iou([0., 0., 21., 1.], [19., 0., 21., 1.]), 0.05);
    }

    #[test]
    fn owned_grid_pixels_cache_identity_and_invalid_late_input_are_atomic() {
        let image = GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-cross-system/beam-two-systems.png"
        ))
        .unwrap();
        let mut owned = crate::stem_image_head_linking::continue_process(
            crate::stem_image_owned::prepare_items(&image).unwrap(),
        )
        .unwrap();
        let snapshot = owned.graph.snapshot();
        let registry = owned.prepared.registry.snapshot();
        let stems: Vec<_> = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "StemInter")
            .map(|n| {
                let k = key(n).unwrap();
                (
                    k,
                    owned.caches[&k.system]
                        .registered_stem(&owned.graph, k.local)
                        .unwrap(),
                )
            })
            .collect();
        assert!(!stems.is_empty());
        let grid = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["glyph_id_namespace"] == "GRID")
            .unwrap();
        let id = grid["glyph_id"].clone();
        let glyphs = owned.upstream_report["grid_ir"]["glyphs"]
            .as_array_mut()
            .unwrap();
        let g = glyphs.iter_mut().find(|g| g["id"] == id).unwrap();
        let old_runs = g["runs"].clone();
        g["runs"] = json!([{ "position":0,"start":0,"length":0 }]);
        assert!(discover(&mut owned).is_err());
        assert_eq!(snapshot, owned.graph.snapshot());
        owned.upstream_report["grid_ir"]["glyphs"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|g| g["id"] == id)
            .unwrap()["runs"] = old_runs;

        // Unknown head shapes still refuse before exclusions can be committed.
        // Recognized small/stemless shapes use the original size branch above;
        // downstream chord/recognition dependencies remain separately guarded.
        let head = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "HeadInter")
            .unwrap();
        let mut special = head.clone();
        special["shape"] = json!("UNKNOWN_HEAD_SHAPE");
        owned
            .graph
            .vertex(key(head).unwrap(), special, false)
            .unwrap();
        let before = owned.graph.snapshot();
        assert_eq!(
            discover(&mut owned).unwrap_err(),
            "foundation_overlap_special_head_dependency_pending"
        );
        assert_eq!(before, owned.graph.snapshot());
        owned
            .graph
            .vertex(key(head).unwrap(), head.clone(), false)
            .unwrap();
        assert_eq!(snapshot, owned.graph.snapshot());
        let report = discover(&mut owned).unwrap();
        assert!(
            !report["resolved_existing_GRID_glyph_ids"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(registry, owned.prepared.registry.snapshot());
        for (k, stem) in stems {
            let current = owned.caches[&k.system]
                .registered_stem(&owned.graph, k.local)
                .unwrap();
            assert!(Arc::ptr_eq(&stem, &current));
            assert_eq!(stem.snapshot().unwrap(), current.snapshot().unwrap());
        }
        // The new boundary admits typed non-support exclusions, not arbitrary
        // external relations. Unsupported support still refuses atomically.
        owned
            .graph
            .relations(
                1,
                Phase::Head,
                vec![(
                    key(head).unwrap(),
                    key(grid).unwrap(),
                    json!({"kind":"UnknownSupport"}),
                )],
            )
            .unwrap();
        let before = owned.graph.snapshot();
        assert_eq!(
            owned.graph.contextualize_head_stems().unwrap_err(),
            "unsupported_head_stem_context_external_relation"
        );
        assert_eq!(before, owned.graph.snapshot());
    }
}
