// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::music_measure_layout::{Stack, StackOwner};
use crate::music_measures::Measure;
use crate::music_voices::{Slot, StackTiming, Voice};
use std::cell::RefCell;

struct HeldStack {
    owner: StackOwner,
    measures: Vec<usize>,
    voices: Vec<(usize, Voice)>,
}
struct Trace {
    held: Vec<HeldStack>,
    frames: Vec<Value>,
}
thread_local! {
    static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
}
fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/official-page-duration-replay/observed.json"
    ))
    .unwrap()
}
fn same_slots(before: &[Slot], after: &[Slot]) {
    assert_eq!(before.len(), after.len());
    for (a, b) in before.iter().zip(after) {
        assert!(Arc::ptr_eq(&a.identity, &b.identity));
        assert_eq!(
            (a.id, a.time_offset, a.x_offset),
            (b.id, b.time_offset, b.x_offset)
        );
        assert_eq!(a.incoming_ids, b.incoming_ids);
    }
}
pub(crate) struct TraceGuard {
    active: bool,
}
impl Drop for TraceGuard {
    fn drop(&mut self) {
        if self.active {
            TRACE.with(|capture| {
                capture.borrow_mut().take();
            });
        }
    }
}
impl TraceGuard {
    pub(crate) fn finish(mut self) -> Value {
        self.active = false;
        TRACE.with(|capture| {
            json!(
                capture
                    .borrow_mut()
                    .take()
                    .expect("PAGE trace enabled")
                    .frames
            )
        })
    }
}
pub(crate) fn begin(music: &OwnedMusic) -> TraceGuard {
    let mut held = Vec::new();
    for owner in music
        .measure_layout
        .stacks
        .iter()
        .filter(|s| s.stack.system == 1 && matches!(s.stack.index, 1 | 3))
    {
        let measures = owner
            .measures
            .iter()
            .map(|geometry| {
                music
                    .owner
                    .graph
                    .require_actual_music_measure(geometry)
                    .unwrap();
                assert!(Arc::ptr_eq(&geometry.stack, &owner.stack));
                music
                    .measures
                    .iter()
                    .position(|m| {
                        m.geometry
                            .as_ref()
                            .is_some_and(|g| Arc::ptr_eq(g, geometry))
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let voices = measures
            .iter()
            .flat_map(|mi| {
                music
                    .voices
                    .iter()
                    .enumerate()
                    .filter(move |(_, v)| v.measure == *mi)
            })
            .map(|(vi, v)| {
                for id in &v.chord_ids {
                    let chord = music
                        .chords
                        .iter()
                        .find(|c| c.id == *id && c.measure == v.measure)
                        .unwrap();
                    let actual = music
                        .owner
                        .graph
                        .chord_voice_owner(chord.key, &music.voices)
                        .unwrap()
                        .unwrap();
                    assert!(Arc::ptr_eq(&actual.identity, &v.identity));
                }
                (vi, v.clone())
            })
            .collect();
        held.push(HeldStack {
            owner: owner.clone(),
            measures,
            voices,
        });
    }
    assert_eq!(held.len(), 2);
    TRACE.with(|trace| {
        assert!(trace.borrow().is_none());
        *trace.borrow_mut() = Some(Trace {
            held,
            frames: Vec::new(),
        });
    });
    TraceGuard { active: true }
}
// This hook only reads actual staged owners after the production caller has
// validated their registration. Reports never supply an owner or timing value.
pub(super) fn record(
    phase: &str,
    stack: &Arc<Stack>,
    indices: &[usize],
    voices: &[Voice],
    measures: &[Measure],
    timing: &StackTiming,
    checked: usize,
) {
    TRACE.with(|capture| {
        let mut capture=capture.borrow_mut();
        let Some(trace)=capture.as_mut() else {return};
        let Some(held)=trace.held.iter().find(|h|Arc::ptr_eq(&h.owner.stack,stack)) else {return};
        assert_eq!(indices,held.measures);
        let old=held.owner.timing.as_ref().unwrap();
        assert!(Arc::ptr_eq(&old.generation,&timing.generation));
        assert_eq!((old.expected,old.actual),(timing.expected,timing.actual));
        same_slots(&old.slots,&timing.slots);
        assert!(held.voices.iter().any(|(vi,v)|*vi==checked && Arc::ptr_eq(&v.identity,&voices[checked].identity)));
        let peers=indices.iter().zip(&held.owner.measures).map(|(&mi,geometry)| {
            let measure=&measures[mi];
            assert!(measure.geometry.as_ref().is_some_and(|g|Arc::ptr_eq(g,geometry)));
            assert!(Arc::ptr_eq(&geometry.stack,stack));
            assert_eq!((measure.timing.expected,measure.timing.actual,measure.timing.excess),(timing.expected,timing.actual,timing.excess));
            same_slots(&old.slots,&measure.timing.slots);
            let own_voices=held.voices.iter().filter(|(_,v)|v.measure==mi).map(|(vi,v)| {
                let now=&voices[*vi];
                assert!(Arc::ptr_eq(&v.identity,&now.identity));
                assert_eq!((v.id,v.measure,v.starting_staff,v.kind,v.chords_initialized,v.measure_rest_chord),(now.id,now.measure,now.starting_staff,now.kind,now.chords_initialized,now.measure_rest_chord));
                assert_eq!(v.chord_ids,now.chord_ids);
                assert_eq!(v.slots.len(),now.slots.len());
                for ((id,a),(new_id,b)) in v.slots.iter().zip(&now.slots) {
                    assert_eq!((id,a.chord_id,a.continuing),(new_id,b.chord_id,b.continuing));
                }
                json!({"id":now.id,"termination":now.termination.map(crate::music_time::format_delta),"excess":now.excess.map(crate::music_time::format_fraction)})
            }).collect::<Vec<_>>();
            json!({"part":measure.part,"expected":measure.timing.expected,"actual":measure.timing.actual,"excess":measure.timing.excess,"abnormal":measure.timing.abnormal,"voices":own_voices})
        }).collect::<Vec<_>>();
        trace.frames.push(json!({"system":stack.system,"stack_index":stack.index,"phase":phase,
            "checked_part":measures[voices[checked].measure].part,"checked_voice_id":voices[checked].id,
            "expected":timing.expected,"actual":timing.actual,"excess":timing.excess,"peers":peers,
            "actual_Stack_Measure_Voice_Slot_Arcs_and_Chord_memberships_retained":true}));
    });
}
fn voice<'a>(frame: &'a Value, part: &Value, id: &Value) -> &'a Value {
    frame["peers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["part"] == *part)
        .unwrap()["voices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == *id)
        .unwrap()
}
fn only_checked_voice_may_change(before: &Value, after: &Value) {
    for p in before["peers"].as_array().unwrap() {
        for v in p["voices"].as_array().unwrap() {
            if p["part"] != before["checked_part"] || v["id"] != before["checked_voice_id"] {
                assert_eq!(v, voice(after, &p["part"], &v["id"]));
            }
        }
    }
}
pub(crate) fn compare_ordinary(frames: &Value) -> Value {
    let expected = source();
    std::fs::write("/tmp/real-rest-PAGE-duration-trace-native.json", serde_json::to_vec_pretty(&json!({"scope":"ordinary PAGE actual staged owners; original before reload nulls qualified separately", "frames":frames,"comparison_complete":false,"quality_accepted":false})).unwrap()).unwrap();
    let mut rows = Vec::new();
    for replay in expected["replays"].as_array().unwrap() {
        let actual = frames
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| {
                f["system"] == replay["system"] && f["stack_index"] == replay["stack_index"]
            })
            .collect::<Vec<_>>();
        let source = replay["frames"].as_array().unwrap();
        assert_eq!(actual.len(), source.len());
        assert_eq!(actual.len(), 6);
        for (pair, original) in actual
            .as_chunks::<2>()
            .0
            .iter()
            .zip(source.as_chunks::<2>().0.iter())
        {
            for (&frame, original) in pair.iter().zip(original) {
                for field in [
                    "phase",
                    "checked_part",
                    "checked_voice_id",
                    "expected",
                    "actual",
                    "excess",
                ] {
                    assert_eq!(frame[field], original[field], "PAGE frame field {field}");
                }
                assert_eq!(
                    frame["actual_Stack_Measure_Voice_Slot_Arcs_and_Chord_memberships_retained"],
                    true
                );
            }
            only_checked_voice_may_change(pair[0], pair[1]);
            only_checked_voice_may_change(&original[0], &original[1]);
            let part = &pair[1]["checked_part"];
            let id = &pair[1]["checked_voice_id"];
            assert_eq!(voice(pair[1], part, id), voice(&original[1], part, id));
        }
        assert_eq!(
            actual.last().unwrap()["peers"],
            source.last().unwrap()["peers"]
        );
        rows.push(
            json!({"system":replay["system"],"stack_index":replay["stack_index"],
            "original_reloaded_before":source[0]["peers"],"native_live_before":actual[0]["peers"],
            "checked_Voice_results_and_unchanged_other_Voices_match":true}),
        );
    }
    let observation = json!({"comparisons":rows,"frames":frames,"comparison_complete":true,"quality_accepted":false,"qualification":"Original saved-PAGE reload starts with null transient termination; native live RHYTHMS already computed it. Each checked result and unaffected peers are compared without clearing native prior state."});
    std::fs::write(
        "/tmp/real-rest-PAGE-duration-trace-native.json",
        serde_json::to_vec_pretty(&observation).unwrap(),
    )
    .unwrap();
    observation
}
pub(crate) fn declared_shared_growth(music: &mut OwnedMusic) -> Value {
    let original = source();
    let control = &original["declared_shared_growth"];
    let si = music
        .measure_layout
        .stacks
        .iter()
        .position(|s| {
            s.stack.system == control["system"].as_u64().unwrap()
                && s.stack.index == control["stack_index"].as_u64().unwrap() as usize
        })
        .unwrap();
    let indices = music.measure_layout.stacks[si]
        .measures
        .iter()
        .map(|g| {
            music
                .measures
                .iter()
                .position(|m| {
                    m.geometry
                        .as_ref()
                        .is_some_and(|actual| Arc::ptr_eq(actual, g))
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 2);
    let selected = music
        .voices
        .iter()
        .position(|v| {
            indices.contains(&v.measure)
                && music.measures[v.measure].part
                    == control["before"]["checked_part"].as_u64().unwrap()
                && v.id == control["before"]["checked_voice_id"].as_u64().unwrap() as usize
        })
        .unwrap();
    let last = *music.voices[selected].chord_ids.last().unwrap();
    let chord = music
        .chords
        .iter()
        .find(|c| c.id == last && c.measure == music.voices[selected].measure)
        .unwrap();
    assert_eq!(
        json!(
            music
                .owner
                .graph
                .actual_chord_time_offset(chord.key)
                .unwrap()
        ),
        control["actual_last_chord_offset"]
    );
    assert_eq!(
        json!(music.owner.graph.chord_duration(chord.key).unwrap()),
        control["actual_last_chord_duration"]
    );
    let graph = music.owner.graph.snapshot();
    let chord_storage = music.chords.as_ptr();
    let old_voices = music.voices.clone();
    let old_measures = music.measures.clone();
    let old_stacks = music.measure_layout.stacks.clone();
    for stack in &old_stacks {
        music.require_recognized_page_classification(stack).unwrap();
    }
    let small = Some(Fraction::new(1, 64).unwrap());
    music.measure_layout.stacks[si]
        .timing
        .as_mut()
        .unwrap()
        .excess = small;
    for &mi in &indices {
        music.measures[mi].timing.excess = small;
    }
    let execution = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let trace = begin(music);
        let result = music.fix_recognized_page_measures();
        (result, trace.finish())
    }));
    let after_graph = music.owner.graph.snapshot();
    // Restore held typed state before inspecting the returned result or resuming
    // a callback assertion panic. Existing PAGE metadata is never cleared first.
    music.voices.clone_from_slice(&old_voices);
    for (m, held) in music.measures.iter_mut().zip(&old_measures) {
        m.timing = held.timing.clone();
    }
    for (stack, held) in music.measure_layout.stacks.iter_mut().zip(&old_stacks) {
        stack.timing = held.timing.clone();
        stack.page_identity = held.page_identity;
        stack.page_classification = held.page_classification.clone();
    }
    let (result, frames) = match execution {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    std::fs::write("/tmp/real-rest-PAGE-declared-shared-growth-native.json",serde_json::to_vec_pretty(&json!({"frames":frames,"PAGE_result":result,"comparison_complete":false,"declared_control_values_restored":true,"quality_accepted":false})).unwrap()).unwrap();
    let result = result.unwrap();
    assert_eq!(graph, after_graph);
    assert_eq!(music.chords.as_ptr(), chord_storage);
    for (actual, held) in music.measures.iter().zip(&old_measures) {
        assert!(Arc::ptr_eq(
            actual.geometry.as_ref().unwrap(),
            held.geometry.as_ref().unwrap()
        ));
        same_slots(&actual.timing.slots, &held.timing.slots);
    }
    for (actual, held) in music.voices.iter().zip(&old_voices) {
        assert!(Arc::ptr_eq(&actual.identity, &held.identity));
    }
    for (actual, held) in music.measure_layout.stacks.iter().zip(&old_stacks) {
        assert!(Arc::ptr_eq(&actual.stack, &held.stack));
        assert!(Arc::ptr_eq(
            &actual.timing.as_ref().unwrap().generation,
            &held.timing.as_ref().unwrap().generation
        ));
        same_slots(
            &actual.timing.as_ref().unwrap().slots,
            &held.timing.as_ref().unwrap().slots,
        );
        assert_eq!(actual.page_identity, held.page_identity);
        music
            .require_recognized_page_classification(actual)
            .unwrap();
    }
    let selected_frames = frames
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["system"] == control["system"] && f["stack_index"] == control["stack_index"])
        .collect::<Vec<_>>();
    assert_eq!(selected_frames.len(), 6);
    let source_before = &control["before"];
    let source_after = &control["after"];
    for (i, pair) in selected_frames.as_chunks::<2>().0.iter().enumerate() {
        only_checked_voice_may_change(pair[0], pair[1]);
        for (j, frame) in pair.iter().enumerate() {
            let expected_excess = if i == 2 && j == 1 {
                &source_after["excess"]
            } else {
                &source_before["excess"]
            };
            assert_eq!(&frame["excess"], expected_excess);
            for peer in frame["peers"].as_array().unwrap() {
                assert_eq!(&peer["excess"], expected_excess);
            }
        }
    }
    for (actual, expected) in selected_frames[4..]
        .iter()
        .zip([source_before, source_after])
    {
        for field in [
            "phase",
            "checked_part",
            "checked_voice_id",
            "expected",
            "actual",
            "excess",
            "peers",
        ] {
            assert_eq!(
                actual[field], expected[field],
                "declared PAGE growth {field}"
            );
        }
    }
    let observation = json!({"frames":frames,"PAGE_result":result,"declared_in_memory_control":true,"comparison_complete":true,"quality_accepted":false,
        "old_Voice_excess_kept":true,"shared_growth_visible_in_both_actual_peer_Measures_immediately":true,
        "held_Voice_timing_PAGE_metadata_and_certificates_restored":true,"actual_owner_and_relation_manual_identity_preserved":true,
        "qualification":"Original direct control checks the last excess Voice only. Native runs the ordinary PAGE loop; the first two zero-termination checks preserving1/64 are source-derived controls, not additional original runtime frames."});
    std::fs::write(
        "/tmp/real-rest-PAGE-declared-shared-growth-native.json",
        serde_json::to_vec_pretty(&observation).unwrap(),
    )
    .unwrap();
    observation
}
