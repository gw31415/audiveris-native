// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original StemsRetriever.Finalizer beam-side support and SigReducer beam
//! endpoint checks, on actual retained SIG objects. Bounded ordinary beam/hook groups.
use crate::shared_sig::{Graph, Key, Phase};
use serde_json::{Value, json};
mod raster;
fn key(n: &Value) -> Result<Key, String> {
    serde_json::from_value(n["origin"].clone()).map_err(|_| "owned beam origin required".into())
}
/// AbstractBeamInter.getCompetingHook: an Exclusion neighbor with the same
/// actual physical glyph, not merely overlapping geometry or equal bounds.
pub(crate) fn competing_hook(snapshot: &Value, beam: &Value) -> Option<usize> {
    if beam["kind"] != "BeamInter" || !beam["glyph_id"].is_u64() {
        return None;
    }
    for edge in snapshot["relations"].as_array()? {
        if edge["kind"] != "Exclusion" {
            continue;
        }
        let other = if edge["source"] == beam["id"] {
            &edge["target"]
        } else if edge["target"] == beam["id"] {
            &edge["source"]
        } else {
            continue;
        };
        if let Some(hook) = snapshot["inters"].as_array()?.iter().find(|n| {
            n["id"] == *other
                && n["kind"] == "BeamHookInter"
                && n["shape"] == "BEAM_HOOK"
                && n["glyph_id"] == beam["glyph_id"]
                && n["origin"]["system"] == beam["origin"]["system"]
        }) {
            return hook["origin"]["local"].as_u64().map(|v| v as usize);
        }
    }
    None
}
pub(crate) fn boost_sides(graph: &mut Graph) -> Result<Value, String> {
    boost_sides_impl(graph, None, true)
}
pub(crate) fn boost_system_sides(graph: &mut Graph, system: u64) -> Result<Value, String> {
    boost_sides_impl(graph, Some(system), false)
}
fn boost_sides_impl(
    graph: &mut Graph,
    system: Option<u64>,
    reject_aliases: bool,
) -> Result<Value, String> {
    let contextualize = |graph: &mut Graph| match system {
        Some(id) => graph.contextualize_system_head_stems(id),
        None => graph.contextualize_head_stems(),
    };
    contextualize(graph)?;
    let snap = graph.snapshot();
    let nodes = snap["inters"].as_array().unwrap();
    let edges = snap["relations"].as_array().unwrap();
    let mut added = Vec::new();
    for b in nodes.iter().filter(|n| {
        matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
            && system.is_none_or(|id| n["origin"]["system"] == id)
    }) {
        // Original cross-system orphan purge has no work only when this beam
        // has actual stems and no counterpart sharing its physical glyph.
        if reject_aliases
            && nodes.iter().any(|n| {
                matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter"))
                    && n["id"] != b["id"]
                    && n["glyph_id"] == b["glyph_id"]
                    && n["origin"]["system"] != b["origin"]["system"]
            })
        {
            return Err("cross system beam finalizer pending".into());
        }
        // AbstractBeamInter.isGood overrides the general contextual-grade
        // check: Finalizer uses intrinsic Grades.goodBeamGrade (0.35).
        if b["grade"].as_f64().ok_or("beam intrinsic grade required")? < 0.35 {
            continue;
        }
        for bs in edges.iter().filter(|e| {
            e["kind"] == "BeamStemRelation"
                && e["source"] == b["id"]
                && e["beam_portion"] != "CENTER"
        }) {
            for hs in edges
                .iter()
                .filter(|e| e["kind"] == "HeadStemRelation" && e["target"] == bs["target"])
            {
                let h = nodes
                    .iter()
                    .find(|n| n["id"] == hs["source"])
                    .ok_or("beam finalizer head required")?;
                let shape = h["shape"]
                    .as_str()
                    .ok_or("beam finalizer head shape required")?;
                if !crate::head_seed_search::ALL_STEM_SHAPES.contains(&shape) {
                    return Err("beam finalizer other shapes pending".into());
                }
                if matches!(shape, "NOTEHEAD_BLACK_SMALL" | "NOTEHEAD_VOID_SMALL") {
                    continue; // Original Shape.isSmallHead.
                }
                let grade = 0.5 * (bs["grade"].as_f64().unwrap() + hs["grade"].as_f64().unwrap());
                let rel = json!({"kind":"BeamHeadRelation","grade":grade,"on_beam_side":true,"source_ratio":1.,"target_ratio":1.+grade});
                graph.relations(
                    key(b)?.system,
                    Phase::Beam,
                    vec![(key(b)?, key(h)?, rel.clone())],
                )?;
                added.push(json!({"beam":b["id"],"head":h["id"],"relation":rel}));
            }
        }
    }
    contextualize(graph)?;
    Ok(
        json!({"beam_side_supports":added,"scope":"StemsRetriever.Finalizer.boostBeamSides; standalone legacy boundary retains alias refusal, owned image epilog validates/purges aliases before system boosting; full STEMS pending"}),
    )
}
/// Combined original beam/hook control boundary. The image-owned adapter calls
/// the checks separately and contextualizes/purges after each.
#[cfg(test)]
pub(crate) fn check_ends(graph: &mut Graph) -> Result<Value, String> {
    let mut removed = check_hooks(graph)?;
    removed.extend(check_beams(graph)?);
    graph.contextualize_head_stems()?;
    Ok(
        json!({"removed_beam_ids":removed,"scope":"SigReducer.checkHooksHaveStem and checkBeamsHaveBothStems; legacy combined diagnostic boundary, image reduction contextualizes/purges between checks"}),
    )
}

pub(crate) fn check_beams(graph: &mut Graph) -> Result<Vec<Value>, String> {
    let snap = graph.snapshot();
    let nodes = snap["inters"].as_array().unwrap();
    let edges = snap["relations"].as_array().unwrap();
    let mut removed = Vec::new();
    for b in nodes.iter().filter(|n| n["kind"] == "BeamInter") {
        for side in ["LEFT", "RIGHT"] {
            if !edges.iter().any(|e| {
                e["kind"] == "BeamStemRelation"
                    && e["source"] == b["id"]
                    && e["beam_portion"] == side
            }) {
                graph.remove_music_inter(key(b)?)?;
                removed.push(b["id"].clone());
                break;
            }
        }
    }
    Ok(removed)
}

/// SigReducer.checkHooksHaveStem: one actual non-CENTER stem is enough.
pub(crate) fn check_hooks(graph: &mut Graph) -> Result<Vec<Value>, String> {
    let snap = graph.snapshot();
    let nodes = snap["inters"].as_array().unwrap();
    let edges = snap["relations"].as_array().unwrap();
    let mut removed = Vec::new();
    for b in nodes.iter().filter(|n| n["kind"] == "BeamHookInter") {
        if !edges.iter().any(|e| {
            e["kind"] == "BeamStemRelation"
                && e["source"] == b["id"]
                && e["beam_portion"] != "CENTER"
        }) {
            graph.remove_music_inter(key(b)?)?;
            removed.push(b["id"].clone());
        }
    }
    // AbstractEnsemble/Containment removal leaves an empty ensemble removable.
    for g in nodes.iter().filter(|n| n["kind"] == "BeamGroupInter") {
        if !graph.snapshot()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "Containment" && e["source"] == g["id"])
        {
            graph.remove_music_inter(key(g)?)?;
        }
    }
    Ok(removed)
}
/// BeamHeadCleaner.process removes temporary beam/head supports at LINKS.
/// Owned BeamStem relationships remain the source of chord beam membership.
pub(crate) fn clean_head_supports(graph: &mut Graph) -> Result<Vec<usize>, String> {
    let snap = graph.snapshot();
    let ids = snap["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "BeamHeadRelation")
        .map(|r| r["id"].as_u64().unwrap() as usize)
        .collect::<Vec<_>>();
    graph.remove_beam_head_supports(&ids)?;
    // Original BeamHeadCleaner.process removes edges only. It does not
    // contextualize/purge again; preserve grades until a later explicit pass.
    Ok(ids)
}
/// PageCleaner.visit(AbstractBeamInter)/processArea in the restored HEADS
/// context. The certified Java2D AA-off normalized one-pixel contour branch
/// is rectangular after contour normalization. Wider strokes and slopes that
/// cross normalized pixel rows stay pending.
pub(crate) fn rectangle_rows(b: [f64; 4], margin: f32) -> Result<Vec<[i64; 3]>, String> {
    if !(1.0..1.5).contains(&margin)
        || b.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || b[0] >= b[2]
        || b[1] >= b[3]
    {
        return Err("beam PageCleaner invalid/wider stroke rectangle pending".into());
    }
    // Java2D DMarlin NearestPixelQuarter for an AA-off normalized stroke:
    // floor(coord + .25) + .25. This thin contour paints that integer pixel.
    // In the certified pen-width branch it contains the rectangular fill too.
    let pixel = |v: f64| (v + 0.25).floor() as i64;
    Ok((pixel(b[1])..=pixel(b[3]))
        .map(|y| [y, pixel(b[0]), pixel(b[2]) + 1])
        .collect())
}
pub(crate) fn area_rows(n: &Value, interline: usize) -> Result<Vec<[i64; 3]>, String> {
    let restore = |v: &Value| -> Result<f64, String> {
        let x = v.as_f64().ok_or("beam eraser geometry required")?;
        crate::stem_head_reference_context::to_xml(Some(x))?
            .ok_or("beam eraser geometry required".to_owned())?
            .parse()
            .map_err(|_| "beam eraser rounded geometry required".into())
    };
    let l: [f64; 4] = [
        restore(&n["median"][0])?,
        restore(&n["median"][1])?,
        restore(&n["median"][2])?,
        restore(&n["median"][3])?,
    ];
    let h = restore(&n["height"])?;
    if !matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter")) {
        return Err("beam PageCleaner body required".into());
    }
    raster::rows(l, h, (interline as f64 * 0.1) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_beam_head_cleaner_retains_context_until_explicit_recontextualization() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-head-cleaner/observed.json"
        ))
        .unwrap();
        let mut graph = Graph::default();
        let head = Key {
            system: 1,
            phase: Phase::Head,
            local: 1,
        };
        let beam = Key {
            system: 1,
            phase: Phase::Beam,
            local: 1,
        };
        let hid = graph
            .vertex(
                head,
                json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK",
            "grade":oracle["head_intrinsic"],"bounds":[300,250,12,8],"glyph_id":11}),
                false,
            )
            .unwrap();
        let bid = graph
            .vertex(
                beam,
                json!({"kind":"BeamInter","shape":"BEAM",
            "grade":oracle["beam_intrinsic"],"bounds":[290,208,60,4],"glyph_id":12}),
                false,
            )
            .unwrap();
        graph
            .relations(
                1,
                Phase::Beam,
                vec![(
                    beam,
                    head,
                    json!({"kind":"BeamHeadRelation",
            "grade":oracle["relation_grade"],"on_beam_side":true,
            "source_ratio":1.,"target_ratio":1.+oracle["relation_grade"].as_f64().unwrap()}),
                )],
            )
            .unwrap();
        graph.contextualize_head_stems().unwrap();
        let observe = |g: &Graph| {
            let snapshot = g.snapshot();
            let nodes = snapshot["inters"].as_array().unwrap();
            let h = nodes.iter().find(|n| n["id"] == hid).unwrap();
            let b = nodes.iter().find(|n| n["id"] == bid).unwrap();
            json!({"head_grade":h["grade"],"head_contextual_grade":h["contextual_grade"],
                "beam_grade":b["grade"],"beam_contextual_grade":b["contextual_grade"],
                "head_removed":false,"beam_removed":false,"live_vertices":nodes.len(),
                "live_relations":snapshot["relations"].as_array().unwrap().len()})
        };
        assert_eq!(observe(&graph), oracle["before"]);
        let before = graph.snapshot();
        assert_eq!(clean_head_supports(&mut graph).unwrap().len(), 1);
        assert_eq!(observe(&graph), oracle["after"]);
        assert_eq!(graph.snapshot()["inters"], before["inters"]);
        let after = graph.snapshot();
        assert!(clean_head_supports(&mut graph).unwrap().is_empty());
        assert_eq!(graph.snapshot(), after);
        // A later explicitly ordered contextualization remains available.
        graph.contextualize_head_stems().unwrap();
        assert_eq!(
            observe(&graph)["head_contextual_grade"],
            oracle["head_intrinsic"]
        );
    }
    #[test]
    fn original_beam_side_support_uses_intrinsic_good_grade() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-epilog/groups-epilog.json"
        ))
        .unwrap();
        for c in oracle["grade_controls"].as_array().unwrap() {
            let mut graph = Graph::default();
            let beam = Key {
                system: 1,
                phase: Phase::Beam,
                local: 1,
            };
            let stem = Key {
                system: 1,
                phase: Phase::Stem,
                local: 1,
            };
            let head = Key {
                system: 1,
                phase: Phase::Head,
                local: 1,
            };
            graph.vertex(beam,json!({"kind":"BeamInter","glyph_id":1,"grade":c["grade"],"contextual_grade":c["declared_contextual_grade"]}),false).unwrap();
            graph
                .vertex(stem, json!({"kind":"StemInter","grade":0.8}), false)
                .unwrap();
            graph
                .vertex(
                    head,
                    json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK","grade":0.8}),
                    false,
                )
                .unwrap();
            graph.relations(1,Phase::Beam,vec![(beam,stem,json!({"kind":"BeamStemRelation","beam_portion":"LEFT","grade":1.,"source_ratio":5.,"target_ratio":5.}))]).unwrap();
            graph.relations(1,Phase::Stem,vec![(head,stem,json!({"kind":"HeadStemRelation","local_relation_id":1,"grade":1.,"source_ratio":5.,"target_ratio":11.}))]).unwrap();
            // One side/head subset: original image controls have five supports.
            let result = boost_sides(&mut graph).unwrap();
            assert_eq!(
                result["beam_side_supports"].as_array().unwrap().len(),
                usize::from(c["is_good"].as_bool().unwrap())
            );
            assert!(
                graph.snapshot()["inters"][0]["contextual_grade"]
                    .as_f64()
                    .unwrap()
                    > 0.4
            );
        }
    }
    #[test]
    fn original_hook_area_and_one_side_reduction_retain_registered_endpoints() {
        let areas: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-hooks/areas.json"
        ))
        .unwrap();
        for c in areas["cases"].as_array().unwrap() {
            let n = json!({"kind":c["kind"],"median":c["median"],"height":c["height"]});
            assert_eq!(json!(area_rows(&n, 11).unwrap()), c["white_rows"]);
        }
        let controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-hooks/competition.json"
        ))
        .unwrap();
        for c in controls["hook_endpoint_controls"].as_array().unwrap() {
            let mut graph = Graph::default();
            let beam = Key {
                system: 1,
                phase: Phase::Beam,
                local: 1,
            };
            let stem = Key {
                system: 1,
                phase: Phase::Stem,
                local: 1,
            };
            let bid = graph
                .vertex(
                    beam,
                    json!({"kind":"BeamHookInter","shape":"BEAM_HOOK","grade":0.8}),
                    false,
                )
                .unwrap();
            let sid = graph
                .vertex(stem, json!({"kind":"StemInter","grade":0.8}), false)
                .unwrap();
            if c["portion"] != "NONE" {
                let portion = &c["portion"];
                let target = if portion == "CENTER" { 4. } else { 11. };
                graph.relations(1,Phase::Beam,vec![(beam,stem,json!({"kind":"BeamStemRelation","beam_portion":portion,"grade":1.,"source_ratio":5.,"target_ratio":target}))]).unwrap();
            }
            check_ends(&mut graph).unwrap();
            let snap = graph.snapshot();
            let nodes = snap["inters"].as_array().unwrap();
            assert_eq!(
                !nodes.iter().any(|n| n["id"] == bid),
                c["hook_removed"].as_bool().unwrap()
            );
            assert!(nodes.iter().any(|n| n["id"] == sid));
            assert!(
                check_ends(&mut graph).unwrap()["removed_beam_ids"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
    }
    #[test]
    fn original_page_cleaner_beam_area_contour_and_uncertified_branches() {
        let builder: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beams/builder-observed.json"
        ))
        .unwrap();
        let observed: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beams/continuation-observed.json"
        ))
        .unwrap();
        let mut beam = json!({"kind":"BeamInter","median":builder["cases"][0]["beam_median"],"height":builder["cases"][0]["beam_height"]});
        assert_eq!(
            json!(area_rows(&beam, 11).unwrap()),
            observed["systems"][0]["PageCleaner_beam_white_rows"]
        );
        let controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-subdivisions/raster-controls.json"
        ))
        .unwrap();
        for case in controls["cases"].as_array().unwrap() {
            let bounds = serde_json::from_value(case["area_bounds"].clone()).unwrap();
            assert_eq!(
                json!(rectangle_rows(bounds, 1.1).unwrap()),
                case["white_rows"]
            );
        }
        let controls: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-groups/raster-controls.json"
        ))
        .unwrap();
        for c in controls["cases"].as_array().unwrap() {
            let n = json!({"kind":"BeamInter","median":c["median"],"height":c["height"]});
            assert_eq!(json!(area_rows(&n, 11).unwrap()), c["white_rows"]);
        }
        let continuation: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-subdivisions/eighth-continuation.json"
        ))
        .unwrap();
        beam["median"] = json!([288., 209., 470., 209.]);
        beam["height"] = json!(6.1);
        assert_eq!(
            json!(area_rows(&beam, 11).unwrap()),
            continuation["systems"][0]["PageCleaner_beam_white_rows"]
        );
        beam["median"][3] = json!(210.);
        let slope: Value = serde_json::from_str(include_str!(
            "../fixtures/official-beam-sloped-raster/observed.json"
        ))
        .unwrap();
        assert_eq!(
            json!(area_rows(&beam, 11).unwrap()),
            slope["cases"][0]["white_rows"]
        );
        beam["median"][3] = beam["median"][1].clone();
        beam["height"] = json!(0.);
        assert!(area_rows(&beam, 11).is_err());
        beam["height"] = json!(6.);
        assert!(area_rows(&beam, 22).is_err());
    }
    #[test]
    fn missing_end_removes_actual_beam_group_but_retains_other_endpoint_objects() {
        for portions in [vec![], vec!["LEFT"], vec!["RIGHT"], vec!["LEFT", "RIGHT"]] {
            let mut g = Graph::default();
            let b = Key {
                system: 1,
                phase: Phase::Beam,
                local: 1,
            };
            let group = Key {
                system: 1,
                phase: Phase::Group,
                local: 1,
            };
            let bid = g
                .vertex(
                    b,
                    json!({"kind":"BeamInter","grade":0.8,"median":[0.,0.,10.,0.]}),
                    false,
                )
                .unwrap();
            g.vertex(
                group,
                json!({"kind":"BeamGroupInter","members":[bid]}),
                false,
            )
            .unwrap();
            g.relations(
                1,
                Phase::Group,
                vec![(group, b, json!({"kind":"Containment"}))],
            )
            .unwrap();
            for (i, p) in portions.iter().enumerate() {
                let s = Key {
                    system: 1,
                    phase: Phase::Stem,
                    local: i + 1,
                };
                g.vertex(s, json!({"kind":"StemInter","grade":0.8}), false)
                    .unwrap();
                g.relations(1,Phase::Beam,vec![(b,s,json!({"kind":"BeamStemRelation","beam_portion":p,"grade":1.,"source_ratio":5.,"target_ratio":11.}))]).unwrap();
            }
            let result = check_ends(&mut g).unwrap();
            let snap = g.snapshot();
            assert_eq!(
                result["removed_beam_ids"].as_array().unwrap().len(),
                usize::from(portions.len() != 2)
            );
            assert_eq!(
                snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|n| matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter")))
                    .count(),
                usize::from(portions.len() == 2)
            );
            assert_eq!(
                snap["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|n| n["kind"] == "StemInter")
                    .count(),
                portions.len()
            );
            assert!(
                check_ends(&mut g).unwrap()["removed_beam_ids"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
    }
}

#[cfg(test)]
mod raster_tests;
#[cfg(test)]
pub(crate) use raster_tests::assert_real_rasters;
