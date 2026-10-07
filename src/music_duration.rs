// SPDX-License-Identifier: AGPL-3.0-or-later
//! Uncached ordinary AbstractChord.getDuration on live SIG members.
//! Exact ordinary fractions, with a separate sixteenth-grid export adapter;
//! special/mirrored heads and tuplets remain separate dependencies.
use crate::music_time::Fraction;
use serde_json::Value;
fn note_duration(shape: &str) -> Result<(u32, &'static str), String> {
    match shape {
        "NOTEHEAD_BLACK" | "QUARTER_REST" => Ok((1, "quarter")),
        "NOTEHEAD_VOID" | "HALF_REST" => Ok((2, "half")),
        _ => Err(format!(
            "musicxml_note_duration_outside_quarter_half_scope:{shape}"
        )),
    }
}
#[derive(Clone, Copy)]
pub(crate) struct ChordDuration {
    pub(crate) intrinsic: Option<u32>,
    pub(crate) note_type: &'static str,
    pub(crate) beams_or_flags: usize,
    pub(crate) dots: usize,
}
pub(crate) struct ExactChordDuration {
    pub(crate) intrinsic: Option<Fraction>,
    pub(crate) note_type: &'static str,
    beams_or_flags: usize,
    dots: usize,
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
        note_type: exact.note_type,
        beams_or_flags: exact.beams_or_flags,
        dots: exact.dots,
    })
}

pub(crate) fn live_exact(snapshot: &Value, id: usize) -> Result<ExactChordDuration, String> {
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
    if edges
        .iter()
        .any(|e| e["kind"] == "ChordTupletRelation" && (e["source"] == id || e["target"] == id))
    {
        return Err("musicxml_tuplet_duration_pending".into());
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
                if shape != "NOTEHEAD_BLACK"
                    || (!flags.is_empty() && !beams.is_empty())
                    || flags.len() > 1
                    || beams.len() > 2
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
                note_type = if count == 1 { "eighth" } else { "16th" };
            }
        }
        let live_chord = nodes
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("musicxml_duration_live_chord_required")?;
        let dots = live_chord["dots_number"].as_u64().unwrap_or(0) as usize;
        let mut duration = Fraction::new(base, 4 << count)?;
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
