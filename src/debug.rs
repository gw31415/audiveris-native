//! Debug overlay: draws recognized items over the binary image.

use crate::binary::{Bitmap, Rect};
use image::{Rgb, RgbImage};

pub struct Overlay {
    pub img: RgbImage,
}

impl Overlay {
    pub fn new(bin: &Bitmap) -> Self {
        let mut img = RgbImage::new(bin.width as u32, bin.height as u32);
        for (i, &b) in bin.data.iter().enumerate() {
            let v = if b { 150 } else { 255 };
            img.put_pixel(
                (i % bin.width) as u32,
                (i / bin.width) as u32,
                Rgb([v, v, v]),
            );
        }
        Self { img }
    }
    pub fn dot(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if x >= 0 && y >= 0 && (x as u32) < self.img.width() && (y as u32) < self.img.height() {
            self.img.put_pixel(x as u32, y as u32, Rgb(c));
        }
    }
    pub fn rect(&mut self, r: Rect, c: [u8; 3]) {
        for x in r.x..r.right() {
            self.dot(x, r.y, c);
            self.dot(x, r.bottom() - 1, c);
        }
        for y in r.y..r.bottom() {
            self.dot(r.x, y, c);
            self.dot(r.right() - 1, y, c);
        }
    }
    pub fn line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, c: [u8; 3]) {
        let n = ((x1 - x0).abs().max((y1 - y0).abs()) as i32).max(1);
        for i in 0..=n {
            let t = i as f64 / n as f64;
            self.dot(
                (x0 + t * (x1 - x0)).round() as i32,
                (y0 + t * (y1 - y0)).round() as i32,
                c,
            );
        }
    }
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        self.img.save(path).map_err(|e| e.to_string())
    }
}
