// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original HeadSeedScale XML Double1Adapter context, kept separate from raw
//! in-memory statistics. Existing raw reference APIs and page reports are unchanged.
use crate::head_stem_relation::{self, HeadSide, VerticalSide};
use audiveris_template_bank::Bank;
/// NumberFormat Locale.US, grouping off, maximum fraction digits1, minimum0,
/// HALF_EVEN. Null and negative zero are retained. Unsupported nonfinite/large
/// native inputs are explicit refusals, not JVM string-parser emulation.
pub fn to_xml(value: Option<f64>) -> Result<Option<String>, String> {
    value
        .map(|v| {
            if !v.is_finite() || v.abs() > 4_000_000. {
                return Err("unsupported head seed XML DX".into());
            }
            let mut text = format!("{v:.1}");
            if text.ends_with(".0") {
                text.truncate(text.len() - 2);
            }
            Ok(text)
        })
        .transpose()
}
/// Restore the exact canonical output of to_xml. This exposes the explicit
/// persistence boundary; it does not silently round live HeadSeedScale values.
pub fn restored_dx(values: [Option<f64>; 2]) -> Result<[Option<f64>; 2], String> {
    let mut restored = [None, None];
    for (i, v) in values.iter().enumerate() {
        restored[i] = to_xml(*v)?
            .map(|s| s.parse::<f64>().map_err(|e| e.to_string()))
            .transpose()?;
    }
    Ok(restored)
}
pub struct Head<'a> {
    pub bank: &'a Bank,
    pub point_size: u16,
    pub shape: &'a str,
    pub bounds: [i32; 4],
    pub side: HeadSide,
    pub vertical: VerticalSide,
}
/// Reference point for a deliberately declared XML-round-tripped scale context.
/// Call existing stem_reference_point directly for transient raw context.
pub fn reference(head: &Head<'_>, raw_learned_dx: [Option<f64>; 2]) -> Result<[f64; 2], String> {
    let h = head;
    head_stem_relation::stem_reference_point(
        h.bank,
        h.point_size,
        h.shape,
        h.bounds,
        h.side,
        h.vertical,
        restored_dx(raw_learned_dx)?,
    )
}
