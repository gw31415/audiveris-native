// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use std::sync::Arc;

fn image() -> GrayImage {
    GrayImage::decode(include_bytes!(
        "../../fixtures/official-image-musicxml-parallel-voices/parallel-quarters.png"
    ))
    .unwrap()
}

#[test]
fn selected_cli_rebinds_real_voices_once_and_keeps_unselected_slots() {
    let mut export = recognize(&image()).unwrap();
    let xml = export.xml.clone();
    let chords = export.owner.chords.as_ptr();
    let measures = export.owner.measures.as_ptr();
    let old = export.owner.voices.clone();
    let pool = export
        .owner
        .owner
        .graph
        .retained_voice_observations()
        .as_array()
        .unwrap()
        .len();
    let slots = export
        .owner
        .measures
        .iter()
        .map(|m| {
            m.timing
                .slots
                .iter()
                .map(|s| s.identity.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let actual = export.reprocess_stack_rhythms(&[2, 1, 2]).unwrap();
    assert_eq!(actual["requested_stack_numbers"], json!([2, 1, 2]));
    assert_eq!(actual["processed_stack_numbers"], json!([2, 1]));
    assert_eq!(export.xml, xml);
    assert_eq!(export.owner.chords.as_ptr(), chords);
    assert_eq!(export.owner.measures.as_ptr(), measures);
    assert_eq!(
        export
            .owner
            .owner
            .graph
            .retained_voice_observations()
            .as_array()
            .unwrap()
            .len(),
        pool + old.len()
    );
    for v in &old {
        let retained = export
            .owner
            .owner
            .graph
            .retained_voice_owner(&v.identity)
            .unwrap();
        assert!(Arc::ptr_eq(&retained.identity, &v.identity));
        assert_eq!(retained.chord_ids, v.chord_ids);
        assert!(
            !export
                .owner
                .voices
                .iter()
                .any(|n| Arc::ptr_eq(&n.identity, &v.identity))
        );
    }
    for (mi, old_slots) in slots.iter().enumerate() {
        for (new, old) in export.owner.measures[mi].timing.slots.iter().zip(old_slots) {
            assert!(!Arc::ptr_eq(&new.identity, old));
        }
    }
    for event in export.owner.owner.prepared.report["events"]
        .as_array()
        .unwrap()
    {
        let mi = event["measure_number"].as_u64().unwrap() as usize - 1;
        let mut bound = event.clone();
        crate::music_voice_export::bind_event(
            &export.owner.owner.graph,
            &export.owner.voices,
            &export.owner.measures[mi],
            mi,
            &mut bound,
        )
        .unwrap();
        assert_eq!(&bound, event);
    }
    let untouched = &export.owner.measures[1].timing;
    let storage = untouched.slots.as_ptr();
    let refs = untouched
        .slots
        .iter()
        .map(|s| s.identity.clone())
        .collect::<Vec<_>>();
    let clocks = (
        untouched.expected,
        untouched.actual,
        untouched.excess,
        untouched.abnormal,
    );
    export.reprocess_stack_rhythms(&[1]).unwrap();
    let untouched = &export.owner.measures[1].timing;
    assert_eq!(untouched.slots.as_ptr(), storage);
    assert_eq!(
        (
            untouched.expected,
            untouched.actual,
            untouched.excess,
            untouched.abnormal
        ),
        clocks
    );
    for (s, r) in untouched.slots.iter().zip(&refs) {
        assert!(Arc::ptr_eq(&s.identity, r));
    }
    assert_eq!(export.xml, xml);
    for flag in [
        "musicxml_complete",
        "automatic_raster_pipeline_complete",
        "heads_complete",
        "stems_complete",
    ] {
        assert_eq!(export.owner.owner.prepared.report[flag], false);
    }
}

#[test]
fn selected_cli_late_member_failure_restores_original_storage_and_retries() {
    let mut export = recognize(&image()).unwrap();
    let xml = export.xml.clone();
    let index = export
        .owner
        .chords
        .iter()
        .position(|c| c.measure == 1)
        .unwrap();
    let member = export.owner.chords[index].members[0].clone();
    export.owner.chords[index].members.push(member);
    // Internal malformed-owner injection reaches the second-stack guard;
    // public callers cannot grant provenance by changing their report.
    export.provenance = provenance(&export.owner);
    let before = provenance(&export.owner);
    let report = export.owner.owner.prepared.report.clone();
    let graph = export.owner.owner.graph.snapshot();
    let chords = export.owner.chords.clone();
    let chord_storage = export.owner.chords.as_ptr();
    let measure_storage = export.owner.measures.as_ptr();
    let voices = export.owner.voices.clone();
    let slots = export
        .owner
        .measures
        .iter()
        .map(|m| {
            (
                m.timing.slots.as_ptr(),
                m.timing
                    .slots
                    .iter()
                    .map(|s| s.identity.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        export.reprocess_stack_rhythms(&[1, 2, 1]).unwrap_err(),
        "musicxml_actual_chord_members_required"
    );
    assert_eq!(provenance(&export.owner), before);
    assert_eq!(export.owner.owner.graph.snapshot(), graph);
    assert_eq!(export.owner.owner.prepared.report, report);
    assert_eq!(export.xml, xml);
    assert_eq!(export.owner.chords.as_ptr(), chord_storage);
    assert_eq!(export.owner.measures.as_ptr(), measure_storage);
    for (v, old) in export.owner.voices.iter().zip(&voices) {
        assert!(Arc::ptr_eq(&v.identity, &old.identity));
    }
    for (c, old) in export.owner.chords.iter().zip(&chords) {
        assert_eq!(c.members, old.members);
        assert_eq!(c.stem, old.stem);
        assert_eq!(c.bounds, old.bounds);
        assert_eq!(c.key, old.key);
        assert_eq!(c.measure, old.measure);
    }
    for (m, (storage, refs)) in export.owner.measures.iter().zip(&slots) {
        assert_eq!(m.timing.slots.as_ptr(), *storage);
        for (s, r) in m.timing.slots.iter().zip(refs) {
            assert!(Arc::ptr_eq(&s.identity, r));
        }
    }
    for bad in [vec![], vec![0], vec![1, 3]] {
        assert_eq!(
            export.reprocess_stack_rhythms(&bad).unwrap_err(),
            "music_reprocess_actual_stack_owner_required"
        );
        assert_eq!(provenance(&export.owner), before);
    }
    export.owner.chords[index].members.pop();
    export.provenance = provenance(&export.owner);
    export.reprocess_stack_rhythms(&[1, 2, 1]).unwrap();
    assert_eq!(export.xml, xml);
    let events = export.owner.owner.prepared.report["events"]
        .as_array_mut()
        .unwrap();
    let duplicate = events
        .iter()
        .find(|e| e["measure_number"] == 2)
        .unwrap()
        .clone();
    events.push(duplicate);
    export.provenance = provenance(&export.owner);
    let before_render = provenance(&export.owner);
    let before_slots = export
        .owner
        .measures
        .iter()
        .map(|m| {
            (
                m.timing.slots.as_ptr(),
                m.timing
                    .slots
                    .iter()
                    .map(|s| s.identity.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let before_voices = export.owner.voices.clone();
    assert_eq!(
        export.reprocess_stack_rhythms(&[1]).unwrap_err(),
        "musicxml_voice_unique_owned_event_required"
    );
    assert_eq!(provenance(&export.owner), before_render);
    assert_eq!(export.owner.measures.as_ptr(), measure_storage);
    assert_eq!(export.owner.chords.as_ptr(), chord_storage);
    assert_eq!(export.xml, xml);
    for (m, (storage, refs)) in export.owner.measures.iter().zip(&before_slots) {
        assert_eq!(m.timing.slots.as_ptr(), *storage);
        for (s, r) in m.timing.slots.iter().zip(refs) {
            assert!(Arc::ptr_eq(&s.identity, r));
        }
    }
    for (v, old) in export.owner.voices.iter().zip(&before_voices) {
        assert!(Arc::ptr_eq(&v.identity, &old.identity));
    }
    export.owner.owner.prepared.report["events"]
        .as_array_mut()
        .unwrap()
        .pop();
    export.provenance = provenance(&export.owner);
    export.reprocess_stack_rhythms(&[1]).unwrap();
    assert_eq!(export.xml, xml);
    export.owner.owner.prepared.report["selected_rhythm_reprocess"]["processed_stack_numbers"] =
        json!([99]);
    assert_eq!(
        export.reprocess_stack_rhythms(&[1]).unwrap_err(),
        "music_reprocess_unedited_retained_export_required"
    );
    assert_eq!(export.xml, xml);
}
