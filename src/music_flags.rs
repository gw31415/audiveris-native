// SPDX-License-Identifier: AGPL-3.0-or-later
//! AbstractFlagInter.lookupLink/createValidAdded, ordinary one/two flags.
//! Actual symbol glyphs and retained stem/head relations; no selected input IDs.
use crate::shared_sig::{Graph, Key};
use serde_json::{Value, json};
pub(crate) fn value(shape: &str) -> Option<usize> {
    match shape {
        "FLAG_1" | "FLAG_1_DOWN" => Some(1),
        "FLAG_2" | "FLAG_2_DOWN" => Some(2),
        _ => None,
    }
}
fn rect(v: &Value) -> Result<[i64; 4], String> {
    serde_json::from_value(v.clone()).map_err(|_| "flag_native_bounds_required".into())
}
fn line(v: &Value) -> Result<[f64; 4], String> {
    let l: [f64; 4] =
        serde_json::from_value(v.clone()).map_err(|_| "flag_native_stem_median_required")?;
    if l.iter().any(|x| !x.is_finite()) || l[1] >= l[3] {
        return Err("flag_invalid_stem_median".into());
    }
    Ok(l)
}
fn intersects(a: [i64; 4], b: [i64; 4]) -> bool {
    if a[2] <= 0 || a[3] <= 0 || b[2] <= 0 || b[3] <= 0 {
        return false;
    }
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
/// AbstractConnection.setInOutGaps, original intrinsic relation profile0.
pub(crate) fn relation_grade(dx: f64, dy: f64) -> Result<f64, String> {
    if !dx.is_finite() || !dy.is_finite() || dy < 0. {
        return Err("flag_nonfinite_or_negative_gap".into());
    }
    let x = if dx >= 0. {
        (1. - dx / 0.3).clamp(0., 1.)
    } else {
        (1. + dx / 0.3).clamp(0., 1.)
    };
    let y = (1. - dy / 0.5).clamp(0., 1.);
    Ok(if dx >= 0. {
        (x * x * y).powf(1. / 3.)
    } else {
        (x * y).sqrt()
    })
}
/// Original has first-valid-by-abscissa selection. Current native automatic
/// system profile1 is used for lookup; the FlagInter's relation profile is0.
pub(crate) fn lookup(
    shape: &str,
    b: [i64; 4],
    il: usize,
    max_stem: usize,
    system: u64,
    snapshot: &Value,
) -> Result<Option<(Key, Value)>, String> {
    let up = match shape {
        "FLAG_1" | "FLAG_2" => true,
        "FLAG_1_DOWN" | "FLAG_2_DOWN" => false,
        _ => return Err("flag_shape_outside_single_ordinary_scope".into()),
    };
    if b[2] <= 0 || b[3] <= 0 || il == 0 || max_stem == 0 {
        return Err("flag_invalid_lookup_geometry".into());
    }
    let foot = (b[3] as f64 / 2.5).round_ties_even() as i64;
    let reference = [
        b[0] as f64,
        if up { b[1] + foot } else { b[1] + b[3] - foot } as f64,
    ];
    let ygap = (il as f64 * 0.75).round_ties_even() as i64;
    let lookup = [
        b[0] - 1 - max_stem as i64,
        if up {
            b[1] + ygap
        } else {
            b[1] + b[3] - foot - ygap
        },
        2 * max_stem as i64,
        foot,
    ];
    let check_y = reference[1]
        + if up {
            -(foot / 2) as f64
        } else {
            (foot / 2) as f64
        };
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("flag_owned_SIG_required")?;
    let mut stems = nodes
        .iter()
        .filter(|n| n["kind"] == "StemInter" && n["origin"]["system"] == system)
        .collect::<Vec<_>>();
    stems.sort_by_key(|s| s["bounds"][0].as_i64());
    for stem in stems {
        if !intersects(lookup, rect(&stem["bounds"])?) {
            continue;
        }
        let l = line(&stem["median"])?;
        let x = l[0] + (reference[1] - l[1]) * (l[2] - l[0]) / (l[3] - l[1]);
        let dx = (reference[0] - x) / il as f64;
        let dy = ((l[1] - reference[1]).max(0.).max(reference[1] - l[3])) / il as f64;
        let grade = relation_grade(dx, dy)?;
        if grade < 0.1 {
            continue;
        }
        let midpoint = (l[1] + l[3]) / 2.;
        if (up && check_y >= midpoint) || (!up && check_y <= midpoint) {
            continue;
        }
        let mut context = snapshot.clone();
        context["interline"] = json!(il);
        let direction = crate::stem_head_foundation_plan::direction(&context, stem);
        if (up && direction == 1) || (!up && direction == -1) {
            continue;
        }
        let ey = if up { b[1] } else { b[1] + b[3] - 1 } as f64;
        let ex = l[0] + (ey - l[1]) * (l[2] - l[0]) / (l[3] - l[1]);
        let key: Key = serde_json::from_value(stem["origin"].clone())
            .map_err(|_| "flag_actual_stem_origin_required")?;
        return Ok(Some((
            key,
            json!({"kind":"FlagStemRelation","grade":grade,"dx":dx,"dy":dy,"extension_point":[ex,ey],"source_ratio":1.+3.*grade,"target_ratio":1.+3.*grade,"lookup_profile":1,"profile":0}),
        )));
    }
    Ok(None)
}
/// Complete the original contextualize-and-weak-purge subset for flags. Other
/// supported symbol overlaps and multiple stems stay explicit pending cases.
pub(crate) fn contextualize(graph: &mut Graph) -> Result<Value, String> {
    let initial = graph.snapshot();
    let flags = initial["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "FlagInter")
        .collect::<Vec<_>>();
    if flags.is_empty() {
        return Ok(json!({"flags":0,"removed":[]}));
    }
    graph.contextualize_head_stem_flags()?;
    let snap = graph.snapshot();
    let mut removed = Vec::new();
    for f in snap["inters"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "FlagInter")
    {
        if f["contextual_grade"]
            .as_f64()
            .ok_or("flag_context_grade_required")?
            < 0.5
            && f["frozen"] != true
        {
            let key = serde_json::from_value(f["origin"].clone())
                .map_err(|_| "flag_owned_origin_required")?;
            graph.remove_music_inter(key)?;
            removed.push(f["id"].clone());
        }
    }
    graph.contextualize_head_stem_flags()?;
    Ok(
        json!({"flags":flags.len(),"removed":removed,"scope":"ordinary one/two flags, actual HeadStem/FlagStem support/context and weak purge; full supported symbol conflict reduction pending"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_lookup_gap_and_polarity_boundaries() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-flags/dependency-controls.json"
        ))
        .unwrap();
        for c in o["grades"].as_array().unwrap() {
            assert!(
                (relation_grade(c["dx"].as_f64().unwrap(), c["dy"].as_f64().unwrap()).unwrap()
                    - c["grade"].as_f64().unwrap())
                .abs()
                    < 1e-12
            )
        }
        for c in o["lookup"].as_array().unwrap() {
            let actual = lookup(
                c["shape"].as_str().unwrap(),
                rect(&c["bounds"]).unwrap(),
                11,
                3,
                1,
                &o["snapshot"],
            )
            .unwrap();
            if c["result"].is_null() {
                assert!(actual.is_none(), "{c}")
            } else {
                let (k, r) = actual.unwrap();
                assert_eq!(k.local as u64, c["result"]["stem_id"].as_u64().unwrap());
                for field in ["grade", "dx", "dy"] {
                    assert!(
                        (r[field].as_f64().unwrap() - c["result"][field].as_f64().unwrap()).abs()
                            < 1e-12,
                        "{field}:{c}"
                    )
                }
                assert_eq!(r["extension_point"], c["result"]["extension_point"]);
            }
        }
        let b = [290, 207, 12, 41];
        assert!(
            lookup("FLAG_1_DOWN", b, 11, 3, 1, &o["snapshot"])
                .unwrap()
                .is_none()
        );
        assert!(
            lookup("FLAG_1", b, 11, 3, 2, &o["snapshot"])
                .unwrap()
                .is_none()
        );
        assert!(
            lookup("FLAG_1", b, 11, 3, 1, &json!({"inters":[],"relations":[]}))
                .unwrap()
                .is_none()
        );
        assert!(relation_grade(f64::NAN, 0.).is_err());
        assert!(relation_grade(0., -1.).is_err());
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-chord-members/flag2-original-control.json"
        ))
        .unwrap();
        let c = &original["control"];
        let mut snapshot = original["snapshot"].clone();
        snapshot["interline"] = original["interline"].clone();
        let nodes = snapshot["inters"].as_array().unwrap();
        let stem = nodes.iter().find(|n| n["id"] == c["stem_id"]).unwrap();
        assert_eq!(
            json!(crate::stem_head_foundation_plan::direction(&snapshot, stem)),
            c["stem_direction"]
        );
        assert_eq!(json!(value(c["shape"].as_str().unwrap())), c["value"]);
        let (key, relation) = lookup(
            c["shape"].as_str().unwrap(),
            serde_json::from_value(c["bounds"].clone()).unwrap(),
            original["interline"].as_u64().unwrap() as usize,
            original["max_stem"].as_u64().unwrap() as usize,
            original["system"].as_u64().unwrap(),
            &snapshot,
        )
        .unwrap()
        .unwrap();
        assert_eq!(json!(key.local), c["stem_id"]);
        for field in ["grade", "dx", "dy"] {
            assert!((relation[field].as_f64().unwrap() - c[field].as_f64().unwrap()).abs() < 1e-12);
        }
        assert_eq!(relation["extension_point"], c["extension_point"]);
    }
}

mod competition;
#[cfg(test)]
pub(crate) use competition::reduce_overlaps;
pub(crate) use competition::{detect_overlaps, reduce_exclusions};
