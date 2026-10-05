// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded SigReducer.detectOverlaps and SIGraph.reduceExclusions at LINKS.
//! Rest/TimeWhole only, no support grade contributions or frozen conflicts.
use crate::{
    beam_dispatch,
    shared_sig::{Graph, Key, Phase},
};
use audiveris_staff_removal::Orientation;
use serde_json::{Value, json};
fn symbol(n: &Value) -> bool {
    matches!(n["kind"].as_str(), Some("RestInter" | "TimeWholeInter"))
}
fn key(n: &Value) -> Result<Key, String> {
    let k: Key =
        serde_json::from_value(n["origin"].clone()).map_err(|_| "music_symbol_origin_required")?;
    if k.phase != Phase::Symbol {
        return Err("music_symbol_origin_phase_required".into());
    }
    Ok(k)
}
fn overlap(a: &Value, b: &Value) -> Result<bool, String> {
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
            if !matches!(e["kind"].as_str(), Some("Containment" | "Exclusion")) {
                return Err("music_links_symbol_support_pending".into());
            }
            if e["kind"] == "Exclusion"
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
    let mut weak = Vec::new();
    for n in symbols {
        let grade = n["grade"]
            .as_f64()
            .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
            .ok_or("music_links_grade_required")?;
        if grade < 0.5 && n["frozen"] != true {
            next.remove_music_inter(key(n)?)?;
            weak.push(n["id"].clone());
        } else {
            let mut data = n.clone();
            data["contextual_grade"] = json!(grade);
            next.vertex(key(n)?, data, false)?;
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
        json!({"overlap_exclusions_inserted":overlaps,"removed_weak_symbol_ids":weak,"removed_conflict_symbol_ids":removed,"scope":"Rest/TimeWhole physical overlaps, no support/frozen conflicts; original strict best-grade exclusion order; containing empty chords cleaned by owner","full_LINKS_complete":false}),
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
