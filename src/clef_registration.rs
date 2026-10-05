// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Native ClefBuilder.registerClefs. Final key-dependent selection is separate.
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Debug, Deserialize)]
pub struct Metric {
    pub shape: String,
    pub interline: usize,
    pub layout_width: f64,
    pub layout_height: f64,
    pub centroid_offset_x: f64,
    pub centroid_offset_y: f64,
    pub area_pitch_offset: f64,
}

/// Pinned Bravura geometry, generated once from the official Linux distribution.
/// This loads numerical metrics only; runtime does not rasterize fonts or run Java.
pub fn builtin_metrics() -> Result<Vec<Metric>, String> {
    use sha2::{Digest, Sha256};
    const BYTES: &[u8] = include_bytes!("../assets/clef-font-metrics.json");
    if format!("{:x}", Sha256::digest(BYTES))
        != "2a16b2938a16248dcb1ef298b856381846c80ee9127f20da979c63289ed3e371"
    {
        return Err("clef_font_metrics_integrity".into());
    }
    let bank: Value = serde_json::from_slice(BYTES).map_err(|e| e.to_string())?;
    if bank["family"] != "Bravura"
        || bank["upstream_commit"] != "9e1e55cd2746037d059345881c53e6a6754bffbd"
        || bank["jar_sha256"] != "9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5"
        || bank["font_sha256"] != "dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2"
    {
        return Err("clef_font_metrics_provenance".into());
    }
    let metrics: Vec<Metric> =
        serde_json::from_value(bank["entries"].clone()).map_err(|e| e.to_string())?;
    if metrics.len() != 768 {
        return Err("clef_font_metrics_coverage".into());
    }
    Ok(metrics)
}

pub fn analyze(
    candidates: &Value,
    grid: &Value,
    headers: &Value,
    interline: usize,
) -> Result<Value, String> {
    register(candidates, grid, headers, interline, &builtin_metrics()?)
}

fn bounds(value: &Value) -> Result<[i64; 4], String> {
    let a = value
        .as_array()
        .filter(|a| a.len() == 4)
        .ok_or("invalid_clef_bounds")?;
    let mut out = [0; 4];
    for (i, n) in a.iter().enumerate() {
        out[i] = n
            .as_i64()
            .filter(|v| (-4_000_000..=4_000_000).contains(v))
            .ok_or("invalid_clef_bounds")?;
    }
    if out[2] <= 0 || out[3] <= 0 {
        return Err("invalid_clef_dimensions".into());
    }
    Ok(out)
}

fn glyph_centroid_x(glyph: &Value, b: [i64; 4]) -> Result<i64, String> {
    if glyph["orientation"] != "VERTICAL" {
        return Err("unsupported_clef_glyph_orientation".into());
    }
    let runs = glyph["runs"]
        .as_array()
        .filter(|a| a.len() <= 262144)
        .ok_or("invalid_clef_runs")?;
    let mut weight = 0i64;
    let mut x_sum = 0i64;
    let mut previous = None;
    for r in runs {
        let a = r
            .as_array()
            .filter(|a| a.len() == 3)
            .ok_or("invalid_clef_run")?;
        let x = a[0]
            .as_i64()
            .filter(|x| *x >= 0 && *x < b[2])
            .ok_or("invalid_clef_run_x")?;
        let y = a[1]
            .as_i64()
            .filter(|y| *y >= 0 && *y < b[3])
            .ok_or("invalid_clef_run_y")?;
        let n = a[2]
            .as_i64()
            .filter(|n| *n > 0 && *n <= b[3] - y)
            .ok_or("invalid_clef_run_length")?;
        if let Some((px, end)) = previous
            && (x < px || (x == px && y < end))
        {
            return Err("overlapping_or_unsorted_clef_runs".into());
        }
        previous = Some((x, y + n));
        weight += n;
        if weight > 4_000_000 {
            return Err("unsupported_clef_glyph_budget".into());
        }
        x_sum += (b[0] + x) * n;
    }
    if weight == 0 {
        return Err("empty_clef_glyph".into());
    }
    Ok((x_sum as f64 / weight as f64).round_ties_even() as i64)
}

/// No input is mutated. Font geometry must be from the exact pinned official bank.
/// Candidate ties preserve the existing ClefKind EnumMap order.
pub fn register(
    candidates: &Value,
    grid: &Value,
    headers: &Value,
    interline: usize,
    metrics: &[Metric],
) -> Result<Value, String> {
    if interline == 0 || interline > 128 {
        return Err("unsupported_clef_interline".into());
    }
    let staffs = grid["staffs"].as_array().ok_or("missing_clef_staffs")?;
    let mut output = Vec::new();
    for candidate in candidates["staffs"]
        .as_array()
        .ok_or("missing_clef_candidates")?
    {
        let sid = candidate["staff_id"]
            .as_u64()
            .ok_or("invalid_clef_staff_id")?;
        let staff = staffs
            .iter()
            .find(|s| s["id"] == sid)
            .ok_or("missing_clef_staff")?;
        let specific = crate::clef_candidates::staff_interline(staff, interline)?;
        let header = headers["staff_headers"]
            .as_array()
            .ok_or("missing_clef_headers")?
            .iter()
            .find(|h| h["staff_id"] == sid)
            .ok_or("missing_clef_header")?;
        let first = candidate["first_best"]
            .as_array()
            .ok_or("missing_clef_first_candidates")?;
        let selected = if first.is_empty() {
            candidate["second_best"]
                .as_array()
                .ok_or("missing_clef_second_candidates")?
        } else {
            first
        };
        if selected.len() > 8 {
            return Err("unsupported_clef_candidate_count".into());
        }
        let mut ordered = selected.iter().collect::<Vec<_>>();
        for c in &ordered {
            if !c["grade"].as_f64().is_some_and(|g| (0.0..=1.).contains(&g)) {
                return Err("invalid_clef_candidate_grade".into());
            }
        }
        ordered.sort_by(|a, b| {
            b["grade"]
                .as_f64()
                .unwrap()
                .total_cmp(&a["grade"].as_f64().unwrap())
        });
        let mut inters = Vec::new();
        let mut stop = None;
        for c in ordered {
            let shape = c["shape"].as_str().ok_or("invalid_clef_shape")?;
            let matches = metrics
                .iter()
                .filter(|m| m.shape == shape && m.interline == specific)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(format!("unsupported_clef_font_metric:{shape}:{specific}"));
            }
            let m = matches[0];
            if !m.layout_width.is_finite()
                || !m.layout_height.is_finite()
                || m.layout_width <= 0.
                || m.layout_height <= 0.
                || m.layout_width > 16384.
                || m.layout_height > 16384.
                || !m.centroid_offset_x.is_finite()
                || !m.centroid_offset_y.is_finite()
                || !m.area_pitch_offset.is_finite()
                || m.centroid_offset_x.abs() > 0.5
                || m.centroid_offset_y.abs() > 0.5
            {
                return Err("invalid_clef_font_metric".into());
            }
            // Original uses staff-specific symbol bounds but sheet-main font
            // centroid offsets. Validate that second lookup as strictly as the first.
            let centroid_matches = metrics
                .iter()
                .filter(|m| m.shape == shape && m.interline == interline)
                .collect::<Vec<_>>();
            if centroid_matches.len() != 1 {
                return Err(format!(
                    "unsupported_clef_centroid_metric:{shape}:{interline}"
                ));
            }
            let centroid = centroid_matches[0];
            if !centroid.centroid_offset_x.is_finite() || centroid.centroid_offset_x.abs() > 0.5 {
                return Err("invalid_clef_centroid_metric".into());
            }
            let g = bounds(&c["glyph"]["bounds"])?;
            let mut b = [
                g[0] + g[2] / 2 - (m.layout_width / 2.).round_ties_even() as i64,
                g[1] + g[3] / 2 - (m.layout_height / 2.).round_ties_even() as i64,
                m.layout_width.round_ties_even() as i64,
                m.layout_height.round_ties_even() as i64,
            ];
            if b[2] <= 0 || b[3] <= 0 {
                return Err("unsupported_empty_clef_font_bounds".into());
            }
            let symbol_x =
                (b[0] as f64 + b[2] as f64 / 2. + b[2] as f64 * centroid.centroid_offset_x)
                    .round_ties_even() as i64;
            b[0] += glyph_centroid_x(&c["glyph"], g)? - symbol_x;
            // ClefInter overrides AbstractPitchedInter.setBounds: preserve kind.pitch.
            let pitch = match c["kind"].as_str() {
                Some("TREBLE" | "MEZZO_SOPRANO") => 2.,
                Some("BASS" | "TENOR") => -2.,
                Some("BARITONE" | "ALTO" | "PERCUSSION") => 0.,
                Some("SOPRANO") => 4.,
                _ => return Err("unsupported_clef_kind".into()),
            };
            if inters.is_empty() {
                stop = Some((g[0] + g[2]).min(b[0] + b[2]) - 1);
            }
            inters.push(json!({"shape":shape,"kind":c["kind"],"grade":c["grade"],"pitch":pitch,"bounds":b,"glyph":c["glyph"]}));
        }
        let mut exclusions = Vec::new();
        for source in 0..inters.len() {
            for target in source + 1..inters.len() {
                exclusions.push(json!({"source":source,"target":target,"cause":"OVERLAP"}));
            }
        }
        output.push(json!({"staff_id":sid,"header_start":header["start"],"header_stop":header["stop"],"clef_stop":stop,"inters":inters,"exclusions":exclusions}));
    }
    // HeaderBuilder.setSystemClefStop uses the largest RELATIVE clef offset.
    let mut offsets = std::collections::BTreeMap::<u64, i64>::new();
    for row in &output {
        let sid = row["staff_id"].as_u64().unwrap();
        let staff = staffs.iter().find(|s| s["id"] == sid).unwrap();
        let system = staff["system_id"]
            .as_u64()
            .ok_or("missing_clef_system_id")?;
        let start = row["header_start"]
            .as_i64()
            .ok_or("invalid_clef_header_start")?;
        let max = offsets.entry(system).or_default();
        if let Some(stop) = row["clef_stop"].as_i64() {
            *max = (*max).max(stop - start);
        }
    }
    for row in &mut output {
        let staff = staffs.iter().find(|s| s["id"] == row["staff_id"]).unwrap();
        let system = staff["system_id"].as_u64().unwrap();
        if offsets[&system] > 0 {
            row["header_stop"] = json!(row["header_start"].as_i64().unwrap() + offsets[&system]);
        }
    }
    Ok(
        json!({"stage":"HEADERS.registerClefs","staffs":output,"clef_selection_complete":false,"headers_complete":false,"note_recognition_complete":false,"pending":["key-dependent clef contextualization and final selection","key and time signatures"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inputs() -> (Value, Value, Value, Vec<Metric>) {
        let lines = (0..5)
            .map(|i| json!({"points":[[0., 100.+i as f64*10.],[300.,100.+i as f64*10.]]}))
            .collect::<Vec<_>>();
        let grid = json!({"staffs":[{"id":1,"system_id":1,"lines":lines}]});
        let headers = json!({"staff_headers":[{"staff_id":1,"start":80,"stop":80}]});
        let glyph =
            json!({"bounds":[100,110,5,11],"orientation":"VERTICAL","runs":[[0,0,11],[4,0,1]]});
        let c = json!({"staff_id":1,"first_best":[{"shape":"G_CLEF","kind":"TREBLE","grade":0.7,"glyph":glyph}],"second_best":[]});
        let metrics = vec![Metric {
            shape: "G_CLEF".into(),
            interline: 10,
            layout_width: 7.,
            layout_height: 12.,
            centroid_offset_x: 0.1,
            centroid_offset_y: 0.4,
            area_pitch_offset: 1.,
        }];
        (json!({"staffs":[c]}), grid, headers, metrics)
    }
    #[test]
    fn bbox_centroid_translation_ignores_y_and_preserves_clef_pitch() {
        let (c, g, h, m) = inputs();
        let r = register(&c, &g, &h, 10, &m).unwrap();
        let s = &r["staffs"][0];
        // Integer glyph area center=(102,115); initial box=(98,109,7,12).
        // Symbol centroid x=rint(102.2)=102, glyph centroid x=rint(100.333)=100.
        assert_eq!(s["inters"][0]["bounds"], json!([96, 109, 7, 12]));
        assert_eq!(s["clef_stop"], 102);
        assert_eq!(s["inters"][0]["pitch"], 2.0);
        assert_eq!(r["headers_complete"], false);
    }
    #[test]
    fn reverse_grade_sort_is_stable_and_exclusions_are_all_pairs() {
        let (mut c, g, h, m) = inputs();
        let a = c["staffs"][0]["first_best"][0].clone();
        let mut b = a.clone();
        b["kind"] = json!("BASS");
        let mut d = a.clone();
        d["kind"] = json!("ALTO");
        d["grade"] = json!(0.8);
        c["staffs"][0]["first_best"] = json!([a, b, d]);
        let r = register(&c, &g, &h, 10, &m).unwrap();
        let s = &r["staffs"][0];
        assert_eq!(s["inters"][0]["kind"], "ALTO");
        assert_eq!(s["inters"][1]["kind"], "TREBLE");
        assert_eq!(s["inters"][2]["kind"], "BASS");
        assert_eq!(
            s["exclusions"],
            json!([{"source":0,"target":1,"cause":"OVERLAP"},{"source":0,"target":2,"cause":"OVERLAP"},{"source":1,"target":2,"cause":"OVERLAP"}])
        );
    }
    #[test]
    fn second_pass_empty_cases_and_unsupported_metrics() {
        let (mut c, g, h, m) = inputs();
        let expected = register(&c, &g, &h, 10, &m).unwrap();
        c["staffs"][0]["second_best"] = c["staffs"][0]["first_best"].clone();
        c["staffs"][0]["first_best"] = json!([]);
        assert_eq!(register(&c, &g, &h, 10, &m).unwrap(), expected);
        assert!(
            register(&c, &g, &h, 11, &m)
                .unwrap_err()
                .starts_with("unsupported_clef_font_metric")
        );
        c["staffs"][0]["second_best"] = json!([]);
        let r = register(&c, &g, &h, 10, &m).unwrap();
        assert_eq!(r["staffs"][0]["clef_stop"], Value::Null);
        assert_eq!(r["staffs"][0]["inters"], json!([]));
    }
    #[test]
    fn centroid_uses_pixel_coordinates_and_ties_even() {
        let g = json!({"orientation":"VERTICAL","runs":[[0,0,1],[1,0,1]]});
        assert_eq!(glyph_centroid_x(&g, [100, 20, 2, 1]).unwrap(), 100);
        assert_eq!(glyph_centroid_x(&g, [101, 20, 2, 1]).unwrap(), 102);
        let bad = json!({"orientation":"VERTICAL","runs":[[0,0,2],[0,1,1]]});
        assert!(glyph_centroid_x(&bad, [0, 0, 2, 3]).is_err());
        assert!(bounds(&json!([i64::MIN, 0, 1, 1])).is_err());
    }
}

#[cfg(test)]
mod column_tests {
    use super::*;
    #[test]
    fn column_offsets_are_relative_per_system_and_do_not_select_clef() {
        let g = json!({"staffs":[{"id":1,"system_id":1},{"id":2,"system_id":1},{"id":3,"system_id":2}]});
        let h = json!({"staff_headers":[{"staff_id":1,"start":80,"stop":80},{"staff_id":2,"start":90,"stop":90},{"staff_id":3,"start":1000,"stop":1000}]});
        let candidate = json!({"shape":"G_CLEF","kind":"TREBLE","grade":0.7,"glyph":{"bounds":[100,110,5,11],"orientation":"VERTICAL","runs":[[0,0,11],[4,0,1]]}});
        let c = json!({"staffs":[{"staff_id":1,"first_best":[candidate],"second_best":[]},{"staff_id":2,"first_best":[],"second_best":[]},{"staff_id":3,"first_best":[],"second_best":[]}]});
        let m = vec![Metric {
            shape: "G_CLEF".into(),
            interline: 10,
            layout_width: 7.,
            layout_height: 12.,
            centroid_offset_x: 0.1,
            centroid_offset_y: 0.4,
            area_pitch_offset: 1.,
        }];
        let actual = register(&c, &g, &h, 10, &m).unwrap();
        assert_eq!(actual["staffs"][0]["header_stop"], 102);
        assert_eq!(actual["staffs"][1]["header_stop"], 112);
        assert_eq!(actual["staffs"][2]["header_stop"], 1000);
        assert_eq!(actual["staffs"][1]["clef_stop"], Value::Null);
        assert_eq!(actual["clef_selection_complete"], false);
    }
}
