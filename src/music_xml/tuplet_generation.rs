// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    TupletPlan,
    tuplet_groups::{existing_on, measure_index},
};
use crate::{music_font_symbols::TupletFontSymbols, music_measures::OwnedMusic, shared_sig::Key};

impl OwnedMusic {
    /// Fresh implicit lifecycle on retained ordinary single-staff owners.
    /// SheetStub font selection and automatic rhythm/export admission follow.
    pub fn generate_rhythm_tuplet(
        &mut self,
        number: usize,
        group: &[Key],
        font: &TupletFontSymbols,
    ) -> Result<Key, String> {
        let mi = measure_index(&self.measures, number)?;
        if let Some(key) = existing_on(&self.owner.graph, &self.measures, &self.chords, mi, group)?
        {
            return Ok(key);
        }
        let old_graph = self.owner.graph.clone();
        let held = self
            .measures
            .iter_mut()
            .map(|measure| {
                let stage = measure.tuplets.clone();
                let tuplets = std::mem::replace(&mut measure.tuplets, stage);
                let stage = measure.stack_tuplets.clone();
                let stack = std::mem::replace(&mut measure.stack_tuplets, stage);
                (tuplets, stack)
            })
            .collect::<Vec<_>>();
        let result = self.generate_tuplet_on(number, mi, group, font);
        if result.is_err() {
            self.owner.graph = old_graph;
            for (measure, (tuplets, stack)) in self.measures.iter_mut().zip(held) {
                measure.tuplets = tuplets;
                measure.stack_tuplets = stack;
            }
        }
        result
    }
    fn generate_tuplet_on(
        &mut self,
        number: usize,
        mi: usize,
        group: &[Key],
        font: &TupletFontSymbols,
    ) -> Result<Key, String> {
        let TupletPlan::New { shape, bounds } = self.plan_rhythm_tuplet(number, group, font)?
        else {
            return Err("music_tuplet_fresh_plan_required".into());
        };
        let measure = &self.measures[mi];
        let system = measure.system;
        let staffs = self
            .owner
            .staffs
            .iter()
            .filter(|s| s.system == system)
            .collect::<Vec<_>>();
        if staffs.len() != 1
            || staffs[0].id != measure.staff
            || self
                .measures
                .iter()
                .filter(|m| m.system == system)
                .any(|m| m.staff != measure.staff || m.part != measure.part)
        {
            return Err("music_tuplet_shared_stack_allocation_pending".into());
        }
        let x = bounds[0]
            .checked_add(bounds[2] / 2)
            .ok_or("music_tuplet_actual_center_required")?;
        // In the retained single-staff/single-barline path these are the
        // actual Measure abscissae. Source includes right and excludes left;
        // added may locate another Stack, or none, before generator membership.
        let added_stack = self
            .measures
            .iter()
            .position(|m| m.system == system && x > m.left && x <= m.right);
        let key = self.owner.graph.fresh_tuplet_key(system)?;
        #[cfg(test)]
        let clocks = self
            .chords
            .iter()
            .filter(|ch| ch.measure == mi)
            .map(|ch| ch.key)
            .collect::<Vec<_>>();
        self.owner.graph.register_fresh_tuplet(key, shape, bounds)?;
        #[cfg(test)]
        self.observe_tuplet_owner_addition(key, None, "vertex_registered", mi, &clocks)?;
        self.owner.graph.attach_fresh_tuplet(key)?;
        if self.owner.graph.tuplet_part(key)?.is_some() {
            return Err("music_tuplet_fresh_part_must_be_null".into());
        }
        if let Some(stack) = added_stack {
            self.measures[stack].stack_tuplets.push(key);
        }
        self.owner.graph.set_tuplet_abnormal(key, true)?;
        self.owner.graph.set_tuplet_abnormal(key, false)?;
        self.measures[mi].tuplets.push(key);
        for &chord in group {
            let _edge = self
                .owner
                .graph
                .add_generated_chord_tuplet(chord, key, shape)?;
            #[cfg(test)]
            self.observe_tuplet_owner_addition(
                key,
                Some(_edge),
                "chord_tuplet_added",
                mi,
                &clocks,
            )?;
        }
        Ok(key)
    }
    #[cfg(test)]
    fn observe_tuplet_owner_addition(
        &mut self,
        key: Key,
        edge: Option<usize>,
        phase: &'static str,
        mi: usize,
        clocks: &[Key],
    ) -> Result<(), String> {
        let mut stack = self.measures[mi].tuplets.clone();
        for &key in &self.measures[mi].stack_tuplets {
            if !stack.contains(&key) {
                stack.push(key);
            }
        }
        let containers = self
            .measures
            .iter()
            .filter(|m| m.system == key.system)
            .map(|m| {
                let measure = m.tuplets.contains(&key);
                let unassigned = m.stack_tuplets.contains(&key);
                [measure, measure || unassigned, unassigned]
            })
            .collect();
        self.owner.graph.observe_tuplet_addition(
            key,
            edge,
            phase,
            crate::shared_sig::TupletAdditionOwners {
                measure: &self.measures[mi].tuplets,
                stack: &stack,
                clocks,
                containers,
            },
        )
    }
}
