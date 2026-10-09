// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

fn treble() -> ClefAttributes {
    ClefAttributes {
        sign: "G",
        line: 2,
        octave_change: 0,
    }
}
fn bass() -> ClefAttributes {
    ClefAttributes {
        sign: "F",
        line: 4,
        octave_change: 0,
    }
}

#[test]
fn initial_void_creates_no_key_or_clef_and_explicit_zero_is_an_event() {
    let mut state = HeaderState::default();
    let empty = state.advance(&[None], &[None], true).unwrap();
    assert!(empty.keys.is_empty() && empty.clefs.is_empty());
    assert_eq!(state.keys, vec![None]);
    assert_eq!(state.clefs, vec![None]);
    let explicit = state.advance(&[Some(0)], &[Some(treble())], false).unwrap();
    assert_eq!(explicit.keys, vec![(None, 0)]);
    assert_eq!(explicit.clefs, vec![(1, treble())]);
}

#[test]
fn partial_system_header_keeps_absent_staff_key_and_clef() {
    let mut state = HeaderState::default();
    let first = state
        .advance(&[Some(1), Some(1)], &[Some(treble()), Some(bass())], true)
        .unwrap();
    assert_eq!(first.keys, vec![(None, 1)]);
    let next = state
        .advance(&[Some(1), None], &[Some(treble()), None], true)
        .unwrap();
    assert!(next.keys.is_empty() && next.clefs.is_empty());
    assert_eq!(state.keys, vec![Some(1), Some(1)]);
    assert_eq!(state.clefs, vec![Some(treble()), Some(bass())]);
}

#[test]
fn whole_measure_void_cancels_nonzero_keys_only_at_system_start() {
    let mut state = HeaderState::default();
    state
        .advance(&[Some(2), None], &[None, None], true)
        .unwrap();
    let middle = state.advance(&[None, None], &[None, None], false).unwrap();
    assert!(middle.keys.is_empty());
    assert_eq!(state.keys, vec![Some(2), None]);
    let start = state.advance(&[None, None], &[None, None], true).unwrap();
    assert_eq!(start.keys, vec![(None, 0)]);
    assert_eq!(state.keys, vec![Some(0), Some(0)]);
    assert!(
        state
            .advance(&[None, None], &[None, None], true)
            .unwrap()
            .keys
            .is_empty()
    );
}

#[test]
fn key_changes_are_numbered_for_partial_headers_then_collapse_globally() {
    let mut state = HeaderState::default();
    let first = state
        .advance(&[Some(-1), None], &[None, None], true)
        .unwrap();
    assert_eq!(first.keys, vec![(Some(1), -1)]);
    let second = state
        .advance(&[None, Some(2)], &[None, None], false)
        .unwrap();
    assert_eq!(second.keys, vec![(Some(2), 2)]);
    assert_eq!(state.keys, vec![Some(-1), Some(2)]);
    let collapse = state
        .advance(&[Some(2), Some(2)], &[None, None], false)
        .unwrap();
    assert_eq!(collapse.keys, vec![(None, 2)]);
    assert_eq!(state.keys, vec![Some(2), Some(2)]);
}

#[test]
fn clef_identity_is_semantic_and_changed_staff_is_emitted_alone() {
    let mut state = HeaderState::default();
    state
        .advance(&[None, None], &[Some(treble()), Some(bass())], true)
        .unwrap();
    let changed = state
        .advance(&[None, None], &[None, Some(treble())], false)
        .unwrap();
    assert_eq!(changed.clefs, vec![(2, treble())]);
    assert!(
        state
            .advance(&[None, None], &[None, Some(treble())], true)
            .unwrap()
            .clefs
            .is_empty()
    );
    let before = HeaderState {
        keys: state.keys.clone(),
        clefs: state.clefs.clone(),
    };
    assert!(state.advance(&[None], &[None], false).is_err());
    assert_eq!(state, before);
}

impl OwnedMusic {
    /// Compare real owners after LINKS; source PAGE export supplies header-only
    /// observations. This does not run native rhythm or claim exported notes.
    pub(crate) fn assert_der_winter_nullable_reader_and_logical_headers(&self) -> Value {
        let source: Value = serde_json::from_str(include_str!(
            "../../../fixtures/official-null-header/pitch-export-original.json"
        ))
        .unwrap();
        let before = self.owner.graph.snapshot();
        let chord_storage = self.chords.as_ptr();
        let geometries = self
            .measures
            .iter()
            .map(|m| m.geometry.as_ref().unwrap().clone())
            .collect::<Vec<_>>();
        let clocks = self
            .measures
            .iter()
            .map(|m| {
                json!({
                    "expected":m.timing.expected.map(crate::music_time::format_fraction),
                    "slots":crate::music_voices::slots_projection(&m.timing.slots)
                })
            })
            .collect::<Vec<_>>();
        let reader = note_pitch::Reader::new(self);
        let mut header_rows = Vec::new();
        let mut null_rows = Vec::new();
        let mut final_keys = Vec::new();
        for part in self
            .measures
            .iter()
            .map(|m| m.part)
            .collect::<BTreeSet<_>>()
        {
            let mut state = HeaderState::default();
            let mut previous_system = None;
            let mut rows = Vec::new();
            for (mi, measure) in self
                .measures
                .iter()
                .enumerate()
                .filter(|(_, m)| m.part == part)
            {
                let geometry = measure.geometry.as_ref().unwrap();
                // No native Slot timing is inferred here. All as-built headers
                // are collected for this source-backed header-only comparison.
                let (keys, clefs) = actual_headers(self, &before, mi, None).unwrap();
                let changes = state
                    .advance(&keys, &clefs, previous_system != Some(measure.system))
                    .unwrap();
                rows.push(json!({
                    "keys":changes.keys.iter().map(|(number,fifths)|json!({"number":number,"fifths":fifths})).collect::<Vec<_>>(),
                    "clefs":changes.clefs.iter().map(|(number,clef)| {
                        let mut row=json!({"number":if geometry.part.staffs.len()>1{Some(number)}else{None},"sign":clef.sign,"line":clef.line});
                        if clef.octave_change!=0 { row["octave_change"]=json!(clef.octave_change); }
                        row
                    }).collect::<Vec<_>>()
                }));
                for original in source["LINKS"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|row| row["system"] == measure.system)
                {
                    let matching = self
                        .chords
                        .iter()
                        .filter(|c| c.measure == mi && c.kind == ChordKind::Head)
                        .flat_map(|c| c.members.iter())
                        .filter(|head| {
                            head["bounds"] == original["bounds"]
                                && head["staff_id"] == original["staff"]
                        })
                        .collect::<Vec<_>>();
                    if matching.is_empty() {
                        continue;
                    }
                    assert_eq!(matching.len(), 1);
                    let head = matching[0];
                    let key: Key = serde_json::from_value(head["origin"].clone()).unwrap();
                    let native = self.read_actual_note_pitch(key).unwrap();
                    let staff_index = geometry
                        .part
                        .staffs
                        .iter()
                        .position(|s| head["staff_id"] == s.id)
                        .unwrap();
                    let exported = reader
                        .read_for_export(
                            head["id"].as_u64().unwrap() as usize,
                            state.keys[staff_index],
                        )
                        .unwrap();
                    let declared_plus1 = reader
                        .read_for_export(head["id"].as_u64().unwrap() as usize, Some(1))
                        .unwrap();
                    null_rows.push(json!({"head":key,"original":original,"native":native,"exported":exported,"declared_plus1":declared_plus1,
                        "absolute_pitch":crate::head_musical_pitch::absolute_pitch(None,head["pitch"].as_f64().unwrap()).unwrap()}));
                }
                previous_system = Some(measure.system);
            }
            header_rows.push(json!({"part":part,"measures":rows}));
            final_keys.push(json!({"part":part,"keys":state.keys}));
        }
        let observation = json!({"headers":header_rows,"null_physical_Staff_heads":null_rows,"final_logical_keys":final_keys,
            "stage":"ordinary native LINKS; header-only preview before rhythm",
            "qualification":"Actual Measure/Staff Arcs and live Graph are read; Key/Clef membership is as-built spatial collection until the ordered registry exists. Native rhythm and final MusicXML note equivalence are not claimed."});
        std::fs::write(
            "/tmp/real-null-pitch-headers-native-observed.json",
            serde_json::to_vec_pretty(&observation).unwrap(),
        )
        .unwrap();
        assert_eq!(null_rows.len(), 20);
        for row in &null_rows {
            let original = &row["original"];
            let native = &row["native"];
            for field in ["bounds", "step", "octave"] {
                assert_eq!(native[field], original[field]);
            }
            assert_eq!(native["staff"], original["staff"]);
            assert!(native["effective_clef_id"].is_null() && native["effective_key_id"].is_null());
            assert!(native["key_fifths"].is_null());
            assert_eq!(native["alter"], original["alter_with_declared_null_fifths"]);
            assert_eq!(row["absolute_pitch"], original["absolute_pitch"]);
            assert_eq!(
                row["declared_plus1"]["alter"],
                original["alter_with_declared_plus1_fifths"]
            );
            assert_eq!(row["exported"]["export_key_fifths"], 1);
            assert_eq!(
                row["exported"]["alter"],
                original["alter_with_declared_plus1_fifths"]
            );
            assert!(
                row["exported"]["effective_clef_id"].is_null()
                    && row["exported"]["effective_key_id"].is_null()
            );
        }
        assert_eq!(
            header_rows.len(),
            source["exported_parts"].as_array().unwrap().len()
        );
        for (native, original) in header_rows
            .iter()
            .zip(source["exported_parts"].as_array().unwrap())
        {
            let original_headers = original["measures"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| json!({"keys":m["keys"],"clefs":m["clefs"]}))
                .collect::<Vec<_>>();
            assert_eq!(native["measures"], json!(original_headers));
        }
        assert_eq!(before, self.owner.graph.snapshot());
        assert_eq!(chord_storage, self.chords.as_ptr());
        for (m, g) in self.measures.iter().zip(&geometries) {
            assert!(Arc::ptr_eq(m.geometry.as_ref().unwrap(), g));
        }
        assert_eq!(clocks,self.measures.iter().map(|m|json!({"expected":m.timing.expected.map(crate::music_time::format_fraction),"slots":crate::music_voices::slots_projection(&m.timing.slots)})).collect::<Vec<_>>());
        observation
    }
}

impl OwnedMusic {
    pub(crate) fn observe_der_winter_internal_rhythm_export(&mut self) -> Result<Value, String> {
        fn state(m: &OwnedMusic) -> Value {
            json!({
                "graph":m.owner.graph.snapshot(),
                "voices":crate::music_voices::projection(&m.voices,&m.measures),
                "voice_chords_initialized":m.voices.iter().map(|v|v.chords_initialized).collect::<Vec<_>>(),
                "measures":m.measures.iter().map(|m|json!({"system":m.system,"part":m.part,"index":m.index,
                    "expected":m.timing.expected.map(crate::music_time::format_fraction),
                    "actual":m.timing.actual.map(crate::music_time::format_fraction),
                    "excess":m.timing.excess.map(crate::music_time::format_fraction),
                    "abnormal":m.timing.abnormal,
                    "slots":crate::music_voices::slots_projection(&m.timing.slots),
                    "tuplets":m.tuplets,"stack_tuplets":m.stack_tuplets})).collect::<Vec<_>>(),
                "PAGE_ids":m.measure_layout.stacks.iter().map(|s|s.page_identity.map(|i|json!({"number":i.number,"implicit":i.pickup}))).collect::<Vec<_>>(),
                "stacks":m.measure_layout.stacks.iter().map(|s|s.timing.as_ref().map(|t|json!({
                    "expected":t.expected.map(crate::music_time::format_fraction),
                    "actual":t.actual.map(crate::music_time::format_fraction),
                    "excess":t.excess.map(crate::music_time::format_fraction),
                    "slots":crate::music_voices::slots_projection(&t.slots)}))).collect::<Vec<_>>()
            })
        }
        fn same_slots(a: &[crate::music_voices::Slot], b: &[crate::music_voices::Slot]) -> bool {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(a, b)| Arc::ptr_eq(&a.identity, &b.identity))
        }
        let before = state(self);
        let chord_storage = self.chords.as_ptr();
        let chord_ids = self
            .chords
            .iter()
            .map(|c| (c.key, c.id, c.measure, c.stem, c.members.clone()))
            .collect::<Vec<_>>();
        let measures = self
            .measures
            .iter()
            .map(|m| m.geometry.as_ref().unwrap().clone())
            .collect::<Vec<_>>();
        let stacks = self
            .measure_layout
            .stacks
            .iter()
            .map(|s| s.stack.clone())
            .collect::<Vec<_>>();
        let voices = self.voices.clone();
        let measure_slots = self
            .measures
            .iter()
            .map(|m| m.timing.slots.clone())
            .collect::<Vec<_>>();
        let stack_clocks = self
            .measure_layout
            .stacks
            .iter()
            .map(|s| s.timing.clone())
            .collect::<Vec<_>>();
        let rhythm = self.process_recognized_page_rhythm();
        let after_rhythm = state(self);
        let owners_retained = chord_storage == self.chords.as_ptr()
            && chord_ids
                == self
                    .chords
                    .iter()
                    .map(|c| (c.key, c.id, c.measure, c.stem, c.members.clone()))
                    .collect::<Vec<_>>()
            && measures.len() == self.measures.len()
            && measures
                .iter()
                .zip(&self.measures)
                .all(|(old, m)| m.geometry.as_ref().is_some_and(|g| Arc::ptr_eq(old, g)))
            && stacks.len() == self.measure_layout.stacks.len()
            && stacks
                .iter()
                .zip(&self.measure_layout.stacks)
                .all(|(old, s)| Arc::ptr_eq(old, &s.stack));
        let mut observation = json!({
            "schema":"audiveris-native.internal-real-image-continuation.v1",
            "input":"fixtures/real-rest-page/der-winter-gray.png",
            "input_is_existing_actual_LINKS_owner":true,
            "public_factory_admission_checked":false,
            "source_or_truth_used_as_recognition_input":false,
            "quality_accepted":false,"xml_written":false,
            "actual_Chord_Measure_Stack_owners_retained":owners_retained,
            "before_rhythm":before,"after_rhythm":after_rhythm
        });
        let save = |value: &Value| {
            std::fs::write(
                "/tmp/real-rest-native-internal-rhythm-export.json",
                serde_json::to_vec_pretty(value).unwrap(),
            )
            .unwrap();
        };
        let rhythm = match rhythm {
            Ok(result) => result,
            Err(error) => {
                let same_voices = voices.len() == self.voices.len()
                    && voices
                        .iter()
                        .zip(&self.voices)
                        .all(|(a, b)| Arc::ptr_eq(&a.identity, &b.identity));
                let same_measure_slots = measure_slots.len() == self.measures.len()
                    && measure_slots
                        .iter()
                        .zip(&self.measures)
                        .all(|(old, m)| same_slots(old, &m.timing.slots));
                let same_stack_slots = stack_clocks.len() == self.measure_layout.stacks.len()
                    && stack_clocks
                        .iter()
                        .zip(&self.measure_layout.stacks)
                        .all(|(old, s)| match (old, &s.timing) {
                            (None, None) => true,
                            (Some(a), Some(b)) => same_slots(&a.slots, &b.slots),
                            _ => false,
                        });
                observation["failed_stage"] = json!("ordinary_native_page_rhythm");
                observation["error"] = json!(error);
                observation["rollback_state_equal"] = json!(before == after_rhythm);
                observation["rollback_actual_Voice_Slot_identities_retained"] =
                    json!(same_voices && same_measure_slots && same_stack_slots);
                save(&observation);
                assert!(owners_retained && same_voices && same_measure_slots && same_stack_slots);
                assert_eq!(before, after_rhythm);
                return Err(error);
            }
        };
        observation["rhythm"] = rhythm;
        let before_page = state(self);
        let page_voices = self
            .voices
            .iter()
            .map(|v| v.identity.clone())
            .collect::<Vec<_>>();
        let page_slots = self
            .measures
            .iter()
            .map(|m| m.timing.slots.clone())
            .collect::<Vec<_>>();
        let page_trace_guard = crate::music_xml::page_measures::trace_tests::begin(self);
        let page_result = self.fix_recognized_page_measures();
        let page_trace = page_trace_guard.finish();
        let after_page = state(self);
        observation["after_PAGE"] = after_page.clone();
        let page = match page_result {
            Ok(page) => page,
            Err(error) => {
                observation["failed_stage"] = json!("ordinary_native_PAGE_measures");
                observation["error"] = json!(error);
                observation["PAGE_rollback_state_equal"] = json!(before_page == after_page);
                save(&observation);
                assert_eq!(before_page, after_page);
                return Err(error);
            }
        };
        observation["PAGE_measures"] = page.clone();
        observation["PAGE_duration_trace"] =
            crate::music_xml::page_measures::trace_tests::compare_ordinary(&page_trace);
        save(&observation);
        crate::music_xml::page_measures::tests::assert_der_winter_persisted_page_state(self, &page);
        assert_eq!(before_page["graph"], after_page["graph"]);
        assert_eq!(page_voices.len(), self.voices.len());
        assert!(
            page_voices
                .iter()
                .zip(&self.voices)
                .all(|(held, v)| { Arc::ptr_eq(held, &v.identity) })
        );
        assert_eq!(page_slots.len(), self.measures.len());
        assert!(
            page_slots
                .iter()
                .zip(&self.measures)
                .all(|(held, m)| { same_slots(held, &m.timing.slots) })
        );
        assert!(
            measures
                .iter()
                .zip(&self.measures)
                .all(|(held, m)| { m.geometry.as_ref().is_some_and(|g| Arc::ptr_eq(held, g)) })
        );
        assert!(
            stacks
                .iter()
                .zip(&self.measure_layout.stacks)
                .all(|(held, s)| { Arc::ptr_eq(held, &s.stack) })
        );
        assert_eq!(chord_storage, self.chords.as_ptr());
        observation["PAGE_actual_Voice_Slot_Chord_Measure_Stack_identities_retained"] = json!(true);
        // Declared generation change has the same clock values and actual owners.
        // Reclassification is still required; old source PAGE metadata survives.
        let fresh_stack = 0;
        let held_identity = self.measure_layout.stacks[fresh_stack].page_identity;
        let held_generation = std::mem::replace(
            &mut self.measure_layout.stacks[fresh_stack]
                .timing
                .as_mut()
                .unwrap()
                .generation,
            Arc::new(()),
        );
        let stale = self
            .require_recognized_page_classification(&self.measure_layout.stacks[fresh_stack])
            .unwrap_err();
        assert_eq!(stale, "musicxml_stale_PAGE_classification");
        assert_eq!(
            self.measure_layout.stacks[fresh_stack].page_identity,
            held_identity
        );
        self.measure_layout.stacks[fresh_stack]
            .timing
            .as_mut()
            .unwrap()
            .generation = held_generation;
        self.require_recognized_page_classification(&self.measure_layout.stacks[fresh_stack])
            .unwrap();
        observation["declared_new_generation_rejects_old_PAGE_without_erasing_metadata"] =
            json!(true);
        // Declared late-stage failure uses the held native Stack timing, never
        // a report replacement; earlier staged numbering must not be published.
        let before_failure = state(self);
        let previous_ids = self
            .measure_layout
            .stacks
            .iter()
            .map(|s| s.page_identity)
            .collect::<Vec<_>>();
        let previous_voices = self
            .voices
            .iter()
            .map(|v| v.identity.clone())
            .collect::<Vec<_>>();
        let previous_slots = self
            .measures
            .iter()
            .flat_map(|m| m.timing.slots.iter().map(|s| s.identity.clone()))
            .collect::<Vec<_>>();
        let late = self.measure_layout.stacks.len() - 1;
        let held_timing = self.measure_layout.stacks[late].timing.take();
        let missing_timing = self.fix_recognized_page_measures().unwrap_err();
        self.measure_layout.stacks[late].timing = held_timing;
        assert_eq!(missing_timing, "music_page_actual_rhythm_required");
        assert_eq!(
            previous_ids,
            self.measure_layout
                .stacks
                .iter()
                .map(|s| s.page_identity)
                .collect::<Vec<_>>()
        );
        assert_eq!(before_failure, state(self));
        assert!(
            previous_voices
                .iter()
                .zip(&self.voices)
                .all(|(old, v)| Arc::ptr_eq(old, &v.identity))
        );
        assert!(
            previous_slots
                .iter()
                .zip(self.measures.iter().flat_map(|m| &m.timing.slots))
                .all(|(old, s)| Arc::ptr_eq(old, &s.identity))
        );
        assert_eq!(chord_storage, self.chords.as_ptr());
        observation["declared_late_missing_Stack_timing_keeps_previous_PAGE_and_clocks"] =
            json!(true);
        let before_growth = state(self);
        observation["declared_shared_excess_growth"] =
            crate::music_xml::page_measures::trace_tests::declared_shared_growth(self);
        assert_eq!(before_growth, state(self));
        let exported = self.write_recognized_musicxml("Der Winter");
        let after_export = state(self);
        observation["export_preserved_graph_and_clocks"] = json!(after_page == after_export);
        match exported {
            Ok((xml, export)) => {
                crate::music_xml::slur_export::tests::assert_same_real_page(self, &export, &xml);
                std::fs::write("/tmp/real-rest-native-internal.musicxml", xml).unwrap();
                observation["xml_written"] = json!(true);
                observation["xml_path"] = json!("/tmp/real-rest-native-internal.musicxml");
                observation["export"] = export;
                observation["stage"] = json!("internal_native_MusicXML_written");
            }
            Err(error) => {
                observation["failed_stage"] = json!("ordinary_native_recognized_export");
                observation["error"] = json!(error);
                save(&observation);
                assert!(owners_retained);
                assert_eq!(after_page, after_export);
                return Err(error);
            }
        }
        save(&observation);
        assert!(owners_retained);
        assert_eq!(after_page, after_export);
        Ok(observation)
    }
}
