// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Original BeamsBuilder.createBeams/checkBeamGlyph/createBeamInters boundary.
// Initial registered candidates only; extensions, late hooks, beam groups,
// multiple-measure rests and later music interpretation are downstream.
use crate::{
    GrayImage, MAX_PIXELS,
    beam_dispatch::{self, Group},
    beam_impacts::{Evaluator, Impacts, ItemAreas, Parameters},
    beam_prolog::Prolog,
    beam_structure::{self, BeamItem, BeamStructure, ItemParameters, VerticalSide},
    scale::Scale,
};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde_json::{Value, json};
const MAX_WORK: usize = 1_000_000_000;
const MAX_INTERS: usize = 100_000;
struct Budget(usize);
impl Budget {
    fn take(&mut self, n: usize) -> Result<(), String> {
        self.0 = self
            .0
            .checked_add(n)
            .filter(|v| *v <= MAX_WORK)
            .ok_or("unsupported_beam_candidate_work")?;
        Ok(())
    }
}
/// Native identity is (system_id, id); these are not original JVM allocation IDs.
/// glyph_id refers to the shared prolog registry and retains its group ownership.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateInter {
    id: usize,
    glyph_id: usize,
    removed: bool,
    pub kind: &'static str,
    pub shape: &'static str,
    pub bounds: [i32; 4],
    pub item: BeamItem,
    pub impacts: Impacts,
    pub glyph: GlyphPixels,
}
impl CandidateInter {
    pub fn is_removed(&self) -> bool {
        self.removed
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn glyph_id(&self) -> usize {
        self.glyph_id
    }
    fn canonical(&self) -> Value {
        let names = ["wdth", "minH", "maxH", "core", "belt", "jit"];
        json!({"kind":self.kind,"shape":self.shape,"bounds":self.bounds,"grade":self.impacts.grade,"contextual_grade":null,"glyph":beam_dispatch::glyph_json(&self.glyph),"median":self.item.median,"height":self.item.height,"impacts":{"grade":self.impacts.grade,"intrinsic_ratio":0.8,"checks":names.into_iter().enumerate().map(|(i,name)|json!({"name":name,"weight":crate::beam_impacts::IMPACT_WEIGHTS[i],"impact":self.impacts.values[i]})).collect::<Vec<_>>()},"staff_id":null})
    }
}

/// Candidates in registration order, with typed identity-based exclusions.
/// Fresh pre-group phase only: no chords, ensembles or beam groups exist here.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemCandidates {
    system_id: u64,
    ordered_spots: Vec<Value>,
    assigned_spot_ids: Vec<usize>,
    inters: Vec<CandidateInter>,
    exclusions: Vec<(usize, usize)>,
    lifecycle_work: usize,
    state_changed: bool,
}
impl SystemCandidates {
    pub fn empty(system_id: u64) -> Self {
        Self {
            system_id,
            ordered_spots: Vec::new(),
            assigned_spot_ids: Vec::new(),
            inters: Vec::new(),
            exclusions: Vec::new(),
            lifecycle_work: 0,
            state_changed: false,
        }
    }
    pub fn active(&self, id: usize) -> Option<&CandidateInter> {
        self.inters.iter().find(|v| v.id == id && !v.removed)
    }
    /// Read the raw order including tombstones; lookup itself excludes removed inters.
    pub fn lookup_side(
        &self,
        target: usize,
        side: crate::beam_extension_geometry::HorizontalSide,
        reach: Option<f64>,
        interline: usize,
    ) -> Result<crate::beam_side_lookup::SideLookup, String> {
        let raw = self
            .inters
            .iter()
            .map(|v| crate::beam_side_lookup::LookupBeam {
                id: v.id,
                item: v.item.clone(),
                bounds: v.bounds,
                removed: v.removed,
            })
            .collect::<Vec<_>>();
        crate::beam_side_lookup::get_side_beam(&raw, target, side, reach, interline)
    }
    pub fn add_exclusion(&mut self, a: usize, b: usize) -> Result<(), String> {
        if a == b || self.active(a).is_none() || self.active(b).is_none() {
            return Err("invalid_beam_exclusion_endpoints".into());
        }
        if !self
            .exclusions
            .iter()
            .any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
        {
            self.exclusions.push((a, b));
            self.state_changed = true;
        }
        Ok(())
    }
    /// Append to raw registration order; canonical glyph registration adds no group.
    pub fn register_merge(
        &mut self,
        image: &GrayImage,
        registry: &mut beam_dispatch::Registry,
        candidate: &crate::beam_merge::MergeCandidate,
    ) -> Result<usize, String> {
        if self.inters.len() >= MAX_INTERS {
            return Err("unsupported_beam_candidate_inter_count".into());
        }
        let mut budget = Budget(self.lifecycle_work);
        let (bounds, glyph) = retrieve_glyph(image, &candidate.item, &mut budget)?;
        let glyph_id = registry.register(&glyph)?;
        self.lifecycle_work = budget.0;
        self.state_changed = true;
        let id = self.inters.len() + 1;
        self.inters.push(CandidateInter {
            id,
            glyph_id,
            removed: false,
            kind: if candidate.is_small {
                "SmallBeamInter"
            } else {
                "BeamInter"
            },
            shape: if candidate.is_small {
                "BEAM_SMALL"
            } else {
                "BEAM"
            },
            bounds,
            item: candidate.item.clone(),
            impacts: candidate.impacts.clone(),
            glyph,
        });
        Ok(id)
    }
    /// Tombstones remain in raw order; active lookup and incident exclusions vanish.
    pub fn remove(&mut self, id: usize) -> Result<(), String> {
        let inter = self
            .inters
            .iter_mut()
            .find(|v| v.id == id)
            .ok_or("unknown_beam_candidate_id")?;
        if !inter.removed {
            inter.removed = true;
            self.exclusions.retain(|&(a, b)| a != id && b != id);
            self.state_changed = true;
        }
        Ok(())
    }
    /// Apply an already evaluated merge, matching register/append/remove order.
    /// Rejection is a complete no-op. Eligibility lookup remains the caller's job.
    pub fn apply_merge(
        &mut self,
        image: &GrayImage,
        registry: &mut beam_dispatch::Registry,
        first: usize,
        second: usize,
        candidate: Option<&crate::beam_merge::MergeCandidate>,
    ) -> Result<Option<usize>, String> {
        let Some(candidate) = candidate else {
            return Ok(None);
        };
        if first == second || self.active(first).is_none() || self.active(second).is_none() {
            return Err("invalid_beam_merge_endpoints".into());
        }
        let id = self.register_merge(image, registry, candidate)?;
        self.remove(first)?;
        self.remove(second)?;
        Ok(Some(id))
    }
    pub fn system_id(&self) -> u64 {
        self.system_id
    }
    /// Original late hook registration appends without removing its parent,
    /// then records the originating spot (the stage's sorted pool stays fixed).
    pub fn register_hook(
        &mut self,
        image: &GrayImage,
        registry: &mut beam_dispatch::Registry,
        candidate: &crate::beam_merge::MergeCandidate,
        spot: usize,
    ) -> Result<usize, String> {
        registry.get(spot)?;
        let id = self.register_merge(image, registry, candidate)?;
        let hook = self.inters.last_mut().unwrap();
        hook.kind = "BeamHookInter";
        hook.shape = "BEAM_HOOK";
        self.assigned_spot_ids.push(spot);
        Ok(id)
    }
    pub fn inters(&self) -> &[CandidateInter] {
        &self.inters
    }
    pub fn exclusions(&self) -> &[(usize, usize)] {
        &self.exclusions
    }
    pub fn assigned_spot_ids(&self) -> &[usize] {
        &self.assigned_spot_ids
    }
    pub fn ordered_spots(&self) -> &[Value] {
        &self.ordered_spots
    }
    fn report(&self) -> Value {
        let mut canonical = self
            .inters
            .iter()
            .filter(|v| !v.removed)
            .map(CandidateInter::canonical)
            .collect::<Vec<_>>();
        canonical.sort_by_key(Value::to_string);
        let mut relations = self.exclusions.iter().map(|(a,b)| json!({"kind":"Exclusion","source":self.inters.iter().find(|v|v.id==*a).unwrap().canonical(),"target":self.inters.iter().find(|v|v.id==*b).unwrap().canonical()})).collect::<Vec<_>>();
        relations.sort_by_key(Value::to_string);
        json!({"system_id":self.system_id,"ordered_spots":self.ordered_spots,"assigned_spot_ids":self.assigned_spot_ids,"inters":canonical,"relations":relations})
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateAnalysis {
    systems: Vec<SystemCandidates>,
    work_units: usize,
    standard: ItemParameters,
    small: Option<ItemParameters>,
}
impl CandidateAnalysis {
    pub fn system_mut(&mut self, id: u64) -> Option<&mut SystemCandidates> {
        self.systems.iter_mut().find(|s| s.system_id == id)
    }
    pub fn systems(&self) -> &[SystemCandidates] {
        &self.systems
    }
    /// Preserve the initial report until mutated; then expose only active candidates.
    pub fn report(&self) -> Value {
        let systems = self
            .systems
            .iter()
            .map(SystemCandidates::report)
            .collect::<Vec<_>>();
        let stage = if self.systems.iter().any(|s| s.state_changed) {
            "BEAMS.native_candidate_state"
        } else {
            "BEAMS.native_initial_candidates"
        };
        let work_units = self
            .systems
            .iter()
            .fold(self.work_units, |n, s| n.saturating_add(s.lifecycle_work));
        json!({"stage":stage,"initial_beam_candidates_complete":true,"beams_complete":false,"musicxml_complete":false,"systems":systems,"work_units":work_units,"parameters":{"standard":self.standard,"small":self.small},"pending":["beam extensions and late hooks","beam groups and multiple-measure rests","LEDGERS, HEADS, stems, rhythm and MusicXML"],"supported_scope":"fresh default native prolog; explicit 4M-pixel, geometry, sampling and work bounds; no smallBeams override"})
    }
}
fn retrieve_glyph(
    image: &GrayImage,
    item: &BeamItem,
    budget: &mut Budget,
) -> Result<([i32; 4], GlyphPixels), String> {
    let area = ItemAreas::new(item, 0, 0, 0)?;
    let b = area.core_bounds();
    let left = (b[0] as i64).max(0);
    let top = (b[1] as i64).max(0);
    let right = (b[0] as i64 + b[2] as i64).min(image.width as i64);
    let bottom = (b[1] as i64 + b[3] as i64).min(image.height as i64);
    if right <= left || bottom <= top {
        return Err("unsupported_empty_beam_candidate_bounds".into());
    }
    let width = (right - left) as usize;
    let height = (bottom - top) as usize;
    budget.take(
        width
            .checked_mul(height)
            .ok_or("unsupported_beam_candidate_work")?,
    )?;
    let mut runs = Vec::new();
    for x in left..right {
        let mut start = None;
        for y in top..bottom {
            let black = image.pixels[y as usize * image.width + x as usize] == 0
                && area.core_contains([x as f64, y as f64])?;
            if black {
                if start.is_none() {
                    start = Some(y);
                }
            } else if let Some(s) = start.take() {
                runs.push(Run {
                    position: (x - left) as usize,
                    start: (s - top) as usize,
                    length: (y - s) as usize,
                });
            }
        }
        if let Some(s) = start {
            runs.push(Run {
                position: (x - left) as usize,
                start: (s - top) as usize,
                length: (bottom - s) as usize,
            });
        }
    }
    if runs.is_empty() {
        return Err("unsupported_empty_beam_candidate_glyph".into());
    }
    Ok((
        [left as i32, top as i32, width as i32, height as i32],
        GlyphPixels {
            left: left as i32,
            top: top as i32,
            width,
            height,
            orientation: Orientation::Vertical,
            runs,
        },
    ))
}
struct Context<'a> {
    image: &'a GrayImage,
    prolog: &'a mut Prolog,
    interline: usize,
    inters: Vec<CandidateInter>,
    exclusions: Vec<(usize, usize)>,
    budget: &'a mut Budget,
}
impl Context<'_> {
    fn register(
        &mut self,
        kind: &'static str,
        shape: &'static str,
        item: &BeamItem,
        impacts: Impacts,
    ) -> Result<usize, String> {
        if self.inters.len() >= MAX_INTERS {
            return Err("unsupported_beam_candidate_inter_count".into());
        }
        let (bounds, glyph) = retrieve_glyph(self.image, item, self.budget)?;
        let glyph_id = self.prolog.registry.register(&glyph)?;
        let id = self.inters.len() + 1;
        self.inters.push(CandidateInter {
            id,
            glyph_id,
            removed: false,
            kind,
            shape,
            bounds,
            item: item.clone(),
            impacts,
            glyph,
        });
        Ok(id)
    }
    fn check_glyph(
        &mut self,
        glyph: &GlyphPixels,
        p: ItemParameters,
    ) -> Result<Option<&'static str>, String> {
        let geometry = beam_structure::glyph_geometry(glyph)?;
        if (glyph.width as f64) < p.min_beam_width_low {
            return Ok(Some("too narrow"));
        }
        if geometry.mean_thickness < p.min_height_low {
            return Ok(Some("too slim"));
        }
        let Some(line) = geometry.center_line else {
            return Ok(Some("vertical"));
        };
        if ((line[3] - line[1]) / (line[2] - line[0])).abs() > 1.0 {
            return Ok(Some("too steep"));
        }
        let mut structure = BeamStructure::new(glyph, p)?;
        let distance = structure.compute_lines()?;
        if distance.is_none_or(|d| d > 0.15 * self.interline as f64) {
            return Ok(Some("wavy or inconsistent borders"));
        }
        if structure.width().ok_or("empty_successful_beam_structure")? < p.min_beam_width_low {
            return Ok(Some("too narrow width"));
        }
        if structure.compare_slopes() > 0.065 {
            return Ok(Some("diverging beams"));
        }
        structure.adjust_sides();
        structure.extend_middle_lines();
        structure.split_lines();
        let lines = structure.lines();
        if lines.is_empty() {
            return Err("empty_successful_beam_structure".into());
        }
        let jitter = match (
            structure.compute_jitter(&lines[0], VerticalSide::Top),
            structure.compute_jitter(lines.last().unwrap(), VerticalSide::Bottom),
        ) {
            (Ok(top), Ok(bottom)) => 0.5 * (top + bottom),
            _ => return Ok(Some("no good item")),
        };
        let distance_impact = 1.0 - jitter / 0.02;
        let evaluator = Evaluator::new(self.image, self.interline)?;
        let pixels = Parameters::new(self.interline)?;
        let mut accepted = false;
        for (index, line) in lines.iter().enumerate() {
            for item in &line.items {
                let max_area = ItemAreas::new(
                    item,
                    pixels.belt_margin_dx,
                    pixels.belt_margin_dy,
                    pixels.belt_margin_dy,
                )?;
                let cells = |b: [i32; 4]| {
                    (b[2].max(0) as usize)
                        .checked_mul(b[3].max(0) as usize)
                        .ok_or("unsupported_beam_candidate_work")
                };
                self.budget.take(
                    cells(max_area.core_bounds())?
                        .checked_add(cells(max_area.outer_bounds())?)
                        .and_then(|n| n.checked_mul(2))
                        .ok_or("unsupported_beam_candidate_work")?,
                )?;
                let mut hook = None;
                if !p.is_small
                    && item.median[2] - item.median[0] <= p.max_hook_width
                    && let Some(impacts) = evaluator
                        .compute_hook_impacts(item, distance_impact, &p)?
                        .impacts
                        .filter(Impacts::accepted_as_hook)
                {
                    hook = Some(self.register("BeamHookInter", "BEAM_HOOK", item, impacts)?);
                    accepted = true;
                }
                if let Some(impacts) = evaluator
                    .compute_beam_impacts(
                        item,
                        index == 0,
                        index + 1 == lines.len(),
                        distance_impact,
                        &p,
                    )?
                    .impacts
                    .filter(Impacts::accepted_as_beam)
                {
                    let (kind, shape) = if p.is_small {
                        ("SmallBeamInter", "BEAM_SMALL")
                    } else {
                        ("BeamInter", "BEAM")
                    };
                    let beam = self.register(kind, shape, item, impacts)?;
                    accepted = true;
                    if let Some(hook) = hook {
                        self.exclusions.push((hook, beam));
                    }
                }
            }
        }
        Ok(if accepted { None } else { Some("no good item") })
    }
}
/// Fresh default buildBeams initialization through createBeams, before removal
/// of assigned spots, extensions, late hooks, groups and epilog. On failure the
/// caller should discard this phase context; earlier serialized prolog survives.
pub fn analyze(image: &GrayImage, prolog: &mut Prolog, scale: &Scale) -> Result<Value, String> {
    Ok(analyze_typed(image, prolog, scale)?.report())
}

/// Retain native candidate IDs, registry references, registration order and
/// exclusions for later graph work. This still stops at initial candidates.
pub fn analyze_typed(
    image: &GrayImage,
    prolog: &mut Prolog,
    scale: &Scale,
) -> Result<CandidateAnalysis, String> {
    if prolog.head_sizing.music_font_unsupported.is_some() {
        return Err("incomplete_beam_prolog_dependency".into());
    }
    if image.width == 0
        || image.height == 0
        || image
            .width
            .checked_mul(image.height)
            .is_none_or(|n| n > MAX_PIXELS || n != image.pixels.len())
        || image.pixels.iter().any(|v| *v != 0 && *v != 255)
    {
        return Err("invalid_beam_candidates_image".into());
    }
    let standard = ItemParameters::new(scale.interline.main, scale.beam.thickness as f64, false)?;
    let small = scale
        .small_beam
        .map(|b| ItemParameters::new(scale.interline.main, b.thickness as f64, true))
        .transpose()?;
    let mut work = Budget(0);
    let mut inputs = Vec::new();
    let mut total_spots = 0usize;
    for system in &prolog.systems {
        let mut ids = Vec::new();
        for &id in &system.glyph_ids {
            let entry = prolog.registry.get(id)?;
            if !entry.groups.contains(&Group::BeamSpot) {
                continue;
            }
            total_spots += 1;
            if total_spots > 100_000 {
                return Err("unsupported_beam_candidate_spot_count".into());
            }
            let n = entry.glyph.runs.len();
            let estimate = n
                .checked_mul(n)
                .and_then(|v| v.checked_mul(8))
                .and_then(|v| {
                    v.checked_add(entry.glyph.runs.iter().map(|r| r.length).sum::<usize>())
                })
                .and_then(|v| v.checked_mul(if small.is_some() { 2 } else { 1 }))
                .ok_or("unsupported_beam_candidate_work")?;
            work.take(estimate)?;
            ids.push(id);
        }
        ids.sort_by_key(|id| {
            let g = &prolog.registry.get(*id).unwrap().glyph;
            (g.top, g.left, *id)
        });
        inputs.push((system.system_id, ids));
    }
    let mut systems = Vec::new();
    for (system_id, ids) in inputs {
        let mut context = Context {
            image,
            prolog,
            interline: scale.interline.main,
            inters: Vec::new(),
            exclusions: Vec::new(),
            budget: &mut work,
        };
        let mut traces = Vec::new();
        let mut assigned = Vec::new();
        for id in ids {
            let glyph = context.prolog.registry.get(id)?.glyph.clone();
            let first = context.check_glyph(&glyph, standard)?;
            let mut reason = first;
            if reason.is_some()
                && let Some(p) = small
            {
                reason = context.check_glyph(&glyph, p)?;
            }
            if reason.is_none() {
                assigned.push(id);
            }
            traces.push(json!({"spot_id":id,"glyph":beam_dispatch::glyph_json(&glyph),"standard_reason":first,"final_reason":reason,"assigned":reason.is_none()}));
        }
        systems.push(SystemCandidates {
            system_id,
            ordered_spots: traces,
            assigned_spot_ids: assigned,
            inters: context.inters,
            exclusions: context.exclusions,
            lifecycle_work: 0,
            state_changed: false,
        });
    }
    Ok(CandidateAnalysis {
        systems,
        work_units: work.0,
        standard,
        small,
    })
}
