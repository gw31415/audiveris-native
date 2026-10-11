//! Texts (Audiveris TEXTS step): words read by the Tesseract OCR engine on
//! the residual image (staff lines, notes and symbols removed). Tesseract is
//! run as an external program; without it no text is read.

use crate::binary::{Bitmap, Rect};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Word {
    pub bounds: Rect,
    pub text: String,
    pub conf: f64,
}

/// `languages`: Tesseract language names, e.g. "eng" or "deu+eng".
pub fn ocr(residual: &Bitmap, languages: &str) -> Vec<Word> {
    let dir = std::env::temp_dir().join(format!("audiveris-native-ocr-{}", std::process::id()));
    if std::fs::create_dir_all(&dir).is_err() {
        return Vec::new();
    }
    let png = dir.join("page.png");
    let img = image::GrayImage::from_fn(residual.width as u32, residual.height as u32, |x, y| {
        image::Luma([if residual.get(x as i32, y as i32) { 0 } else { 255 }])
    });
    let words = img
        .save(&png)
        .ok()
        .and_then(|_| {
            std::process::Command::new("tesseract")
                .arg(&png)
                .arg("stdout")
                .args(["-l", languages, "--psm", "11", "tsv"])
                .stderr(std::process::Stdio::null())
                .output()
                .ok()
        })
        .filter(|o| o.status.success())
        .map(|o| parse_tsv(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    words
}

fn parse_tsv(tsv: &str) -> Vec<Word> {
    tsv.lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            if f.len() < 12 || f[11].trim().is_empty() {
                return None;
            }
            let n = |i: usize| f[i].parse::<f64>().ok();
            Some(Word {
                bounds: Rect::new(n(6)? as i32, n(7)? as i32, n(8)? as i32, n(9)? as i32),
                text: f[11].trim().to_string(),
                conf: n(10)?,
            })
        })
        .collect()
}
