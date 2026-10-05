// SPDX-License-Identifier: AGPL-3.0-or-later
//! Head CLinker.buildLuArea at Audiveris 5.11. Declared reference points and
//! part/beam limit lines are inputs; target-beam selection and beam-side areas
//! are separate dependencies. No registry, cache, head relation or SIG writes.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HeadRequest {
    pub reference: [f64; 2],
    pub x_direction: i8,
    pub y_direction: i8,
    pub slope: f64,
    pub slope_margin: f64,
    pub max_head_in_dx: i32,
    pub max_head_out_dx: i32,
    /// [x, y, width, height], bottom edge is y + height - 1.
    pub part_bounds: [i32; 4],
    pub limit: Option<[f64; 4]>,
}
#[derive(Debug, Serialize)]
pub struct Geometry {
    /// Original Path2D vertex order, prior to Java Area canonicalization.
    /// A collapsed horizontal path has no area and returns an empty polygon.
    pub polygon: Vec<[f64; 2]>,
    pub theoretical_line: [f64; 4],
}

// LineUtil.intersection, preserving Java's floating point operation order.
fn intersection(a: [f64; 4], b: [f64; 4]) -> [f64; 2] {
    let [x1, y1, x2, y2] = a;
    let [x3, y3, x4, y4] = b;
    let denominator = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    let cross_a = x1 * y2 - y1 * x2;
    let cross_b = x3 * y4 - y3 * x4;
    [
        (cross_a * (x3 - x4) - (x1 - x2) * cross_b) / denominator,
        (cross_a * (y3 - y4) - (y1 - y2) * cross_b) / denominator,
    ]
}

/// Generate the head lookup trapezoid from reference, directions, pixel
/// tolerances and a declared part/line limit. Only observed zero skew is
/// supported. Empty geometry is representable; downstream collectors may
/// refuse a zero-length theoretical line. Inputs remain unchanged.
pub fn head(request: &HeadRequest) -> Result<Geometry, String> {
    let r = request;
    if ![-1, 1].contains(&r.x_direction)
        || ![-1, 1].contains(&r.y_direction)
        || r.slope != 0.
        || !r.slope_margin.is_finite()
        || !(0.0..=1.0).contains(&r.slope_margin)
        || !(0..=1024).contains(&r.max_head_in_dx)
        || !(0..=1024).contains(&r.max_head_out_dx)
        || r.max_head_in_dx + r.max_head_out_dx == 0
        || r.reference
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || r.part_bounds.iter().any(|v| !(0..=4_000_000).contains(v))
        || r.part_bounds[2] == 0
        || r.part_bounds[3] == 0
        || r.part_bounds[1] + r.part_bounds[3] > 4_000_000
        || r.limit.is_some_and(|l| {
            l.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.) || l[0] == l[2]
        })
    {
        return Err("unsupported_head_lookup_area_input".into());
    }
    let [x, y] = r.reference;
    let end_y = match r.limit {
        Some(line) => intersection(line, [x, 0., x, 1000.])[1],
        None if r.y_direction > 0 => (r.part_bounds[1] + r.part_bounds[3] - 1) as f64,
        None => r.part_bounds[1] as f64,
    };
    let delta_y = end_y - y;
    let margin = (r.x_direction as i32 * r.y_direction as i32) as f64 * r.slope_margin;
    let in_x = x - (r.x_direction as i32 * r.max_head_in_dx) as f64;
    let out_x = x + (r.x_direction as i32 * r.max_head_out_dx) as f64;
    let target = intersection(
        [x, y, x - 100. * r.slope, y + 100.],
        [0., end_y, 100., end_y],
    );
    let polygon = if delta_y == 0. {
        vec![]
    } else {
        vec![
            [out_x, y],
            [in_x, y],
            [in_x + (-r.slope - margin) * delta_y, end_y],
            [out_x + (-r.slope + margin) * delta_y, end_y],
        ]
    };
    if polygon
        .iter()
        .flatten()
        .chain(target.iter())
        .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
    {
        return Err("unsupported_head_lookup_area_result".into());
    }
    // Opposite-side declared limits can make a self-crossing trapezoid.
    // Java Area may split these into multiple rings; that case is not certified.
    if !polygon.is_empty() && (polygon[2][0] - polygon[3][0]) * (in_x - out_x) <= 0. {
        return Err("unsupported_head_lookup_area_crossing".into());
    }
    Ok(Geometry {
        polygon,
        theoretical_line: [x, y, target[0], target[1]],
    })
}
