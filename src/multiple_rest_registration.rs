// SPDX-License-Identifier: AGPL-3.0-or-later
// Explicit pre-cleanup graph records, with native phase-local identities.
use crate::{
    beam_dispatch::Registry,
    multiple_rest_detection::{RestPlan, SerifPlan},
    sections::Section,
};
#[derive(Clone, Debug, PartialEq)]
pub struct RestInter {
    pub id: usize,
    pub source_beam: usize,
    pub staff_id: u64,
    pub median: [f64; 4],
    pub height: f64,
    pub grade: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SerifInter {
    pub id: usize,
    pub glyph_id: usize,
    pub plan: SerifPlan,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    pub rest_id: usize,
    pub serif_id: usize,
    pub left: bool,
}
#[derive(Default, Debug)]
pub struct RestGraph {
    pub rests: Vec<RestInter>,
    pub serifs: Vec<SerifInter>,
    pub relations: Vec<Relation>,
    next_id: usize,
}
impl RestGraph {
    /// Register rest then LEFT and RIGHT serif vertices/edges in encounter order.
    /// This explicit phase graph does not delete source beams or group vertices.
    /// A missing original filament is an error, not a fabricated glyph.
    pub fn register(
        &mut self,
        plans: &[RestPlan],
        sections: &[Section],
        interline: usize,
        registry: &mut Registry,
    ) -> Result<(), String> {
        let mut ids = std::collections::BTreeSet::new();
        let existing = self
            .rests
            .iter()
            .map(|r| r.source_beam)
            .collect::<std::collections::BTreeSet<_>>();
        if plans.len() > 10000 || self.next_id.checked_add(plans.len() * 3).is_none() {
            return Err("rest_graph_budget".into());
        }
        let mut built = Vec::new();
        for p in plans {
            if !ids.insert(p.beam_id)
                || existing.contains(&p.beam_id)
                || p.beam_id == 0
                || p.staff_id == 0
                || !p.grade.is_finite()
                || !p.height.is_finite()
                || p.height <= 0.
                || !p.median.iter().all(|v| v.is_finite())
                || !p.serifs[0].left
                || p.serifs[1].left
            {
                return Err("invalid_rest_registration_plan".into());
            }
            let mut glyphs = Vec::new();
            for serif in &p.serifs {
                glyphs.push(
                    crate::serif_glyphs::build(&serif.peak, sections, interline)?
                        .ok_or("missing_original_serif_filament")?,
                );
            }
            built.push(glyphs);
        }
        for (p, glyphs) in plans.iter().zip(built) {
            self.next_id += 1;
            let rest_id = self.next_id;
            self.rests.push(RestInter {
                id: rest_id,
                source_beam: p.beam_id,
                staff_id: p.staff_id,
                median: p.median,
                height: p.height,
                grade: p.grade,
            });
            for (serif, glyph) in p.serifs.iter().zip(glyphs) {
                let glyph_id = registry.register(&glyph)?;
                self.next_id += 1;
                let serif_id = self.next_id;
                self.serifs.push(SerifInter {
                    id: serif_id,
                    glyph_id,
                    plan: serif.clone(),
                });
                self.relations.push(Relation {
                    rest_id,
                    serif_id,
                    left: serif.left,
                });
            }
        }
        Ok(())
    }
}

impl RestGraph {
    /// Explicit final phase, called after successful registration of every rest.
    /// Existing beam glyphs remain registered. Candidate tombstones, incident
    /// exclusions/supports and ensemble membership are updated without regrouping.
    pub fn finish_cleanup(
        &self,
        state: &mut crate::beam_candidates::SystemCandidates,
        groups: &mut crate::beam_groups::GroupPass,
    ) -> Result<(), String> {
        for rest in &self.rests {
            let relations = self
                .relations
                .iter()
                .filter(|r| r.rest_id == rest.id)
                .collect::<Vec<_>>();
            if relations.len() != 2
                || !relations[0].left
                || relations[1].left
                || relations
                    .iter()
                    .any(|r| self.serifs.iter().filter(|s| s.id == r.serif_id).count() != 1)
            {
                return Err("incomplete_rest_registration".into());
            }
        }
        let removed = self.rests.iter().map(|r| r.source_beam).collect::<Vec<_>>();
        if removed.iter().any(|id| state.active(*id).is_none()) {
            return Err("inactive_rest_source_beam".into());
        }
        let beams = state
            .inters()
            .iter()
            .map(|b| crate::beam_groups::GroupBeam {
                id: b.id(),
                bounds: b.bounds,
                item: b.item.clone(),
            })
            .collect::<Vec<_>>();
        let updated = crate::beam_groups::remove_members(groups, &beams, &removed)?;
        for id in removed {
            state.remove(id)?;
        }
        *groups = updated;
        Ok(())
    }
}
