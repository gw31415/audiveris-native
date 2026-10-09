// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
//! Automatic short Slurs before SYMBOLS.
//! Retained component/raw-pixel extraction differs from original Skeleton/trunk provenance.
//! All source IDs, expected coordinates and fixture masks belong only in observation tests.
use crate::shared_sig::slur_owners::{NativeChord, NativeHead, NativeOwners};
use crate::{
    GrayImage, beam_dispatch,
    grid_ir::{PartGeometry, StaffGeometry, SystemGeometry},
    music_measure_layout::MeasureLayout,
    music_measures::OwnedMusic,
    shared_sig::{Graph, Key},
};
use audiveris_staff_removal::{GlyphPixels, Orientation, Run};
use serde_json::{Value, json};
use std::sync::Arc;
#[cfg(test)]
mod boundary_tests;
mod clump;
mod extension_boxes;
pub(crate) mod fit;
#[cfg(test)]
pub(crate) mod image_tests;
pub(crate) mod lookup;
pub(crate) mod raw;

pub(crate) struct Candidate {
    components: Vec<Arc<GlyphPixels>>,
    points: Vec<fit::Pixel>,
    curve: lookup::Curve,
    grade: f64,
    glyph: GlyphPixels,
}
#[derive(Debug)]
pub(crate) struct Relation {
    pub(crate) side: lookup::Side,
    pub(crate) euclidean: f64,
}
impl Relation {
    pub(crate) fn projection(&self) -> Value {
        json!({"kind":"SlurHeadRelation","side":if self.side==lookup::Side::Left{"LEFT"}else{"RIGHT"},
            "euclidean":self.euclidean,"manual":false,"grade":1.0,"source_ratio":6.0,"target_ratio":1.0})
    }
}
pub(crate) struct Endpoint {
    pub(crate) head: Key,
    pub(crate) chord: Key,
    pub(crate) staff: Arc<StaffGeometry>,
    pub(crate) measure: Arc<MeasureLayout>,
}
struct PendingSlur {
    candidate: Arc<Candidate>,
    system: Arc<SystemGeometry>,
    endpoints: [Endpoint; 2],
    relations: [Arc<Relation>; 2],
}
pub(crate) struct Slur {
    pub(crate) key: Key,
    pub(crate) system: Arc<SystemGeometry>,
    pub(crate) part: Arc<PartGeometry>,
    pub(crate) endpoints: [Endpoint; 2],
    pub(crate) relations: [Arc<Relation>; 2],
    pub(crate) relation_ids: [usize; 2],
    pub(crate) glyph_id: usize,
    candidate: Arc<Candidate>,
}
impl Slur {
    pub(crate) fn curve(&self) -> lookup::Curve {
        self.candidate.curve
    }
}
pub(crate) struct PartSlurs {
    pub(crate) part: Arc<PartGeometry>,
    pub(crate) slurs: Vec<Arc<Slur>>,
}
pub(crate) struct State {
    pub(crate) slurs: Vec<Arc<Slur>>,
    pub(crate) parts: Vec<PartSlurs>,
    #[cfg(test)]
    pub(crate) raw_candidate_count: usize,
    #[cfg(test)]
    pub(crate) clump_count: usize,
}
fn glyph_view(g: &GlyphPixels) -> Result<fit::GlyphRuns, String> {
    if g.orientation != Orientation::Vertical {
        return Err("slur_actual_vertical_components_required".into());
    }
    Ok(fit::GlyphRuns {
        bounds: fit::Rect {
            x: g.left,
            y: g.top,
            width: i32::try_from(g.width).map_err(|_| "slur_component_budget")?,
            height: i32::try_from(g.height).map_err(|_| "slur_component_budget")?,
        },
        runs: g
            .runs
            .iter()
            .map(|r| {
                Ok(fit::VerticalRun {
                    x: i32::try_from(r.position).map_err(|_| "slur_run_budget")?,
                    y: i32::try_from(r.start).map_err(|_| "slur_run_budget")?,
                    length: i32::try_from(r.length).map_err(|_| "slur_run_budget")?,
                })
            })
            .collect::<Result<_, String>>()?,
    })
}
fn retrieved_glyph(s: &raw::SupportEvidence) -> Result<GlyphPixels, String> {
    if !s.accepted_by_run_support_only {
        return Err("slur_raw_support_required".into());
    }
    let [left, top, width, height] = s.glyph_bounds;
    Ok(GlyphPixels {
        left,
        top,
        width: usize::try_from(width).map_err(|_| "slur_glyph_width")?,
        height: usize::try_from(height).map_err(|_| "slur_glyph_height")?,
        orientation: Orientation::Vertical,
        runs: s
            .accepted_runs
            .iter()
            .map(|r| {
                Ok(Run {
                    position: usize::try_from(r.x - left).map_err(|_| "slur_run_x")?,
                    start: usize::try_from(r.start - top).map_err(|_| "slur_run_y")?,
                    length: r.length,
                })
            })
            .collect::<Result<_, String>>()?,
    })
}
fn candidates(
    m: &OwnedMusic,
    components: &[Arc<GlyphPixels>],
) -> Result<Vec<Arc<Candidate>>, String> {
    let image = &m.owner.raw_binary;
    let binary = raw::BinaryImage::new(image.width, image.height, &image.pixels)
        .map_err(|e| format!("slur_binary:{e}"))?;
    let il = m.owner.interline as f64;
    let views = components
        .iter()
        .map(|g| glyph_view(g))
        .collect::<Result<Vec<_>, _>>()?;
    let extracted = fit::extract_short(&views, il).map_err(|e| format!("slur_extract:{e:?}"))?;
    let extension_boxes =
        extension_boxes::grouped_extension_boxes(&extracted.candidates, components)?;
    let mut out = Vec::new();
    for c in extracted.candidates {
        let selected = c
            .component_indices
            .iter()
            .map(|&i| Arc::clone(&components[i]))
            .collect::<Vec<_>>();
        let (points, support) = if selected.len() == 2 {
            let (left_index, a, b) = extension_boxes::gap_ends(&c, components)?;
            let extension = *extension_boxes
                .get(&left_index)
                .ok_or("slur_all_reachable_extension_box_required")?;
            #[cfg(test)]
            image_tests::gap_input(&selected, &c.points);
            if !m
                .owner
                .graph
                .slur_gap_is_clear(&m.owner.systems, a, b, extension)?
            {
                continue;
            }
            let refined = raw::refine_gap_foreground(&binary, &c.points, a, b, il)
                .map_err(|e| format!("slur_raw_gap:{e}"))?;
            if !refined.gap.accepted_by_gap_only {
                continue;
            }
            (refined.points, refined.support)
        } else {
            let support = raw::retrieve_support(&binary, &c.points, il)
                .map_err(|e| format!("slur_raw_run:{e}"))?;
            (c.points, support)
        };
        let fitted = match fit::fit_short(&points, il) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if !fitted.math_accepted_before_binary_and_head_links
            || !support.accepted_by_run_support_only
        {
            continue;
        }
        let bezier = fitted
            .bezier
            .ok_or("slur_actual_fitted_curve_required")?
            .map(|p| [p.x, p.y]);
        out.push(Arc::new(Candidate {
            components: selected,
            points,
            curve: lookup::Curve {
                bezier,
                above: fitted
                    .selected_after_impacts
                    .as_ref()
                    .ok_or("slur_model_required")?
                    .above,
            },
            grade: fitted.impacts.ok_or("slur_impacts_required")?.grade,
            glyph: retrieved_glyph(&support)?,
        }));
    }
    Ok(out)
}
struct LinkView<'a, C, H> {
    candidate: &'a lookup::HeadCandidate<'a, C, H>,
    relation: Arc<Relation>,
}
impl<C, H> clump::PendingLink for LinkView<'_, C, H> {
    type Chord = C;
    fn chord(&self) -> &C {
        self.candidate.chord
    }
    fn euclidean(&self) -> f64 {
        self.relation.euclidean
    }
}
fn union(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    let x = a[0].min(b[0]);
    let y = a[1].min(b[1]);
    [
        x,
        y,
        (a[0] + a[2]).max(b[0] + b[2]) - x,
        (a[1] + a[3]).max(b[1] + b[3]) - y,
    ]
}
fn rect_hits(a: [i64; 4], b: [i32; 4]) -> bool {
    a[0] < (b[0] + b[2]) as i64
        && (b[0] as i64) < a[0] + a[2]
        && a[1] < (b[1] + b[3]) as i64
        && (b[1] as i64) < a[1] + a[3]
}
fn endpoint(link: &LinkView<'_, NativeChord<'_>, NativeHead<'_>>) -> Result<Endpoint, String> {
    let c = link.candidate.chord;
    let h = link.candidate.head;
    if c.stem.is_none() || c.beams.as_ref().is_none_or(Vec::is_empty) {
        return Err("slur_stemless_or_unbeamed_endpoint_comparison_pending".into());
    }
    Ok(Endpoint {
        head: h.key,
        chord: c.owner.key,
        staff: Arc::clone(&h.staff),
        measure: Arc::clone(
            c.measure
                .geometry
                .as_ref()
                .ok_or("slur_actual_measure_layout_required")?,
        ),
    })
}
fn choose(
    m: &OwnedMusic,
    graph: &mut Graph,
    group: &[&Arc<Candidate>],
) -> Result<Option<PendingSlur>, String> {
    let geometries = group
        .iter()
        .map(|c| {
            lookup::define_lookup(c.curve, m.owner.interline as f64)
                .map_err(|e| format!("slur_lookup_area:{e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut bounds = [
        geometries[0].area[0].bounds(),
        geometries[0].area[1].bounds(),
    ];
    for g in &geometries[1..] {
        for (side, bounds) in bounds.iter_mut().enumerate() {
            *bounds = union(*bounds, g.area[side].bounds());
        }
    }
    let systems = m.owner.systems.clone();
    for system in systems {
        if !system.area.intersects_rect(bounds[0].map(f64::from))?
            || !system.area.intersects_rect(bounds[1].map(f64::from))?
        {
            continue;
        }
        let owners = NativeOwners::borrow(m, graph, &system)?;
        let filtered: [Vec<_>; 2] = std::array::from_fn(|side| {
            owners
                .chords
                .iter()
                .filter(|c| rect_hits(c.bounds, bounds[side]))
                .collect()
        });
        let mut lookups = Vec::new();
        for (candidate, geometry) in group.iter().zip(&geometries) {
            let left = lookup::lookup_chords(
                &owners,
                candidate.curve,
                geometry,
                lookup::Side::Left,
                &filtered[0],
            )
            .map_err(|e| format!("slur_left_lookup:{e}"))?;
            let right = lookup::lookup_chords(
                &owners,
                candidate.curve,
                geometry,
                lookup::Side::Right,
                &filtered[1],
            )
            .map_err(|e| format!("slur_right_lookup:{e}"))?;
            if left
                .iter()
                .chain(&right)
                .any(|l| l.chord.mirror_key.is_some())
            {
                return Err("slur_mirror_positive_comparison_pending".into());
            }
            lookups.push([left, right]);
        }
        // One relation owner per pending lookup record, before pair/clump selection.
        let links = lookups
            .iter()
            .map(|sides| {
                std::array::from_fn::<_, 2, _>(|side| {
                    sides[side]
                        .iter()
                        .map(|c| {
                            let relation = Arc::new(Relation {
                                side: c.side,
                                euclidean: c.chord_euclidean,
                            });
                            #[cfg(test)]
                            image_tests::pending(&relation);
                            LinkView {
                                candidate: c,
                                relation,
                            }
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        let mut entries = Vec::new();
        for (index, ((candidate, geometry), sides)) in
            group.iter().zip(&geometries).zip(&lookups).enumerate()
        {
            let Some(pair) = lookup::select_pair(
                &owners,
                candidate.curve,
                geometry,
                false,
                &sides[0],
                &sides[1],
            )
            .map_err(|e| format!("slur_pair:{e}"))?
            else {
                continue;
            };
            if pair.iter().any(Option::is_none) {
                return Err("slur_ordinary_orphan_admission_pending".into());
            }
            let retained = std::array::from_fn(|side| {
                links[index][side]
                    .iter()
                    .find(|l| std::ptr::eq(l.candidate, pair[side].unwrap()))
            });
            entries.push(clump::Entry {
                candidate: *candidate,
                point_count: candidate.points.len(),
                links: retained,
            });
        }
        let Some(best) = clump::select(&entries, |a, b| std::ptr::eq(a.owner, b.owner))
            .map_err(|e| format!("slur_clump:{e:?}"))?
        else {
            continue;
        };
        let left = best.links[0].unwrap();
        let right = best.links[1].unwrap();
        if !Arc::ptr_eq(&left.candidate.head.staff, &right.candidate.head.staff) {
            return Err("slur_cross_staff_comparison_pending".into());
        }
        let pitch = |h: &NativeHead<'_>| {
            h.body["pitch"]
                .as_f64()
                .filter(|v| v.is_finite())
                .map(f64::round_ties_even)
                .ok_or("slur_actual_integer_pitch_required")
        };
        if pitch(left.candidate.head)? == pitch(right.candidate.head)? {
            return Err("slur_provisional_tie_space_clear_collision_pending".into());
        }
        let endpoints = [endpoint(left)?, endpoint(right)?];
        if !Arc::ptr_eq(&endpoints[0].measure.part, &endpoints[1].measure.part) {
            return Err("slur_cross_part_dispatch_comparison_pending".into());
        }
        if best.candidate.grade < 0.4 {
            return Err("slur_weak_optional_glyph_cleaner_pending".into());
        }
        return Ok(Some(PendingSlur {
            candidate: Arc::clone(best.candidate),
            system,
            endpoints,
            relations: [Arc::clone(&left.relation), Arc::clone(&right.relation)],
        }));
    }
    Ok(None)
}
fn curve_bounds(curve: lookup::Curve) -> [i64; 4] {
    let lo: [f64; 2] = std::array::from_fn(|i| {
        curve
            .bezier
            .iter()
            .map(|p| p[i])
            .fold(f64::INFINITY, f64::min)
            .floor()
    });
    let hi: [f64; 2] = std::array::from_fn(|i| {
        curve
            .bezier
            .iter()
            .map(|p| p[i])
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil()
    });
    [
        lo[0] as i64,
        lo[1] as i64,
        (hi[0] - lo[0]) as i64,
        (hi[1] - lo[1]) as i64,
    ]
}
/// Fresh automatic source path only. All refusal paths precede publication.
/// `erased` is the existing native pre-classifier cleaner output, not a fixture.
fn stage(
    m: &OwnedMusic,
    mut erased: GrayImage,
    components: Vec<GlyphPixels>,
) -> Result<Staged, String> {
    if m.slurs.is_some() {
        return Err("slur_fresh_stage_required".into());
    }
    let components = components.into_iter().map(Arc::new).collect::<Vec<_>>();
    let raw = candidates(m, &components)?;
    let identities = raw
        .iter()
        .map(|c| clump::ComponentCandidate {
            candidate: c,
            components: c.components.iter().map(Arc::as_ref).collect(),
        })
        .collect::<Vec<_>>();
    let groups =
        clump::component_groups(&identities).map_err(|e| format!("slur_component_clumps:{e:?}"))?;
    let mut graph = m.owner.graph.clone();
    #[cfg(test)]
    if image_tests::active() {
        graph.enable_fresh_slur_trace();
    }
    let mut plans = Vec::new();
    for group in &groups {
        let candidates = group.iter().map(|c| c.candidate).collect::<Vec<_>>();
        if let Some(plan) = choose(m, &mut graph, &candidates)? {
            plans.push(plan);
        }
    }
    let mut registry = m.owner.prepared.registry.clone();
    let mut slurs = Vec::new();
    for plan in plans {
        let key = graph.next_curve_key(plan.system.id)?;
        let glyph_id = registry.register(&plan.candidate.glyph)?;
        #[cfg(test)]
        graph.record_fresh_slur("ClumpPruner$ClumpLinker.doLink.entry", key, None);
        graph.vertex(key,json!({"kind":"SlurInter","shape":if plan.candidate.curve.above{"SLUR_ABOVE"}else{"SLUR_BELOW"},"above":plan.candidate.curve.above,"curve":plan.candidate.curve.bezier,"bounds":curve_bounds(plan.candidate.curve),"glyph_id":glyph_id,"glyph":beam_dispatch::glyph_json(&registry.get(glyph_id)?.glyph),"grade":plan.candidate.grade,"manual":false,"is_tie":false}),false)?;
        let mut ids = [0; 2];
        for (side, id) in ids.iter_mut().enumerate() {
            *id = graph.register_fresh_slur_relation(
                key,
                plan.endpoints[side].head,
                Arc::clone(&plan.relations[side]),
            )?;
        }
        #[cfg(test)]
        graph.record_fresh_slur("ClumpPruner$ClumpLinker.doLink.return", key, None);
        slurs.push(Arc::new(Slur {
            key,
            system: plan.system,
            part: Arc::clone(&plan.endpoints[0].measure.part),
            endpoints: plan.endpoints,
            relations: plan.relations,
            relation_ids: ids,
            glyph_id,
            candidate: plan.candidate,
        }));
    }
    // No admitted provisional ties: source collision handling has no candidates.
    // dispatchToParts occurs after every clump's complete vertex/edge lifecycle.
    let mut parts: Vec<PartSlurs> = Vec::new();
    for system in &m.owner.systems {
        for slur in slurs
            .iter()
            .filter(|slur| Arc::ptr_eq(&slur.system, system))
        {
            #[cfg(test)]
            image_tests::part_event(&mut graph, "Part.addSlur.entry", slur, &parts);
            // The source appends the same Slur while its cached Part is still null.
            if let Some(p) = parts.iter_mut().find(|p| Arc::ptr_eq(&p.part, &slur.part)) {
                p.slurs.push(Arc::clone(slur));
            } else {
                parts.push(PartSlurs {
                    part: Arc::clone(&slur.part),
                    slurs: vec![Arc::clone(slur)],
                });
            }
            #[cfg(test)]
            image_tests::part_event(&mut graph, "Part.addSlur.return", slur, &parts);
            #[cfg(test)]
            image_tests::part_event(&mut graph, "AbstractInter.setPart.entry", slur, &parts);
            graph.set_fresh_slur_part(slur.key, slur.part.id)?;
            #[cfg(test)]
            image_tests::part_event(&mut graph, "AbstractInter.setPart.return", slur, &parts);
        }
    }
    let state = State {
        slurs,
        parts,
        #[cfg(test)]
        raw_candidate_count: raw.len(),
        #[cfg(test)]
        clump_count: groups.len(),
    };
    state.validate(m, &graph, &registry)?;
    for slur in &state.slurs {
        let actual = &registry.get(slur.glyph_id)?.glyph;
        for r in &actual.runs {
            for y in r.start..r.start + r.length {
                let x = actual.left + i32::try_from(r.position).map_err(|_| "slur_glyph_x")?;
                let y = actual.top + i32::try_from(y).map_err(|_| "slur_glyph_y")?;
                if x < 0 || y < 0 || x as usize >= erased.width || y as usize >= erased.height {
                    return Err("slur_glyph_outside_retained_image".into());
                }
                erased.pixels[y as usize * erased.width + x as usize] = 255;
            }
        }
    }
    Ok(Staged {
        registry,
        graph,
        cleaned: erased,
        state,
    })
}
struct Staged {
    registry: beam_dispatch::Registry,
    graph: Graph,
    state: State,
    cleaned: GrayImage,
}
#[derive(Clone, Copy)]
enum Publication {
    Normal,
    #[cfg(test)]
    RefuseAtBoundary,
}
fn publish(
    m: &mut OwnedMusic,
    staged: &mut Option<Staged>,
    mode: Publication,
) -> Result<GrayImage, String> {
    match mode {
        Publication::Normal => {}
        #[cfg(test)]
        Publication::RefuseAtBoundary => {
            return Err("slur_injected_before_publication".into());
        }
    }
    let Staged {
        registry,
        graph,
        state,
        cleaned,
    } = staged.take().ok_or("slur_prepared_stage_required")?;
    m.owner.prepared.registry.commit_extension(registry)?;
    m.owner.graph = graph;
    m.slurs = Some(state);
    // Registry is the final fallible commit; only owned moves and projection follow.
    m.owner.prepared.report["shared_sig"] = m.owner.graph.snapshot();
    Ok(cleaned)
}
pub(crate) fn process(
    m: &mut OwnedMusic,
    erased: GrayImage,
    components: Vec<GlyphPixels>,
) -> Result<GrayImage, String> {
    #[cfg(test)]
    let before = image_tests::start(m, &erased);
    let mut staged = Some(stage(m, erased, components)?);
    #[cfg(test)]
    if let Some(before) = &before {
        let failure = publish(m, &mut staged, Publication::RefuseAtBoundary);
        assert_eq!(
            failure.err().as_deref(),
            Some("slur_injected_before_publication")
        );
        assert!(
            staged.is_some(),
            "the same fully prepared registration must survive the injected refusal"
        );
        before.assert_rollback(m);
    }
    let cleaned = publish(m, &mut staged, Publication::Normal)?;
    #[cfg(test)]
    if let Some(before) = before {
        image_tests::complete(m, &before, &cleaned);
    }
    Ok(cleaned)
}

impl State {
    fn validate(
        &self,
        m: &OwnedMusic,
        graph: &Graph,
        registry: &beam_dispatch::Registry,
    ) -> Result<(), String> {
        let mut keys = std::collections::BTreeSet::new();
        for slur in &self.slurs {
            if !keys.insert(slur.key)
                || slur.key.phase != crate::shared_sig::Phase::Curve
                || slur.key.system != slur.system.id
                || slur.part.system != slur.system.id
                || !m.owner.systems.iter().any(|s| Arc::ptr_eq(s, &slur.system))
                || !m.owner.parts.iter().any(|p| Arc::ptr_eq(p, &slur.part))
            {
                return Err("slur_registered_System_Part_owner_required".into());
            }
            let (body, removed) = graph.retained_inter(slur.key)?;
            if removed
                || body["kind"] != "SlurInter"
                || body["manual"] != false
                || body["is_tie"] != false
                || body["abnormal"] != false
                || body["curve"] != json!(slur.candidate.curve.bezier)
                || body["above"] != slur.candidate.curve.above
                || body["grade"] != slur.candidate.grade
                || body["glyph_id"] != slur.glyph_id
                || registry.get(slur.glyph_id)?.glyph != slur.candidate.glyph
            {
                return Err("slur_registered_curve_and_physical_glyph_binding_required".into());
            }
            graph.validate_fresh_slur_part(slur.key, slur.part.id)?;
            for (side, endpoint) in slur.endpoints.iter().enumerate() {
                let chord = m
                    .chords
                    .iter()
                    .find(|c| c.key == endpoint.chord)
                    .ok_or("slur_registered_actual_Chord_required")?;
                let measure = m
                    .measures
                    .get(chord.measure)
                    .and_then(|v| v.geometry.as_ref())
                    .ok_or("slur_registered_actual_Measure_required")?;
                let (head, removed) = graph.retained_inter(endpoint.head)?;
                if removed
                    || head["kind"] != "HeadInter"
                    || endpoint.head.system != slur.system.id
                    || endpoint.chord.system != slur.system.id
                    || head["staff_id"] != endpoint.staff.id
                    || endpoint.staff.system != slur.system.id
                    || !m
                        .owner
                        .staffs
                        .iter()
                        .any(|s| Arc::ptr_eq(s, &endpoint.staff))
                    || !Arc::ptr_eq(measure, &endpoint.measure)
                    || !Arc::ptr_eq(&measure.part, &slur.part)
                    || !measure
                        .part
                        .staffs
                        .iter()
                        .any(|s| Arc::ptr_eq(s, &endpoint.staff))
                    || !m
                        .measure_layout
                        .stacks
                        .iter()
                        .any(|s| Arc::ptr_eq(&s.stack, &measure.stack))
                    || graph.id(endpoint.chord) != Some(chord.id)
                    || !graph.has_relation(endpoint.chord, endpoint.head, "Containment")
                {
                    return Err(
                        "slur_registered_actual_Head_Chord_Staff_Measure_path_required".into(),
                    );
                }
                let relation = &slur.relations[side];
                if relation.side
                    != if side == 0 {
                        lookup::Side::Left
                    } else {
                        lookup::Side::Right
                    }
                    || graph.fresh_slur_relation_id(slur.key, endpoint.head, relation.side)?
                        != slur.relation_ids[side]
                    || !Arc::ptr_eq(
                        graph.fresh_slur_relation_owner(slur.relation_ids[side])?,
                        relation,
                    )
                {
                    return Err("slur_registered_same_selected_relation_required".into());
                }
            }
            let memberships = self
                .parts
                .iter()
                .filter(|p| Arc::ptr_eq(&p.part, &slur.part))
                .flat_map(|p| &p.slurs)
                .filter(|s| Arc::ptr_eq(s, slur))
                .count();
            if memberships != 1 {
                return Err("slur_registered_single_actual_Part_membership_required".into());
            }
        }
        let mut part_owners = std::collections::BTreeSet::new();
        for part in &self.parts {
            if !part_owners.insert((part.part.system, part.part.id))
                || !m.owner.parts.iter().any(|p| Arc::ptr_eq(p, &part.part))
                || part.slurs.is_empty()
                || part.slurs.iter().any(|s| {
                    !Arc::ptr_eq(&s.part, &part.part)
                        || !self
                            .slurs
                            .iter()
                            .any(|registered| Arc::ptr_eq(s, registered))
                })
            {
                return Err("slur_registered_Part_collection_owner_required".into());
            }
        }
        for system in &m.owner.systems {
            let actual = graph
                .live_inter_keys(system.id, "SlurInter")
                .into_iter()
                .filter(|k| k.phase == crate::shared_sig::Phase::Curve)
                .collect::<Vec<_>>();
            let held = self
                .slurs
                .iter()
                .filter(|s| Arc::ptr_eq(&s.system, system))
                .map(|s| s.key)
                .collect::<Vec<_>>();
            if actual != held {
                return Err("slur_registered_System_SIG_encounter_order_required".into());
            }
        }
        Ok(())
    }
}
