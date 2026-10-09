// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded SigReducer.detectOverlaps and SIGraph.reduceExclusions at LINKS.
//! Rest/TimeWhole, including incoming zero-target augmentation support; no frozen conflicts.
use crate::{
    beam_dispatch,
    shared_sig::{Graph, Key, Phase},
};
use audiveris_staff_removal::Orientation;
use serde_json::{Value, json};
fn symbol(n: &Value) -> bool {
    // SigReducer.reduce excludes Staff.getHeaderInters from overlap candidates.
    // The owned header phase is populated only by selected frozen time imports.
    let header = n["kind"] == "TimeWholeInter"
        && n["origin"]["phase"] == "MUSIC_TIME"
        && n["frozen"] == true;
    !header && matches!(n["kind"].as_str(), Some("RestInter" | "TimeWholeInter"))
}
fn key(n: &Value) -> Result<Key, String> {
    let k: Key =
        serde_json::from_value(n["origin"].clone()).map_err(|_| "music_symbol_origin_required")?;
    if k.phase != Phase::Symbol {
        return Err("music_symbol_origin_phase_required".into());
    }
    Ok(k)
}
pub(crate) fn overlap(a: &Value, b: &Value) -> Result<bool, String> {
    let rect = |n: &Value| -> Result<[i64; 4], String> {
        serde_json::from_value(n["bounds"].clone())
            .map_err(|_| "music_symbol_bounds_required".into())
    };
    let a_box = rect(a)?;
    let b_box = rect(b)?;
    if [a_box[0], a_box[1], b_box[0], b_box[1]]
        .iter()
        .any(|n| !(-4_000_000..=4_000_000).contains(n))
    {
        return Err("music_symbol_overlap_geometry_pending".into());
    }
    if [a_box[2], a_box[3], b_box[2], b_box[3]]
        .iter()
        .any(|n| *n <= 0 || *n > 4096)
    {
        return Err("music_symbol_overlap_geometry_pending".into());
    }
    let w = (a_box[0] + a_box[2]).min(b_box[0] + b_box[2]) - a_box[0].max(b_box[0]);
    let h = (a_box[1] + a_box[3]).min(b_box[1] + b_box[3]) - a_box[1].max(b_box[1]);
    if w <= 0 || h <= 0 {
        return Ok(false);
    }
    let intersection = w * h;
    if intersection as f64 / ((a_box[2] * a_box[3] + b_box[2] * b_box[3] - intersection) as f64)
        < 0.05
    {
        return Ok(false);
    }
    let a_g = beam_dispatch::glyph_from_json(&a["glyph"])?;
    let b_g = beam_dispatch::glyph_from_json(&b["glyph"])?;
    if a_g.orientation != Orientation::Vertical || b_g.orientation != Orientation::Vertical {
        return Err("music_symbol_vertical_glyph_required".into());
    }
    // AbstractInter.overlaps -> Glyphs.intersect(false), not bounds alone.
    let pixels = crate::clef_candidates::pixels(&a_g).collect::<std::collections::BTreeSet<_>>();
    Ok(crate::clef_candidates::pixels(&b_g).any(|p| pixels.contains(&p)))
}
fn weak_rest_flag_exclusion(n: &Value, e: &Value, nodes: &[Value]) -> bool {
    if n["kind"] != "RestInter"
        || n["frozen"] == true
        || !n["grade"]
            .as_f64()
            .is_some_and(|g| g.is_finite() && (0.0..0.5).contains(&g))
    {
        return false;
    }
    let other_id = if e["source"] == n["id"] {
        &e["target"]
    } else {
        &e["source"]
    };
    let Some(flag) = nodes
        .iter()
        .find(|v| v["id"] == *other_id && v["kind"] == "FlagInter")
    else {
        return false;
    };
    flag["origin"]["phase"] == "SYMBOL"
        && flag["origin"]["system"] == n["origin"]["system"]
        && flag["contextual_grade"]
            .as_f64()
            .is_some_and(|g| g.is_finite() && (0.5..=1.0).contains(&g))
        && !n["glyph_id"].is_null()
        && flag["glyph_id"] == n["glyph_id"]
        && flag["glyph"] == n["glyph"]
}
pub(crate) fn reduce(graph: &mut Graph) -> Result<Value, String> {
    let mut next = graph.clone();
    let initial = graph.snapshot();
    let nodes = initial["inters"]
        .as_array()
        .ok_or("music_links_SIG_required")?;
    let symbols = nodes.iter().filter(|n| symbol(n)).collect::<Vec<_>>();
    for n in &symbols {
        key(n)?;
        for e in initial["relations"]
            .as_array()
            .ok_or("music_links_relations_required")?
        {
            if e["source"] != n["id"] && e["target"] != n["id"] {
                continue;
            }
            if e["kind"] == "AugmentationRelation"
                && n["kind"] == "RestInter"
                && e["target"] == n["id"]
                && e["target_ratio"] == 1.0
                && nodes
                    .iter()
                    .any(|v| v["id"] == e["source"] && v["kind"] == "AugmentationDotInter")
            {
                // Support.getTargetCoeff is zero for AugmentationRelation:
                // contextualization cannot rescue a weak rest interpretation.
                continue;
            }
            if !matches!(e["kind"].as_str(), Some("Containment" | "Exclusion")) {
                return Err("music_links_symbol_support_pending".into());
            }
            if e["kind"] == "Exclusion"
                && !weak_rest_flag_exclusion(n, e, nodes)
                && ["source", "target"]
                    .iter()
                    .any(|endpoint| !nodes.iter().any(|v| v["id"] == e[*endpoint] && symbol(v)))
            {
                return Err("music_symbol_cross_kind_exclusion_pending".into());
            }
        }
    }
    let mut sorted = symbols.clone();
    sorted.sort_by_key(|n| n["bounds"][0].as_i64());
    let mut overlaps = Vec::new();
    for (i, a) in sorted.iter().enumerate() {
        for b in &sorted[i + 1..] {
            let ak = key(a)?;
            let bk = key(b)?;
            if ak.system != bk.system || !overlap(a, b)? {
                continue;
            }
            if a["frozen"] == true || b["frozen"] == true {
                return Err("music_symbol_frozen_overlap_pending".into());
            }
            let snap = next.snapshot();
            if snap["relations"].as_array().unwrap().iter().any(|e| {
                e["kind"] == "Exclusion"
                    && ((e["source"] == a["id"] && e["target"] == b["id"])
                        || (e["target"] == a["id"] && e["source"] == b["id"]))
            }) {
                continue;
            }
            next.relations(
                ak.system,
                Phase::Symbol,
                vec![(ak, bk, json!({"kind":"Exclusion","cause":"OVERLAP"}))],
            )?;
            overlaps.push(json!([a["id"], b["id"]]));
        }
    }
    // Contextualize all targets before the first removal callback. Zero-target
    // augmentation and non-Support exclusions cannot raise a Rest grade.
    let mut grades = Vec::new();
    for n in &symbols {
        let grade = n["grade"]
            .as_f64()
            .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
            .ok_or("music_links_grade_required")?;
        let mut data = (*n).clone();
        data["contextual_grade"] = json!(grade);
        next.vertex(key(n)?, data, false)?;
        grades.push(grade);
    }
    let mut weak = Vec::new();
    for (n, grade) in symbols.iter().zip(grades) {
        if grade < 0.5 && n["frozen"] != true {
            next.remove_music_inter(key(n)?)?;
            weak.push(n["id"].clone());
        }
    }
    let mut removed = Vec::new();
    loop {
        let snap = next.snapshot();
        let nodes = snap["inters"].as_array().unwrap();
        let mut best: Option<(&Value, f64)> = None;
        for e in snap["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Exclusion")
        {
            let a = nodes.iter().find(|n| n["id"] == e["source"]).unwrap();
            let b = nodes.iter().find(|n| n["id"] == e["target"]).unwrap();
            if !symbol(a) && !symbol(b) {
                continue;
            }
            if !symbol(a) || !symbol(b) {
                return Err("music_symbol_cross_kind_exclusion_pending".into());
            }
            if a["frozen"] == true || b["frozen"] == true {
                return Err("music_symbol_frozen_overlap_pending".into());
            }
            let grade = a["contextual_grade"]
                .as_f64()
                .unwrap()
                .max(b["contextual_grade"].as_f64().unwrap());
            if grade > best.as_ref().map_or(0., |v| v.1) {
                best = Some((e, grade));
            }
        }
        let Some((e, _)) = best else { break };
        let a = nodes.iter().find(|n| n["id"] == e["source"]).unwrap();
        let b = nodes.iter().find(|n| n["id"] == e["target"]).unwrap();
        // Original strict comparison preserves source on ties; relation order
        // breaks equal best-edge grades. Containment supplies no grade boost.
        let weaker =
            if a["contextual_grade"].as_f64().unwrap() < b["contextual_grade"].as_f64().unwrap() {
                a
            } else {
                b
            };
        next.remove_music_inter(key(weaker)?)?;
        removed.push(weaker["id"].clone());
    }
    *graph = next;
    Ok(
        json!({"overlap_exclusions_inserted":overlaps,"removed_weak_symbol_ids":weak,"removed_conflict_symbol_ids":removed,"scope":"Rest/TimeWhole physical overlaps and incoming zero-target augmentation support, other support/frozen conflicts pending; original strict best-grade exclusion order; containing empty chords cleaned by owner","full_LINKS_complete":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn oracle() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-compound/observed.json"
        ))
        .unwrap()
    }
    fn add(graph: &mut Graph, local: usize, glyph: &Value, grade: f64) {
        graph.vertex(Key { system: 1, phase: Phase::Symbol, local }, json!({"kind":"RestInter","shape":"QUARTER_REST","grade":grade,"bounds":glyph["bounds"],"glyph":glyph}),false).unwrap();
    }
    #[test]
    fn original_compound_part_conflicts_preserve_stronger_and_source_on_tie() {
        let o = oracle();
        for c in o["conflict_controls"].as_array().unwrap() {
            let mut graph = Graph::default();
            add(
                &mut graph,
                1,
                &o["compound_glyph"],
                c["source_grade"].as_f64().unwrap(),
            );
            add(
                &mut graph,
                2,
                &o["source_part"],
                c["target_grade"].as_f64().unwrap(),
            );
            let report = reduce(&mut graph).unwrap();
            assert_eq!(
                report["overlap_exclusions_inserted"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            let s = graph.snapshot();
            for (id, field) in [(1, "source_removed"), (2, "target_removed")] {
                assert_eq!(
                    s["tombstones"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|n| n["id"] == id),
                    c[field].as_bool().unwrap()
                );
            }
            assert!(s["relations"].as_array().unwrap().is_empty());
        }
    }
    #[test]
    fn incoming_rest_augmentation_does_not_rescue_a_weak_target() {
        let f: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-augmentation/half-rest-original-support.json"
        ))
        .unwrap();
        let original = &f["cases"][0];
        let rk = Key {
            system: 1,
            phase: Phase::Symbol,
            local: 1,
        };
        let dk = Key { local: 2, ..rk };
        for case in original["declared_target_grade_controls"]
            .as_array()
            .unwrap()
        {
            let mut graph = Graph::default();
            let rest = graph.vertex(rk, json!({"kind":"RestInter","shape":"HALF_REST","bounds":original["rest_bounds"],"grade":case["declared_target_grade"]}), false).unwrap();
            graph.vertex(dk, json!({"kind":"AugmentationDotInter","shape":"AUGMENTATION_DOT","bounds":original["dot_bounds"],"grade":0.767}), false).unwrap();
            graph.relations(1, Phase::Symbol, vec![(dk,rk,json!({"kind":"AugmentationRelation","grade":0.701,"source_ratio":original["source_ratio"],"target_ratio":original["target_ratio"]}))]).unwrap();
            let mut unsupported = graph.clone();
            unsupported.relations(1, Phase::Symbol, vec![(dk,rk,json!({"kind":"AugmentationRelation","grade":0.701,"source_ratio":original["source_ratio"],"target_ratio":1.1}))]).unwrap();
            let before = unsupported.snapshot();
            assert_eq!(
                reduce(&mut unsupported).unwrap_err(),
                "music_links_symbol_support_pending"
            );
            assert_eq!(unsupported.snapshot(), before);
            let report = reduce(&mut graph).unwrap();
            let after = graph.snapshot();
            if case["original_contextual_target_grade"].as_f64().unwrap() < 0.5 {
                assert_eq!(report["removed_weak_symbol_ids"], json!([rest]));
                assert!(after["relations"].as_array().unwrap().is_empty());
                assert!(
                    after["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|n| n["id"] != rest)
                );
            } else {
                let rest = after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == rest)
                    .unwrap();
                assert_eq!(
                    rest["contextual_grade"],
                    case["original_contextual_target_grade"]
                );
                assert!(
                    report["removed_weak_symbol_ids"]
                        .as_array()
                        .unwrap()
                        .is_empty()
                );
            }
        }
    }
    #[test]
    fn unsupported_support_is_atomic_and_empty_input_has_no_fabricated_conflicts() {
        let o = oracle();
        let mut graph = Graph::default();
        assert!(
            reduce(&mut graph).unwrap()["removed_conflict_symbol_ids"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        add(&mut graph, 1, &o["compound_glyph"], 0.7);
        add(&mut graph, 2, &o["source_part"], 0.6);
        graph
            .relations(
                1,
                Phase::Symbol,
                vec![(
                    Key {
                        system: 1,
                        phase: Phase::Symbol,
                        local: 1,
                    },
                    Key {
                        system: 1,
                        phase: Phase::Symbol,
                        local: 2,
                    },
                    json!({"kind":"UnsupportedSymbolSupport"}),
                )],
            )
            .unwrap();
        let before = graph.snapshot();
        assert_eq!(
            reduce(&mut graph).unwrap_err(),
            "music_links_symbol_support_pending"
        );
        assert_eq!(graph.snapshot(), before);
    }
}
