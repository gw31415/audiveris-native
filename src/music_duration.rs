// SPDX-License-Identifier: AGPL-3.0-or-later
//! Uncached ordinary AbstractChord.getDuration on live SIG members.
//! Exact ordinary fractions, with a separate sixteenth-grid export adapter;
//! special/mirrored heads and tuplets remain separate dependencies.
use crate::music_time::Fraction;
use serde_json::Value;
fn note_duration(shape: &str) -> Result<(Fraction, &'static str), String> {
    let (denominator, note_type) = match shape {
        "NOTEHEAD_BLACK" | "QUARTER_REST" => (4, "quarter"),
        "NOTEHEAD_VOID" | "HALF_REST" => (2, "half"),
        "EIGHTH_REST" => (8, "eighth"),
        "ONE_16TH_REST" => (16, "16th"),
        "ONE_32ND_REST" => (32, "32nd"),
        "ONE_64TH_REST" => (64, "64th"),
        "ONE_128TH_REST" => (128, "128th"),
        _ => return Err(format!("musicxml_note_duration_shape_pending:{shape}")),
    };
    Ok((Fraction::new(1, denominator)?, note_type))
}
#[derive(Clone, Copy)]
pub(crate) struct ChordDuration {
    pub(crate) intrinsic: Option<u32>,
}
#[derive(Clone, Copy)]
pub(crate) struct ExactChordDuration {
    pub(crate) intrinsic: Option<Fraction>,
    pub(crate) note_type: &'static str,
    pub(crate) beams_or_flags: usize,
    pub(crate) dots: usize,
}

pub(crate) fn live(snapshot: &Value, id: usize) -> Result<ChordDuration, String> {
    let exact = live_exact(snapshot, id)?;
    Ok(ChordDuration {
        intrinsic: exact
            .intrinsic
            .map(|d| {
                d.ticks()
                    .map_err(|_| "musicxml_dotted_duration_finer_than_sixteenth_pending".to_owned())
            })
            .transpose()?,
    })
}

pub(crate) fn live_exact(snapshot: &Value, id: usize) -> Result<ExactChordDuration, String> {
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("missing_music_inters")?;
    let chord = nodes
        .iter()
        .find(|n| n["id"] == id)
        .ok_or("musicxml_duration_live_chord_required")?;
    if !matches!(
        chord["kind"].as_str(),
        Some("HeadChordInter" | "RestChordInter")
    ) {
        return Err("musicxml_duration_live_chord_required".into());
    }
    if snapshot["relations"]
        .as_array()
        .ok_or("missing_music_relations")?
        .iter()
        .any(|e| e["kind"] == "ChordTupletRelation" && (e["source"] == id || e["target"] == id))
    {
        return Err("musicxml_tuplet_duration_pending".into());
    }
    live_intrinsic(snapshot, id)
}
pub(crate) fn live_intrinsic(snapshot: &Value, id: usize) -> Result<ExactChordDuration, String> {
    let nodes = snapshot["inters"]
        .as_array()
        .ok_or("missing_music_inters")?;
    let edges = snapshot["relations"]
        .as_array()
        .ok_or("missing_music_relations")?;
    let chord = nodes
        .iter()
        .find(|n| n["id"] == id)
        .ok_or("musicxml_duration_live_chord_required")?;
    if !matches!(
        chord["kind"].as_str(),
        Some("HeadChordInter" | "RestChordInter")
    ) {
        return Err("musicxml_duration_live_chord_required".into());
    }
    let members = edges
        .iter()
        .filter(|e| e["kind"] == "Containment" && e["source"] == id)
        .map(|e| {
            nodes
                .iter()
                .find(|n| n["id"] == e["target"])
                .ok_or("musicxml_duration_live_note_required")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let first = members.first().ok_or("musicxml_chord_member_required")?;
    let note = nodes
        .iter()
        .find(|n| n["id"] == first["id"])
        .ok_or("musicxml_duration_live_note_required")?;
    let shape = note["shape"]
        .as_str()
        .ok_or("musicxml_note_shape_required")?;
    let rest = note["kind"] == "RestInter";
    let mut member_ids = std::collections::BTreeSet::new();
    for member in &members {
        let live = nodes
            .iter()
            .find(|n| n["id"] == member["id"])
            .ok_or("musicxml_duration_live_note_required")?;
        let id = live["id"]
            .as_u64()
            .ok_or("musicxml_chord_member_required")?;
        if !member_ids.insert(id)
            || live["kind"] != note["kind"]
            || live["shape"] != note["shape"]
            || (rest && members.len() != 1)
        {
            // Mirrored mixed shapes need the original per-note type/filled
            // notehead branch; their duration cannot admit ordinary export.
            return Err("musicxml_mixed_or_special_chord_members_pending".into());
        }
    }
    let whole = rest && crate::music_voices::is_measure_rest_shape(shape, false);
    let result = if whole {
        ExactChordDuration {
            intrinsic: None,
            note_type: "whole",
            beams_or_flags: 0,
            dots: 0,
        }
    } else {
        let (base, mut note_type) = note_duration(shape)?;
        let mut count = 0;
        if !rest {
            let beams = crate::music_voices::chord_beams(snapshot, id)?;
            let stems = edges
                .iter()
                .filter(|e| e["kind"] == "ChordStemRelation" && e["source"] == id)
                .collect::<Vec<_>>();
            if stems.len() > 1 {
                return Err("musicxml_actual_chord_stem_required".into());
            }
            let stem = stems
                .first()
                .and_then(|e| e["target"].as_u64())
                .map(|v| v as usize);
            let flags = nodes
                .iter()
                .filter(|n| {
                    n["kind"] == "FlagInter"
                        && stem.is_some_and(|id| {
                            edges.iter().any(|e| {
                                e["kind"] == "FlagStemRelation"
                                    && e["source"] == n["id"]
                                    && e["target"] == id
                            })
                        })
                })
                .collect::<Vec<_>>();
            if !flags.is_empty() || !beams.is_empty() {
                if !matches!(shape, "NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
                    || (!flags.is_empty() && !beams.is_empty())
                    || flags.len() > 1
                    || beams.len() > 3
                    || flags
                        .first()
                        .is_some_and(|f| !matches!(f["value"].as_u64(), Some(1 | 2)))
                {
                    return Err("musicxml_flagged_void_rest_or_multiple_flags_pending".into());
                }
                count = flags
                    .iter()
                    .map(|f| f["value"].as_u64().unwrap() as usize)
                    .sum::<usize>()
                    + beams.len();
                if shape == "NOTEHEAD_VOID"
                    && edges.iter().any(|e| {
                        e["kind"] == "MirrorRelation"
                            && (e["source"] == note["id"] || e["target"] == note["id"])
                    })
                {
                    return Err("musicxml_mirrored_void_duration_pending".into());
                }
                // A void head without a mirror keeps its half-note base before
                // flag division; treating every flagged head as black loses it.
                note_type = match (shape, count) {
                    ("NOTEHEAD_VOID", 1) => "quarter",
                    ("NOTEHEAD_VOID", 2) | ("NOTEHEAD_BLACK", 1) => "eighth",
                    ("NOTEHEAD_VOID", 3) | ("NOTEHEAD_BLACK", 2) => "16th",
                    ("NOTEHEAD_BLACK", 3) => "32nd",
                    _ => return Err("musicxml_beam_flag_duration_type_pending".into()),
                };
            }
        }
        let live_chord = nodes
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("musicxml_duration_live_chord_required")?;
        let dots = live_chord["dots_number"].as_u64().unwrap_or(0) as usize;
        let mut duration = base.times(Fraction::new(1, 1 << count)?)?;
        match dots {
            0 => {}
            1 => duration = duration.times(Fraction::new(3, 2)?)?,
            2 => duration = duration.times(Fraction::new(7, 4)?)?,
            _ => return Err("musicxml_invalid_dot_count".into()),
        }
        ExactChordDuration {
            intrinsic: Some(duration),
            note_type,
            beams_or_flags: count,
            dots,
        }
    };
    Ok(result)
}

#[cfg(test)]
mod image_tests;
#[cfg(test)]
pub(crate) use image_tests::assert_real_duration_reads;

#[cfg(test)]
mod rest_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn short_rest_base_durations_match_direct_original_shape_calls() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-rest-durations/observed.json"
        ))
        .unwrap();
        let mut compared = 0;
        for case in original["cases"].as_array().unwrap() {
            let shape = case["shape"].as_str().unwrap();
            if matches!(shape, "LONG_REST" | "BREVE_REST" | "WHOLE_REST") {
                continue; // Measure-rest duration remains the separate existing owner path.
            }
            let (duration, _) = note_duration(shape).unwrap();
            assert_eq!(duration.to_string(), case["duration"]);
            compared += 1;
        }
        assert_eq!(compared, 9);
    }

    #[test]
    fn dotted_short_rest_keeps_exact_duration_and_the_legacy_tick_boundary() {
        for (shape, dots, duration, note_type, ticks) in [
            ("EIGHTH_REST", 0, "1/8", "eighth", Some(2)),
            ("EIGHTH_REST", 1, "3/16", "eighth", Some(3)),
            ("EIGHTH_REST", 2, "7/32", "eighth", None),
            ("ONE_16TH_REST", 0, "1/16", "16th", Some(1)),
            ("ONE_16TH_REST", 1, "3/32", "16th", None),
            ("ONE_16TH_REST", 2, "7/64", "16th", None),
            ("ONE_32ND_REST", 0, "1/32", "32nd", None),
            ("ONE_64TH_REST", 0, "1/64", "64th", None),
            ("ONE_128TH_REST", 0, "1/128", "128th", None),
        ] {
            let snapshot = json!({"inters":[
                {"id":1,"kind":"RestChordInter","dots_number":dots,"manual":true},
                {"id":2,"kind":"RestInter","shape":shape}],
                "relations":[{"kind":"Containment","source":1,"target":2}]});
            let before = snapshot.clone();
            let actual = live_exact(&snapshot, 1).unwrap();
            assert_eq!(actual.intrinsic, Some(Fraction::parse(duration).unwrap()));
            assert_eq!(actual.note_type, note_type);
            assert_eq!(actual.beams_or_flags, 0);
            assert_eq!(actual.dots, dots);
            match ticks {
                Some(ticks) => assert_eq!(live(&snapshot, 1).unwrap().intrinsic, Some(ticks)),
                None => assert_eq!(
                    live(&snapshot, 1).err().unwrap(),
                    "musicxml_dotted_duration_finer_than_sixteenth_pending"
                ),
            }
            assert_eq!(snapshot, before);
        }
    }
}
