// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original BeamLinker.VLinker.getCloserLimit in the admitted zero-skew context.
//! This is an internal image-builder dependency, not a caller-selected limit API.
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) struct Beam {
    pub id: u64,
    pub group: u64,
    pub bounds: [i32; 4],
    pub median: [f64; 4],
    pub height: f64,
    pub grade: f64,
    pub hook: bool,
}

#[derive(Deserialize)]
pub(crate) struct Search {
    pub group: u64,
    pub beam_bounds: [i32; 4],
    pub system_bounds: [i32; 4],
    pub reference: [f64; 2],
    pub theoretical_line: [f64; 4],
    pub direction: i8,
    pub left: bool,
    pub interline: i32,
}
fn intersects(a: [i32; 4], b: [i32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
/// Neighbor bounds, grade (not contextual grade), group identity and theoretical
/// line all come from the live image owner. Java's stable nearest-border sort
/// uses the direction-facing border, then returns the opposite border.
pub(crate) fn select(s: &Search, beams: &[Beam]) -> Result<Option<(u64, [f64; 4])>, String> {
    let t = s.theoretical_line;
    if ![-1, 1].contains(&s.direction)
        || !(1..=1024).contains(&s.interline)
        || t.iter()
            .chain(s.reference.iter())
            .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
        // LineUtil's determinant arithmetic can round the zero-skew target x.
        || (t[0] - t[2]).abs() > 1e-8
        || t[0] != s.reference[0]
        || beams.len() > 2048
        || [s.beam_bounds, s.system_bounds]
            .iter()
            .any(|b| b.iter().any(|v| v.unsigned_abs() > 4_000_000) || b[2] <= 0 || b[3] <= 0)
    {
        return Err("unsupported_beam_closer_limit_context".into());
    }
    for b in beams {
        if b.id == 0
            || b.median
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 4_000_000.)
            || b.median[0] >= b.median[2]
            || !b.height.is_finite()
            || b.height <= 0.
            || !(0.0..=1.0).contains(&b.grade)
            || b.height > 4_000_000.
            || b.bounds.iter().any(|v| v.unsigned_abs() > 4_000_000)
            || b.bounds[2] <= 0
            || b.bounds[3] <= 0
        {
            return Err("unsupported_beam_closer_limit_beam".into());
        }
    }
    let il = f64::from(s.interline);
    let margin = s.interline; // StemsRetriever.vicinityMargin = 1.0 interline.
    let side_dx = (0.25 * il).round_ties_even();
    let group_dy = (5.0 * il).round_ties_even();
    let fat = [
        s.beam_bounds[0] - margin,
        s.system_bounds[1],
        s.beam_bounds[2] + 2 * margin,
        s.system_bounds[3],
    ];
    let mut neighbors = beams.iter().collect::<Vec<_>>();
    // getNeighboringInters returns the BY_ABSCISSA system collection order.
    neighbors.sort_by_key(|b| b.bounds[0]);
    let x = s.reference[0];
    neighbors.retain(|b| {
        if !intersects(fat, b.bounds)
            || b.group == s.group
            // AbstractBeamInter overrides AbstractInter.isGood: 0.35, not 0.4.
            || b.grade < 0.35
            || b.hook
            || !crate::stem_head_targets::intersects(b.median, t)
        {
            return false;
        }
        let cross = crate::stem_head_targets::intersection(t, b.median);
        let dx = (cross[0] - b.median[if s.left { 0 } else { 2 }]).abs();
        !((cross[1] - s.reference[1]).abs() <= group_dy && dx < side_dx)
    });
    neighbors.sort_by(|a, b| {
        let distance = |b: &Beam| {
            // StemsRetriever.getTargetPt uses LineUtil.intersection even at
            // zero skew. Keep its arithmetic; yAtX can change fractional ties.
            let shift = f64::from(s.direction) * b.height / 2.;
            let border = [
                b.median[0],
                b.median[1] + shift,
                b.median[2],
                b.median[3] + shift,
            ];
            let target = crate::stem_head_targets::intersection(
                [x, s.reference[1], x, s.reference[1] + 100.],
                border,
            );
            f64::from(s.direction) * (target[1] - s.reference[1])
        };
        distance(a).total_cmp(&distance(b))
    });
    Ok(neighbors.first().map(|b| {
        let shift = -f64::from(s.direction) * b.height / 2.;
        (
            b.id,
            [
                b.median[0],
                b.median[1] + shift,
                b.median[2],
                b.median[3] + shift,
            ],
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn original_alien_limits_preserve_grade_group_side_boundaries_and_border_order() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-image-musicxml-beam-groups/closer-controls.json"
        ))
        .unwrap();
        for c in oracle["controls"].as_array().unwrap() {
            let s: Search = serde_json::from_value(c["search"].clone()).unwrap();
            let beams: Vec<Beam> = serde_json::from_value(c["beams"].clone()).unwrap();
            let selected = select(&s, &beams)
                .unwrap()
                .map(|(id, border)| json!({"id":id,"border":border}));
            assert_eq!(json!(selected), c["selected"], "{}", c["name"]);
        }
    }
}
