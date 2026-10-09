// SPDX-License-Identifier: AGPL-3.0-or-later
//! One observation hook in the already required DerWinter target; no image decoder/test here.
use super::*;
use std::{cell::RefCell, collections::BTreeSet, sync::Weak};
#[derive(Default)]
struct Journal {
    pending: Vec<Weak<Relation>>,
    gap_input: Option<Value>,
    gap_decisions: Vec<Value>,
}
thread_local! { static JOURNAL: RefCell<Option<Journal>> = const { RefCell::new(None) }; }
pub(crate) fn begin_der_winter() {
    JOURNAL.with(|j| {
        assert!(j.borrow().is_none());
        *j.borrow_mut() = Some(Journal::default());
    });
}
pub(super) fn active() -> bool {
    JOURNAL.with(|j| j.borrow().is_some())
}
pub(super) fn pending(relation: &Arc<Relation>) {
    JOURNAL.with(|j| {
        if let Some(j) = j.borrow_mut().as_mut() {
            j.pending.push(Arc::downgrade(relation));
        }
    });
}
pub(super) fn gap_input(components: &[Arc<GlyphPixels>], points: &[fit::Pixel]) {
    JOURNAL.with(|j| {
        if let Some(j) = j.borrow_mut().as_mut() {
            assert!(j.gap_input.is_none());
            j.gap_input = Some(json!({
                "components":components.iter().map(|g|json!({
                    "owner_address":Arc::as_ptr(g)as usize,"glyph":beam_dispatch::glyph_json(g)
                })).collect::<Vec<_>>(),
                "points":points.iter().map(|p|[p.x,p.y]).collect::<Vec<_>>()
            }));
        }
    });
}
pub(crate) fn gap_result(mut decision: Value) {
    JOURNAL.with(|j| {
        if let Some(j) = j.borrow_mut().as_mut() {
            decision["actual_native_input"] = j.gap_input.take().expect("gap input must precede owner lookup");
            j.gap_decisions.push(decision);
            std::fs::write("/tmp/real-slur-gap-native-decisions.json",serde_json::to_vec_pretty(&json!({
                "scope":"Actual existing DerWinter lifecycle hook. Native inputs and live owners observed before filtering; no source coordinates or IDs selected candidates.",
                "rows":j.gap_decisions
            })).unwrap()).unwrap();
        }
    });
}

fn retained_owners(m: &OwnedMusic) -> Value {
    json!({
        "chords":m.chords.iter().map(|c|json!({"address":c as *const _ as usize,"value":format!("{c:?}")})).collect::<Vec<_>>(),
        "systems":m.owner.systems.iter().map(|a|Arc::as_ptr(a)as usize).collect::<Vec<_>>(),
        "staffs":m.owner.staffs.iter().map(|a|Arc::as_ptr(a)as usize).collect::<Vec<_>>(),
        "parts":m.owner.parts.iter().map(|a|Arc::as_ptr(a)as usize).collect::<Vec<_>>(),
        "voices":m.voices.iter().map(|v|json!({"identity":Arc::as_ptr(&v.identity)as usize,"address":v as *const _ as usize,
            "id":v.id,"measure":v.measure,"chords_initialized":v.chords_initialized,"chords":v.chord_ids,
            "termination":v.termination.map(crate::music_time::format_delta),"excess":v.excess,"slots":v.slots.iter().map(|(id,s)|json!([id,s.chord_id,s.continuing])).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "measures":m.measures.iter().map(|v|json!({"address":v as *const _ as usize,"layout":v.geometry.as_ref().map(|a|Arc::as_ptr(a)as usize),
            "part":v.geometry.as_ref().map(|a|Arc::as_ptr(&a.part)as usize),"stack":v.geometry.as_ref().map(|a|Arc::as_ptr(&a.stack)as usize),
            "expected":v.timing.expected,"actual":v.timing.actual,"abnormal":v.timing.abnormal,"excess":v.timing.excess,
            "slots":crate::music_voices::slots_projection(&v.timing.slots),"slot_owners":v.timing.slots.iter().map(|s|Arc::as_ptr(&s.identity)as usize).collect::<Vec<_>>(),
            "tuplets":v.tuplets,"stack_tuplets":v.stack_tuplets
        })).collect::<Vec<_>>(),
        "stacks":m.measure_layout.stacks.iter().map(|v|json!({"stack":Arc::as_ptr(&v.stack)as usize,
            "measures":v.measures.iter().map(|a|Arc::as_ptr(a)as usize).collect::<Vec<_>>(),
            "timing":v.timing.as_ref().map(|t|json!({"generation":Arc::as_ptr(&t.generation)as usize,"expected":t.expected,"actual":t.actual,"excess":t.excess,
                "slots":crate::music_voices::slots_projection(&t.slots),"slot_owners":t.slots.iter().map(|s|Arc::as_ptr(&s.identity)as usize).collect::<Vec<_>>()}))
        })).collect::<Vec<_>>()
    })
}
pub(super) struct Before {
    owners: Value,
    graph: Value,
    clocks: Value,
    report: Value,
    registry: Value,
    run_addresses: Vec<usize>,
    raw_address: usize,
    raw_pixels: Vec<u8>,
    erased: GrayImage,
}
pub(super) fn start(m: &OwnedMusic, erased: &GrayImage) -> Option<Before> {
    active().then(|| Before {
        owners: retained_owners(m),
        graph: m.owner.graph.slur_preservation_observation(),
        clocks: m.owner.graph.slur_chord_clock_observation(),
        report: m.owner.prepared.report.clone(),
        registry: m.owner.prepared.registry.snapshot(),
        run_addresses: (1..=m.owner.prepared.registry.len())
            .map(|id| {
                m.owner
                    .prepared
                    .registry
                    .get(id)
                    .unwrap()
                    .glyph
                    .runs
                    .as_ptr() as usize
            })
            .collect(),
        raw_address: m.owner.raw_binary.pixels.as_ptr() as usize,
        raw_pixels: m.owner.raw_binary.pixels.clone(),
        erased: GrayImage {
            width: erased.width,
            height: erased.height,
            pixels: erased.pixels.clone(),
        },
    })
}
impl Before {
    pub(super) fn assert_rollback(&self, m: &OwnedMusic) {
        assert_eq!(retained_owners(m), self.owners);
        assert_eq!(m.owner.graph.slur_preservation_observation(), self.graph);
        assert_eq!(m.owner.prepared.report, self.report);
        assert_eq!(m.owner.prepared.registry.snapshot(), self.registry);
        assert!(m.slurs.is_none());
        self.assert_prior_owners(m);
    }
    fn assert_prior_owners(&self, m: &OwnedMusic) {
        assert_eq!(retained_owners(m), self.owners);
        assert_eq!(
            m.owner.raw_binary.pixels.as_ptr() as usize,
            self.raw_address
        );
        assert_eq!(m.owner.raw_binary.pixels, self.raw_pixels);
        assert_eq!(m.owner.graph.slur_chord_clock_observation(), self.clocks);
        let registry = m.owner.prepared.registry.snapshot();
        for (i, address) in self.run_addresses.iter().enumerate() {
            assert_eq!(
                m.owner
                    .prepared
                    .registry
                    .get(i + 1)
                    .unwrap()
                    .glyph
                    .runs
                    .as_ptr() as usize,
                *address
            );
            assert_eq!(registry[i], self.registry[i]);
        }
    }
}
pub(super) fn part_event(
    graph: &mut Graph,
    hook: &'static str,
    slur: &Arc<Slur>,
    parts: &[PartSlurs],
) {
    let memberships = parts
        .iter()
        .filter(|p| Arc::ptr_eq(&p.part, &slur.part))
        .flat_map(|p| &p.slurs)
        .filter(|s| Arc::ptr_eq(s, slur))
        .count();
    graph.record_fresh_slur_membership(hook, slur.key, memberships);
}
fn normalized_native(state: &Value) -> Value {
    json!({"present":state["present"],"abnormal":state["abnormal"],"cached_part":!state["part_cached"].is_null(),"part_memberships":state["part_memberships"],
        "heads":state["head_relations"].as_array().unwrap().iter().map(|r|r["side"].clone()).collect::<Vec<_>>(),"manual":state["manual"],"tie":state["tie"]})
}
fn normalized_source(state: &Value) -> Value {
    json!({"present":state["in_actual_SIG"],"abnormal":state["abnormal"],"cached_part":!state["part_raw"].is_null(),"part_memberships":state["actual_part_memberships"].as_array().unwrap().len(),
        "heads":state["live_head_relations"].as_array().unwrap().iter().map(|r|r["side"].clone()).collect::<Vec<_>>(),"manual":state["manual"],"tie":state["tie"]})
}
pub(super) fn complete(m: &OwnedMusic, before: &Before, cleaned: &GrayImage) {
    before.assert_prior_owners(m);
    let journal = JOURNAL.with(|j| j.borrow_mut().take().unwrap());
    let state = m.slurs.as_ref().unwrap();
    let original: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-slur-fresh-lifecycle/observed.json"
    ))
    .unwrap();
    let expected = original["after_slurs"].as_array().unwrap();
    assert_eq!(state.slurs.len(), expected.len());
    let mut used_source = BTreeSet::new();
    let mut erased_pixels = BTreeSet::new();
    let mut rows = Vec::new();
    let clone = m.owner.graph.clone();
    let trace = m.owner.graph.fresh_slur_trace();
    let first_part = trace
        .iter()
        .position(|e| e.hook == "Part.addSlur.entry")
        .unwrap();
    assert!(
        trace[..first_part]
            .iter()
            .all(|e| !e.hook.starts_with("Part.") && !e.hook.starts_with("AbstractInter."))
    );
    assert!(
        trace[first_part..]
            .iter()
            .all(|e| e.hook.starts_with("Part.") || e.hook.starts_with("AbstractInter."))
    );
    let mut native_dispatch = Vec::new();
    for s in &m.owner.systems {
        for slur in &state.slurs {
            if Arc::ptr_eq(s, &slur.system) {
                native_dispatch.push(slur.key);
            }
        }
    }
    assert_eq!(
        trace
            .iter()
            .filter(|e| e.hook == "Part.addSlur.entry")
            .map(|e| e.slur)
            .collect::<Vec<_>>(),
        native_dispatch
    );
    for slur in &state.slurs {
        assert!(m.owner.systems.iter().any(|s| Arc::ptr_eq(s, &slur.system)));
        assert!(m.owner.parts.iter().any(|p| Arc::ptr_eq(p, &slur.part)));
        let part = state
            .parts
            .iter()
            .find(|p| Arc::ptr_eq(&p.part, &slur.part))
            .unwrap();
        assert_eq!(
            part.slurs.iter().filter(|s| Arc::ptr_eq(s, slur)).count(),
            1
        );
        let head_bounds = slur
            .endpoints
            .iter()
            .map(|e| m.owner.graph.retained_inter(e.head).unwrap().0["bounds"].clone())
            .collect::<Vec<_>>();
        let matching = expected
            .iter()
            .filter(|s| {
                s["system"] == slur.system.id
                    && s["part_id_raw"] == slur.part.id
                    && s["live_head_relations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|r| r["target"]["bounds_raw"].clone())
                        .collect::<Vec<_>>()
                        == head_bounds
            })
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "post-computation full endpoint join");
        let source = matching[0];
        assert!(used_source.insert(source["object"].as_str().unwrap()));
        let src = original["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["slur"]["object"] == source["object"])
            .collect::<Vec<_>>();
        let actual = trace
            .iter()
            .filter(|e| e.slur == slur.key)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), src.len());
        for (a, b) in actual.iter().zip(src) {
            assert_eq!(
                a.hook,
                b["hook"].as_str().unwrap().rsplit('/').next().unwrap()
            );
            assert_eq!(normalized_native(&a.state), normalized_source(&b["slur"]));
            assert_eq!(
                a.state["chord_clocks"], before.clocks,
                "clock must exist at callback entry"
            );
            if let Some(weak) = &a.relation {
                let owner = weak.upgrade().unwrap();
                let side = if owner.side == lookup::Side::Left {
                    0
                } else {
                    1
                };
                assert!(Arc::ptr_eq(&owner, &slur.relations[side]));
                assert!(b["event_edge_is_receiver"] == true && b["same_pending_relation"] == true);
            }
        }
        for (side, end) in slur.endpoints.iter().enumerate() {
            let owner = m
                .owner
                .graph
                .fresh_slur_relation_owner(slur.relation_ids[side])
                .unwrap();
            assert!(Arc::ptr_eq(owner, &slur.relations[side]));
            assert!(Arc::ptr_eq(
                clone
                    .fresh_slur_relation_owner(slur.relation_ids[side])
                    .unwrap(),
                owner
            ));
            assert_eq!(
                journal
                    .pending
                    .iter()
                    .filter(|w| w.upgrade().is_some_and(|r| Arc::ptr_eq(&r, owner)))
                    .count(),
                1
            );
            let same_data_different_object = Arc::new(Relation {
                side: owner.side,
                euclidean: owner.euclidean,
            });
            assert_eq!(same_data_different_object.projection(), owner.projection());
            assert!(!Arc::ptr_eq(owner, &same_data_different_object));
            assert_eq!(
                m.owner
                    .graph
                    .fresh_slur_relation_id(slur.key, end.head, owner.side)
                    .unwrap(),
                slur.relation_ids[side]
            );
            let chord = m.chords.iter().find(|c| c.key == end.chord).unwrap();
            assert!(Arc::ptr_eq(
                m.measures[chord.measure].geometry.as_ref().unwrap(),
                &end.measure
            ));
            assert!(m.owner.staffs.iter().any(|s| Arc::ptr_eq(s, &end.staff)));
        }
        let glyph = &m.owner.prepared.registry.get(slur.glyph_id).unwrap().glyph;
        for r in &glyph.runs {
            for y in r.start..r.start + r.length {
                let i = (glyph.top as usize + y) * cleaned.width + glyph.left as usize + r.position;
                assert_eq!(m.owner.raw_binary.pixels[i], 0);
                assert_eq!(cleaned.pixels[i], 255);
                erased_pixels.insert(i);
            }
        }
        rows.push(json!({"native_slur":slur.key,"original_slur_run_id":source["id"],"head_bounds":head_bounds,
            "curve":slur.candidate.curve.bezier,"original_fresh_curve":source["curve"],"relation_ids":slur.relation_ids,
            "glyph":beam_dispatch::glyph_json(glyph),"runtime_contracts_match":true}));
    }
    assert_eq!(
        journal
            .pending
            .iter()
            .filter(|w| w.strong_count() > 0)
            .count(),
        2 * state.slurs.len()
    );
    for (i, (&a, &b)) in before.erased.pixels.iter().zip(&cleaned.pixels).enumerate() {
        if !erased_pixels.contains(&i) {
            assert_eq!(a, b, "unowned cleanup pixel {i}");
        }
    }
    std::fs::write("/tmp/real-slur-native-lifecycle-v2.json",serde_json::to_vec_pretty(&json!({
        "scope":"native retained owner lifecycle against original fresh CURVES callbacks; native column/raw-gap extraction is not original Skeleton provenance; not MusicXML quality acceptance",
        "raw_eligible":state.raw_candidate_count,"clumps":state.clump_count,"pending_relations":journal.pending.len(),
        "late_publication_refusal_verified":true,"same_prepared_stage_published_after_refusal":true,"slurs":rows
    })).unwrap()).unwrap();
}
