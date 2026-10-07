// SPDX-License-Identifier: AGPL-3.0-or-later
//! Uncached ordinary AbstractChord.getDuration on live SIG members.
//! Exact sixteenth grid, supported ordinary shapes and two beam/flag levels;
//! special/mirrored heads and tuplets remain separate dependencies.
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

pub(crate) fn live(snapshot: &Value, id: usize) -> Result<ChordDuration, String> {
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
        ChordDuration {
            intrinsic: None,
            note_type: "whole",
            beams_or_flags: 0,
            dots: 0,
        }
    } else {
        let (base, mut note_type) = note_duration(shape)?;
        let mut duration = base * 4;
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
                duration /= 1 << count;
                note_type = if count == 1 { "eighth" } else { "16th" };
            }
        }
        let live_chord = nodes
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("musicxml_duration_live_chord_required")?;
        let dots = live_chord["dots_number"].as_u64().unwrap_or(0) as usize;
        match dots {
            0 => {}
            1 if duration.is_multiple_of(2) => duration = duration / 2 * 3,
            1 => return Err("musicxml_dotted_duration_finer_than_sixteenth_pending".into()),
            2 if duration.is_multiple_of(4) => duration = duration / 4 * 7,
            2 => return Err("musicxml_dotted_duration_finer_than_sixteenth_pending".into()),
            _ => return Err("musicxml_invalid_dot_count".into()),
        }
        ChordDuration {
            intrinsic: Some(duration),
            note_type,
            beams_or_flags: count,
            dots,
        }
    };
    Ok(result)
}
