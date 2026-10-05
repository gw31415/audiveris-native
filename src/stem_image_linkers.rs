// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual native image head corners, current HEADS exclusions and other-head targets.
use crate::{GrayImage, stem_image_sources, stem_image_stumps};
use audiveris_area::LinearArea;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
struct Head {
    id: usize,
    head_bounds: [i32; 4],
    center: [f64; 2],
    shape: String,
    removed: bool,
    competing_ids: Vec<usize>,
    corners: Vec<Value>,
}
fn rows<'a>(v: &'a Value, max: usize, label: &str) -> Result<&'a [Value], String> {
    v.as_array()
        .filter(|v| v.len() <= max)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid image linkers {label}"))
}
fn shape(s: &str) -> Result<(), String> {
    match s {
        "NOTEHEAD_BLACK" | "NOTEHEAD_VOID" => Ok(()),
        _ => Err("unsupported image linker note duration".into()),
    }
}
fn select(
    heads: &[Head],
    source: usize,
    source_shape: &str,
    reference: [f64; 2],
    vertical: &str,
    polygon: &[[f64; 2]],
    min_dy: i32,
) -> Result<Vec<Value>, String> {
    shape(source_shape)?;
    if heads.len() > 4096
        || polygon.len() > 128
        || !(0..=1024).contains(&min_dy)
        || reference
            .iter()
            .any(|v| !v.is_finite() || !(-4_000_000.0..=4_000_000.0).contains(v))
    {
        return Err("invalid image head linker input".into());
    }
    let direction = match vertical {
        "TOP" => -1.,
        "BOTTOM" => 1.,
        _ => return Err("invalid image head linker direction".into()),
    };
    let area = if polygon.is_empty() {
        None
    } else {
        Some(LinearArea::polygon(polygon)?)
    };
    let mut ids = BTreeSet::new();
    for h in heads {
        shape(&h.shape)?;
        if h.id == 0
            || !ids.insert(h.id)
            || h.head_bounds
                .iter()
                .any(|v| !(-4_000_000..=4_000_000).contains(v))
            || h.head_bounds[2] <= 0
            || h.head_bounds[3] <= 0
            || h.center
                .iter()
                .any(|v| !v.is_finite() || !(-4_000_000.0..=4_000_000.0).contains(v))
            || h.competing_ids.len() > 4096
            || h.corners.len() != 4
        {
            return Err("invalid image head linker catalog".into());
        }
    }
    // Empty upstream head catalog is a valid no-op; otherwise source must exist.
    let competing = if heads.is_empty() {
        &[][..]
    } else {
        heads
            .iter()
            .find(|h| h.id == source)
            .ok_or("missing image source head")?
            .competing_ids
            .as_slice()
    };
    let threshold = reference[1] + direction * f64::from(min_dy);
    let mut order = heads.iter().collect::<Vec<_>>();
    order.sort_by_key(|h| h.head_bounds[0]); // original stable Inters.byAbscissa: bounds.x
    let mut out = Vec::new();
    for h in order {
        if h.removed
            || h.id == source
            || competing.contains(&h.id)
            || h.shape != source_shape
            || direction * (h.center[1] - threshold) < 0.
        {
            continue;
        }
        if !match &area {
            Some(a) => a.intersects_rect(h.head_bounds.map(f64::from))?,
            None => false,
        } {
            continue;
        }
        for side in ["LEFT", "RIGHT"] {
            let mut matching = h
                .corners
                .iter()
                .filter(|c| c["side"] == side && c["vertical"] == vertical);
            let c = matching.next().ok_or("missing image target corner")?;
            if matching.next().is_some() {
                return Err("duplicate image target corner".into());
            }
            let point: [f64; 2] = serde_json::from_value(c["reference"].clone())
                .map_err(|_| "invalid image target reference")?;
            if match &area {
                Some(a) => a.contains(point)?,
                None => false,
            } {
                out.push(c.clone())
            }
        }
    }
    Ok(out)
}
/// Conditions are derived from the native image output; no corner request/oracle input.
/// The physical registry returned by the upstream stump/source path stays owned.
pub fn prepare(image: &GrayImage, page: &Value) -> Result<stem_image_stumps::Prepared, String> {
    prepare_with_registry(image, page, None)
}
pub(crate) fn prepare_with_registry(
    image: &GrayImage,
    page: &Value,
    registry: Option<crate::beam_dispatch::Registry>,
) -> Result<stem_image_stumps::Prepared, String> {
    let mut owned = stem_image_sources::prepare_with_registry(image, page, registry)?;
    let source_plan = owned.report.clone();
    let sig = &page["head_registration"]["shared_sig"];
    let vertices = rows(&sig["inters"], 16384, "live HEADS vertices")?;
    let edges = rows(&sig["relations"], 100000, "live HEADS relations")?;
    let mut origins = BTreeMap::new();
    for v in vertices {
        if v["kind"] == "HeadInter" {
            let id = v["id"].as_u64().ok_or("invalid live head ID")?;
            if v["origin"]["phase"] != "HEAD" || origins.insert(id, v).is_some() {
                return Err("invalid live head origin".into());
            }
        }
    }
    let binary = crate::vertical::binarize(image, |_, _, _| {});
    let scale = crate::scale::analyze(&binary)
        .result
        .map_err(|e| e.code().to_owned())?;
    let min_dy = (scale.interline.main as f64 * 0.25).round_ties_even() as i32;
    let mut result = Vec::new();
    let mut total = 0usize;
    for sys in rows(&source_plan["systems"], 64, "systems")? {
        let system = sys["system"]
            .as_u64()
            .ok_or("invalid image linker system")?;
        let cases = rows(&sys["cases"], 16384, "corners")?;
        let mut grouped: BTreeMap<usize, Vec<Value>> = BTreeMap::new();
        let mut head_order = Vec::new();
        for c in cases {
            let h = &c["head"];
            let id = h["head_id"].as_u64().ok_or("invalid image head identity")? as usize;
            if !grouped.contains_key(&id) {
                head_order.push(id)
            }
            let v = origins
                .get(&(id as u64))
                .ok_or("missing live HEADS origin")?;
            if v["bounds"] != h["head_bounds"]
                || v["shape"] != h["shape"]
                || v["origin"]["system"] != system
            {
                return Err("inconsistent live HEADS origin".into());
            }
            let mut corner = h.clone();
            corner["stump"] = c["stump"].clone();
            corner["head_origin"] = v["origin"].clone();
            grouped.entry(id).or_default().push(corner);
        }
        let mut catalog = Vec::new();
        for id in head_order {
            let v = origins[&(id as u64)];
            let b: [i32; 4] = serde_json::from_value(v["bounds"].clone())
                .map_err(|_| "invalid live head bounds")?;
            let mut competing = Vec::new();
            for edge in edges.iter().filter(|e| e["kind"] == "Exclusion") {
                let a = edge["source"]
                    .as_u64()
                    .ok_or("invalid live exclusion source")? as usize;
                let b = edge["target"]
                    .as_u64()
                    .ok_or("invalid live exclusion target")? as usize;
                if a == id {
                    competing.push(b)
                } else if b == id {
                    competing.push(a)
                }
            }
            competing.sort_unstable();
            competing.dedup();
            catalog.push(json!({"id":id,"head_bounds":b,"center":[b[0]+b[2]/2,b[1]+b[3]/2],"shape":v["shape"],"removed":false,"competing_ids":competing,"corners":grouped.remove(&id).expect("owned corners")}));
        }
        let heads: Vec<Head> =
            serde_json::from_value(json!(catalog)).map_err(|_| "invalid image head catalog")?;
        let mut output = Vec::new();
        for c in cases {
            let h = &c["head"];
            let poly: Vec<[f64; 2]> = serde_json::from_value(c["polygon"].clone())
                .map_err(|_| "invalid image linker polygon")?;
            let selected = select(
                &heads,
                h["head_id"].as_u64().ok_or("invalid source ID")? as usize,
                h["shape"].as_str().ok_or("invalid source shape")?,
                serde_json::from_value(h["reference"].clone())
                    .map_err(|_| "invalid source reference")?,
                h["vertical"].as_str().ok_or("invalid source direction")?,
                &poly,
                min_dy,
            )?;
            total = total
                .checked_add(selected.len())
                .filter(|v| *v <= 2_000_000)
                .ok_or("image linker work budget")?;
            output.push(json!({"head":h,"selected_other_head_linkers":selected}));
        }
        result.push(json!({"system":system,"min_head_head_dy":min_dy,"head_catalog":catalog,"cases":output}));
    }
    owned.report = json!({"schema":"audiveris-native.stem-image-linkers.v1","source_plan":source_plan,"systems":result,"corner_count":source_plan["corner_count"],"selected_other_head_linker_occurrences":total,"other_head_linker_selection_complete":true,"beam_sibling_linker_selection_complete":false,"final_stem_glyph_selection_complete":false,"automatic_raster_pipeline_complete":false,"page_cache_sig_complete":false,"epilog_complete":false,"heads_complete":false,"stems_complete":false,"pending":["beam sibling targets and beam-origin builders; section/chunk/items/final prefix glyph","same owned live HEADS Graph/retriever/cache/head attachments and contextual cleaner/epilog","fresh transient lifecycle; nonzero skew/scans/special staff/full24 pages"]});
    Ok(owned)
}
pub fn analyze(image: &GrayImage, page: &Value) -> Result<Value, String> {
    Ok(prepare(image, page)?.report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_other_head_boundary_observations_and_invalid_identity() {
        let o: Value = serde_json::from_str(include_str!(
            "../fixtures/official-stem-image-linkers/controls.json"
        ))
        .unwrap();
        let full: Vec<Head> = serde_json::from_value(o["control_head_catalog"].clone()).unwrap();
        for c in o["controls"].as_array().unwrap() {
            let polygon = c["area_path"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| p["type"] == 0 || p["type"] == 1)
                .map(|p| serde_json::from_value(p["coordinates"].clone()).unwrap())
                .collect::<Vec<[f64; 2]>>();
            let hs = if c["name"] == "empty-heads" {
                &[][..]
            } else {
                full.as_slice()
            };
            let actual = select(
                hs,
                c["source_id"].as_u64().unwrap() as usize,
                c["head"]["shape"].as_str().unwrap(),
                serde_json::from_value(c["head"]["reference"].clone()).unwrap(),
                c["head"]["vertical"].as_str().unwrap(),
                &polygon,
                c["min_head_head_dy"].as_i64().unwrap() as i32,
            )
            .unwrap();
            assert_eq!(
                actual,
                c["selected"].as_array().unwrap().clone(),
                "{}",
                c["name"]
            );
        }
        let first = &full[0];
        let p = [[0., 0.], [2000., 0.], [2000., 3000.], [0., 3000.]];
        assert!(select(&full, 0, "NOTEHEAD_BLACK", [0., 0.], "BOTTOM", &p, 3).is_err());
        assert!(select(&full, first.id, "WHOLE_NOTE", [0., 0.], "BOTTOM", &p, 3).is_err());
    }
}
