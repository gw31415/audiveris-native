//! Image loading, binarization (Audiveris AdaptiveFilter) and basic raster utilities.

use std::path::Path;

/// Binary raster, row-major, `true` = black (foreground).
#[derive(Clone)]
pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<bool>,
}

impl Bitmap {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, data: vec![false; width * height] }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < self.width
            && (y as usize) < self.height
            && self.data[y as usize * self.width + x as usize]
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, v: bool) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.data[y as usize * self.width + x as usize] = v;
        }
    }

    /// Length of the vertical black run containing (x, y), 0 if white.
    pub fn vrun_at(&self, x: i32, y: i32) -> (i32, i32) {
        if !self.get(x, y) {
            return (y, y - 1);
        }
        let mut a = y;
        while self.get(x, a - 1) {
            a -= 1;
        }
        let mut b = y;
        while self.get(x, b + 1) {
            b += 1;
        }
        (a, b)
    }

    pub fn hrun_at(&self, x: i32, y: i32) -> (i32, i32) {
        if !self.get(x, y) {
            return (x, x - 1);
        }
        let mut a = x;
        while self.get(a - 1, y) {
            a -= 1;
        }
        let mut b = x;
        while self.get(b + 1, y) {
            b += 1;
        }
        (a, b)
    }

    /// Per-pixel vertical run length (0 for white pixels).
    pub fn vertical_run_lengths(&self) -> Vec<u16> {
        let mut out = vec![0u16; self.data.len()];
        for x in 0..self.width {
            let mut y = 0;
            while y < self.height {
                if self.data[y * self.width + x] {
                    let s = y;
                    while y < self.height && self.data[y * self.width + x] {
                        y += 1;
                    }
                    let len = (y - s).min(u16::MAX as usize) as u16;
                    for yy in s..y {
                        out[yy * self.width + x] = len;
                    }
                } else {
                    y += 1;
                }
            }
        }
        out
    }

    pub fn horizontal_run_lengths(&self) -> Vec<u16> {
        let mut out = vec![0u16; self.data.len()];
        for y in 0..self.height {
            let row = &self.data[y * self.width..(y + 1) * self.width];
            let mut x = 0;
            while x < self.width {
                if row[x] {
                    let s = x;
                    while x < self.width && row[x] {
                        x += 1;
                    }
                    let len = (x - s).min(u16::MAX as usize) as u16;
                    for v in &mut out[y * self.width + s..y * self.width + x] {
                        *v = len;
                    }
                } else {
                    x += 1;
                }
            }
        }
        out
    }

    pub fn count_rect(&self, r: Rect) -> usize {
        let mut n = 0;
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                if self.get(x, y) {
                    n += 1;
                }
            }
        }
        n
    }
}

/// Integer rectangle (x, y, width, height).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
    pub fn cx(&self) -> f64 {
        self.x as f64 + self.w as f64 / 2.0
    }
    pub fn cy(&self) -> f64 {
        self.y as f64 + self.h as f64 / 2.0
    }
    pub fn intersection(&self, o: &Rect) -> Option<Rect> {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        (r > x && b > y).then(|| Rect::new(x, y, r - x, b - y))
    }
    pub fn intersects(&self, o: &Rect) -> bool {
        self.intersection(o).is_some()
    }
    pub fn union(&self, o: &Rect) -> Rect {
        let x = self.x.min(o.x);
        let y = self.y.min(o.y);
        Rect::new(x, y, self.right().max(o.right()) - x, self.bottom().max(o.bottom()) - y)
    }
    pub fn grow(&self, dx: i32, dy: i32) -> Rect {
        Rect::new(self.x - dx, self.y - dy, self.w + 2 * dx, self.h + 2 * dy)
    }
    pub fn area(&self) -> i32 {
        self.w * self.h
    }
}

/// Load any supported image file as 8-bit gray (alpha composited on white).
pub fn load_gray(path: &Path) -> Result<(usize, usize, Vec<u8>), String> {
    let img = image::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    let mut gray = Vec::with_capacity(w * h);
    for p in rgba.pixels() {
        let [r, g, b, a] = p.0;
        let lum = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
        let a = a as f64 / 255.0;
        gray.push((lum * a + 255.0 * (1.0 - a)).round().clamp(0.0, 255.0) as u8);
    }
    Ok((w, h, gray))
}

/// Audiveris AdaptiveFilter: 37x37 window, threshold = 0.7*mean + 0.9*stddev,
/// pixel is black when value <= threshold.
pub fn binarize(width: usize, height: usize, gray: &[u8]) -> Bitmap {
    let stride = width + 1;
    let mut sums = vec![0u64; stride * (height + 1)];
    let mut squares = vec![0u64; stride * (height + 1)];
    for y in 0..height {
        let mut rs = 0u64;
        let mut rq = 0u64;
        for x in 0..width {
            let v = gray[y * width + x] as u64;
            rs += v;
            rq += v * v;
            let i = (y + 1) * stride + x + 1;
            sums[i] = sums[i - stride] + rs;
            squares[i] = squares[i - stride] + rq;
        }
    }
    let mut out = Bitmap::new(width, height);
    for y in 0..height {
        let y1 = y.saturating_sub(18);
        let y2 = (y + 19).min(height);
        for x in 0..width {
            let x1 = x.saturating_sub(18);
            let x2 = (x + 19).min(width);
            let area = ((x2 - x1) * (y2 - y1)) as f64;
            let rect = |a: &[u64]| {
                (a[y2 * stride + x2] + a[y1 * stride + x1] - a[y1 * stride + x2] - a[y2 * stride + x1])
                    as f64
                    / area
            };
            let mean = rect(&sums);
            let std = (rect(&squares) - mean * mean).abs().sqrt();
            out.data[y * width + x] = (gray[y * width + x] as f64) <= 0.7 * mean + 0.9 * std;
        }
    }
    out
}

/// A connected component (8-connectivity), with its pixel list.
#[derive(Clone, Debug)]
pub struct Component {
    pub bounds: Rect,
    pub pixels: Vec<(i32, i32)>,
}

impl Component {
    pub fn weight(&self) -> usize {
        self.pixels.len()
    }
    /// Local boolean mask, row-major, sized to bounds.
    pub fn mask(&self) -> Vec<bool> {
        let b = self.bounds;
        let mut m = vec![false; (b.w * b.h) as usize];
        for &(x, y) in &self.pixels {
            m[((y - b.y) * b.w + (x - b.x)) as usize] = true;
        }
        m
    }
    pub fn merge(parts: &[&Component]) -> Component {
        let mut bounds = parts[0].bounds;
        let mut pixels = Vec::new();
        for p in parts {
            bounds = bounds.union(&p.bounds);
            pixels.extend_from_slice(&p.pixels);
        }
        Component { bounds, pixels }
    }
}

/// Connected components of black pixels within `roi` (whole image if None).
pub fn components(img: &Bitmap, roi: Option<Rect>) -> Vec<Component> {
    let roi = roi
        .and_then(|r| r.intersection(&Rect::new(0, 0, img.width as i32, img.height as i32)))
        .unwrap_or(Rect::new(0, 0, img.width as i32, img.height as i32));
    let mut seen = vec![false; (roi.w * roi.h) as usize];
    let mut out = Vec::new();
    let mut stack = Vec::new();
    for y0 in roi.y..roi.bottom() {
        for x0 in roi.x..roi.right() {
            let i0 = ((y0 - roi.y) * roi.w + (x0 - roi.x)) as usize;
            if seen[i0] || !img.get(x0, y0) {
                continue;
            }
            seen[i0] = true;
            stack.push((x0, y0));
            let mut pixels = Vec::new();
            let (mut minx, mut miny, mut maxx, mut maxy) = (x0, y0, x0, y0);
            while let Some((x, y)) = stack.pop() {
                pixels.push((x, y));
                minx = minx.min(x);
                maxx = maxx.max(x);
                miny = miny.min(y);
                maxy = maxy.max(y);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < roi.x || ny < roi.y || nx >= roi.right() || ny >= roi.bottom() {
                            continue;
                        }
                        let i = ((ny - roi.y) * roi.w + (nx - roi.x)) as usize;
                        if !seen[i] && img.get(nx, ny) {
                            seen[i] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }
            out.push(Component {
                bounds: Rect::new(minx, miny, maxx - minx + 1, maxy - miny + 1),
                pixels,
            });
        }
    }
    out
}

/// Chamfer 3-4 distance to the nearest black pixel. Pixels in `unknown` are
/// treated as background for propagation (they are reported separately).
pub fn chamfer_distance(img: &Bitmap) -> Vec<i32> {
    let (w, h) = (img.width, img.height);
    let inf = i32::MAX / 4;
    let mut d: Vec<i32> = img.data.iter().map(|&b| if b { 0 } else { inf }).collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut v = d[i];
            if x > 0 {
                v = v.min(d[i - 1] + 3);
            }
            if y > 0 {
                v = v.min(d[i - w] + 3);
                if x > 0 {
                    v = v.min(d[i - w - 1] + 4);
                }
                if x + 1 < w {
                    v = v.min(d[i - w + 1] + 4);
                }
            }
            d[i] = v;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            let mut v = d[i];
            if x + 1 < w {
                v = v.min(d[i + 1] + 3);
            }
            if y + 1 < h {
                v = v.min(d[i + w] + 3);
                if x + 1 < w {
                    v = v.min(d[i + w + 1] + 4);
                }
                if x > 0 {
                    v = v.min(d[i + w - 1] + 4);
                }
            }
            d[i] = v;
        }
    }
    d
}
