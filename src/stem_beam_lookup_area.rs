// SPDX-License-Identifier: AGPL-3.0-or-later
//! Beam VLinker.buildLuArea at original 5.11. Border, staff neighborhoods and
//! profile-converted gap pixels are declared inputs; no whole-page completion.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub reference: [f64; 2],
    pub border: [f64; 4],
    pub direction: i8,
    pub slope: f64,
    pub slope_margin: f64,
    pub half_lookup_dx: f64,
    pub max_seed_dy_ratio: f64,
    pub gap_pixels: i32,
    pub system_bounds: [i32; 4],
    pub surrounding_part_bounds: Vec<[i32; 4]>,
    pub limit: Option<[f64; 4]>,
}
#[derive(Debug, Serialize)]
pub struct Geometry {
    pub polygon: Vec<[f64; 2]>,
    pub theoretical_line: [f64; 4],
}
// Original LineUtil.intersection. Keep the arithmetic order of the separately
// certified head implementation without changing its frozen source contract.
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
fn rectangle_valid(b: &[i32; 4]) -> bool {
    b.iter().all(|v| (0..=4_000_000).contains(v))
        && b[2] > 0
        && b[3] > 0
        && b[0] + b[2] <= 4_000_000
        && b[1] + b[3] <= 4_000_000
}
/// Derive a single convex beam search path. The default end combines the
/// system edge and all declared surrounding part edges, preserving -1 on
/// inclusive part bottoms. Limits use original yAtX arithmetic. Inputs and
/// registry/cache/head/SIG state are unchanged.
pub fn generate(r: &Request) -> Result<Geometry, String> {
    if ![-1, 1].contains(&r.direction)
        || r.slope != 0.
        || !r.slope_margin.is_finite()
        || !(0.0..=1.0).contains(&r.slope_margin)
        || !r.half_lookup_dx.is_finite()
        || !(0.0..=1024.0).contains(&r.half_lookup_dx)
        || r.half_lookup_dx == 0.
        || !r.max_seed_dy_ratio.is_finite()
        || !(0.0..=1.0).contains(&r.max_seed_dy_ratio)
        || !(0..=1024).contains(&r.gap_pixels)
        || r.reference
            .iter()
            .chain(r.border.iter())
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || r.border[0] == r.border[2]
        || !rectangle_valid(&r.system_bounds)
        || r.surrounding_part_bounds.len() > 4096
        || r.surrounding_part_bounds
            .iter()
            .any(|b| !rectangle_valid(b))
        || r.limit.is_some_and(|l| {
            l.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.) || l[0] == l[2]
        })
    {
        return Err("unsupported_beam_lookup_area_input".into());
    }
    let [x, y] = r.reference;
    let left = intersection(
        r.border,
        [x - r.half_lookup_dx, 0., x - r.half_lookup_dx, 1000.],
    );
    let right = intersection(
        r.border,
        [x + r.half_lookup_dx, 0., x + r.half_lookup_dx, 1000.],
    );
    let end_y = match r.limit {
        Some(l) => intersection(l, [x, 0., x, 1000.])[1],
        None => {
            let mut end = if r.direction < 0 {
                (r.system_bounds[1] + r.system_bounds[3]) as f64
            } else {
                r.system_bounds[1] as f64
            };
            for b in &r.surrounding_part_bounds {
                end = if r.direction > 0 {
                    end.max((b[1] + b[3] - 1) as f64)
                } else {
                    end.min(b[1] as f64)
                };
            }
            end
        }
    };
    let delta_y = end_y - y;
    let margin = r.direction as f64 * r.slope_margin;
    let seed_dy = r.direction as f64 * r.max_seed_dy_ratio * r.gap_pixels as f64;
    let polygon = vec![
        [left[0], left[1] + seed_dy],
        [right[0], right[1] + seed_dy],
        [right[0] + (-r.slope + margin) * delta_y, end_y],
        [left[0] + (-r.slope - margin) * delta_y, end_y],
    ];
    let target = intersection(
        [x, y, x - 100. * r.slope, y + 100.],
        [0., end_y, 100., end_y],
    );
    if polygon
        .iter()
        .flatten()
        .chain(target.iter())
        .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
    {
        return Err("unsupported_beam_lookup_area_result".into());
    }
    let mut sign = 0f64;
    for i in 0..4 {
        let (a, b, c) = (polygon[i], polygon[(i + 1) % 4], polygon[(i + 2) % 4]);
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        if cross == 0. || (sign != 0. && cross.signum() != sign) {
            return Err("unsupported_beam_lookup_area_nonconvex".into());
        }
        sign = cross.signum();
    }
    Ok(Geometry {
        polygon,
        theoretical_line: [x, y, target[0], target[1]],
    })
}
