// SPDX-License-Identifier: AGPL-3.0-or-later
//! HeadChordInter.getBounds/getLeadingNote/computeLocations, integer Rectangle
//! and Point semantics. A stem's full box is not the chord's head geometry.
use serde_json::Value;

pub(crate) struct Geometry {
    pub bounds: Option<[i64; 4]>,
    pub leading_id: Option<u64>,
    pub head_location: Option<[i64; 2]>,
    pub tail_location: Option<[i64; 2]>,
}
fn rectangle(n: &Value) -> Result<[i64; 4], String> {
    let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
        .map_err(|_| "head_chord_member_bounds_required")?;
    if b[2] <= 0 || b[3] <= 0 || b.iter().any(|v| v.unsigned_abs() > 4_000_000) {
        return Err("invalid_head_chord_member_bounds".into());
    }
    Ok(b)
}
fn center(b: [i64; 4]) -> [i64; 2] {
    [b[0] + b[2] / 2, b[1] + b[3] / 2]
}
pub(crate) fn ordered<'a>(heads: &[&'a Value]) -> Result<Vec<&'a Value>, String> {
    ordered_notes(heads, "HeadInter")
}
fn ordered_notes<'a>(heads: &[&'a Value], kind: &str) -> Result<Vec<&'a Value>, String> {
    for h in heads {
        rectangle(h)?;
        if h["kind"] != kind || h["id"].as_u64().is_none_or(|id| id == 0) {
            return Err("head_chord_actual_head_required".into());
        }
    }
    let mut sorted = heads.to_vec();
    sorted.sort_by_key(|h| std::cmp::Reverse(center(rectangle(h).unwrap())[1]));
    Ok(sorted)
}
fn notes_bounds(heads: &[&Value], kind: &str) -> Result<Option<[i64; 4]>, String> {
    let mut union: Option<[i64; 4]> = None;
    for h in ordered_notes(heads, kind)? {
        let b = rectangle(h)?;
        union = Some(if let Some(a) = union {
            let x = a[0].min(b[0]);
            let y = a[1].min(b[1]);
            [
                x,
                y,
                (a[0] + a[2]).max(b[0] + b[2]) - x,
                (a[1] + a[3]).max(b[1] + b[3]) - y,
            ]
        } else {
            b
        });
    }
    Ok(union)
}
pub(crate) fn calculate(heads: &[&Value], stem: Option<&Value>) -> Result<Geometry, String> {
    calculate_notes(heads, stem, "HeadInter")
}
/// RestChordInter inherits the stemless AbstractChord geometry.
pub(crate) fn calculate_rest(rests: &[&Value]) -> Result<Geometry, String> {
    calculate_notes(rests, None, "RestInter")
}
fn calculate_notes(heads: &[&Value], stem: Option<&Value>, kind: &str) -> Result<Geometry, String> {
    let notes = ordered_notes(heads, kind)?;
    let mut bounds = notes_bounds(&notes, kind)?;
    let stem_box = stem
        .map(|s| {
            if s["kind"] != "StemInter" {
                return Err("head_chord_actual_stem_required".into());
            }
            rectangle(s)
        })
        .transpose()?;
    // Highest note for a stemless chord; farthest from stem middle otherwise.
    // Strict greater-than preserves the first bottom-up note on a distance tie.
    let leading = if let Some(b) = stem_box {
        let mid = center(b)[1];
        let mut best = None;
        let mut best_distance = -1;
        for h in notes {
            let distance = (center(rectangle(h)?)[1] - mid).abs();
            if distance > best_distance {
                best = Some(h);
                best_distance = distance;
            }
        }
        best
    } else {
        notes.last().copied()
    };
    let (head_location, tail_location) = if let Some(h) = leading {
        let hc = center(rectangle(h)?);
        if let Some(b) = stem_box {
            let tail = [
                b[0] + b[2] / 2,
                if center(b)[1] < hc[1] {
                    b[1]
                } else {
                    b[1] + b[3] - 1
                },
            ];
            (Some([tail[0], hc[1]]), Some(tail))
        } else {
            (Some(hc), Some(hc))
        }
    } else {
        // Original location getters throw without a leading note. Bounds alone
        // can fall back to the stem box; do not invent a usable note location.
        (None, None)
    };
    if let Some(b) = stem_box {
        bounds = Some(if let Some(a) = bounds {
            let p = tail_location.ok_or("head_chord_tail_location_required")?;
            let x = a[0].min(p[0]);
            let y = a[1].min(p[1]);
            // java.awt.Rectangle.add(Point), no extra pixel on max coordinates.
            [
                x,
                y,
                (a[0] + a[2]).max(p[0]) - x,
                (a[1] + a[3]).max(p[1]) - y,
            ]
        } else {
            b
        });
    }
    Ok(Geometry {
        bounds,
        leading_id: leading.and_then(|h| h["id"].as_u64()),
        head_location,
        tail_location,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        shared_sig::{Graph, Key, Phase},
        stem_overlap_ownership::Ownership,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    #[test]
    fn original_stem_bound_chord_tail_bounds_and_bottom_up_leading_note() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-chord-geometry/controls.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut graph = Graph::default();
            let mut keys = BTreeMap::new();
            for (i, n) in case["nodes"].as_array().unwrap().iter().enumerate() {
                let phase = match n["kind"].as_str().unwrap() {
                    "HeadInter" => Phase::Head,
                    "StemInter" => Phase::Stem,
                    "HeadChordInter" => Phase::Chord,
                    _ => panic!("unknown original node"),
                };
                let key = Key {
                    system: 1,
                    phase,
                    local: i + 1,
                };
                let mut data = n.clone();
                for field in [
                    "label",
                    "member_labels",
                    "leading_label",
                    "head_location",
                    "tail_location",
                    "location_boundary",
                ] {
                    data.as_object_mut().unwrap().remove(field);
                }
                graph.vertex(key, data, false).unwrap();
                keys.insert(n["label"].as_str().unwrap(), key);
            }
            for e in case["initial_relations"].as_array().unwrap() {
                graph
                    .relations(
                        1,
                        Phase::Chord,
                        vec![(
                            keys[e["source"].as_str().unwrap()],
                            keys[e["target"].as_str().unwrap()],
                            json!({"kind":e["kind"]}),
                        )],
                    )
                    .unwrap();
            }
            let snapshot = graph.snapshot();
            let owner = Ownership::new(&snapshot).unwrap();
            let cid = graph.id(keys["chord"]).unwrap() as u64;
            let expected = case["nodes"].as_array().unwrap().last().unwrap();
            let ordered_labels = owner.members[&cid]
                .iter()
                .map(|id| {
                    keys.iter()
                        .find(|(_, k)| graph.id(**k) == Some(*id as usize))
                        .unwrap()
                        .0
                })
                .collect::<Vec<_>>();
            assert_eq!(json!(ordered_labels), expected["member_labels"]);
            let heads: Vec<_> = owner.members[&cid]
                .iter()
                .map(|id| owner.nodes[id])
                .collect();
            let stem_id = graph.id(keys["stem"]).unwrap() as u64;
            let geometry = calculate(&heads, Some(owner.nodes[&stem_id])).unwrap();
            assert_eq!(json!(geometry.bounds), expected["bounds"]);
            let leading = expected["leading_label"]
                .as_str()
                .map(|l| graph.id(keys[l]).unwrap() as u64);
            assert_eq!(geometry.leading_id, leading);
            let head: Option<[f64; 2]> =
                serde_json::from_value(expected["head_location"].clone()).unwrap();
            let tail: Option<[f64; 2]> =
                serde_json::from_value(expected["tail_location"].clone()).unwrap();
            assert_eq!(geometry.head_location.map(|p| p.map(|v| v as f64)), head);
            assert_eq!(geometry.tail_location.map(|p| p.map(|v| v as f64)), tail);
            assert!(owner.mirror(cid).is_none());
            // A serialized bbox cannot substitute for the actual typed stem.
            let mut stale = snapshot.clone();
            stale["inters"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["id"] == cid)
                .unwrap()["stem_id"] = json!(cid);
            assert_eq!(
                Ownership::new(&stale).err().unwrap(),
                "stale_overlap_chord_stem"
            );
            let mut stale = snapshot.clone();
            let chord = stale["inters"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["id"] == cid)
                .unwrap();
            chord["bounds"][2] = json!(chord["bounds"][2].as_i64().unwrap() + 1);
            assert_eq!(
                Ownership::new(&stale).err().unwrap(),
                "stale_overlap_chord_bounds"
            );
            assert_eq!(graph.snapshot(), snapshot);
        }
        let empty = calculate(&[], None).unwrap();
        assert!(
            empty.bounds.is_none() && empty.leading_id.is_none() && empty.head_location.is_none()
        );
    }
}
