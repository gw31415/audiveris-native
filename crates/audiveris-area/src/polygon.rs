// Copyright (c) 1995, 2021, Oracle and/or its affiliates. All rights reserved.
// Copyright (c) 1998, 2018, Oracle and/or its affiliates. All rights reserved.
// Rust translation: 2026 audiveris-native contributors.
// SPDX-License-Identifier: GPL-2.0-only WITH Classpath-exception-2.0
// Polygon.intersects / Crossings.accumulateLine with EvenOdd interval parity.
// The Classpath exception is retained for this modified independent library.

pub fn polygon_intersects_rect(
    points: &[[i32; 2]],
    [x, y, w, h]: [i32; 4],
) -> Result<bool, String> {
    if points.len() > 128
        || points
            .iter()
            .flatten()
            .chain([x, y, w, h].iter())
            .any(|v| v.unsigned_abs() > 16_000_000)
    {
        return Err("invalid_polygon_geometry".into());
    }
    if points.is_empty() || w <= 0 || h <= 0 {
        return Ok(false);
    }
    let min_x = points.iter().map(|p| p[0]).min().unwrap();
    let max_x = points.iter().map(|p| p[0]).max().unwrap();
    let min_y = points.iter().map(|p| p[1]).min().unwrap();
    let max_y = points.iter().map(|p| p[1]).max().unwrap();
    if min_x == max_x
        || min_y == max_y
        || x + w <= min_x
        || y + h <= min_y
        || x >= max_x
        || y >= max_y
    {
        return Ok(false);
    }
    let (xlo, ylo, xhi, yhi) = (x as f64, y as f64, (x + w) as f64, (y + h) as f64);
    let mut endpoints = Vec::new();
    let mut previous = points[points.len() - 1];
    for &point in points {
        let (mut a, mut b) = (previous, point);
        previous = point;
        if a[1] > b[1] {
            std::mem::swap(&mut a, &mut b);
        }
        let ([x0, y0], [x1, y1]) = (a.map(f64::from), b.map(f64::from));
        if yhi <= y0 || ylo >= y1 || (x0 >= xhi && x1 >= xhi) {
            continue;
        }
        if y0 == y1 {
            if x0 >= xlo || x1 >= xlo {
                return Ok(true);
            }
            continue;
        }
        let (dx, dy) = (x1 - x0, y1 - y0);
        let (xs, ys) = if y0 < ylo {
            (x0 + (ylo - y0) * dx / dy, ylo)
        } else {
            (x0, y0)
        };
        let (xe, ye) = if yhi < y1 {
            (x0 + (yhi - y0) * dx / dy, yhi)
        } else {
            (x1, y1)
        };
        if xs >= xhi && xe >= xhi {
            continue;
        }
        if xs > xlo || xe > xlo {
            return Ok(true);
        }
        if ys < ye {
            endpoints.extend([ys, ye]);
        }
    }
    endpoints.sort_by(f64::total_cmp);
    Ok(endpoints.as_chunks::<2>().0.iter().any(|p| p[0] < p[1]))
}
