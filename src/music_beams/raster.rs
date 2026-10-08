// SPDX-License-Identifier: AGPL-3.0-or-later
pub(super) fn rows(median: [f64; 4], height: f64, margin: f32) -> Result<Vec<[i64; 3]>, String> {
    if !(1.0..1.5).contains(&margin)
        || median
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || !height.is_finite()
        || height <= 0.
        || height > 4096.
        || median[0] >= median[2]
    {
        return Err("beam PageCleaner invalid/wider stroke raster pending".into());
    }
    let raw = [
        [median[0], median[1] - height / 2.],
        [median[2], median[3] - height / 2.],
        [median[2], median[3] + height / 2.],
        [median[0], median[1] + height / 2.],
    ];
    let normalized = raw.map(|p| p.map(|v| (v + 0.25).floor() + 0.25));
    let half = margin as f64 / 2.;
    let mut edges = [([0.; 2], [0.; 2]); 4];
    for i in 0..4 {
        let a = normalized[i];
        let b = normalized[(i + 1) % 4];
        let d = [b[0] - a[0], b[1] - a[1]];
        let length = d[0].hypot(d[1]);
        if length == 0. {
            return Err("beam PageCleaner collapsed contour pending".into());
        }
        edges[i] = (
            [a[0] + half * d[1] / length, a[1] - half * d[0] / length],
            d,
        );
    }
    let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
    let mut outer = [[0.; 2]; 4];
    for i in 0..4 {
        let (a, u) = edges[(i + 3) % 4];
        let (b, v) = edges[i];
        let determinant = cross(u, v);
        if determinant <= 0. {
            return Err("beam PageCleaner nonconvex contour pending".into());
        }
        let t = cross([b[0] - a[0], b[1] - a[1]], v) / determinant;
        outer[i] = [a[0] + t * u[0], a[1] + t * u[1]];
        if (outer[i][0] - normalized[i][0]).hypot(outer[i][1] - normalized[i][1]) > 10. * half {
            return Err("beam PageCleaner bevel join pending".into());
        }
    }
    let minimum = |axis| {
        raw.iter()
            .chain(&outer)
            .map(|p| p[axis])
            .fold(f64::INFINITY, f64::min)
            .floor() as i64
    };
    let maximum = |axis| {
        raw.iter()
            .chain(&outer)
            .map(|p| p[axis])
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil() as i64
    };
    let (left, top, right, bottom) = (minimum(0), minimum(1), maximum(0), maximum(1));
    if (right - left) * (bottom - top) > crate::MAX_PIXELS as i64 {
        return Err("beam PageCleaner raster budget exceeded".into());
    }
    let contains = |polygon: &[[f64; 2]; 4], p: [f64; 2]| {
        (0..4).all(|i| {
            let a = polygon[i];
            let b = polygon[(i + 1) % 4];
            cross([b[0] - a[0], b[1] - a[1]], [p[0] - a[0], p[1] - a[1]]) >= 0.
        })
    };
    let mut rows = Vec::new();
    for y in top..bottom {
        let mut span = None;
        let mut closed = false;
        for x in left..right {
            let p = [x as f64 + 0.5, y as f64 + 0.5];
            // Filling uses the raw area; stroking normalizes its vertices.
            // Keep both when normalization moves a raw foreground boundary.
            if contains(&raw, p) || contains(&outer, p) {
                if closed {
                    return Err("beam PageCleaner disjoint raster row pending".into());
                }
                let first = span.map_or(x, |s: [i64; 2]| s[0]);
                span = Some([first, x + 1]);
            } else if span.is_some() {
                closed = true;
            }
        }
        if let Some([left, right]) = span {
            rows.push([y, left, right]);
        }
    }
    Ok(rows)
}
