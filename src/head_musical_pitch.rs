// SPDX-License-Identifier: AGPL-3.0-or-later
//! Header-clef diatonic candidates, before measure-local clefs/alter/rhythm.
//! Arithmetic follows ClefInter.noteStepOf/octaveOf and getIntegerPitch.
use serde_json::{Value, json};

pub(crate) fn diatonic(
    shape: &str,
    clef_pitch: f64,
    head_pitch: f64,
) -> Result<(&'static str, i32), &'static str> {
    // Keep arithmetic bounded and refuse non-finite values, not Rust saturating casts.
    if !clef_pitch.is_finite()
        || !head_pitch.is_finite()
        || clef_pitch.abs() > 12.
        || head_pitch.abs() > 32.
    {
        return Err("diatonic_pitch_outside_verified_range");
    }
    let c = clef_pitch.round_ties_even() as i32;
    let p = head_pitch.round_ties_even() as i32;
    // Java integer division truncates toward zero; modulo numerators here are positive.
    let (index, octave) = match shape {
        "G_CLEF" | "G_CLEF_SMALL" => ((71 - p) % 7, (34 - p) / 7),
        "G_CLEF_8VA" => ((71 - p) % 7, (34 - p) / 7 + 1),
        "G_CLEF_8VB" => ((71 - p) % 7, (34 - p) / 7 - 1),
        "C_CLEF" => ((72 + c - p) % 7, (28 + c - p) / 7),
        "F_CLEF" | "F_CLEF_SMALL" => ((75 + c - p) % 7, (24 + c - p) / 7),
        "F_CLEF_8VA" => ((73 - p) % 7, (22 - p) / 7 + 1),
        "F_CLEF_8VB" => ((73 - p) % 7, (22 - p) / 7 - 1),
        "PERCUSSION_CLEF" => return Err("unpitched_percussion_requires_separate_mapping"),
        _ => return Err("unsupported_diatonic_clef"),
    };
    Ok((["A", "B", "C", "D", "E", "F", "G"][index as usize], octave))
}

/// Derived only from the retained live Graph snapshot and native header result.
/// No duration, accidentals, default meter, final note or musical Graph is invented.
pub(crate) fn header_candidates(graph: &Value, upstream: &Value) -> Value {
    let mut heads = graph["inters"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|n| n["kind"] == "HeadInter")
        .collect::<Vec<_>>();
    heads.sort_by_key(|n| {
        (
            n["origin"]["system"].as_u64(),
            n["staff_id"].as_u64(),
            n["bounds"][0].as_i64(),
            n["id"].as_u64(),
        )
    });
    let rows = heads.into_iter().map(|h| {
        let matches = upstream["headers"]["staffs"].as_array().into_iter().flatten()
            .filter(|s| s["system_id"] == h["origin"]["system"] && s["staff_id"] == h["staff_id"])
            .collect::<Vec<_>>();
        let result = (|| {
            if matches.len() != 1 { return Err("unique_native_staff_header_required"); }
            let clef = &matches[0]["clef"];
            let shape = clef["shape"].as_str().ok_or("recognized_native_header_clef_required")?;
            let cp = clef["pitch"].as_f64().ok_or("native_clef_pitch_required")?;
            let hp = h["pitch"].as_f64().ok_or("native_head_pitch_required")?;
            diatonic(shape, cp, hp).map(|(step, octave)| (step, octave, clef))
        })();
        let mut row = json!({"head_id":h["id"],"origin":h["origin"],"staff_id":h["staff_id"],"bounds":h["bounds"],"relative_pitch":h["pitch"],"step":null,"octave":null,"alter":null,"duration":null,"measure":null,"voice":null,"final_note":false});
        match result {
            Ok((step, octave, clef)) => {
                row["step"] = json!(step); row["octave"] = json!(octave);
                row["header_clef_id"] = clef["id"].clone();
                row["header_clef_shape"] = clef["shape"].clone();
                row["status"] = json!("header_diatonic_candidate");
            }
            Err(e) => row["status"] = json!(e),
        }
        row
    }).collect::<Vec<_>>();
    json!({"scope":"recognized native header clef only; original Measure.getClefBefore, mid-system clefs, octave shifts and alteration remain unconnected","candidates":rows,"key_and_individual_accidentals_checked":false,"duration_and_meter_checked":false,"musical_notes_complete":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_clef_controls_include_ties_even_and_octave_transposing_shapes() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-clef-musical-pitch/controls.json"
        ))
        .unwrap();
        let cases = oracle["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 180);
        for c in cases {
            let (s, o) = diatonic(
                c["shape"].as_str().unwrap(),
                c["clef_pitch"].as_f64().unwrap(),
                c["head_pitch"].as_f64().unwrap(),
            )
            .unwrap();
            assert_eq!(s, c["step"].as_str().unwrap(), "{c}");
            assert_eq!(o as i64, c["octave"].as_i64().unwrap(), "{c}");
        }
    }
    #[test]
    fn unavailable_headers_and_invalid_pitch_do_not_invent_musical_notes() {
        assert!(diatonic("G_CLEF", 2., f64::NAN).is_err());
        assert!(diatonic("G_CLEF", f64::INFINITY, 0.).is_err());
        assert!(diatonic("G_CLEF", 2., 33.).is_err());
        assert!(diatonic("PERCUSSION_CLEF", 0., 0.).is_err());
        assert!(diatonic("UNKNOWN", 0., 0.).is_err());
        assert_eq!(
            header_candidates(&json!({"inters":[]}), &json!({}))["candidates"],
            json!([])
        );
        let head = json!({"id":1,"kind":"HeadInter","origin":{"system":1},"staff_id":1,"pitch":0.,"bounds":[100,100,10,10]});
        let g = json!({"inters":[head]});
        let row = header_candidates(&g, &json!({}))["candidates"][0].clone();
        assert_eq!(row["status"], "unique_native_staff_header_required");
        assert_eq!(row["step"], Value::Null);
        assert_eq!(row["alter"], Value::Null);
        assert_eq!(row["final_note"], false);
    }
}
