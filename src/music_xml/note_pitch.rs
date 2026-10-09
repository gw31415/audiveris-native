// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measures::{Chord, ChordKind};
use crate::shared_sig::Phase;
use std::collections::BTreeSet;

pub(crate) struct Reader<'a> {
    music: &'a OwnedMusic,
    snapshot: Value,
}
impl<'a> Reader<'a> {
    pub(crate) fn new(music: &'a OwnedMusic) -> Self {
        Self {
            music,
            snapshot: music.owner.graph.snapshot(),
        }
    }
    fn node(&self, id: usize) -> Result<&Value, String> {
        self.snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("music_pitch_live_Inter_required".into())
    }
    fn chord(&self, id: usize) -> Result<&Chord, String> {
        let parents = self.snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "Containment" && e["target"] == id)
            .collect::<Vec<_>>();
        if parents.len() != 1 {
            return Err("music_pitch_actual_HeadChord_required".into());
        }
        let chord = self
            .music
            .chords
            .iter()
            .find(|c| {
                c.kind == ChordKind::Head
                    && c.id == parents[0]["source"]
                    && c.members.iter().any(|n| n["id"] == id)
            })
            .ok_or("music_pitch_actual_HeadChord_required")?;
        if self.music.owner.graph.id(chord.key) != Some(chord.id) {
            return Err("music_pitch_actual_HeadChord_required".into());
        }
        Ok(chord)
    }
    fn center(n: &Value) -> Result<[i64; 2], String> {
        let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
            .map_err(|_| "music_pitch_bounds_required")?;
        Ok([b[0] + b[2] / 2, b[1] + b[3] / 2])
    }
    fn diatonic(&self, id: usize) -> Result<(&'static str, i32, Option<usize>), String> {
        let n = self.node(id)?;
        if n["kind"] != "HeadInter" {
            return Err("music_pitch_actual_Head_required".into());
        }
        let chord = self.chord(id)?;
        let measure = &self.music.measures[chord.measure];
        let geometry = measure
            .geometry
            .as_ref()
            .ok_or("music_pitch_actual_Measure_required")?;
        self.music
            .owner
            .graph
            .require_actual_music_measure(geometry)?;
        let staff = n["staff_id"]
            .as_u64()
            .ok_or("music_pitch_actual_Staff_required")?;
        if !geometry.part.staffs.iter().any(|s| {
            s.id == staff
                && s.system == measure.system
                && self
                    .music
                    .owner
                    .staffs
                    .iter()
                    .any(|held| std::sync::Arc::ptr_eq(s, held))
        }) {
            return Err("music_pitch_actual_Part_Staff_required".into());
        }
        let point = Self::center(n)?;
        let mut clefs = Vec::new();
        for c in self.snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| {
                c["kind"] == "ClefInter"
                    && c["origin"]["system"] == measure.system
                    && c["staff_id"] == staff
            })
        {
            let center = Self::center(c)?;
            if center[0] <= point[0] {
                clefs.push((center[0], c));
            }
        }
        let clef = clefs.into_iter().max_by_key(|(x, _)| *x).map(|(_, c)| c);
        let numerical_clef = clef
            .map(|c| -> Result<_, String> {
                Ok((
                    c["shape"]
                        .as_str()
                        .ok_or("music_pitch_Clef_shape_required")?,
                    c["pitch"]
                        .as_f64()
                        .ok_or("music_pitch_Clef_pitch_required")?,
                ))
            })
            .transpose()?;
        // The static G fallback has no graph identity and must not borrow a
        // previous system's physical clef or the writer's carried XML clef.
        let (step, octave) = crate::head_musical_pitch::diatonic_for_clef(
            numerical_clef,
            n["pitch"]
                .as_f64()
                .ok_or("music_pitch_Head_pitch_required")?,
        )
        .map_err(str::to_owned)?;
        Ok((
            step,
            octave,
            clef.map(|c| c["id"].as_u64().unwrap() as usize),
        ))
    }
    fn key_before(&self, id: usize) -> Result<Option<&Value>, String> {
        let chord = self.chord(id)?;
        let measure = &self.music.measures[chord.measure];
        let geometry = measure
            .geometry
            .as_ref()
            .ok_or("music_pitch_actual_Measure_required")?;
        let staff_id = self.node(id)?["staff_id"]
            .as_u64()
            .ok_or("music_pitch_actual_Staff_required")?;
        let staff = geometry
            .part
            .staffs
            .iter()
            .find(|s| s.id == staff_id)
            .ok_or("music_pitch_actual_Part_Staff_required")?;
        let mut keys = Vec::new();
        for key in self.snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| {
                n["kind"] == "KeyInter"
                    && n["origin"]["system"] == measure.system
                    && n["staff_id"] == staff_id
            })
        {
            let x = Self::center(key)?[0];
            for previous in self.music.measures.iter().filter(|m| {
                m.system == measure.system && m.part == measure.part && m.index <= measure.index
            }) {
                let g = previous
                    .geometry
                    .as_ref()
                    .ok_or("music_pitch_actual_Measure_required")?;
                let [left, right] = g.abscissae(staff)?;
                if left < x && x <= right {
                    keys.push((previous.index, key));
                    break;
                }
            }
        }
        // Measure.getKey takes the first registered key on the Staff. Reverse
        // Measure traversal must preserve that encounter order for same-Measure keys.
        keys.sort_by_key(|(index, _)| std::cmp::Reverse(*index));
        Ok(keys.first().map(|(_, key)| *key))
    }
    fn accidental(&self, id: usize) -> Result<Option<&Value>, String> {
        let edge = self.snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "AlterHeadRelation" && e["target"] == id);
        let Some(edge) = edge else {
            return Ok(None);
        };
        let n = self.node(
            edge["source"]
                .as_u64()
                .ok_or("music_pitch_actual_Alter_required")? as usize,
        )?;
        if n["kind"] != "AlterInter" {
            return Err("music_pitch_actual_Alter_required".into());
        }
        Ok(Some(n))
    }
    fn measure_accidental(&self, id: usize) -> Result<Option<&Value>, String> {
        if let Some(a) = self.accidental(id)? {
            return Ok(Some(a));
        }
        let chord = self.chord(id)?;
        let note = self.node(id)?;
        let pitch = self.diatonic(id)?;
        let mut heads = Vec::new();
        for c in self
            .music
            .chords
            .iter()
            .filter(|c| c.measure == chord.measure && c.kind == ChordKind::Head)
        {
            for h in &c.members {
                let h = self.node(h["id"].as_u64().unwrap() as usize)?;
                heads.push((Self::center(h)?[0], h));
            }
        }
        heads.sort_by_key(|(x, _)| std::cmp::Reverse(*x));
        let mut started = false;
        for (_, h) in heads {
            let other = h["id"].as_u64().unwrap() as usize;
            if other == id {
                started = true;
            } else if started && h["staff_id"] == note["staff_id"] {
                let other_pitch = self.diatonic(other)?;
                if (pitch.0, pitch.1) == (other_pitch.0, other_pitch.1)
                    && let Some(a) = self.accidental(other)?
                {
                    return Ok(Some(a));
                }
            }
        }
        Ok(None)
    }
    fn alteration(
        &self,
        id: usize,
        fifths: Option<i64>,
        visited: &mut BTreeSet<usize>,
    ) -> Result<i32, String> {
        if !visited.insert(id) {
            return Err("music_pitch_tie_cycle".into());
        }
        if let Some(a) = self.measure_accidental(id)? {
            return match a["shape"].as_str() {
                Some("FLAT") => Ok(-1),
                Some("DOUBLE_FLAT") => Ok(-2),
                Some("NATURAL") => Ok(0),
                Some("SHARP") => Ok(1),
                Some("DOUBLE_SHARP") => Ok(2),
                _ => Err("music_pitch_Alter_shape_required".into()),
            };
        }
        for edge in self.snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                e["kind"] == "SlurHeadRelation" && e["target"] == id && e["side"] == "RIGHT"
            })
        {
            let slur = edge["source"]
                .as_u64()
                .ok_or("music_pitch_actual_tie_required")? as usize;
            let s = self.node(slur)?;
            if s["kind"] != "SlurInter" {
                return Err("music_pitch_actual_tie_required".into());
            }
            if s["is_tie"] != true {
                continue;
            }
            let start = self.snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| {
                    e["kind"] == "SlurHeadRelation" && e["source"] == slur && e["side"] == "LEFT"
                });
            if let Some(start) = start {
                return self.alteration(
                    start["target"].as_u64().unwrap() as usize,
                    fifths,
                    visited,
                );
            }
            return Err("music_pitch_cross_system_tie_extension_read_pending".into());
        }
        crate::music_headers::key_alter(self.diatonic(id)?.0, fifths)
    }
    pub(crate) fn read(&self, id: usize) -> Result<Value, String> {
        self.read_with_key(id, None)
    }
    pub(crate) fn read_for_export(
        &self,
        id: usize,
        exported_fifths: Option<i64>,
    ) -> Result<Value, String> {
        self.read_with_key(id, Some(exported_fifths))
    }
    fn read_with_key(
        &self,
        id: usize,
        export_key_fifths: Option<Option<i64>>,
    ) -> Result<Value, String> {
        let chord = self.chord(id)?;
        let measure = &self.music.measures[chord.measure];
        let n = self.node(id)?;
        let (step, octave, clef) = self.diatonic(id)?;
        let key = self.key_before(id)?;
        let fifths = key
            .map(|k| {
                k["fifths"]
                    .as_i64()
                    .ok_or("music_pitch_key_fifths_required")
            })
            .transpose()?;
        let accidental =
            |a: Option<&Value>| a.map(|a| json!({"shape":a["shape"],"bounds":a["bounds"]}));
        // Some(None) is an empty exported key cache, not a request to fall
        // back to this physical Measure's key. Ordinary reads retain local keys.
        let alteration_fifths = export_key_fifths.unwrap_or(fifths);
        let mut result = json!({"bounds":n["bounds"],"staff":n["staff_id"],"part":measure.part,"measure_index":measure.index,"step":step,"octave":octave,"alter":self.alteration(id,alteration_fifths,&mut BTreeSet::new())?,"key_fifths":fifths,"local_accidental":accidental(self.accidental(id)?),"measure_accidental":accidental(self.measure_accidental(id)?),"effective_clef_id":clef,"effective_key_id":key.map(|k|k["id"].clone())});
        if let Some(exported) = export_key_fifths {
            result["export_key_fifths"] = json!(exported);
        }
        Ok(result)
    }
}

impl OwnedMusic {
    /// Read pitch from this live Head and its retained Staff/Chord/Measure.
    pub fn read_actual_note_pitch(&self, head: Key) -> Result<Value, String> {
        if head.phase != Phase::Head {
            return Err("music_pitch_actual_Head_required".into());
        }
        let id = self
            .owner
            .graph
            .id(head)
            .ok_or("music_pitch_live_Head_required")?;
        Reader::new(self).read(id)
    }
}

#[cfg(test)]
pub(crate) mod image_tests;
