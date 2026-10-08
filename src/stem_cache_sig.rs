// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected createStem cache and explicit SIG vertex lifecycle. Parts selection,
//! head connections, cleaner and epilog are separate dependencies.
use crate::{
    GrayImage,
    beam_dispatch::{self, Registry},
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_builder, stem_checks, stem_glyph,
};
use audiveris_staff_removal::GlyphPixels;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct CachedStem {
    created: Arc<stem_builder::CreatedStem>,
    state: Mutex<Value>,
}
impl CachedStem {
    pub(crate) fn removal_guard(&self) -> Result<std::sync::MutexGuard<'_, Value>, String> {
        self.state.lock().map_err(|_| "poisoned_cached_stem".into())
    }
    pub fn glyph_id(&self) -> usize {
        self.created.glyph_id
    }
    pub fn snapshot(&self) -> Result<Value, String> {
        self.state
            .lock()
            .map(|v| v.clone())
            .map_err(|_| "poisoned_cached_stem".into())
    }
}
#[derive(Debug)]
pub struct Creation {
    pub glyph_id: usize,
    pub cache_hit: bool,
    pub stem: Option<Arc<CachedStem>>,
}
#[derive(Debug)]
pub struct Retriever {
    cache: stem_builder::Cache,
    stems: BTreeMap<usize, Arc<CachedStem>>,
}
impl Retriever {
    // Resolve the actual canonical Arc before the automatic reduction commits.
    // Every registered cache binding is checked against the retained live Graph.
    pub(crate) fn registered_stem(
        &self,
        graph: &Graph,
        local: usize,
    ) -> Result<Arc<CachedStem>, String> {
        self.existing(graph)?;
        let stem = self
            .stems
            .get(&local)
            .ok_or("missing_owned_foundation_stem")?;
        let state = stem.snapshot()?;
        if state["removed"] == true || state["in_sig"] != true {
            return Err("inactive_owned_foundation_stem".into());
        }
        Ok(Arc::clone(stem))
    }
    pub fn new(system: u64) -> Result<Self, String> {
        Ok(Self {
            cache: stem_builder::Cache::new(system)?,
            stems: BTreeMap::new(),
        })
    }
    pub fn create_selected(
        &mut self,
        registry: &mut Registry,
        parts: &[GlyphPixels],
        image: &GrayImage,
        parameters: &stem_checks::Parameters,
    ) -> Result<Creation, String> {
        let c = self
            .cache
            .create_selected(registry, parts, image, parameters)?;
        let stem = if let Some(created) = c.stem {
            if let Some(prior) = self.stems.get(&c.glyph_id) {
                if !Arc::ptr_eq(&prior.created, &created) {
                    return Err("changed_cached_stem_identity".into());
                }
                Some(Arc::clone(prior))
            } else {
                let mut state = created.record.clone();
                state["removed"] = json!(false);
                let stem = Arc::new(CachedStem {
                    created,
                    state: Mutex::new(state),
                });
                self.stems.insert(c.glyph_id, Arc::clone(&stem));
                Some(stem)
            }
        } else {
            None
        };
        Ok(Creation {
            glyph_id: c.glyph_id,
            cache_hit: c.cache_hit,
            stem,
        })
    }
    fn key(&self, stem: &CachedStem) -> Key {
        Key {
            system: self.cache.system_id(),
            phase: Phase::Stem,
            local: stem.glyph_id(),
        }
    }
    fn owned(&self, stem: &Arc<CachedStem>) -> Result<(), String> {
        if !self
            .stems
            .get(&stem.glyph_id())
            .is_some_and(|s| Arc::ptr_eq(s, stem))
        {
            return Err("foreign_cached_stem".into());
        }
        Ok(())
    }
    fn record(&self, registry: &Registry, stem: &CachedStem) -> Result<Value, String> {
        let glyph = &registry.get(stem.glyph_id())?.glyph;
        if beam_dispatch::glyph_json(glyph) != stem.created.record["glyph"] {
            return Err("stem_builder_cache_registry_mismatch".into());
        }
        let mut r = stem_glyph::record(
            stem.glyph_id(),
            glyph,
            stem.created.record["grade"]
                .as_f64()
                .ok_or("invalid_cached_stem_grade")?,
        )?;
        r["glyph_id"] = json!(stem.glyph_id());
        r["glyph_id_namespace"] = json!("shared physical registry");
        r["impacts"] = stem.created.record["impacts"].clone();
        r["removed"] = stem.snapshot()?["removed"].clone();
        Ok(r)
    }
    // Retain every unrelated STEM vertex/edge as well as all earlier phases.
    // Tombstones owned by this retriever remain recorded and never resurrect.
    fn existing(&self, graph: &Graph) -> Result<(Vec<Value>, Vec<HeadStemAttachment>), String> {
        let snapshot = graph.snapshot();
        let inters = snapshot["inters"]
            .as_array()
            .ok_or("invalid_cached_sig_snapshot")?;
        let by_id = inters
            .iter()
            .map(|v| (v["id"].as_u64().unwrap() as usize, v))
            .collect::<BTreeMap<_, _>>();
        let mut records = Vec::new();
        for v in inters {
            let key: Key = serde_json::from_value(v["origin"].clone())
                .map_err(|_| "invalid_cached_sig_origin")?;
            if key.system == self.cache.system_id() && key.phase == Phase::Stem {
                let mut r = v.clone();
                r["id"] = json!(key.local);
                records.push(r);
            }
        }
        for stem in self.stems.values() {
            let state = stem.snapshot()?;
            if state["in_sig"] != true {
                continue;
            }
            let key = self.key(stem);
            if graph.id(key).map(|v| json!(v)) != Some(state["id"].clone()) {
                return Err("stale_cached_stem_sig".into());
            }
            if state["removed"] == true {
                if records.iter().any(|r| r["id"] == key.local) {
                    return Err("resurrected_cached_stem_sig".into());
                }
                let mut r = stem_glyph::record(
                    key.local,
                    &beam_dispatch::glyph_from_json(&state["glyph"])?,
                    state["grade"].as_f64().unwrap(),
                )?;
                for field in ["median", "bounds", "width"] {
                    r[field] = state[field].clone();
                }
                r["removed"] = json!(true);
                r["impacts"] = state["impacts"].clone();
                r["glyph_id"] = json!(key.local);
                records.push(r);
            } else {
                let r = records
                    .iter()
                    .find(|r| r["id"] == key.local)
                    .ok_or("stale_cached_stem_sig")?;
                for field in ["glyph", "grade", "median", "bounds", "width", "impacts"] {
                    if r[field] != state[field] {
                        return Err("changed_cached_stem_sig_record".into());
                    }
                }
            }
        }
        let mut attachments = Vec::new();
        for e in snapshot["relations"]
            .as_array()
            .ok_or("invalid_cached_sig_relations")?
        {
            if e["kind"] != "HeadStemRelation" || e["system_id"] != self.cache.system_id() {
                continue;
            }
            let source = by_id
                .get(&(e["source"].as_u64().ok_or("invalid_cached_sig_endpoint")? as usize))
                .ok_or("missing_cached_sig_endpoint")?;
            let target = by_id
                .get(&(e["target"].as_u64().ok_or("invalid_cached_sig_endpoint")? as usize))
                .ok_or("missing_cached_sig_endpoint")?;
            let head: Key = serde_json::from_value(source["origin"].clone())
                .map_err(|_| "invalid_cached_sig_origin")?;
            let target: Key = serde_json::from_value(target["origin"].clone())
                .map_err(|_| "invalid_cached_sig_origin")?;
            attachments.push(HeadStemAttachment {
                local: e["local_relation_id"]
                    .as_u64()
                    .ok_or("invalid_cached_sig_relation_id")? as usize,
                head,
                stem_local: target.local,
                relation: e.clone(),
            });
        }
        Ok((records, attachments))
    }
    /// Explicit original addVertex boundary; the physical glyph is read from
    /// its registered ID and never passed through GlyphFactory again.
    /// Native shared IDs differ from JVM IDs; cached Arc identity is stable.
    pub fn register(
        &mut self,
        registry: &Registry,
        graph: &mut Graph,
        stem: &Arc<CachedStem>,
    ) -> Result<usize, String> {
        self.owned(stem)?;
        let state = stem.snapshot()?;
        if state["removed"] == true {
            return Err("removed_cached_stem_cannot_register".into());
        }
        let (mut records, attachments) = self.existing(graph)?;
        let record = self.record(registry, stem)?;
        if state["in_sig"] == true {
            return graph
                .id(self.key(stem))
                .ok_or("stale_cached_stem_sig".into());
        }
        if graph.id(self.key(stem)).is_some() {
            return Err("cached_stem_sig_local_collision".into());
        }
        records.push(record);
        let mut next = graph.clone();
        next.sync_stems(self.cache.system_id(), &records, &attachments)?;
        let id = next
            .id(self.key(stem))
            .ok_or("missing_registered_cached_stem")?;
        let mut state = stem.state.lock().map_err(|_| "poisoned_cached_stem")?;
        state["id"] = json!(id);
        state["in_sig"] = json!(true);
        *graph = next;
        Ok(id)
    }
    /// Removing a registered stem prunes incident edges and updates abnormal
    /// heads, while the retriever cache retains the same removed object.
    /// Re-registration of a removed cache is refused until separately certified.
    pub fn remove(&mut self, graph: &mut Graph, stem: &Arc<CachedStem>) -> Result<(), String> {
        self.owned(stem)?;
        let state = stem.snapshot()?;
        if state["in_sig"] != true {
            return Err("unregistered_cached_stem_remove".into());
        }
        let (mut records, mut attachments) = self.existing(graph)?;
        if state["removed"] == true {
            return Ok(());
        }
        let local = stem.glyph_id();
        let record = records
            .iter_mut()
            .find(|r| r["id"] == local)
            .ok_or("missing_registered_cached_stem")?;
        record["removed"] = json!(true);
        attachments.retain(|e| e.stem_local != local);
        let mut next = graph.clone();
        next.sync_stems(self.cache.system_id(), &records, &attachments)?;
        let mut state = stem.state.lock().map_err(|_| "poisoned_cached_stem")?;
        state["removed"] = json!(true);
        *graph = next;
        Ok(())
    }
}
