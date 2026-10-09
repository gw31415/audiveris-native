// SPDX-License-Identifier: AGPL-3.0-or-later
//! CurvesBuilder non-crossable filter on the borrowed pre-CURVES Graph.
use super::Graph;
use crate::{grid_ir::SystemGeometry, music_slurs::fit::Pixel};
use audiveris_area::LinearArea;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    ExtensionBoundsClear,
    ExtensionAreaClear,
    SegmentBoundsClear,
    StripAreaClear,
    Crosses,
}

fn valid_bounds(r: [f64; 4]) -> Result<(), String> {
    if r.iter().any(|v| !v.is_finite() || v.abs() > 1_000_000.) || r[2] <= 0. || r[3] <= 0. {
        return Err("slur_actual_obstacle_bounds_required".into());
    }
    Ok(())
}

fn rectangles_intersect(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] + a[2] > b[0] && a[1] + a[3] > b[1] && a[0] < b[0] + b[2] && a[1] < b[1] + b[3]
}

fn beam_area(m: [f64; 4], height: f64) -> Result<LinearArea, String> {
    if m.iter().any(|x| !x.is_finite()) || m[0] >= m[2] || !height.is_finite() || height <= 0. {
        return Err("slur_actual_Beam_area_geometry_required".into());
    }
    let dy = height / 2.;
    LinearArea::polygon(&[
        [m[0], m[1] - dy],
        [m[2], m[3] - dy],
        [m[2], m[3] + dy],
        [m[0], m[1] + dy],
    ])
}

fn strip(a: Pixel, b: Pixel) -> Result<LinearArea, String> {
    let horizontal =
        (i64::from(b.x) - i64::from(a.x)).abs() >= (i64::from(b.y) - i64::from(a.y)).abs();
    let [mx, my] = if horizontal { [0., 1.] } else { [1., 0.] };
    let [ax, ay, bx, by] = [
        f64::from(a.x),
        f64::from(a.y),
        f64::from(b.x),
        f64::from(b.y),
    ];
    LinearArea::polygon(&[
        [ax - mx, ay - my],
        [bx - mx, by - my],
        [bx + mx, by + my],
        [ax + mx, ay + my],
    ])
}

fn decision(
    bounds: [f64; 4],
    area: Option<&LinearArea>,
    extension: [f64; 4],
    a: Pixel,
    b: Pixel,
) -> Result<Decision, String> {
    valid_bounds(bounds)?;
    valid_bounds(extension)?;
    if !rectangles_intersect(bounds, extension) {
        return Ok(Decision::ExtensionBoundsClear);
    }
    if let Some(area) = area
        && !area.intersects_rect(extension)?
    {
        return Ok(Decision::ExtensionAreaClear);
    }
    if !crate::stem_chord_consistency::intersects(
        [
            f64::from(a.x),
            f64::from(a.y),
            f64::from(b.x),
            f64::from(b.y),
        ],
        bounds,
    ) {
        return Ok(Decision::SegmentBoundsClear);
    }
    if let Some(area) = area
        && !strip(a, b)?.intersects_area(area)?
    {
        return Ok(Decision::StripAreaClear);
    }
    Ok(Decision::Crosses)
}

fn live_area(n: &Value) -> Result<Option<LinearArea>, String> {
    match n["kind"].as_str() {
        Some("HeadInter") if n["area"].is_null() && n["manual"] != true => Ok(None),
        Some("BeamInter" | "BeamHookInter") if n["area"].is_null() && n["manual"] != true => {
            let median = serde_json::from_value(n["median"].clone())
                .map_err(|_| "slur_actual_Beam_median_required")?;
            let height = n["height"]
                .as_f64()
                .ok_or("slur_actual_Beam_height_required")?;
            beam_area(median, height).map(Some)
        }
        _ => Err("slur_non_crossable_owner_Area_comparison_pending".into()),
    }
}

fn can_hide(n: &Value) -> Result<bool, String> {
    if n["frozen"] == true || n["is_frozen"] == true {
        return Ok(true);
    }
    // CurvesCleaner inherits PageCleaner.canHide. The SYMBOLS-specific
    // Head threshold 0.6 does not apply at this boundary.
    Ok(n["contextual_grade"]
        .as_f64()
        .or(n["grade"].as_f64())
        .filter(|d| d.is_finite() && (0.0..=1.0).contains(d))
        .ok_or("slur_actual_obstacle_best_grade_required")?
        >= 0.4)
}

impl Graph {
    pub(crate) fn slur_gap_is_clear(
        &self,
        systems: &[Arc<SystemGeometry>],
        a: Pixel,
        b: Pixel,
        extension: [f64; 4],
    ) -> Result<bool, String> {
        #[cfg(test)]
        let mut observations = Vec::new();
        #[cfg(test)]
        let mut routed = Vec::new();
        let result = (|| {
            valid_bounds(extension)?;
            for system in systems {
                if !system.area.contains([f64::from(a.x), f64::from(a.y)])? {
                    continue;
                }
                #[cfg(test)]
                routed.push(serde_json::json!({"id":system.id,"owner_address":Arc::as_ptr(system) as usize}));
                for v in self
                    .vertices
                    .iter()
                    .filter(|v| !v.removed && v.key.system == system.id)
                {
                    let n = &v.data;
                    let selected = matches!(
                        n["kind"].as_str(),
                        Some("HeadInter" | "BeamInter" | "BeamHookInter")
                    ) || matches!(n["shape"].as_str(), Some("TEXT" | "LYRICS"));
                    if !selected {
                        continue;
                    }
                    #[cfg(test)]
                    let frozen = n["frozen"] == true || n["is_frozen"] == true;
                    if !can_hide(n)? {
                        continue;
                    }
                    let bounds: [f64; 4] = serde_json::from_value(n["bounds"].clone())
                        .map_err(|_| "slur_actual_obstacle_bounds_required")?;
                    valid_bounds(bounds)?;
                    if !rectangles_intersect(bounds, extension) {
                        continue;
                    }
                    #[cfg(test)]
                    let trace_index = observations.len();
                    #[cfg(test)]
                    observations.push(serde_json::json!({"key":v.key,"id":n["id"],"kind":n["kind"],"shape":n["shape"],
                        "owner_address":v as *const _ as usize,"bounds":bounds,"median":n["median"],"height":n["height"],
                        "grade":n["grade"],"contextual_grade":n["contextual_grade"],"frozen":frozen,"manual":n["manual"],
                        "implicit":n["implicit"],"is_implicit":n["is_implicit"],"decision":"pending"}));
                    let area = live_area(n)?;
                    if n["implicit"] == true || n["is_implicit"] == true {
                        #[cfg(test)]
                        {
                            observations[trace_index]["decision"] =
                                serde_json::json!("ImplicitSkipped");
                        }
                        continue;
                    }
                    let outcome = decision(bounds, area.as_ref(), extension, a, b)?;
                    #[cfg(test)]
                    {
                        observations[trace_index]["decision"] =
                            serde_json::json!(format!("{outcome:?}"));
                    }
                    if outcome == Decision::Crosses {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        })();
        #[cfg(test)]
        crate::music_slurs::image_tests::gap_result(serde_json::json!({
            "a":[a.x,a.y],"b":[b.x,b.y],"extension_box":extension,"systems":routed,"obstacles":observations,
            "allowed":result.as_ref().ok(),"error":result.as_ref().err(),"scope":"Actual retained pre-CURVES owners; all current native alternatives for the left component, not original Skeleton reachability."}));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_non_crossable_filter_keeps_area_prefilter_and_segment_boundaries() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-slur-gap-obstacles/observed.json"
        ))
        .unwrap();
        for row in fixture["head_good_grade_controls"].as_array().unwrap() {
            assert_eq!(
                can_hide(&serde_json::json!({"grade":row["grade"]})).unwrap(),
                row["is_contextually_good"].as_bool().unwrap()
            );
        }
        for row in fixture["rows"].as_array().unwrap() {
            let line: [i32; 4] = serde_json::from_value(row["line"].clone()).unwrap();
            let bounds = serde_json::from_value(row["bounds"].clone()).unwrap();
            let ext = serde_json::from_value(row["expected"]["extension_box"].clone()).unwrap();
            let beam: Option<[f64; 5]> = serde_json::from_value(row["beam"].clone()).unwrap();
            let area = beam.map(|m| beam_area([m[0], m[1], m[2], m[3]], m[4]).unwrap());
            let result = decision(
                bounds,
                area.as_ref(),
                ext,
                Pixel {
                    x: line[0],
                    y: line[1],
                },
                Pixel {
                    x: line[2],
                    y: line[3],
                },
            )
            .unwrap();
            assert_eq!(
                result == Decision::Crosses,
                row["expected"]["original_filter_rejects"]
                    .as_bool()
                    .unwrap(),
                "{}",
                row["case"]
            );
        }
    }
}
