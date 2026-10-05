// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original LedgerSuite, Check and CheckSuite. Staff/reference selection is separate.
use crate::{GrayImage, stem_checks::Check};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LedgerStick {
    pub bounds: [i64; 4],
    pub start: [f64; 2],
    pub stop: [f64; 2],
    pub weight: usize,
    pub mean_distance: f64,
}

#[derive(Clone, Debug)]
pub struct Parameters {
    pub main_interline: usize,
    pub specific_interline: usize,
    pub fore: usize,
    pub target_y: f64,
    /// Original lookupLine uses wider length thresholds after a wide prior ledger.
    pub wide_previous: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LedgerCheckResult {
    pub checks: [Check; 7],
    pub grade: f64,
    pub min_threshold: f64,
    pub accepted: bool,
}

pub fn check(
    image: &GrayImage,
    stick: &LedgerStick,
    p: &Parameters,
) -> Result<LedgerCheckResult, String> {
    if !(1..=128).contains(&p.main_interline)
        || !(1..=128).contains(&p.specific_interline)
        || !(1..=128).contains(&p.fore)
        || !p.target_y.is_finite()
    {
        return Err("unsupported_ledger_check_scale_or_target".into());
    }
    if image.width == 0
        || image.height == 0
        || image.width.checked_mul(image.height) != Some(image.pixels.len())
        || image.pixels.len() > crate::MAX_PIXELS
    {
        return Err("invalid_ledger_check_image".into());
    }
    let [x, y, width, height] = stick.bounds;
    if x < 0
        || y < 0
        || width < 1
        || height < 1
        || x.checked_add(width).is_none_or(|v| v > image.width as i64)
        || y.checked_add(height)
            .is_none_or(|v| v > image.height as i64)
        || stick
            .start
            .iter()
            .chain(&stick.stop)
            .any(|v| !v.is_finite())
        || stick.start[0] >= stick.stop[0]
        || stick.weight == 0
        || !stick.mean_distance.is_finite()
        || stick.mean_distance < 0.
    {
        return Err("unsupported_ledger_check_geometry".into());
    }
    let mut convex = 0;
    for xx in [x, x + width - 1] {
        let above = y > 0 && image.pixels[((y - 1) * image.width as i64 + xx) as usize] == 0;
        let below = y + height < image.height as i64
            && image.pixels[((y + height) * image.width as i64 + xx) as usize] == 0;
        if !above && !below {
            convex += 1;
        }
    }
    let thickness = stick.weight as f64 / width as f64;
    let values = [
        thickness / p.main_interline as f64,
        thickness / p.fore as f64,
        width as f64 / p.specific_interline as f64,
        convex as f64,
        stick.mean_distance / p.main_interline as f64,
        (stick.start[1] - p.target_y).abs() / p.specific_interline as f64,
        (stick.stop[1] - p.target_y).abs() / p.specific_interline as f64,
    ];
    let specifications = [
        ("MinTh.", 0., 0.25, true, 0.5),
        ("MaxTh.", 1., 3.25, false, 0.),
        (
            "Length",
            if p.wide_previous { 1.4 } else { 1. },
            if p.wide_previous { 2. } else { 1.5 },
            true,
            4.,
        ),
        ("Convex", -0.5, 2., true, 2.),
        ("Straight", 0., 0.3, false, 1.),
        ("LPitch", 0., 0.35, false, 0.5),
        ("RPitch", 0., 0.35, false, 0.5),
    ];
    let checks = std::array::from_fn(|i| {
        let (name, low, high, covariant, weight) = specifications[i];
        Check {
            name,
            raw_value: values[i],
            low,
            high,
            covariant,
            weight,
            impact: crate::stem_checks::impact(values[i], low, high, covariant),
        }
    });
    let (mut product, mut weight_sum) = (1.0_f64, 0.0_f64);
    for c in &checks {
        weight_sum += c.weight;
        if c.impact == 0. {
            product = 0.;
        } else if c.weight != 0. {
            product *= c.impact.powf(c.weight);
        }
    }
    let grade = product.powf(1. / weight_sum) * 0.8;
    let min_threshold = 0.1 * 0.8;
    Ok(LedgerCheckResult {
        checks,
        grade,
        min_threshold,
        accepted: grade >= min_threshold,
    })
}
