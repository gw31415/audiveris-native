// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    music_measures::OwnedMusic,
    music_time::Fraction,
    shared_sig::{Graph, Key},
};
use std::sync::Arc;

/// Source beat groups before extension and fresh Tuplet generation.
#[derive(Debug)]
pub struct TupletBeatPlan {
    pub voice: Arc<()>,
    pub groups: Vec<Vec<Key>>,
    pub pivot: Option<Key>,
    pub fallback: bool,
}
impl OwnedMusic {
    /// Explicit beat input isolates the original private grouping boundary;
    /// actual time-signature and automatic force admission follow separately.
    pub fn plan_rhythm_voice_beats(
        &mut self,
        number: usize,
        identity: &Arc<()>,
        beat: &str,
    ) -> Result<TupletBeatPlan, String> {
        let mi = super::tuplet_groups::measure_index(&self.measures, number)?;
        let voice = self
            .voices
            .iter()
            .find(|v| Arc::ptr_eq(&v.identity, identity))
            .or_else(|| self.owner.graph.retained_voice_owner(identity))
            .ok_or("music_actual_tuplet_voice_owner_required")?;
        if voice.measure != mi {
            return Err("music_actual_tuplet_voice_owner_required".into());
        }
        let ids = voice.chord_ids.clone();
        let beat = Fraction::parse(beat)?;
        let mut graph = self.owner.graph.clone();
        let initial_groups = build_on(&graph, self, &ids, mi, beat)?;
        let pivot = super::tuplet_groups::first_pivot(&mut graph, &initial_groups)?;
        let fallback = pivot.is_some() && beat == Fraction::new(1, 2)?;
        let groups = if fallback {
            build_on(&graph, self, &ids, mi, Fraction::new(1, 4)?)?
        } else {
            initial_groups.clone()
        };
        self.owner.graph = graph;
        Ok(TupletBeatPlan {
            voice: identity.clone(),
            groups,
            pivot,
            fallback,
        })
    }
}
fn build_on(
    graph: &Graph,
    owner: &OwnedMusic,
    ids: &[usize],
    mi: usize,
    beat: Fraction,
) -> Result<Vec<Vec<Key>>, String> {
    let raw = beat.times(Fraction::new(3, 2)?)?;
    let mut next = Fraction::ZERO;
    let mut groups = Vec::<Vec<Key>>::new();
    for id in ids {
        let ch = owner
            .chords
            .iter()
            .find(|ch| ch.id == *id)
            .ok_or("music_actual_tuplet_chord_owner_required")?;
        let m = owner
            .measures
            .get(ch.measure)
            .ok_or("music_actual_tuplet_measure_required")?;
        graph.validate_tuplet_chord_container(ch.key, ch.id, m.part, m.index, ch.kind)?;
        if ch.key.system != owner.measures[mi].system || ch.key.system != m.system {
            return Err("music_actual_tuplet_chord_owner_required".into());
        }
        let key = ch.key;
        let time = graph
            .actual_chord_time_offset(key)?
            .ok_or("music_tuplet_null_group_chord_time")?;
        if time >= next {
            // Source increments once per matching Chord. Filling a large gap
            // with empty beats changes group count and later force allocation.
            next = next.plus(raw)?;
            groups.push(Vec::new());
        }
        if let Some(group) = groups.last_mut() {
            group.push(key);
        }
    }
    Ok(groups)
}
