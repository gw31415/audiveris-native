//! Head templates (Audiveris TemplateFactory / Template).
//!
//! A notehead glyph of the Bravura font is rasterized at the sheet point size
//! and binarized. Key points are the foreground pixels, the background pixels
//! close to the symbol (chamfer distance <= 8 * pointSize / 80) and the hole
//! pixels of hollow heads. Matching distance is the weighted ratio of key
//! points whose color disagrees with the image (fore 6, hole 4, back 1).

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};

use crate::binary::{Bitmap, Rect, chamfer_distance};

pub static BRAVURA: &[u8] = include_bytes!("../assets/Bravura.otf");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub enum HeadShape {
    Black,
    Void,
    Whole,
    Breve,
}

impl HeadShape {
    pub const ALL: [HeadShape; 4] = [
        HeadShape::Breve,
        HeadShape::Whole,
        HeadShape::Void,
        HeadShape::Black,
    ];
    fn codepoint(self) -> char {
        match self {
            HeadShape::Black => '\u{E0A4}',
            HeadShape::Void => '\u{E0A3}',
            HeadShape::Whole => '\u{E0A2}',
            HeadShape::Breve => '\u{E0A0}',
        }
    }
    pub fn has_hole(self) -> bool {
        self != HeadShape::Black
    }
    pub fn stemless(self) -> bool {
        matches!(self, HeadShape::Whole | HeadShape::Breve)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Fore,
    Back,
    Hole,
}

pub struct Template {
    pub shape: HeadShape,
    pub width: i32,
    pub height: i32,
    pub slim: Rect,
    pub keys: Vec<(i32, i32, Kind)>,
}

/// Image cell states used for matching.
pub const WHITE: u8 = 0;
pub const BLACK: u8 = 1;
pub const UNKNOWN: u8 = 2;

impl Template {
    /// Offset of the MIDDLE_LEFT anchor within the template.
    pub fn middle_left(&self) -> (i32, i32) {
        let ox = self.slim.x as f64;
        let oy = self.slim.y as f64 + self.slim.h as f64 / 2.0;
        ((ox - 0.5 - 0.001).round() as i32, (oy - 0.5).round() as i32)
    }

    /// Slim bounds when anchored MIDDLE_LEFT at (x, y).
    pub fn slim_at(&self, x: i32, y: i32) -> Rect {
        let (ox, oy) = self.middle_left();
        Rect::new(
            x - ox + self.slim.x,
            y - oy + self.slim.y,
            self.slim.w,
            self.slim.h,
        )
    }

    /// Matching distance in [0, 1] anchored MIDDLE_LEFT at (x, y).
    pub fn evaluate(&self, cells: &[u8], w: i32, h: i32, x: i32, y: i32) -> f64 {
        let (ox, oy) = self.middle_left();
        let (ux, uy) = (x - ox, y - oy);
        let mut total = 0.0;
        let mut weights = 0.0;
        for &(kx, ky, kind) in &self.keys {
            let (nx, ny) = (ux + kx, uy + ky);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let c = cells[(ny * w + nx) as usize];
            if c == UNKNOWN {
                continue;
            }
            let (weight, expect_black) = match kind {
                Kind::Fore => (6.0, true),
                Kind::Back => (1.0, false),
                Kind::Hole => (4.0, false),
            };
            weights += weight;
            if (c == BLACK) != expect_black {
                total += weight;
            }
        }
        if weights == 0.0 {
            f64::MAX
        } else {
            total / weights
        }
    }

    /// Ratio of hole key points actually white.
    pub fn hole_white_ratio(&self, cells: &[u8], w: i32, h: i32, x: i32, y: i32) -> f64 {
        let (ox, oy) = self.middle_left();
        let (ux, uy) = (x - ox, y - oy);
        let (mut expected, mut actual) = (0, 0);
        for &(kx, ky, kind) in &self.keys {
            let (nx, ny) = (ux + kx, uy + ky);
            if !matches!(kind, Kind::Hole) || nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let c = cells[(ny * w + nx) as usize];
            if c == UNKNOWN {
                continue;
            }
            expected += 1;
            if c == WHITE {
                actual += 1;
            }
        }
        if expected == 0 {
            0.0
        } else {
            actual as f64 / expected as f64
        }
    }

    /// Ratio of foreground key points actually black (unknown cells ignored).
    pub fn fore_black_ratio(&self, cells: &[u8], w: i32, h: i32, x: i32, y: i32) -> f64 {
        let (ux, uy) = self.origin_at(x, y);
        let (mut n, mut b) = (0, 0);
        for (kx, ky) in self.fore_points() {
            let (nx, ny) = (ux + kx, uy + ky);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let c = cells[(ny * w + nx) as usize];
            if c == UNKNOWN {
                continue;
            }
            n += 1;
            if c == BLACK {
                b += 1;
            }
        }
        if n == 0 { 0.0 } else { b as f64 / n as f64 }
    }

    /// Foreground key points (template coordinates).
    pub fn fore_points(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.keys
            .iter()
            .filter(|k| matches!(k.2, Kind::Fore))
            .map(|k| (k.0, k.1))
    }

    pub fn origin_at(&self, x: i32, y: i32) -> (i32, i32) {
        let (ox, oy) = self.middle_left();
        (x - ox, y - oy)
    }
}

/// Rasterize a Bravura glyph with an em size of `point_size` pixels.
/// Returns a binary bitmap (threshold 140 like Audiveris) and its offset.
pub fn rasterize(c: char, point_size: f64, pad: i32) -> Option<Bitmap> {
    let font = FontRef::try_from_slice(BRAVURA).ok()?;
    let px = point_size * font.height_unscaled() as f64 / font.units_per_em()? as f64;
    let scaled = font.as_scaled(PxScale::from(px as f32));
    let glyph = scaled.scaled_glyph(c);
    let outlined = font.outline_glyph(glyph)?;
    let b = outlined.px_bounds();
    let w = (b.max.x - b.min.x).ceil() as i32 + 2 * pad;
    let h = (b.max.y - b.min.y).ceil() as i32 + 2 * pad;
    let mut bm = Bitmap::new(w as usize, h as usize);
    outlined.draw(|x, y, cov| {
        // gray = 255 * (1 - cov); foreground when gray < 140
        if 255.0 * (1.0 - cov) < 140.0 {
            bm.set(x as i32 + pad, y as i32 + pad, true);
        }
    });
    let _ = point(0.0, 0.0);
    Some(bm)
}

pub fn build(shape: HeadShape, point_size: i32) -> Option<Template> {
    let max_dist = (8.0 * point_size as f64 / 80.0).round() as i32; // chamfer units (3 per pixel)
    let pad = max_dist / 3 + 2;
    let bm = rasterize(shape.codepoint(), point_size as f64, pad)?;
    let (w, h) = (bm.width as i32, bm.height as i32);
    let dist = chamfer_distance(&bm);
    // Holes: background not reachable from the border.
    let mut outside = vec![false; bm.data.len()];
    if shape.has_hole() {
        let mut stack: Vec<(i32, i32)> = Vec::new();
        for x in 0..w {
            stack.push((x, 0));
            stack.push((x, h - 1));
        }
        for y in 0..h {
            stack.push((0, y));
            stack.push((w - 1, y));
        }
        while let Some((x, y)) = stack.pop() {
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let i = (y * w + x) as usize;
            if outside[i] || bm.data[i] {
                continue;
            }
            outside[i] = true;
            stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
        }
    }
    let mut keys = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if bm.data[i] {
                keys.push((x, y, Kind::Fore));
            } else if shape.has_hole() && !outside[i] {
                keys.push((x, y, Kind::Hole));
            } else if dist[i] <= max_dist {
                keys.push((x, y, Kind::Back));
            }
        }
    }
    // Slim bounds: rows/columns with at least 2 foreground pixels.
    let col = |x: i32| (0..h).filter(|&y| bm.get(x, y)).count() >= 2;
    let row = |y: i32| (0..w).filter(|&x| bm.get(x, y)).count() >= 2;
    let x1 = (0..w).find(|&x| col(x))?;
    let x2 = (0..w).rev().find(|&x| col(x))?;
    let y1 = (0..h).find(|&y| row(y))?;
    let y2 = (0..h).rev().find(|&y| row(y))?;
    Some(Template {
        shape,
        width: w,
        height: h,
        slim: Rect::new(x1, y1, x2 - x1 + 1, y2 - y1 + 1),
        keys,
    })
}

/// Width in pixels of the black notehead at a given point size.
pub fn black_head_width(point_size: f64) -> f64 {
    let font = FontRef::try_from_slice(BRAVURA).unwrap();
    let px = point_size * font.height_unscaled() as f64 / font.units_per_em().unwrap() as f64;
    let scaled = font.as_scaled(PxScale::from(px as f32));
    let g = scaled.scaled_glyph('\u{E0A4}');
    scaled.h_advance(g.id) as f64
}
