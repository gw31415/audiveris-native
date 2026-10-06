// SPDX-License-Identifier: AGPL-3.0-or-later
//! SigReducer.analyzeChords for retained ordinary black/void heads and stems.
//! Small/percussion/stemless size contracts require their pending dependencies.
use crate::shared_sig::{Graph, Key};
use serde_json::{Value, json};

fn rectangle(v: &Value) -> Result<[f64; 4], String> {
    let b: [f64; 4] = serde_json::from_value(v["bounds"].clone())
        .map_err(|_| "foundation_chord_bounds_required")?;
    if b.iter().any(|x| !x.is_finite() || x.abs() > 1_000_000.) || b[2] <= 0. || b[3] <= 0. {
        return Err("invalid_foundation_chord_bounds".into());
    }
    Ok(b)
}
fn key(v: &Value) -> Result<Key, String> {
    serde_json::from_value(v["origin"].clone())
        .map_err(|_| "foundation_chord_origin_required".into())
}
// java.awt.Rectangle2D.intersectsLine, including closed boundary contacts.
fn intersects(mut line: [f64; 4], b: [f64; 4]) -> bool {
    let outcode = |x: f64, y: f64| -> u8 {
        (if x < b[0] {
            1
        } else if x > b[0] + b[2] {
            4
        } else {
            0
        }) | (if y < b[1] {
            2
        } else if y > b[1] + b[3] {
            8
        } else {
            0
        })
    };
    let end = outcode(line[2], line[3]);
    if end == 0 {
        return true;
    }
    loop {
        let start = outcode(line[0], line[1]);
        if start == 0 {
            return true;
        }
        if start & end != 0 {
            return false;
        }
        if start & 5 != 0 {
            let x = if start & 4 != 0 { b[0] + b[2] } else { b[0] };
            line[1] += (x - line[0]) * (line[3] - line[1]) / (line[2] - line[0]);
            line[0] = x;
        } else {
            let y = if start & 8 != 0 { b[1] + b[3] } else { b[1] };
            line[0] += (y - line[1]) * (line[2] - line[0]) / (line[3] - line[1]);
            line[1] = y;
        }
    }
}
fn intersects_head(stem: &Value, head: &Value) -> Result<bool, String> {
    let median: [f64; 4] = serde_json::from_value(stem["median"].clone())
        .map_err(|_| "foundation_chord_median_required")?;
    if median
        .iter()
        .any(|x| !x.is_finite() || x.abs() > 1_000_000.)
    {
        return Err("invalid_foundation_chord_median".into());
    }
    let a = rectangle(stem)?;
    let b = rectangle(head)?;
    let area = (a[0] + a[2]).min(b[0] + b[2]) - a[0].max(b[0]);
    let height = (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1]);
    if area <= 0. || height <= 0. || !intersects(median, b) {
        return Ok(false);
    }
    let intersection = area * height;
    Ok(intersection / (a[2] * a[3] + b[2] * b[3] - intersection) >= 0.02)
}

/// Adds native support/exclusions to the retained SIG; no selected-grade API.
pub(crate) fn analyze(graph: &mut Graph) -> Result<Value, String> {
    let input = graph.snapshot();
    crate::stem_head_foundation_plan::validate(&input)?;
    let nodes = input["inters"].as_array().unwrap();
    let edges = input["relations"].as_array().unwrap();
    let mut stems = nodes
        .iter()
        .filter(|n| n["kind"] == "StemInter")
        .collect::<Vec<_>>();
    stems.sort_by(|a, b| {
        b["grade"]
            .as_f64()
            .unwrap()
            .total_cmp(&a["grade"].as_f64().unwrap())
    });
    let mut heads = nodes
        .iter()
        .filter(|n| n["kind"] == "HeadInter")
        .collect::<Vec<_>>();
    for n in &heads {
        rectangle(n)?;
    }
    heads.sort_by(|a, b| {
        a["bounds"][0]
            .as_f64()
            .unwrap()
            .total_cmp(&b["bounds"][0].as_f64().unwrap())
    });
    let mut staged = graph.clone();
    let mut added = Vec::new();
    for stem in stems {
        if stem["grade"].as_f64().unwrap() < 0.35 {
            break;
        }
        let linked = edges
            .iter()
            .filter(|e| e["kind"] == "HeadStemRelation" && e["target"] == stem["id"])
            .map(|e| e["source"].as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        // SigReducer.intersectedHeads uses median/box intersection AND >=0.02
        // bounding-box IOU; already linked heads are removed from this list.
        for head in heads.iter().filter(|h| {
            h["origin"]["system"] == stem["origin"]["system"]
                && !linked.contains(&h["id"].as_u64().unwrap())
        }) {
            if intersects_head(stem, head)?
                && staged.insert_foundation_contact(
                    key(stem)?,
                    key(head)?,
                    "Exclusion",
                    Some("OVERLAP"),
                )?
            {
                added.push(json!({"kind":"Exclusion","cause":"OVERLAP","endpoints":[stem["id"],head["id"]]}));
            }
        }
        // Original duration buckets preserve first HeadStem edge/member order.
        let mut duration_groups: Vec<(&str, Vec<&Value>)> = Vec::new();
        for relation in edges
            .iter()
            .filter(|e| e["kind"] == "HeadStemRelation" && e["target"] == stem["id"])
        {
            let head = nodes
                .iter()
                .find(|n| n["id"] == relation["source"])
                .unwrap();
            let shape = head["shape"].as_str().unwrap();
            if let Some((_, members)) = duration_groups.iter_mut().find(|(s, _)| *s == shape) {
                if !members.iter().any(|n| n["id"] == head["id"]) {
                    members.push(head);
                }
            } else {
                duration_groups.push((shape, vec![head]));
            }
        }
        for (i, (_, members)) in duration_groups.iter().enumerate() {
            for (_, others) in duration_groups.iter().skip(i + 1) {
                for a in members {
                    for b in others {
                        if staged.insert_foundation_contact(
                            key(a)?,
                            key(b)?,
                            "Exclusion",
                            Some("INCOMPATIBLE"),
                        )? {
                            added.push(json!({"kind":"Exclusion","cause":"INCOMPATIBLE","endpoints":[a["id"],b["id"]]}));
                        }
                    }
                }
            }
            for (j, a) in members.iter().enumerate() {
                for b in members.iter().skip(j + 1) {
                    if staged.insert_foundation_contact(
                        key(a)?,
                        key(b)?,
                        "HeadHeadRelation",
                        None,
                    )? {
                        added.push(json!({"kind":"HeadHeadRelation","cause":null,"endpoints":[a["id"],b["id"]]}));
                    }
                }
            }
        }
    }
    *graph = staged;
    Ok(
        json!({"added_relations":added,"scope":"original good-intrinsic-stem threshold/median and inclusive IOU, unlinked-head overlap, ordinary head duration incompatibility/HeadHead support; small/percussion/stemless size branches pending","full_analyzeChords_complete":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_sig::Phase;
    use std::collections::BTreeMap;
    #[test]
    fn original_ordinary_chord_support_duration_conflict_and_unlinked_intersection() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-foundation-chords/controls.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let mut keys = BTreeMap::new();
            let mut labels = BTreeMap::new();
            for (i, n) in case["nodes"].as_array().unwrap().iter().enumerate() {
                let label = n["label"].as_str().unwrap().to_owned();
                let k = Key {
                    system: 1,
                    phase: if n["kind"] == "StemInter" {
                        Phase::Stem
                    } else {
                        Phase::Head
                    },
                    local: i + 1,
                };
                let mut data = n.clone();
                data.as_object_mut().unwrap().remove("label");
                let id = graph.vertex(k, data, false).unwrap();
                keys.insert(label.clone(), k);
                labels.insert(id, label);
            }
            for (i, r) in case["head_connections"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let mut data = r.clone();
                data.as_object_mut().unwrap().remove("source");
                data.as_object_mut().unwrap().remove("target");
                data["local_relation_id"] = json!(i + 1);
                graph
                    .relations(
                        1,
                        Phase::Stem,
                        vec![(
                            keys[r["source"].as_str().unwrap()],
                            keys[r["target"].as_str().unwrap()],
                            data,
                        )],
                    )
                    .unwrap();
            }
            graph.contextualize_head_stems().unwrap();
            let added = analyze(&mut graph).unwrap();
            let snapshot = graph.snapshot();
            let mut actual = Vec::new();
            for r in snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["kind"] == "Exclusion" || r["kind"] == "HeadHeadRelation")
            {
                let mut comparable = json!({"kind":r["kind"],"source":labels[&(r["source"].as_u64().unwrap() as usize)],
                    "target":labels[&(r["target"].as_u64().unwrap() as usize)]});
                if r["kind"] == "Exclusion" {
                    comparable["cause"] = r["cause"].clone();
                }
                actual.push(comparable);
            }
            // Native stores the default Support metadata; original getter
            // ratios are checked through the real contextual grades below.
            let mut expected_relations = case["new_relations"].clone();
            for r in expected_relations.as_array_mut().unwrap() {
                for field in ["grade", "source_ratio", "target_ratio"] {
                    r.as_object_mut().unwrap().remove(field);
                }
            }
            assert_eq!(json!(actual), expected_relations);
            graph.contextualize_head_stems().unwrap();
            let after = graph.snapshot();
            for expected in case["contextual"].as_array().unwrap() {
                let id = graph.id(keys[expected["label"].as_str().unwrap()]).unwrap();
                let n = after["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| n["id"] == id)
                    .unwrap();
                assert!(
                    (n["contextual_grade"].as_f64().unwrap()
                        - expected["contextual_grade"].as_f64().unwrap())
                    .abs()
                        < 1e-12
                );
            }
            assert!(!added["added_relations"].as_array().unwrap().is_empty());
            assert!(
                analyze(&mut graph).unwrap()["added_relations"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(graph.snapshot(), after);
            // Existing original contact prevents inserting its opposite kind.
            assert!(
                !graph
                    .insert_foundation_contact(
                        keys["head0"],
                        keys["head1"],
                        if case["name"] == "same-duration" {
                            "Exclusion"
                        } else {
                            "HeadHeadRelation"
                        },
                        if case["name"] == "same-duration" {
                            Some("INCOMPATIBLE")
                        } else {
                            None
                        }
                    )
                    .unwrap()
            );
        }
        // Inclusive original IOU threshold and actual median intersection.
        let stem = json!({"median":[1.5,-400.,1.5,10.],"bounds":[1.,-400.,1.,410.]});
        let head = json!({"bounds":[0.,0.,10.,10.]});
        assert!(intersects_head(&stem, &head).unwrap());
        let mut below = stem.clone();
        below["bounds"][3] = json!(411.);
        assert!(!intersects_head(&below, &head).unwrap());
        below["median"] = json!([-1., -400., -1., 10.]);
        assert!(!intersects_head(&below, &head).unwrap());
    }
}
