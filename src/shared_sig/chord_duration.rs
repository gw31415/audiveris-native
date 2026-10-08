// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use crate::music_time::Fraction;

fn shape_duration(shape: &str) -> Result<Option<Fraction>, String> {
    let pair = match shape {
        "LONG_REST" => (4, 1),
        "BREVE_REST"
        | "BREVE"
        | "BREVE_SMALL"
        | "BREVE_CROSS"
        | "BREVE_DIAMOND"
        | "BREVE_TRIANGLE_DOWN"
        | "BREVE_CIRCLE_X" => (2, 1),
        "WHOLE_REST"
        | "WHOLE_NOTE"
        | "WHOLE_NOTE_SMALL"
        | "WHOLE_NOTE_CROSS"
        | "WHOLE_NOTE_DIAMOND"
        | "WHOLE_NOTE_TRIANGLE_DOWN"
        | "WHOLE_NOTE_CIRCLE_X" => (1, 1),
        "HALF_REST"
        | "NOTEHEAD_VOID"
        | "NOTEHEAD_VOID_SMALL"
        | "NOTEHEAD_CROSS_VOID"
        | "NOTEHEAD_DIAMOND_VOID"
        | "NOTEHEAD_TRIANGLE_DOWN_VOID"
        | "NOTEHEAD_CIRCLE_X_VOID" => (1, 2),
        "QUARTER_REST"
        | "NOTEHEAD_BLACK"
        | "NOTEHEAD_BLACK_SMALL"
        | "NOTEHEAD_CROSS"
        | "NOTEHEAD_DIAMOND_FILLED"
        | "NOTEHEAD_TRIANGLE_DOWN_FILLED"
        | "NOTEHEAD_CIRCLE_X" => (1, 4),
        "EIGHTH_REST" => (1, 8),
        "ONE_16TH_REST" => (1, 16),
        "ONE_32ND_REST" => (1, 32),
        "ONE_64TH_REST" => (1, 64),
        "ONE_128TH_REST" => (1, 128),
        _ => return Ok(None),
    };
    Fraction::new(pair.0, pair.1).map(Some)
}
fn is_rest(shape: &str) -> bool {
    matches!(
        shape,
        "LONG_REST"
            | "BREVE_REST"
            | "WHOLE_REST"
            | "HALF_REST"
            | "QUARTER_REST"
            | "EIGHTH_REST"
            | "ONE_16TH_REST"
            | "ONE_32ND_REST"
            | "ONE_64TH_REST"
            | "ONE_128TH_REST"
    )
}
impl Graph {
    /// Raw source getter: dot/Tuplet and measure-rest treatment do not enter it.
    pub(crate) fn raw_chord_duration(&mut self, key: Key) -> Result<Option<Fraction>, String> {
        let id = self.endpoint(key)?;
        if key.phase != Phase::Chord
            || !matches!(
                self.vertices[id - 1].data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        {
            return Err("music_raw_actual_chord_required".into());
        }
        let mut notes = Vec::new();
        for eid in self.incident_edges(id) {
            let edge = &self.edges[eid - 1];
            if edge.key.kind == "Containment" && edge.key.source == id {
                let note = &self.vertices[edge.key.target - 1];
                if note.removed
                    || note.key.system != key.system
                    || !matches!(note.data["kind"].as_str(), Some("HeadInter" | "RestInter"))
                {
                    return Err("music_raw_actual_note_required".into());
                }
                let b: [i64; 4] = serde_json::from_value(note.data["bounds"].clone())
                    .map_err(|_| "music_raw_actual_note_bounds_required")?;
                notes.push((edge.key.target, b[1] + b[3] / 2));
            }
        }
        notes.sort_by_key(|(_, y)| std::cmp::Reverse(*y));
        let Some((note, _)) = notes.first().copied() else {
            return Ok(None);
        };
        let shape = self.vertices[note - 1].data["shape"]
            .as_str()
            .ok_or("music_raw_actual_note_shape_required")?
            .to_owned();
        let mut duration = shape_duration(&shape)?;
        if is_rest(&shape) {
            return Ok(duration);
        }
        let mut count = self.actual_chord_beams(key)?.len();
        if self.vertices[id - 1].data["kind"] == "HeadChordInter"
            && let Some(stem) = self.retained_chord_stem(key)?
        {
            let sid = self.id(stem).ok_or("music_raw_actual_stem_required")?;
            if !self.vertices[sid - 1].removed && !self.vertices[sid - 1].removing {
                for eid in self.incident_edges(sid) {
                    let edge = &self.edges[eid - 1];
                    if edge.key.kind == "FlagStemRelation" {
                        let flag = if edge.key.source == sid {
                            edge.key.target
                        } else {
                            edge.key.source
                        };
                        let v = &self.vertices[flag - 1];
                        if v.removed || v.key.system != key.system || v.data["kind"] != "FlagInter"
                        {
                            return Err("music_raw_actual_flag_required".into());
                        }
                        let value = v.data["value"]
                            .as_u64()
                            .and_then(|v| usize::try_from(v).ok())
                            .ok_or("music_raw_actual_flag_value_required")?;
                        count = count
                            .checked_add(value)
                            .ok_or("music_raw_beams_flags_overflow")?;
                    }
                }
            }
        }
        if count > 0 {
            if shape == "NOTEHEAD_VOID"
                && self
                    .incident_edges(note)
                    .iter()
                    .any(|eid| self.edges[*eid - 1].key.kind == "MirrorRelation")
            {
                duration = Some(Fraction::new(1, 4)?);
            }
            let mut raw = duration.ok_or("music_raw_null_duration_with_beams_flags")?;
            for _ in 0..count {
                raw = raw.times(Fraction::new(1, 2)?)?;
            }
            duration = Some(raw);
        }
        Ok(duration)
    }
}
