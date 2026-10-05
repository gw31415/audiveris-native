// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original 5.11 HeadStemRelation dependency kernels with default constants.
//! These do not attach stems, mutate SIG, or certify whole HEADS/STEMS.
use serde::Serialize;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StemPortion {
    StemTop,
    StemMiddle,
    StemBottom,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct GapLimits {
    pub x_in_max: f64,
    pub x_out_max: f64,
    pub y_max: f64,
}
/// Fractions of the main interline; only original profiles 0, 1 and 2 certified.
pub fn gap_limits(profile: u8) -> Result<GapLimits, String> {
    match profile {
        0 => Ok(GapLimits {
            x_in_max: 0.2,
            x_out_max: 0.15,
            y_max: 0.8,
        }),
        1 => Ok(GapLimits {
            x_in_max: 0.4,
            x_out_max: 0.25,
            y_max: 1.2,
        }),
        2 => Ok(GapLimits {
            x_in_max: 0.4,
            x_out_max: 0.35,
            y_max: 1.2,
        }),
        _ => Err("unsupported_head_stem_profile".into()),
    }
}
/// `anchor_y` is the original extension-point ordinate. Preserve strict
/// endpoint margin comparisons and the inclusive midpoint branch, including
/// equal/reversed endpoints in this standalone original geometry kernel.
pub fn stem_portion(
    head_bounds: [i32; 4],
    stem: [f64; 4],
    anchor_y: f64,
) -> Result<StemPortion, String> {
    if head_bounds[2] <= 0
        || head_bounds[3] <= 0
        || head_bounds.iter().any(|v| v.unsigned_abs() > 4_000_000)
        || stem.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || !anchor_y.is_finite()
        || anchor_y.abs() > 4_000_000.
    {
        return Err("invalid_head_stem_geometry".into());
    }
    let margin = head_bounds[3] as f64 * 0.275;
    let midpoint = (stem[1] + stem[3]) / 2.;
    Ok(if anchor_y >= midpoint {
        if anchor_y > stem[3] - margin {
            StemPortion::StemBottom
        } else {
            StemPortion::StemMiddle
        }
    } else if anchor_y < stem[1] + margin {
        StemPortion::StemTop
    } else {
        StemPortion::StemMiddle
    })
}
/// Original unset consistency is 1. Length is in main-interline units. Preserve
/// the division order (small uses 1 / (length / 2.8)), including +Infinity for a
/// zero-length small stem. Callers must handle that before serializing to JSON.
pub fn consistency(small: bool, length_interlines: Option<f64>) -> Result<f64, String> {
    let Some(length) = length_interlines else {
        return Ok(1.);
    };
    if !length.is_finite() || length < 0. || length > 4_000_000. {
        return Err("invalid_head_stem_length".into());
    }
    let ratio = length / 2.8;
    Ok(if small { 1. / ratio } else { ratio })
}
/// Original invading relation gate: signed horizontal fraction <= 0.05 and
/// vertical fraction <= 0, inclusive. Inputs are fractions, never pixel gaps.
pub fn is_invading(dx: f64, dy: f64) -> Result<bool, String> {
    if !dx.is_finite() || !dy.is_finite() || dx.abs() > 4_000_000. || dy.abs() > 4_000_000. {
        return Err("invalid_head_stem_gap".into());
    }
    Ok(dx <= 0.05 && dy <= 0.)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeadSide {
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerticalSide {
    Top,
    Bottom,
}

/// Original AbstractConnection impacts and Support ratios, with unset
/// consistency (1). Signed dx selects the original in/out weight (1/2).
pub fn gap_grade(dx: f64, dy: f64, profile: u8) -> Result<serde_json::Value, String> {
    is_invading(dx, dy)?;
    let limits = gap_limits(profile)?;
    let (raw_x, weight) = if dx >= 0. {
        ((limits.x_out_max - dx) / limits.x_out_max, 2.)
    } else {
        ((limits.x_in_max + dx) / limits.x_in_max, 1.)
    };
    let x = raw_x.clamp(0., 1.);
    let y = ((limits.y_max - dy) / limits.y_max).clamp(0., 1.);
    let grade = (x.powf(weight) * y).powf(1. / (weight + 1.));
    Ok(
        serde_json::json!({"dx":dx,"dy":dy,"x_impact":x,"y_impact":y,"grade":grade,"source_ratio":1.+4.*grade,"target_ratio":1.+10.*grade,"eligible":grade>=0.1}),
    )
}
fn checked_bounds(bounds: [i32; 4]) -> Result<(), String> {
    if bounds[2] <= 0 || bounds[3] <= 0 || bounds.iter().any(|v| v.unsigned_abs() > 4_000_000) {
        return Err("invalid_head_stem_bounds".into());
    }
    Ok(())
}
/// Original HeadInter template offset. `learned_dx` contains *sheet-level
/// learned means for this shape*, never the individual head's seed tally.
/// Supply [None,None] for an unregistered head or absent learned scale.
pub fn stem_reference_point(
    bank: &audiveris_template_bank::Bank,
    size: u16,
    shape: &str,
    bounds: [i32; 4],
    side: HeadSide,
    vertical: VerticalSide,
    learned_dx: [Option<f64>; 2],
) -> Result<[f64; 2], String> {
    checked_bounds(bounds)?;
    if !crate::head_seed_search::ALL_STEM_SHAPES.contains(&shape) {
        return Err("unsupported_head_stem_shape".into());
    }
    if learned_dx
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
    {
        return Err("invalid_head_stem_learned_dx".into());
    }
    let name = match (side, vertical) {
        (HeadSide::Left, VerticalSide::Top) => "TOP_LEFT_STEM",
        (HeadSide::Left, VerticalSide::Bottom) => "BOTTOM_LEFT_STEM",
        (HeadSide::Right, VerticalSide::Top) => "TOP_RIGHT_STEM",
        (HeadSide::Right, VerticalSide::Bottom) => "BOTTOM_RIGHT_STEM",
    };
    let asset = bank.get(shape, size).map_err(|e| e.to_string())?;
    let slim = asset.slim_bounds();
    let (x, y) = asset
        .anchor(name)
        .map_err(|e| e.to_string())?
        .rounded()
        .map_err(|e| e.to_string())?;
    let mean = match learned_dx {
        [Some(l), Some(r)] => Some((l + r) / 2.),
        [Some(d), None] | [None, Some(d)] => Some(d),
        [None, None] => None,
    };
    let px = if let Some(dx) = mean {
        match side {
            HeadSide::Left => bounds[0] as f64 + 0.5 - dx,
            HeadSide::Right => (bounds[0] + bounds[2] - 1) as f64 + dx,
        }
    } else {
        (bounds[0] - slim[0] as i32 + x) as f64
    };
    Ok([px, (bounds[1] - slim[1] as i32 + y) as f64])
}
#[derive(Clone, Copy)]
pub struct ConnectionInput<'a> {
    pub shape: &'a str,
    pub bounds: [i32; 4],
    pub stem: [f64; 4],
    /// Optional HeadLinker corner stump bounds; not the assembled stem union.
    pub glyph_bounds: Option<[i32; 4]>,
    pub vertical: VerticalSide,
    pub interline: usize,
    pub profile: u8,
    pub learned_dx: [Option<f64>; 2],
}
fn relative_ccw(line: [f64; 4], point: [f64; 2]) -> i32 {
    let dx = line[2] - line[0];
    let dy = line[3] - line[1];
    let mut px = point[0] - line[0];
    let mut py = point[1] - line[1];
    let mut ccw = px * dy - py * dx;
    if ccw == 0. {
        ccw = px * dx + py * dy;
        if ccw > 0. {
            px -= dx;
            py -= dy;
            ccw = px * dx + py * dy;
            if ccw < 0. {
                ccw = 0.;
            }
        }
    }
    if ccw < 0. {
        -1
    } else if ccw > 0. {
        1
    } else {
        0
    }
}
/// Original checkRelation for finite, increasing-y stem medians. The physical
/// corner stump contact gate uses its exclusive bottom, independent of fitted
/// stem median. None means no physical stump and uses the template reference.
/// This calculates a relation; it does not select a stem or update a graph.
pub fn check_relation(
    bank: &audiveris_template_bank::Bank,
    size: u16,
    input: ConnectionInput<'_>,
) -> Result<Option<serde_json::Value>, String> {
    checked_bounds(input.bounds)?;
    gap_limits(input.profile)?;
    if !(1..=128).contains(&input.interline)
        || input
            .stem
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || input.stem[1] >= input.stem[3]
    {
        return Err("unsupported_head_stem_median".into());
    }
    if let Some(b) = input.glyph_bounds {
        checked_bounds(b)?;
    }
    let center = [
        (input.bounds[0] + input.bounds[2] / 2) as f64,
        (input.bounds[1] + input.bounds[3] / 2) as f64,
    ];
    let dir = -relative_ccw(input.stem, center);
    let side = if dir < 0 {
        HeadSide::Left
    } else {
        HeadSide::Right
    };
    let reference = stem_reference_point(
        bank,
        size,
        input.shape,
        input.bounds,
        side,
        input.vertical,
        input.learned_dx,
    )?;
    // Original LineUtil.xAtY determinant order is observable after repeated
    // centroid translations during CLinker expansion.
    let x = crate::stem_checks::x_at_y(
        [input.stem[0], input.stem[1]],
        [input.stem[2], input.stem[3]],
        reference[1],
    );
    let dx = dir as f64 * (x - reference[0]);
    let dy = if let Some(b) = input.glyph_bounds {
        let overlap = match input.vertical {
            VerticalSide::Bottom => (b[1] + b[3]) as f64 - input.stem[1],
            VerticalSide::Top => input.stem[3] - b[1] as f64,
        };
        overlap.min(0.).abs()
    } else if reference[1] < input.stem[1] {
        input.stem[1] - reference[1]
    } else if reference[1] > input.stem[3] {
        reference[1] - input.stem[3]
    } else {
        0.
    };
    let dx = dx / input.interline as f64;
    let dy = dy / input.interline as f64;
    let grade = gap_grade(dx, dy, input.profile)?;
    if grade["eligible"] != true {
        return Ok(None);
    }
    let y = match input.vertical {
        VerticalSide::Bottom => input.bounds[1] as f64,
        VerticalSide::Top => (input.bounds[1] + input.bounds[3] - 1) as f64,
    };
    Ok(Some(
        serde_json::json!({"kind":"HeadStemRelation","profile":input.profile,"manual":false,"head_side":side,"extension_point":[x,y],"dx":dx,"dy":dy,"grade":grade["grade"],"consistency":1.,"source_ratio":grade["source_ratio"],"target_ratio":grade["target_ratio"],"invading":is_invading(dx,dy)?}),
    ))
}

/// Original setConsistency(head,stem): small *shape*, fitted median y span,
/// and the sheet main interline. Zero/reversed medians cannot be registered as
/// supported physical stems; the standalone original zero-length kernel stays.
pub fn measured_consistency(shape: &str, stem: [f64; 4], interline: usize) -> Result<f64, String> {
    if !crate::head_seed_search::ALL_STEM_SHAPES.contains(&shape)
        || !(1..=128).contains(&interline)
        || stem.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        || stem[1] >= stem[3]
    {
        return Err("unsupported_measured_stem_consistency".into());
    }
    let length = (stem[3] - stem[1]) / interline as f64;
    let small = matches!(shape, "NOTEHEAD_VOID_SMALL" | "NOTEHEAD_BLACK_SMALL");
    let value = consistency(small, Some(length))?;
    if !value.is_finite() || value <= 0. {
        return Err("unsupported_measured_stem_consistency".into());
    }
    Ok(value)
}
/// Original checked connection followed by explicit physical stem-length
/// consistency population. Grade/eligibility and target support are unchanged.
/// This does not select stems or compute SIG contextual grades.
pub fn check_relation_with_consistency(
    bank: &audiveris_template_bank::Bank,
    size: u16,
    input: ConnectionInput<'_>,
) -> Result<Option<serde_json::Value>, String> {
    let Some(mut relation) = check_relation(bank, size, input)? else {
        return Ok(None);
    };
    let value = measured_consistency(input.shape, input.stem, input.interline)?;
    let grade = relation["grade"].as_f64().unwrap();
    relation["consistency"] = serde_json::json!(value);
    relation["source_ratio"] = serde_json::json!(1. + (4. * value) * grade);
    // Native verification metadata, not a JVM identity or automatic STEMS flag.
    relation["consistency_interline"] = serde_json::json!(input.interline);
    Ok(Some(relation))
}
