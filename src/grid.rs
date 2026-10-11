//! Barlines, systems and parts (Audiveris BarsRetriever / SystemsBuilder).
//!
//! Barline candidates are vertical peaks of black pixels spanning the whole
//! staff height. A peak is kept when both of its ends are clean (no head-like
//! material beyond the staff) or connect to the neighboring staff. Staves
//! linked by a connector belong to the same system; staves joined by a brace
//! belong to the same part.

use crate::binary::{Bitmap, Rect, components};
use crate::scale::Scale;
use crate::staff::Staff;

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Bar {
    pub x0: i32,
    pub x1: i32,
    pub connected_up: bool,
    pub connected_down: bool,
}

impl Bar {
    pub fn width(&self) -> i32 {
        self.x1 - self.x0 + 1
    }
    pub fn cx(&self) -> f64 {
        (self.x0 + self.x1) as f64 / 2.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum BarStyle {
    Regular,
    LightLight,
    LightHeavy,
    HeavyLight,
    Heavy,
}

/// A measure boundary of a whole system.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Boundary {
    pub x0: i32,
    pub x1: i32,
    pub style: BarStyle,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct System {
    pub staves: Vec<usize>,
    /// Groups of staff indices forming parts (in order).
    pub parts: Vec<Vec<usize>>,
    /// Measure boundaries; measure i lies between boundaries i and i+1.
    pub boundaries: Vec<Boundary>,
    pub left: i32,
    pub right: i32,
}

fn column_black(img: &Bitmap, x: i32, y0: i32, y1: i32) -> usize {
    (y0..=y1).filter(|&y| img.get(x, y)).count()
}

/// Staff-level barline peaks.
pub fn staff_bars(img: &Bitmap, staves: &[Staff], si: usize, scale: &Scale) -> Vec<Bar> {
    let s = &staves[si];
    let il = scale.interline;
    let mut peaks: Vec<(i32, i32)> = Vec::new();
    let mut cur: Option<(i32, i32)> = None;
    for x in (s.left - 3)..=(s.right + 3) {
        let y0 = s.top(x as f64).round() as i32;
        let y1 = s.bottom(x as f64).round() as i32;
        let n = column_black(img, x, y0, y1);
        let full = n as f64 >= 0.97 * (y1 - y0 + 1) as f64;
        match (&mut cur, full) {
            (Some(c), true) => c.1 = x,
            (None, true) => cur = Some((x, x)),
            (Some(c), false) => {
                peaks.push(*c);
                cur = None;
            }
            _ => {}
        }
    }
    if let Some(c) = cur {
        peaks.push(c);
    }
    let mut bars = Vec::new();
    for (x0, x1) in peaks {
        if x1 - x0 + 1 > il {
            continue;
        }
        let xm = (x0 + x1) as f64 / 2.0;
        let top = s.top(xm).round() as i32;
        let bot = s.bottom(xm).round() as i32;
        let up = (si > 0)
            .then(|| &staves[si - 1])
            .filter(|p| connected(img, x0, x1, p.bottom(xm).round() as i32, top));
        let down = staves
            .get(si + 1)
            .filter(|n| connected(img, x0, x1, bot, n.top(xm).round() as i32));
        // A bar connected on one side only needs to stop at the staff on the other side.
        let connected = up.is_some() || down.is_some();
        let clean_up = up.is_some() || clean_end(img, x0, x1, top, -1, scale, connected);
        let clean_down = down.is_some() || clean_end(img, x0, x1, bot, 1, scale, connected);
        if clean_up && clean_down {
            bars.push(Bar {
                x0,
                x1,
                connected_up: up.is_some(),
                connected_down: down.is_some(),
            });
        }
    }
    bars
}

/// True if the column band [x0,x1] stays (mostly) black from y0 to y1.
fn connected(img: &Bitmap, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if y1 <= y0 {
        return false;
    }
    let mut best = 0;
    for x in x0..=x1 {
        best = best.max(column_black(img, x, y0, y1));
    }
    best as f64 >= 0.97 * (y1 - y0 + 1) as f64
}

/// Is the area just beyond the staff end (above if dir<0, below if dir>0) free of material?
fn clean_end(
    img: &Bitmap,
    x0: i32,
    x1: i32,
    y: i32,
    dir: i32,
    scale: &Scale,
    stop_only: bool,
) -> bool {
    let il = scale.interline;
    let dx = il / 2;
    let h = (il * 2) / 3;
    // Leave out the staff line itself where it is thicker than usual or the
    // fitted staff is a few pixels off: follow the line's ink beside the bar.
    let ink = |x: i32| (0..=scale.max_line + 2).take_while(|d| img.get(x, y + dir * d)).count() as i32;
    let edge = ink(x0 - 2).min(ink(x1 + 2));
    let (above, below) = (2.max(edge), 3.max(edge));
    let r = if dir < 0 {
        Rect::new(x0 - dx, y - h + 1 - above, x1 - x0 + 1 + 2 * dx, h - 1)
    } else {
        Rect::new(x0 - dx, y + below, x1 - x0 + 1 + 2 * dx, h - 1)
    };
    let n = img.count_rect(r);
    // The bar itself must not continue beyond the staff either.
    let r2 = if dir < 0 {
        Rect::new(x0, y - h + 1 - above, x1 - x0 + 1, h - 1)
    } else {
        Rect::new(x0, y + below, x1 - x0 + 1, h - 1)
    };
    let stops = (img.count_rect(r2) as f64) < 0.3 * r2.area() as f64;
    stops && (stop_only || (n as f64) < 0.08 * r.area() as f64)
}

pub fn build_systems(img: &Bitmap, staves: &mut [Staff], scale: &Scale) -> Vec<System> {
    let il = scale.interline;
    let bars: Vec<Vec<Bar>> = (0..staves.len())
        .map(|i| staff_bars(img, staves, i, scale))
        .collect();
    // Systems: chain staves linked by a connected bar.
    let mut systems: Vec<Vec<usize>> = Vec::new();
    for i in 0..staves.len() {
        let linked = i > 0 && bars[i].iter().any(|b| b.connected_up);
        if linked {
            systems.last_mut().unwrap().push(i);
        } else {
            systems.push(vec![i]);
        }
    }
    let mut out = Vec::new();
    for sys in systems {
        // Align staff horizontal limits within the system.
        let left = sys.iter().map(|&i| staves[i].left).min().unwrap();
        let right = sys.iter().map(|&i| staves[i].right).max().unwrap();
        // Measure boundaries: cluster bar abscissae over the staves.
        let mut all: Vec<Bar> = sys.iter().flat_map(|&i| bars[i].iter().copied()).collect();
        all.sort_by_key(|b| b.x0);
        let mut clusters: Vec<Vec<Bar>> = Vec::new();
        for b in all {
            match clusters.last_mut() {
                Some(c) if (b.cx() - c[0].cx()).abs() <= (il / 2) as f64 => c.push(b),
                _ => clusters.push(vec![b]),
            }
        }
        let quorum = sys.len().div_ceil(2);
        let lines: Vec<Bar> = clusters
            .into_iter()
            .filter(|c| c.len() >= quorum || c.iter().any(|b| b.connected_up || b.connected_down))
            .map(|c| Bar {
                x0: (c.iter().map(|b| b.x0).sum::<i32>() as f64 / c.len() as f64).round() as i32,
                x1: (c.iter().map(|b| b.x1).sum::<i32>() as f64 / c.len() as f64).round() as i32,
                connected_up: false,
                connected_down: false,
            })
            .collect();
        // Group close bar lines (double / final / repeat bars).
        let thick = scale.frac(0.4);
        let mut boundaries: Vec<Boundary> = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            let mut j = i;
            while j + 1 < lines.len() && lines[j + 1].x0 - lines[j].x1 <= il {
                j += 1;
            }
            let group = &lines[i..=j];
            let heavy: Vec<bool> = group.iter().map(|b| b.width() as f64 >= thick).collect();
            let style = match heavy.as_slice() {
                [false] => BarStyle::Regular,
                [true] => BarStyle::Heavy,
                [false, true, ..] => BarStyle::LightHeavy,
                [true, false, ..] => BarStyle::HeavyLight,
                _ => BarStyle::LightLight,
            };
            boundaries.push(Boundary {
                x0: group[0].x0,
                x1: group[j - i].x1,
                style,
            });
            i = j + 1;
        }
        // Ensure a starting boundary at the system left.
        if boundaries.first().is_none_or(|b| b.x0 > left + il) {
            boundaries.insert(
                0,
                Boundary {
                    x0: left,
                    x1: left,
                    style: BarStyle::Regular,
                },
            );
        }
        if boundaries.last().is_none_or(|b| b.x1 < right - il) {
            boundaries.push(Boundary {
                x0: right,
                x1: right,
                style: BarStyle::Regular,
            });
        }
        for &si in &sys {
            staves[si].left = boundaries[0].x1 + 1;
            staves[si].right = boundaries.last().unwrap().x0 - 1;
        }
        let parts = detect_parts(img, staves, &sys, scale);
        out.push(System {
            staves: sys,
            parts,
            boundaries,
            left,
            right,
        });
    }
    out
}

/// Group staves into parts using braces at the left of the system.
fn detect_parts(img: &Bitmap, staves: &[Staff], sys: &[usize], scale: &Scale) -> Vec<Vec<usize>> {
    let il = scale.interline;
    let mut parts: Vec<Vec<usize>> = vec![vec![sys[0]]];
    for w in sys.windows(2) {
        let (a, b) = (&staves[w[0]], &staves[w[1]]);
        let x = a.left.min(b.left);
        let top = a.top(x as f64).round() as i32;
        let bot = b.bottom(x as f64).round() as i32;
        let roi = Rect::new(x - 4 * il, top - il, 4 * il - 1, bot - top + 2 * il);
        let span = (bot - top) as f64;
        let brace = components(img, Some(roi)).into_iter().any(|c| {
            if (c.bounds.h as f64) < 0.85 * span || c.bounds.w > 3 * il {
                return false;
            }
            // A bracket has a straight spine covering nearly all its height; a brace does not.
            let mut best = 0;
            for xx in c.bounds.x..c.bounds.right() {
                let n = c.pixels.iter().filter(|p| p.0 == xx).count();
                best = best.max(n);
            }
            (best as f64) < 0.8 * c.bounds.h as f64
        });
        if brace {
            parts.last_mut().unwrap().push(w[1]);
        } else {
            parts.push(vec![w[1]]);
        }
    }
    parts
}
