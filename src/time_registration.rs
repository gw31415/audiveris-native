// SPDX-License-Identifier: AGPL-3.0-or-later
// Pinned AbstractInter and TimeWholeInter symbol geometry, Bravura only.
use serde_json::Value;
use sha2::{Digest, Sha256};
pub fn builtin_metrics() -> Result<Vec<Value>, String> {
    const BYTES: &[u8] = include_bytes!("../assets/time-font-metrics.json");
    if format!("{:x}", Sha256::digest(BYTES))
        != "1bac706f0bf8c786e8a899daad7cf637ec838bbe4c13f5f063ba60025ec8a8f6"
    {
        return Err("time_font_metrics_integrity".into());
    }
    let bank: Value = serde_json::from_slice(BYTES).map_err(|e| e.to_string())?;
    if bank["family"] != "Bravura"
        || bank["upstream_commit"] != "9e1e55cd2746037d059345881c53e6a6754bffbd"
        || bank["jar_sha256"] != "9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5"
        || bank["font_sha256"] != "dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2"
    {
        return Err("time_font_metrics_provenance".into());
    }
    let entries = bank["entries"].as_array().ok_or("missing_time_metrics")?;
    if entries.len() != 2688 {
        return Err("time_font_metrics_coverage".into());
    }
    Ok(entries.clone())
}
pub fn symbol_bounds(
    shape: &str,
    interline: usize,
    glyph_bounds: [i64; 4],
    metrics: &[Value],
) -> Result<[i64; 4], String> {
    if !(1..=128).contains(&interline)
        || glyph_bounds[2] <= 0
        || glyph_bounds[3] <= 0
        || glyph_bounds
            .iter()
            .any(|v| !(-4_000_000..=4_000_000).contains(v))
    {
        return Err("invalid_time_symbol_geometry".into());
    }
    let mut entries = metrics
        .iter()
        .filter(|m| m["shape"] == shape && m["interline"] == interline);
    let m = entries.next().ok_or("unsupported_time_font_metric")?;
    if entries.next().is_some() {
        return Err("duplicate_time_font_metric".into());
    }
    let cx = glyph_bounds[0] + glyph_bounds[2] / 2;
    let cy = glyph_bounds[1] + glyph_bounds[3] / 2;
    match m["bounds_method"].as_str() {
        Some("numden_dimension_integer_half") => {
            let w = m["dimension_width"]
                .as_i64()
                .ok_or("invalid_time_dimension")?;
            let h = m["dimension_height"]
                .as_i64()
                .ok_or("invalid_time_dimension")?;
            if !(1..=16384).contains(&w) || !(1..=16384).contains(&h) {
                return Err("invalid_time_dimension".into());
            }
            Ok([cx - w / 2, cy - h / 2, w, h])
        }
        Some("layout_rint_half") => {
            let w = m["layout_width"].as_f64().ok_or("invalid_time_layout")?;
            let h = m["layout_height"].as_f64().ok_or("invalid_time_layout")?;
            if !(0.0..=16384.0).contains(&w) || !(0.0..=16384.0).contains(&h) {
                return Err("invalid_time_layout".into());
            }
            Ok([
                cx - (w / 2.).round_ties_even() as i64,
                cy - (h / 2.).round_ties_even() as i64,
                w.round_ties_even() as i64,
                h.round_ties_even() as i64,
            ])
        }
        _ => Err("unsupported_time_bounds_method".into()),
    }
}
