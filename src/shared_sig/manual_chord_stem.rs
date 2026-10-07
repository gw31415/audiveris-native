// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use serde_json::{Value, json};

impl Graph {
    pub(crate) fn add_owned_manual_head_stem(
        &mut self,
        chord: Key,
        head: Key,
        stem: Key,
        relation: Value,
    ) -> Result<(), String> {
        let cid = self.endpoint(chord)?;
        let hid = self.endpoint(head)?;
        self.endpoint(stem)?;
        if self.editing_book.is_none()
            || self.vertices[cid - 1].chord_cache.is_none()
            || self.vertices[cid - 1].data["kind"] != "HeadChordInter"
            || chord.system != head.system
            || stem.system != head.system
            || relation["kind"] != "HeadStemRelation"
            || relation["manual"] != true
        {
            return Err("manual_head_stem_actual_retained_chord_required".into());
        }
        if self.head_chord_owner(hid) != Some(cid) {
            return Err("manual_head_stem_actual_member_owner_required".into());
        }
        // Missing reference points need retained sheet font/learned scale, not
        // a guessed anchor. This unit preserves explicitly supplied properties.
        let point: [f64; 2] = serde_json::from_value(relation["extension_point"].clone())
            .map_err(|_| "manual_head_stem_reference_point_required")?;
        if !relation["head_side"]
            .as_str()
            .is_some_and(|s| matches!(s, "LEFT" | "RIGHT"))
            || point.iter().any(|v| !v.is_finite() || v.abs() > 4_000_000.)
            || !relation["grade"]
                .as_f64()
                .is_some_and(|g| g.is_finite() && (0.0..=1.0).contains(&g))
        {
            return Err("manual_head_stem_declared_relation_properties_required".into());
        }
        let mut next = self.clone();
        next.relations_impl(
            head.system,
            Phase::Stem,
            vec![(head, stem, relation)],
            Some(chord),
        )?;
        *self = next;
        Ok(())
    }

    fn head_chord_owner(&self, head: usize) -> Option<usize> {
        self.incident_edges(head).into_iter().find_map(|eid| {
            let e = &self.edges[eid - 1];
            (e.key.kind == "Containment"
                && e.key.target == head
                && self.vertices[e.key.source - 1].data["kind"] == "HeadChordInter")
                .then_some(e.key.source)
        })
    }

    pub(super) fn replace_manual_chord_stem(
        &mut self,
        head: usize,
        stem: usize,
        retained: Key,
    ) -> Result<(), String> {
        let chord = self
            .head_chord_owner(head)
            .ok_or("manual_head_stem_actual_member_owner_required")?;
        if self.vertices[chord - 1].key != retained {
            return Err("manual_head_stem_retained_owner_changed".into());
        }
        // getStem reads the live first ChordStem, without warming geometry.
        let old = self.incident_edges(chord).into_iter().find(|eid| {
            let e = &self.edges[*eid - 1];
            e.key.source == chord && e.key.kind == "ChordStemRelation"
        });
        if old.is_some_and(|eid| self.edges[eid - 1].key.target == stem) {
            return Ok(());
        }
        if let Some(eid) = old {
            self.remove_chord_identity_edges(&[eid])?;
        }
        let target = self.vertices[stem - 1].key;
        // HeadChord.setStem does not promote the new ChordStem to manual.
        self.relations_impl(
            retained.system,
            Phase::Chord,
            vec![(retained, target, json!({"kind":"ChordStemRelation"}))],
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_manual_callback_keeps_clocks_and_nested_cache_order() {
        let original: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-manual-chord-cache/observed.json"
        ))
        .unwrap();
        let image = crate::GrayImage::decode(include_bytes!(
            "../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = crate::music_xml::recognize(&image).unwrap();
        let initial = export.owner.owner.graph.snapshot();
        let nodes = initial["inters"].as_array().unwrap();
        let head = |bounds: &Value| {
            nodes
                .iter()
                .find(|n| n["kind"] == "HeadInter" && n["bounds"] == *bounds)
                .unwrap()
        };
        let source = head(&original["head_bounds"]);
        let target = head(&original["target_head_bounds"]);
        let edges = initial["relations"].as_array().unwrap();
        let old = edges
            .iter()
            .find(|e| e["kind"] == "HeadStemRelation" && e["source"] == source["id"])
            .unwrap();
        let new_sid = edges
            .iter()
            .find(|e| e["kind"] == "HeadStemRelation" && e["source"] == target["id"])
            .unwrap()["target"]
            .clone();
        let cid = edges
            .iter()
            .find(|e| e["kind"] == "Containment" && e["target"] == source["id"])
            .unwrap()["source"]
            .clone();
        let node = |id: &Value| nodes.iter().find(|n| n["id"] == *id).unwrap();
        let hk: Key = serde_json::from_value(source["origin"].clone()).unwrap();
        let sk: Key = serde_json::from_value(node(&new_sid)["origin"].clone()).unwrap();
        let ck: Key = serde_json::from_value(node(&cid)["origin"].clone()).unwrap();
        let before = node(&cid);
        let mut cold = export.owner.owner.graph.clone();
        cold.vertices[cid.as_u64().unwrap() as usize - 1].chord_cache =
            Some(super::super::chord_cache::ChordCache::default());
        let cold_state = cold.snapshot();
        assert_eq!(
            cold.retained_chord_stem(ck).unwrap(),
            export
                .owner
                .chords
                .iter()
                .find(|c| c.key == ck)
                .unwrap()
                .stem
        );
        assert_eq!(cold.snapshot(), cold_state);
        let ci = export
            .owner
            .chords
            .iter()
            .position(|c| c.key == ck)
            .unwrap();
        let measure = export.owner.chords[ci].measure;
        let actual_chord = &export.owner.chords[ci] as *const _;
        let voices = export
            .owner
            .voices
            .iter()
            .map(|v| (v as *const _, v.chord_ids.clone()))
            .collect::<Vec<_>>();
        let measure_ptr = &export.owner.measures[measure] as *const _;
        let timing = export.owner.measures[measure].timing.slots.clone();
        let cache = &export.owner.owner.caches[&sk.system];
        let canonical = cache
            .registered_stem(&export.owner.owner.graph, sk.local)
            .unwrap();
        let mut declared = old.clone();
        declared["manual"] = json!(true);
        declared["grade"] = json!(0.7);
        declared["local_relation_id"] = json!(
            edges
                .iter()
                .filter_map(|e| e["local_relation_id"].as_u64())
                .max()
                .unwrap()
                + 1
        );
        let graph = &mut export.owner.owner.graph;
        graph.enable_dot_callback_trace();
        graph.observed_chord = Some(ck);
        assert_eq!(
            export
                .owner
                .apply_manual_head_stem_callback(hk, sk, declared)
                .unwrap(),
            ck
        );
        let graph = &export.owner.owner.graph;
        let trace = graph.dot_callback_trace();
        assert_eq!(trace.len(), original["callbacks"].as_array().unwrap().len());
        fn numeric(v: &Value) -> Value {
            match v {
                Value::Number(n) => json!(n.as_f64().unwrap()),
                Value::Array(a) => json!(a.iter().map(numeric).collect::<Vec<_>>()),
                Value::Object(o) => {
                    Value::Object(o.iter().map(|(k, v)| (k.clone(), numeric(v))).collect())
                }
                _ => v.clone(),
            }
        }
        let after = graph.snapshot();
        for (frame, oracle) in trace.iter().zip(original["callbacks"].as_array().unwrap()) {
            assert_eq!(frame["event"], oracle["event"]);
            assert_eq!(frame["stage"], oracle["stage"]);
            let eid = frame["id"].as_u64().unwrap() as usize;
            assert_eq!(graph.edges[eid - 1].key.kind, oracle["kind"]);
            let state = &frame["state"];
            let chord = state["inters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == cid)
                .unwrap();
            let observation = &frame["chord_observation"];
            for field in ["duration", "end_time"] {
                assert_eq!(observation[field], oracle["state"][field], "{field}");
            }
            for field in ["cache_before_getters", "cache_after_getters"] {
                assert_eq!(
                    numeric(&observation[field]),
                    numeric(&oracle["state"][field]),
                    "{field}"
                );
            }
            for field in ["bounds", "head_location", "tail_location"] {
                assert_eq!(
                    numeric(&observation["getters"][field]),
                    numeric(&oracle["state"]["getters"][field]),
                    "{field}"
                );
            }
            let stem = match oracle["state"]["getters"]["stem"].as_str() {
                Some("old") => node(&old["target"])["origin"].clone(),
                Some("new") => node(&new_sid)["origin"].clone(),
                None => Value::Null,
                _ => unreachable!(),
            };
            assert_eq!(observation["getters"]["stem_origin"], stem);
            for field in [
                "voice",
                "slot_id",
                "time_offset",
                "part_id",
                "measure_index",
                "member_ids",
                "dots_number",
            ] {
                assert_eq!(chord[field], before[field], "{field}");
            }
            assert_eq!(
                state["augmentation_dot_ownership"],
                initial["augmentation_dot_ownership"]
            );
            assert_eq!(
                state["editing_book"]["sheet_modified"],
                oracle["state"]["sheet_modified"]
            );
            assert_eq!(
                state["editing_book"]["book_dirty"],
                oracle["state"]["book_dirty"]
            );
            for (id, expect) in [
                (
                    source["id"].clone(),
                    oracle["state"]["head_abnormal"].clone(),
                ),
                (
                    old["target"].clone(),
                    oracle["state"]["stem_abnormal"][0].clone(),
                ),
                (new_sid.clone(), oracle["state"]["stem_abnormal"][1].clone()),
            ] {
                assert_eq!(
                    state["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|n| n["id"] == id)
                        .unwrap()["abnormal"],
                    expect
                );
            }
        }
        assert_eq!(
            trace
                .iter()
                .map(|e| e["depth"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [0, 1, 1, 1, 1, 0]
        );
        let outer = after["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| {
                e["kind"] == "HeadStemRelation"
                    && e["source"] == source["id"]
                    && e["target"] == new_sid
            })
            .unwrap();
        assert_eq!(outer["manual"], true);
        assert_eq!(outer["head_side"], old["head_side"]);
        assert_eq!(outer["extension_point"], old["extension_point"]);
        let inner = after["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "ChordStemRelation" && e["source"] == cid)
            .unwrap();
        assert_ne!(inner["manual"], true);
        assert_eq!(inner["target"], new_sid);
        let owned = &export.owner.chords[ci];
        assert_eq!(actual_chord, owned as *const _);
        assert_eq!(owned.stem, Some(sk));
        assert_eq!(json!(owned.bounds), original["after"]["getters"]["bounds"]);
        assert_eq!(measure_ptr, &export.owner.measures[measure] as *const _);
        for ((pointer, ids), v) in voices.iter().zip(&export.owner.voices) {
            assert_eq!(*pointer, v as *const _);
            assert_eq!(*ids, v.chord_ids);
        }
        assert_eq!(
            timing.len(),
            export.owner.measures[measure].timing.slots.len()
        );
        for (a, b) in timing
            .iter()
            .zip(&export.owner.measures[measure].timing.slots)
        {
            assert_eq!(a.id, b.id);
            assert_eq!(a.time_offset, b.time_offset);
            assert_eq!(a.incoming_ids, b.incoming_ids);
        }
        assert!(std::sync::Arc::ptr_eq(
            &canonical,
            &export.owner.owner.caches[&sk.system]
                .registered_stem(graph, sk.local)
                .unwrap()
        ));
        assert_eq!(graph.chord_duration(ck).unwrap(), Some("7/64".into()));
        assert_eq!(graph.chord_end_time(ck).unwrap(), Some("7/64".into()));
        assert_eq!(
            crate::music_duration::live(&graph.snapshot(), cid.as_u64().unwrap() as usize)
                .err()
                .unwrap(),
            "musicxml_dotted_duration_finer_than_sixteenth_pending"
        );
        // This correction is not yet an admitted export transaction: retained
        // provenance must reject unrelated reprocessing of the edited owner.
        let mut empty = crate::shared_sig::EditingSequence::new(vec![]);
        assert_eq!(
            export
                .apply_editing_sequence(&mut empty, crate::shared_sig::EditingOperation::Do)
                .unwrap_err(),
            "music_reprocess_unedited_retained_export_required"
        );
    }
}
