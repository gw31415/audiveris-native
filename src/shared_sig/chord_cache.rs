// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, Phase};
use serde_json::{Value, json};

#[derive(Clone, Debug, Default)]
pub(super) struct ChordCache {
    bounds: Option<[i64; 4]>,
    head: Option<[i64; 2]>,
    tail: Option<[i64; 2]>,
    beams: Option<Vec<usize>>,
}
impl ChordCache {
    pub(super) fn snapshot(&self) -> Value {
        json!({"bounds":self.bounds,"head_location":self.head,"tail_location":self.tail,"beams_null":self.beams.is_none()})
    }
}
#[derive(Clone, Debug)]
pub(crate) struct ChordRead {
    pub stem: Option<Key>,
    pub bounds: Option<[i64; 4]>,
    pub head: Option<[i64; 2]>,
    pub tail: Option<[i64; 2]>,
}
impl Graph {
    pub(crate) fn bind_retained_chord_caches(
        &mut self,
        chords: &[(Key, Option<Key>)],
    ) -> Result<(), String> {
        if self.editing_book.is_none() {
            return Err("chord_cache_actual_editing_book_required".into());
        }
        for (key, _) in chords {
            let id = self.endpoint(*key)?;
            if key.phase != Phase::Chord || self.vertices[id - 1].data["kind"] != "HeadChordInter" {
                return Err("chord_cache_actual_head_chord_required".into());
            }
        }
        for (key, stem) in chords {
            let id = self.id(*key).unwrap();
            if self.vertices[id - 1].chord_cache.is_none() {
                self.vertices[id - 1].chord_cache = Some(ChordCache::default());
                let read = self.retained_chord_getters(*key)?;
                if read.stem != *stem {
                    return Err("chord_cache_actual_retained_stem_required".into());
                }
                if read.head.is_none() || read.tail.is_none() {
                    return Err("chord_cache_actual_note_locations_required".into());
                }
                self.retained_chord_beams(*key)?;
            }
        }
        Ok(())
    }
    pub(crate) fn retained_chord_getters(&mut self, key: Key) -> Result<ChordRead, String> {
        let id = self.endpoint(key)?;
        if self.vertices[id - 1].chord_cache.is_none() {
            return Err("retained_chord_cache_owner_required".into());
        }
        let mut heads = Vec::new();
        let mut stem = None;
        for eid in self.incident_edges(id) {
            let e = &self.edges[eid - 1];
            if e.key.source == id
                && matches!(e.key.kind.as_str(), "Containment" | "ChordStemRelation")
            {
                self.validate_chord_identity(id, e.key.target, &e.key.kind)?;
                if e.key.kind == "Containment" {
                    heads.push(&self.vertices[e.key.target - 1].data);
                } else if stem.is_none() {
                    stem = Some(e.key.target);
                }
            }
        }
        let geometry = crate::head_chord_geometry::calculate(
            &heads,
            stem.map(|s| &self.vertices[s - 1].data),
        )?;
        let stem_key = stem.map(|s| self.vertices[s - 1].key);
        let cache = self.vertices[id - 1].chord_cache.as_mut().unwrap();
        // A relation is already changed at callback entry; warm getters still
        // return the old cache until the original invalidateCache callback.
        if cache.bounds.is_none() {
            cache.bounds = geometry.bounds;
        }
        if cache.head.is_none() {
            cache.head = geometry.head_location;
            cache.tail = geometry.tail_location;
        }
        Ok(ChordRead {
            stem: stem_key,
            bounds: cache.bounds,
            head: cache.head,
            tail: cache.tail,
        })
    }
    pub(crate) fn retained_chord_beams(&mut self, key: Key) -> Result<Vec<usize>, String> {
        let id = self.endpoint(key)?;
        let cache = self.vertices[id - 1]
            .chord_cache
            .as_ref()
            .ok_or("retained_chord_cache_owner_required")?;
        if let Some(beams) = &cache.beams {
            return Ok(beams.clone());
        }
        let read = self.retained_chord_getters(key)?;
        let mut beams = Vec::new();
        if let Some(stem) = read.stem {
            let sid = self.endpoint(stem)?;
            for eid in self.incident_edges(sid) {
                let e = &self.edges[eid - 1];
                if e.key.kind == "BeamStemRelation" {
                    let bid = if e.key.source == sid {
                        e.key.target
                    } else {
                        e.key.source
                    };
                    let beam = &self.vertices[bid - 1];
                    if beam.key.system != key.system
                        || !matches!(
                            beam.data["kind"].as_str(),
                            Some("BeamInter" | "BeamHookInter")
                        )
                    {
                        return Err("retained_chord_actual_beam_required".into());
                    }
                    let median: [f64; 4] = serde_json::from_value(beam.data["median"].clone())
                        .map_err(|_| "retained_chord_beam_median_required")?;
                    if !median.iter().all(|v| v.is_finite()) || median[0] == median[2] {
                        return Err("retained_chord_beam_median_required".into());
                    }
                    let bounds = read.bounds.ok_or("retained_chord_bounds_required")?;
                    let x = (bounds[0] + bounds[2] / 2) as f64;
                    let y = read.head.ok_or("retained_chord_head_location_required")?[1] as f64;
                    let at_x = median[1]
                        + (x - median[0]) * (median[3] - median[1]) / (median[2] - median[0]);
                    beams.push((bid, (y - at_x).abs()));
                }
            }
        }
        beams.sort_by(|a, b| b.1.total_cmp(&a.1));
        let beams = beams.into_iter().map(|(id, _)| id).collect::<Vec<_>>();
        self.vertices[id - 1].chord_cache.as_mut().unwrap().beams = Some(beams.clone());
        Ok(beams)
    }
    #[cfg(test)]
    fn retained_chord_observation(&mut self, key: Key) -> Result<Value, String> {
        let id = self.endpoint(key)?;
        let before = self.vertices[id - 1]
            .chord_cache
            .as_ref()
            .ok_or("retained_chord_cache_owner_required")?
            .snapshot();
        let read = self.retained_chord_getters(key)?;
        let after = self.vertices[id - 1]
            .chord_cache
            .as_ref()
            .unwrap()
            .snapshot();
        self.retained_chord_beams(key)?;
        Ok(
            json!({"cache_before_getters":before,"getters":{"bounds":read.bounds,"head_location":read.head,"tail_location":read.tail,"stem_origin":read.stem},"cache_after_getters":after}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_retained_cache_invalidation_keeps_timed_owners_and_recomputes_getters() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-manual-chord-cache/observed.json"
        ))
        .unwrap();
        let image = crate::GrayImage::decode(include_bytes!(
            "../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let export = crate::music_xml::recognize(&image).unwrap();
        let mut g = export.owner.owner.graph.clone();
        let initial = g.snapshot();
        let nodes = initial["inters"].as_array().unwrap();
        let head = |bounds: &Value| {
            nodes
                .iter()
                .find(|v| v["kind"] == "HeadInter" && v["bounds"] == *bounds)
                .unwrap()["id"]
                .as_u64()
                .unwrap() as usize
        };
        let h = head(&oracle["head_bounds"]);
        let target = head(&oracle["target_head_bounds"]);
        let edge = |kind: &str, source: usize| {
            initial["relations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["kind"] == kind && r["source"] == source)
                .unwrap()
        };
        let cid = initial["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["kind"] == "Containment" && r["target"] == h)
            .unwrap()["source"]
            .as_u64()
            .unwrap() as usize;
        let chord = nodes.iter().find(|v| v["id"] == cid).unwrap();
        let ck: Key = serde_json::from_value(chord["origin"].clone()).unwrap();
        let new_stem = edge("HeadStemRelation", target)["target"].as_u64().unwrap() as usize;
        let sk: Key = serde_json::from_value(
            nodes.iter().find(|v| v["id"] == new_stem).unwrap()["origin"].clone(),
        )
        .unwrap();
        let old = edge("ChordStemRelation", cid);
        fn numeric(value: &Value) -> Value {
            match value {
                Value::Number(n) => json!(n.as_f64().unwrap()),
                Value::Array(a) => json!(a.iter().map(numeric).collect::<Vec<_>>()),
                Value::Object(o) => {
                    Value::Object(o.iter().map(|(k, v)| (k.clone(), numeric(v))).collect())
                }
                _ => value.clone(),
            }
        }
        let compare = |actual: &Value, expected: &Value| {
            for f in ["cache_before_getters", "cache_after_getters"] {
                assert_eq!(numeric(&actual[f]), numeric(&expected[f]), "{f}");
            }
            for f in ["bounds", "head_location", "tail_location"] {
                let a = actual["getters"][f]
                    .as_array()
                    .map(|v| v.iter().map(|n| n.as_f64().unwrap()).collect::<Vec<_>>());
                let b = expected["getters"][f]
                    .as_array()
                    .map(|v| v.iter().map(|n| n.as_f64().unwrap()).collect::<Vec<_>>());
                assert_eq!(a, b, "{f}");
            }
        };
        compare(
            &g.retained_chord_observation(ck).unwrap(),
            &oracle["before"],
        );
        g.enable_dot_callback_trace();
        g.remove_chord_identity_edges(&[old["id"].as_u64().unwrap() as usize])
            .unwrap();
        compare(
            &g.retained_chord_observation(ck).unwrap(),
            &oracle["callbacks"][2]["state"],
        );
        assert!(g.retained_chord_getters(ck).unwrap().stem.is_none());
        g.relations(
            ck.system,
            Phase::Chord,
            vec![(ck, sk, json!({"kind":"ChordStemRelation"}))],
        )
        .unwrap();
        compare(
            &g.retained_chord_observation(ck).unwrap(),
            &oracle["callbacks"][4]["state"],
        );
        assert_eq!(g.retained_chord_getters(ck).unwrap().stem, Some(sk));
        let original = oracle["callbacks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["kind"] == "ChordStemRelation")
            .collect::<Vec<_>>();
        assert_eq!(g.dot_callback_trace().len(), original.len());
        for (a, b) in g.dot_callback_trace().iter().zip(original) {
            assert_eq!(a["event"], b["event"]);
            assert_eq!(a["stage"], b["stage"]);
            assert_eq!(
                numeric(
                    &a["state"]["inters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|v| v["id"] == cid)
                        .unwrap()["retained_chord_cache"]
                ),
                numeric(&b["state"]["cache_before_getters"])
            );
        }
        let after = g.snapshot();
        let current = after["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == cid)
            .unwrap();
        for f in [
            "voice",
            "slot_id",
            "time_offset",
            "part_id",
            "measure_index",
            "member_ids",
            "dots_number",
        ] {
            assert_eq!(current[f], chord[f], "{f}");
        }
        assert_eq!(
            after["augmentation_dot_ownership"],
            initial["augmentation_dot_ownership"]
        );
        assert_eq!(g.editing_book_state().unwrap()["sheet_modified"], false);
        assert_eq!(
            g.chord_duration(ck).unwrap_err(),
            "musicxml_dotted_duration_finer_than_sixteenth_pending"
        );
        assert_eq!(oracle["after"]["duration"], "7/64");
        assert_eq!(
            oracle["after"]["getters"]["grade"],
            oracle["after"]["getters"]["head_best_grade"]
        );
        assert_eq!(
            current["grade"],
            nodes.iter().find(|v| v["id"] == h).unwrap()["contextual_grade"]
        );
    }
}
