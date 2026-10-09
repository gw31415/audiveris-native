// Copyright (c) 1998, 2018, Oracle and/or its affiliates. All rights reserved.
// Rust translation 2026; SPDX-License-Identifier: GPL-2.0-only WITH Classpath-exception-2.0
// OpenJDK Curve/Order1.accumulateCrossings and Order2/3.nextVertical.
// Include inside audiveris-area's library root. Caller certifies a simple
// noncrossing band path; this is not a general Area normalization operation.
use super::{Area, Edge, cubic_t, polynomial, quadratic_roots, quadratic_t};

impl Edge {
    fn rect_t_for_y(&self, y: f64) -> Result<f64, String> {
        let first = self.points[0];
        let last = *self.points.last().unwrap();
        if y <= first[1] {
            return Ok(0.);
        }
        if y >= last[1] {
            return Ok(1.);
        }
        Ok(match self.points.len() {
            2 => (y - first[1]) / (last[1] - first[1]),
            3 => quadratic_t(y, self.y),
            _ => cubic_t(y, self.y)?,
        })
    }
    fn rect_next_vertical(&self, start: f64, mut end: f64) -> f64 {
        let roots = if self.points.len() == 3 {
            vec![-self.x[1] / (2. * self.x[2])]
        } else {
            quadratic_roots(self.x[1], 2. * self.x[2], 3. * self.x[3])
        };
        for root in roots {
            if root > start && root < end {
                end = root;
            }
        }
        end
    }
    fn rect_accumulate(
        &self,
        [xlo, ylo, xhi, yhi]: [f64; 4],
        intervals: &mut Vec<f64>,
    ) -> Result<bool, String> {
        let first = self.points[0];
        let last = *self.points.last().unwrap();
        if self.xmin >= xhi || last[1] <= ylo || first[1] >= yhi {
            return Ok(false);
        }
        let ystart = first[1].max(ylo);
        let yend = last[1].min(yhi);
        if self.points.len() == 2 {
            let x_for_y = |y: f64| {
                if first[0] == last[0] || y <= first[1] {
                    first[0]
                } else if y >= last[1] {
                    last[0]
                } else {
                    first[0] + (y - first[1]) * (last[0] - first[0]) / (last[1] - first[1])
                }
            };
            let start = x_for_y(ystart);
            let end = x_for_y(yend);
            if start >= xhi && end >= xhi {
                return Ok(false);
            }
            if start > xlo || end > xlo {
                return Ok(true);
            }
            intervals.extend([ystart, yend]);
            return Ok(false);
        }
        let mut t = self.rect_t_for_y(ystart)?;
        let end = self.rect_t_for_y(yend)?;
        let (mut hit_lo, mut hit_hi) = (false, false);
        for _ in 0..4 {
            let x = polynomial(self.x, t);
            if x < xhi {
                if hit_hi || x > xlo {
                    return Ok(true);
                }
                hit_lo = true;
            } else {
                if hit_lo {
                    return Ok(true);
                }
                hit_hi = true;
            }
            if t >= end {
                if hit_lo {
                    intervals.extend([ystart, yend]);
                }
                return Ok(false);
            }
            let next = self.rect_next_vertical(t, end);
            if next <= t {
                return Err("area_rectangle_extrema_progress_required".into());
            }
            t = next;
        }
        Err("area_rectangle_extrema_budget".into())
    }
}
impl Area {
    /// Exact Curve crossing operations for one certified non-self-crossing band.
    /// A caller must not apply this to overlapping/self-crossing unnormalized paths.
    pub fn intersects_rect_for_noncrossing_path(
        &self,
        [x, y, w, h]: [f64; 4],
    ) -> Result<bool, String> {
        if [x, y, w, h]
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 16_000_000.)
        {
            return Err("invalid_area_rectangle".into());
        }
        if w <= 0. || h <= 0. {
            return Ok(false);
        }
        let mut endpoints = Vec::new();
        for edge in &self.edges {
            if edge.rect_accumulate([x, y, x + w, y + h], &mut endpoints)? {
                return Ok(true);
            }
        }
        endpoints.sort_by(f64::total_cmp);
        Ok(endpoints.as_chunks::<2>().0.iter().any(|p| p[0] < p[1]))
    }
}
