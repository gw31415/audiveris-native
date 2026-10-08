// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{EdgeKey, Graph, Key, Phase};
use crate::music_font_symbols::TupletShape;

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct TupletAdditionObservation {
    pub tuplet: Key,
    pub edge: Option<usize>,
    pub phase: &'static str,
    pub measure_tuplets: Vec<Key>,
    pub stack_tuplets: Vec<Key>,
    pub graph: Graph,
    pub sig_attached: bool,
    pub containers: Vec<[bool; 3]>,
    pub candidate_clocks: Option<serde_json::Value>,
}
#[cfg(test)]
pub(crate) struct TupletAdditionOwners<'a> {
    pub measure: &'a [Key],
    pub stack: &'a [Key],
    pub clocks: &'a [Key],
    pub containers: Vec<[bool; 3]>,
}
impl Graph {
    pub(crate) fn fresh_tuplet_key(&self, system: u64) -> Result<Key, String> {
        let local = self
            .keys
            .keys()
            .filter(|k| k.system == system && k.phase == Phase::Symbol)
            .map(|k| k.local)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("music_tuplet_registration_overflow")?;
        Ok(Key {
            system,
            phase: Phase::Symbol,
            local,
        })
    }
    pub(crate) fn register_fresh_tuplet(
        &mut self,
        key: Key,
        shape: TupletShape,
        bounds: [i64; 4],
    ) -> Result<(), String> {
        if self.id(key).is_some() {
            return Err("music_tuplet_fresh_registry_owner_required".into());
        }
        let id=self.vertex(key,serde_json::json!({"kind":"TupletInter","shape":shape.as_str(),"bounds":bounds,"grade":1.0,"implicit":true,"manual":false,"abnormal":false}),false)?;
        // super.addVertex emits its event before SIGraph assigns Inter.sig.
        self.vertices[id - 1]
            .tuplet_cache
            .as_mut()
            .unwrap()
            .sig_attached = false;
        Ok(())
    }
    pub(crate) fn attach_fresh_tuplet(&mut self, key: Key) -> Result<(), String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        let cache = self.vertices[id - 1].tuplet_cache.as_mut().unwrap();
        if cache.sig_attached {
            return Err("music_tuplet_fresh_SIG_owner_required".into());
        }
        cache.sig_attached = true;
        Ok(())
    }
    pub(crate) fn set_tuplet_abnormal(&mut self, key: Key, abnormal: bool) -> Result<(), String> {
        self.tuplet_status(key)?;
        let id = self.endpoint(key)?;
        self.set_owned_abnormal(id, abnormal);
        Ok(())
    }
    pub(crate) fn add_generated_chord_tuplet(
        &mut self,
        chord: Key,
        tuplet: Key,
        shape: TupletShape,
    ) -> Result<usize, String> {
        let source = self.endpoint(chord)?;
        let target = self.endpoint(tuplet)?;
        let ch = &self.vertices[source - 1];
        if ch.key.phase != Phase::Chord
            || ch.key.system != tuplet.system
            || !matches!(
                ch.data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        {
            return Err("music_actual_chord_tuplet_relation_required".into());
        }
        if !self.tuplet_status(tuplet)?.0
            || !self.vertices[target - 1]
                .tuplet_cache
                .as_ref()
                .unwrap()
                .sig_attached
            || self.vertices[target - 1].data["shape"] != shape.as_str()
        {
            return Err("music_tuplet_generated_relation_owner_required".into());
        }
        let ordinal = self
            .edge_keys
            .keys()
            .filter(|e| {
                e.system == tuplet.system
                    && e.phase == Phase::Symbol
                    && e.source == source
                    && e.target == target
                    && e.kind == "ChordTupletRelation"
            })
            .map(|e| e.ordinal)
            .max()
            .map(|n| n.checked_add(1).ok_or("shared_sig_relation_order_budget"))
            .transpose()?
            .unwrap_or(0);
        let coeff = match shape {
            TupletShape::Three => 2.0 * 0.33,
            TupletShape::Six => 2.0 * 0.17,
        };
        self.relations(tuplet.system,Phase::Symbol,vec![(chord,tuplet,serde_json::json!({"kind":"ChordTupletRelation","manual":false,"grade":1.0,"source_ratio":1.0,"target_ratio":1.0+coeff,"_editing_registration_ordinal":ordinal}))])?;
        self.edge_keys
            .get(&EdgeKey {
                system: tuplet.system,
                phase: Phase::Symbol,
                source,
                target,
                kind: "ChordTupletRelation".into(),
                ordinal,
            })
            .copied()
            .ok_or("music_tuplet_generated_relation_registration_required".into())
    }
    #[cfg(test)]
    pub(crate) fn trace_tuplet_additions(&mut self) {
        self.tuplet_additions = Some(Vec::new());
    }
    #[cfg(test)]
    pub(crate) fn trace_tuplet_candidate_clocks(&mut self, clocks: Option<serde_json::Value>) {
        self.tuplet_candidate_clocks = clocks;
    }
    #[cfg(test)]
    pub(crate) fn fail_tuplet_addition_after(&mut self, count: Option<usize>) {
        self.tuplet_addition_failure = count;
    }
    #[cfg(test)]
    pub(crate) fn fail_tuplet_registration_after(&mut self, count: Option<usize>) {
        self.tuplet_registration_failure = count;
    }
    #[cfg(test)]
    pub(crate) fn take_tuplet_additions(&mut self) -> Vec<TupletAdditionObservation> {
        self.tuplet_additions
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    }
    #[cfg(test)]
    pub(crate) fn observe_tuplet_addition(
        &mut self,
        tuplet: Key,
        edge: Option<usize>,
        phase: &'static str,
        owners: TupletAdditionOwners<'_>,
    ) -> Result<(), String> {
        if self.tuplet_additions.is_some() {
            // The original listener reads live durations and warms the factor
            // at this boundary; observing a later final graph loses that order.
            for &key in owners.clocks {
                self.chord_duration(key)?;
                self.chord_end_time(key)?;
            }
            let held = self.tuplet_additions.take();
            let removals = self.tuplet_removals.take();
            let graph = self.clone();
            self.tuplet_removals = removals;
            self.tuplet_additions = held;
            let id = self.endpoint(tuplet)?;
            let sig_attached = self.vertices[id - 1]
                .tuplet_cache
                .as_ref()
                .unwrap()
                .sig_attached;
            self.tuplet_additions
                .as_mut()
                .unwrap()
                .push(TupletAdditionObservation {
                    tuplet,
                    edge,
                    phase,
                    measure_tuplets: owners.measure.to_vec(),
                    stack_tuplets: owners.stack.to_vec(),
                    graph,
                    sig_attached,
                    containers: owners.containers,
                    candidate_clocks: self.tuplet_candidate_clocks.clone(),
                });
        }
        if edge.is_none()
            && self.tuplet_registration_failure.is_some_and(|count| {
                self.tuplet_additions.as_ref().is_some_and(|events| {
                    events
                        .iter()
                        .filter(|event| event.phase == "vertex_registered")
                        .count()
                        == count
                })
            })
        {
            return Err("music_tuplet_addition_observer_failure".into());
        }
        if edge.is_some() && self.tuplet_addition_failure == Some(self.tuplet_chords(tuplet)?.len())
        {
            return Err("music_tuplet_addition_observer_failure".into());
        }
        Ok(())
    }
}
