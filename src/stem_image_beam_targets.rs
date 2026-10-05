// SPDX-License-Identifier: AGPL-3.0-or-later
//! Head-origin image CLinker beam sibling membership, before BLinker stump selection.
use crate::{GrayImage, stem_image_linkers, stem_image_stumps};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
#[derive(Clone, Deserialize, Serialize)]
struct Beam {
    id: usize,
    median: [f64; 4],
    height: f64,
    group: usize,
}
fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|v| v.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image beam targets {label}"))
}
fn intersection(a: [f64; 4], b: [f64; 4]) -> Result<[f64; 2], String> {
    let [x1, y1, x2, y2] = a;
    let [x3, y3, x4, y4] = b;
    let d = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let p = x1 * y2 - y1 * x2;
    let q = x3 * y4 - y3 * x4;
    let point = [
        (p * (x3 - x4) - (x1 - x2) * q) / d,
        (p * (y3 - y4) - (y1 - y2) * q) / d,
    ];
    if point.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.) {
        return Err("unsupported parallel/nonfinite image beam target lines".into());
    }
    Ok(point)
}
fn siblings(beams: &[Beam], point: [f64; 2], dx: i32) -> Result<Vec<usize>, String> {
    if beams.len() > 2048
        || !(0..=1024).contains(&dx)
        || point.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.)
    {
        return Err("invalid image beam sibling parameters".into());
    }
    let mut ids = BTreeSet::new();
    let mut out = Vec::new();
    // Original Skew.skewedVertical uses a1000pixel line; zero skew is the admitted scope.
    let vertical = [point[0], point[1], point[0], point[1] + 1000.];
    for b in beams {
        if b.id == 0
            || !ids.insert(b.id)
            || b.median
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
            || b.median[2] <= b.median[0]
            || !b.height.is_finite()
            || b.height <= 0.
        {
            return Err("invalid image sibling beam geometry/identity".into());
        }
        let hit = intersection(vertical, b.median)?;
        if b.median[0] - f64::from(dx) <= hit[0] && hit[0] <= b.median[2] + f64::from(dx) {
            out.push((b.id, hit[1]));
        }
    }
    // Java Collections.sort is stable and uses Double.compare. Preserve equal-height input order.
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    Ok(out.into_iter().map(|v| v.0).collect())
}
/// Builds actual image head-origin sibling members with native scale/group/target inputs.
/// Full BLinker seed/section stumps and beam-origin builders are separate pending dependencies.
pub fn prepare(image: &GrayImage, page: &Value) -> Result<stem_image_stumps::Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<crate::beam_dispatch::Registry>,
) -> Result<stem_image_stumps::Prepared, String> {
    let mut owned = stem_image_linkers::prepare_with_registry(image, page, registry)?;
    let head_linker_plan = owned.report.clone();
    let source = &head_linker_plan["source_plan"];
    let head = &source["stump_plan"]["seed_plan"]["head_plan"];
    let hs = rows(&head["systems"], 64, "head systems")?;
    let sources = rows(&source["systems"], 64, "source systems")?;
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let dx = (scale.interline.main as f64 * 0.25).round_ties_even() as i32;
    if hs.len() != sources.len() {
        return Err("invalid image beam system count".into());
    }
    let mut result = Vec::new();
    let mut total = 0usize;
    for (h, s) in hs.iter().zip(sources) {
        if h["system"] != s["system"] {
            return Err("invalid image beam system order".into());
        }
        let beams: Vec<Beam> = serde_json::from_value(h["image_context"]["beams"].clone())
            .map_err(|_| "invalid image beam catalog")?;
        let groups = rows(&h["image_context"]["groups"], 2048, "beam groups")?;
        let targets = rows(&h["cases"], 16384, "head targets")?;
        let cases = rows(&s["cases"], 16384, "source corners")?;
        if targets.len() != cases.len() {
            return Err("invalid image beam target count".into());
        }
        let mut output = Vec::new();
        for (t, c) in targets.iter().zip(cases) {
            let hd = &c["head"];
            let veto = hd["shape"] == "NOTEHEAD_VOID"
                && ((hd["side"] == "LEFT" && hd["vertical"] == "TOP")
                    || (hd["side"] == "RIGHT" && hd["vertical"] == "BOTTOM"));
            let mut ids = Vec::new();
            let mut point = None;
            let mut target = Value::Null;
            if let Some(id) = t["target"]["target_beam"].as_u64() {
                let b = beams
                    .iter()
                    .find(|b| b.id == id as usize)
                    .ok_or("unknown image target beam")?;
                target = serde_json::to_value(b).map_err(|e| e.to_string())?;
                if !veto {
                    let line: [f64; 4] = serde_json::from_value(c["theoretical_line"].clone())
                        .map_err(|_| "invalid image beam target line")?;
                    let at = intersection(b.median, line)?;
                    point = Some(at);
                    let mut matches = groups.iter().filter(|g| g["id"] == b.group);
                    let g = matches.next().ok_or("missing image beam group")?;
                    if matches.next().is_some() {
                        return Err("duplicate image beam group".into());
                    }
                    let mut members = Vec::new();
                    let mut seen = BTreeSet::new();
                    for v in rows(&g["members"], 2048, "group members")? {
                        let n = v.as_u64().ok_or("invalid image sibling member ID")? as usize;
                        if !seen.insert(n) {
                            return Err("duplicate image sibling member".into());
                        }
                        let member = beams
                            .iter()
                            .find(|m| m.id == n)
                            .ok_or("unknown image sibling member")?;
                        if member.group != b.group {
                            return Err("inconsistent image sibling group".into());
                        }
                        members.push(member.clone());
                    }
                    ids = siblings(&members, at, dx)?;
                }
            }
            total = total
                .checked_add(ids.len())
                .filter(|v| *v <= 2_000_000)
                .ok_or("image sibling member budget")?;
            let selected = ids
                .iter()
                .map(|id| {
                    serde_json::to_value(
                        beams
                            .iter()
                            .find(|b| b.id == *id)
                            .expect("owned sibling beam"),
                    )
                    .expect("beam serialization")
                })
                .collect::<Vec<_>>();
            output.push(json!({"head":hd,"target_beam":target,"void_side_veto":veto,"intersection":point,"selected_sibling_member_ids":ids,"selected_sibling_members":selected}));
        }
        result.push(json!({"system":h["system"],"max_beam_side_dx":dx,"cases":output}));
    }
    owned.report = json!({"schema":"audiveris-native.stem-image-beam-targets.v1","head_linker_plan":head_linker_plan,"systems":result,"corner_count":head_linker_plan["corner_count"],"selected_sibling_member_occurrences":total,"head_origin_beam_sibling_member_selection_complete":true,"beam_sibling_linker_selection_complete":false,"automatic_raster_pipeline_complete":false,"final_stem_glyph_selection_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["beam seed/section/orphan BLinker stumps and nearest/anchor choice; beam-origin builders","automatic section/chunk/items/final prefix glyph; actual liveGraph/cache/head; cleaner/epilog","fresh transient HEADS lifetime; nonzero skew/scans/special staff/full24 pages"]});
    Ok(owned)
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_sibling_side_ulps_and_empty_duplicate_parallel_equal_height() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-beam-targets/controls.json"
        ))
        .unwrap();
        for c in o["controls"].as_array().unwrap() {
            let bs = c["members"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, v)| Beam {
                    id: i + 1,
                    median: serde_json::from_value(v["median"].clone()).unwrap(),
                    height: v["height"].as_f64().unwrap(),
                    group: 1,
                })
                .collect::<Vec<_>>();
            let ids = siblings(
                &bs,
                serde_json::from_value(c["point"].clone()).unwrap(),
                c["max_beam_side_dx"].as_i64().unwrap() as i32,
            )
            .unwrap();
            let actual = ids
                .into_iter()
                .map(|id| {
                    let b = &bs[id - 1];
                    json!({"median":b.median,"height":b.height})
                })
                .collect::<Vec<_>>();
            let expected = c["selected"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| json!({"median":v["median"],"height":v["height"]}))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{}", c["name"]);
        }
        assert!(siblings(&[], [0., 0.], 3).unwrap().is_empty());
        let b = Beam {
            id: 1,
            median: [0., 0., 10., 0.],
            height: 1.,
            group: 1,
        };
        assert!(siblings(&[b.clone(), b.clone()], [1., 0.], 3).is_err());
        assert_eq!(
            siblings(&[b.clone(), Beam { id: 2, ..b }], [1., 0.], 3).unwrap(),
            vec![1, 2]
        );
        assert!(intersection([0., 0., 1., 0.], [0., 1., 1., 1.]).is_err());
    }
}
