// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original StemsRetriever.Finalizer beam-side support and SigReducer beam
//! endpoint checks, on actual retained SIG objects. Bounded single-beam groups.
use crate::shared_sig::{Graph, Key, Phase};
use serde_json::{Value, json};
fn key(n: &Value) -> Result<Key, String> {
    serde_json::from_value(n["origin"].clone()).map_err(|_| "owned beam origin required".into())
}
pub(crate) fn boost_sides(graph: &mut Graph) -> Result<Value, String> {
    graph.contextualize_head_stems()?;
    let snap = graph.snapshot();
    let nodes = snap["inters"].as_array().unwrap();
    let edges = snap["relations"].as_array().unwrap();
    let mut added = Vec::new();
    for b in nodes.iter().filter(|n| n["kind"] == "BeamInter") {
        // Original cross-system orphan purge has no work only when this beam
        // has actual stems and no counterpart sharing its physical glyph.
        if nodes.iter().any(|n| {
            n["kind"] == "BeamInter" && n["id"] != b["id"] && n["glyph_id"] == b["glyph_id"]
        }) {
            return Err("cross system beam finalizer pending".into());
        }
        if b["contextual_grade"]
            .as_f64()
            .or(b["grade"].as_f64())
            .unwrap()
            < 0.4
        {
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
                if !matches!(
                    h["shape"].as_str(),
                    Some("NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
                ) {
                    return Err("beam finalizer other shapes pending".into());
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
    graph.contextualize_head_stems()?;
    Ok(
        json!({"beam_side_supports":added,"scope":"StemsRetriever.Finalizer.boostBeamSides; linked unique-glyph single-beam groups have no cross-system orphan branch; full finalizer pending"}),
    )
}
/// SigReducer.checkBeamsHaveBothStems. Called after the current head/stem
/// consistency pass; removal uses actual identities and group containment.
pub(crate) fn check_ends(graph: &mut Graph) -> Result<Value, String> {
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
    graph.contextualize_head_stems()?;
    Ok(
        json!({"removed_beam_ids":removed,"scope":"SigReducer.checkBeamsHaveBothStems; hooks/siblings remain admission refusals"}),
    )
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
    graph.contextualize_head_stem_flags()?;
    Ok(ids)
}
/// PageCleaner.visit(AbstractBeamInter)/processArea in the restored HEADS
/// context. The certified Java2D AA-off normalized one-pixel contour branch
/// is rectangular only. Wider strokes/sloping restored borders stay pending.
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
    let margin = (interline as f64 * 0.1) as f32;
    let top = l[1] - h / 2.;
    let bottom = l[1] + h / 2.;
    if n["kind"] != "BeamInter"
        || l[1] != l[3]
        || l[0] >= l[2]
        || h <= 0.
        || margin.round() != 1.
        || [l[0], l[2], top, bottom]
            .iter()
            .any(|v| v.fract() != 0. || v.abs() > 4_000_000.)
    {
        return Err("beam PageCleaner sloping/fractional/wider stroke raster pending".into());
    }
    Ok((top as i64..=bottom as i64)
        .map(|y| [y, l[0] as i64, l[2] as i64 + 1])
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        beam["median"][3] = json!(210.);
        assert!(area_rows(&beam, 11).is_err());
        beam["median"][3] = beam["median"][1].clone();
        beam["height"] = json!(5.);
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
                    .filter(|n| n["kind"] == "BeamInter")
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
