// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original HeadLinker.linkSides/CLinker.link on the native image owner.
//! Bounded ordinary-head/single-beam continuation; full reduction/epilog pending.
use crate::{
    GrayImage,
    beam_dispatch::Registry,
    head_stem_relation,
    shared_sig::{Graph, HeadStemAttachment, Key, Phase},
    stem_cache_sig::{CachedStem, Retriever},
    stem_image_head_expansion::{Context, arr, expand, line},
    stem_image_owned::OwnedItems,
};
use audiveris_template_bank::Bank;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
struct Scheduler<'a> {
    registry: &'a mut Registry,
    cache: &'a mut Retriever,
    graph: &'a mut Graph,
    all: &'a [Value],
    parts: Vec<Value>,
    context: Context<'a>,
    raster: &'a GrayImage,
    max_stem: usize,
    system: u64,
    linked: BTreeSet<(usize, bool)>,
    closed: BTreeSet<(usize, bool)>,
    attachments: Vec<HeadStemAttachment>,
    stems: BTreeMap<usize, Arc<CachedStem>>,
    budget: usize,
}
// BeamLinker.linkSides then linkStumps. All vertical linkers of a side are
// attempted, even after one succeeds. Failed sides suppress the other side
// and intermediate stumps, not already-created stems or the later head phase.
fn schedule_beam(
    builders: &[Value],
    mut attempt: impl FnMut(&Value) -> Result<Value, String>,
) -> Result<Value, String> {
    let mut observations = Vec::new();
    for side in ["LEFT", "RIGHT"] {
        let candidates = builders
            .iter()
            .filter(|b| b["side"] == side)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            continue;
        }
        let mut linked = false;
        for c in candidates {
            let result = attempt(c)?;
            linked |= result["linked"] == true;
            observations.push(result);
        }
        if !linked {
            return Ok(json!(observations));
        }
    }
    for c in builders.iter().filter(|b| b["side"].is_null()) {
        observations.push(attempt(c)?);
    }
    Ok(json!(observations))
}
impl Scheduler<'_> {
    fn beams(&mut self, builders: &[Value]) -> Result<Value, String> {
        schedule_beam(builders, |c| self.beam(c))
    }
    fn beam(&mut self, c: &Value) -> Result<Value, String> {
        let profile = c["profile"].as_u64().unwrap() as u8;
        let e = crate::stem_image_beam_builders::expand(self.registry, c, &self.context)?;
        if e.last < 0 || e.ids.is_empty() || e.relations.is_empty() {
            return Ok(json!({"builder":c,"linked":false}));
        }
        let gs = e
            .ids
            .iter()
            .map(|id| self.registry.get(*id).map(|g| g.glyph.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let p = crate::stem_checks::Parameters::new(self.context.il, self.max_stem, 0., profile)?;
        let Some(stem) = self
            .cache
            .create_selected(self.registry, &gs, self.raster, &p)?
            .stem
        else {
            return Ok(json!({"builder":c,"linked":false}));
        };
        let state = stem.snapshot()?;
        let dir = if c["vertical"] == "BOTTOM" { -1 } else { 1 };
        let Some(rel) = crate::stem_image_head_expansion::beam_relation(
            c,
            line(&state["median"])?,
            dir,
            self.context.il,
            self.context.thickness,
        )?
        else {
            return Ok(json!({"builder":c,"linked":false}));
        };
        self.cache.register(self.registry, self.graph, &stem)?;
        self.stems.insert(stem.glyph_id(), stem.clone());
        let beam = Key {
            system: self.system,
            phase: Phase::Beam,
            local: c["native_beam_id"].as_u64().unwrap() as usize,
        };
        let sk = Key {
            system: self.system,
            phase: Phase::Stem,
            local: stem.glyph_id(),
        };
        self.graph
            .relations(self.system, Phase::Beam, vec![(beam, sk, rel.clone())])?;
        for r in &e.relations {
            let i = self.corner(&r["linker"])?;
            self.add_relations(i, &stem, std::slice::from_ref(r))?;
        }
        Ok(
            json!({"builder":c,"linked":true,"last_index":e.last,"selected_glyph_ids":e.ids,"expanded_line":e.line,"stem":state,"head_relations":e.relations,"beam_relation":rel}),
        )
    }
    fn slot(&self, i: usize) -> (usize, bool) {
        (
            self.all[i]["head"]["head_id"].as_u64().unwrap() as usize,
            self.all[i]["head"]["side"] == "RIGHT",
        )
    }
    fn corner(&self, h: &Value) -> Result<usize, String> {
        self.all
            .iter()
            .position(|c| {
                c["head"]["head_bounds"] == h["head_bounds"]
                    && c["head"]["shape"] == h["shape"]
                    && c["head"]["side"] == h["side"]
                    && c["head"]["vertical"] == h["vertical"]
            })
            .ok_or("missing scheduled corner".into())
    }
    fn opposite(&self, i: usize, vertical: bool) -> Result<usize, String> {
        let mut h = self.all[i]["head"].clone();
        h["side"] = json!(if h["side"] == "LEFT" { "RIGHT" } else { "LEFT" });
        if vertical {
            h["vertical"] = json!(if h["vertical"] == "TOP" {
                "BOTTOM"
            } else {
                "TOP"
            });
        }
        self.corner(&h)
    }
    fn first_after(
        &self,
        i: usize,
        index: isize,
        profile: u8,
    ) -> Result<Option<(usize, usize)>, String> {
        let items = arr(&self.all[i]["builder_items"]["items"])?;
        let gap = self.all[i]["builder_items"]["gap_map"][profile.to_string()]
            .as_i64()
            .ok_or("missing scheduled gap")?;
        for (j, v) in items.iter().enumerate().skip((index + 1) as usize) {
            if v["gap"] == true && v["contrib"].as_i64().ok_or("missing gap contribution")? > gap {
                return Ok(None);
            }
            if v["native_source"]["kind"] == "HeadCorner" {
                return Ok(Some((j, self.corner(&v["native_source"])?)));
            }
        }
        Ok(None)
    }
    fn can_link(&self, i: usize, profile: u8, append: bool, depth: usize) -> Result<bool, String> {
        if depth > self.all.len() {
            return Err("unsupported cyclic head eligibility".into());
        }
        let min = (self.context.il as f64 * 0.85).round_ties_even() as i64;
        let b = &self.all[i]["builder_items"];
        if b["length_map"][profile.to_string()]
            .as_i64()
            .ok_or("missing scheduled length")?
            < min
        {
            return Ok(false);
        }
        let Some((j, other)) = self.first_after(i, 0, profile)? else {
            return Ok(true);
        };
        if !append && self.linked.contains(&self.slot(other)) {
            return Ok(false);
        }
        let gap = (1..j).rev().find(|&k| b["items"][k]["gap"] == true);
        let Some(gap) = gap else { return Ok(true) };
        let diag = self.opposite(i, true)?;
        let concrete = self.all[diag]["builder_items"]["length_map"]["0"]
            .as_i64()
            .ok_or("missing diagonal concrete length")?
            >= min;
        // Other head's opposite horizontal corner has the original starting vertical side.
        let other_diag = self.opposite(other, false)?;
        if concrete || self.can_link(other_diag, 0, false, depth + 1)? {
            return Ok(b["prefix_lengths"][gap]
                .as_i64()
                .ok_or("missing scheduled prefix length")?
                >= min);
        }
        Ok(true)
    }
    fn add_relations(
        &mut self,
        i: usize,
        stem: &Arc<CachedStem>,
        relations: &[Value],
    ) -> Result<(), String> {
        let state = stem.snapshot()?;
        let il = self.context.il;
        let consistency = head_stem_relation::measured_consistency(
            self.all[i]["head"]["shape"].as_str().unwrap(),
            line(&state["median"])?,
            il,
        )?;
        for entry in relations {
            if entry["relation"].is_null() {
                continue;
            }
            // HeadLinker.CLinker.link applies its checked BeamStem relations
            // as well as HeadStem relations. Keep actual cached endpoints and
            // never overwrite a prior beam-origin relation to the same stem.
            if entry["linker"]["kind"] == "Beam" {
                if entry["relation"]["kind"] != "BeamStemRelation" {
                    return Err("invalid scheduled beam support".into());
                }
                let beam = Key {
                    system: self.system,
                    phase: Phase::Beam,
                    local: entry["linker"]["native_beam_id"]
                        .as_u64()
                        .ok_or("missing scheduled beam origin")?
                        as usize,
                };
                if let Some(bid) = self.graph.id(beam) {
                    let sk = Key {
                        system: self.system,
                        phase: Phase::Stem,
                        local: stem.glyph_id(),
                    };
                    let sid = self.graph.id(sk).ok_or("missing scheduled beam stem")?;
                    if !arr(&self.graph.snapshot()["relations"])?.iter().any(|r| {
                        r["kind"] == "BeamStemRelation" && r["source"] == bid && r["target"] == sid
                    }) {
                        self.graph.relations(
                            self.system,
                            Phase::Beam,
                            vec![(beam, sk, entry["relation"].clone())],
                        )?;
                    }
                }
                continue;
            }
            if entry["linker"]["kind"] != "HeadCorner" {
                return Err("unsupported scheduled beam relation".into());
            }
            let target = self.corner(&entry["linker"])?;
            let slot = self.slot(target);
            let head = Key {
                system: self.system,
                phase: Phase::Head,
                local: {
                    let snapshot = self.graph.snapshot();
                    arr(&snapshot["inters"])?
                        .iter()
                        .find(|h| h["id"] == slot.0)
                        .ok_or("missing owned scheduled head")?["origin"]["local"]
                        .as_u64()
                        .ok_or("missing scheduled head origin")? as usize
                },
            };
            if !self
                .attachments
                .iter()
                .any(|a| a.head == head && a.stem_local == stem.glyph_id())
            {
                let mut relation = entry["relation"].clone();
                relation["consistency"] = json!(consistency);
                relation["consistency_interline"] = json!(il);
                relation["source_ratio"] =
                    json!(1. + 4. * consistency * relation["grade"].as_f64().unwrap());
                self.attachments.push(HeadStemAttachment {
                    local: self.attachments.len() + 1,
                    head,
                    stem_local: stem.glyph_id(),
                    relation,
                });
            }
            self.linked.insert(slot);
        }
        let snapshot = self.graph.snapshot();
        let records = arr(&snapshot["inters"])?
            .iter()
            .filter(|v| v["origin"]["system"] == self.system && v["origin"]["phase"] == "STEM")
            .map(|v| {
                let mut r = v.clone();
                r["id"] = v["origin"]["local"].clone();
                r
            })
            .collect::<Vec<_>>();
        self.graph
            .sync_stems(self.system, &records, &self.attachments)?;
        Ok(())
    }
    fn reuse(&self, i: usize, last: isize) -> Result<Option<Arc<CachedStem>>, String> {
        for item in arr(&self.all[i]["builder_items"]["items"])?
            .iter()
            .take((last + 1) as usize)
        {
            let h = if item["native_source"]["kind"] == "StartingHeadCorner" {
                &self.all[i]["head"]
            } else if item["native_source"]["kind"] == "HeadCorner" {
                &item["native_source"]
            } else {
                continue;
            };
            let slot = self.slot(self.corner(h)?);
            for a in &self.attachments {
                if self.graph.id(a.head) == Some(slot.0)
                    && (a.relation["head_side"] == "RIGHT") == slot.1
                {
                    return Ok(Some(
                        self.stems
                            .get(&a.stem_local)
                            .ok_or("missing live cached stem")?
                            .clone(),
                    ));
                }
            }
        }
        Ok(None)
    }
    fn link(&mut self, i: usize, profile: u8, append: bool) -> Result<bool, String> {
        if self.budget == 0 {
            return Err("unsupported head scheduler recursion budget".into());
        }
        self.budget -= 1;
        self.context.profile = profile;
        let e = expand(self.registry, &self.all[i], &self.parts[i], &self.context)?;
        self.parts[i]["theoretical_line"] = json!(e.line);
        if e.last == -1 || e.ids.is_empty() {
            return Ok(false);
        }
        let mut stem = if append { self.reuse(i, e.last)? } else { None };
        if stem.is_none() {
            let gs = e
                .ids
                .iter()
                .map(|id| self.registry.get(*id).map(|g| g.glyph.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            let p =
                crate::stem_checks::Parameters::new(self.context.il, self.max_stem, 0., profile)?;
            stem = self
                .cache
                .create_selected(self.registry, &gs, self.raster, &p)?
                .stem;
        }
        let Some(stem) = stem else { return Ok(false) };
        self.cache.register(self.registry, self.graph, &stem)?;
        self.stems.insert(stem.glyph_id(), stem.clone());
        self.add_relations(i, &stem, &e.relations)?;
        if let Some((_, next)) = self.first_after(i, e.last, profile)?
            && !self.closed.contains(&self.slot(next))
        {
            let h = &self.all[next]["head"];
            let grade = self.context.heads[&json!([h["head_bounds"], h["shape"]]).to_string()];
            for prof in 0..=if grade >= 0.3 { 3 } else { 1 } {
                if self.link(next, prof, append)? {
                    break;
                }
            }
        }
        Ok(true)
    }
    fn sides(&mut self, head: usize, profile: u8, append: bool) -> Result<bool, String> {
        let mut linked = false;
        for right in [false, true] {
            let slot = (head, right);
            if self.linked.contains(&slot) {
                linked = true;
                continue;
            }
            if !append && self.closed.contains(&slot) {
                continue;
            }
            let top = self
                .all
                .iter()
                .position(|c| {
                    c["head"]["head_id"] == head
                        && (c["head"]["side"] == "RIGHT") == right
                        && c["head"]["vertical"] == "TOP"
                })
                .ok_or("missing top corner")?;
            let bottom = self
                .all
                .iter()
                .position(|c| {
                    c["head"]["head_id"] == head
                        && (c["head"]["side"] == "RIGHT") == right
                        && c["head"]["vertical"] == "BOTTOM"
                })
                .ok_or("missing bottom corner")?;
            let a = self.can_link(top, profile, append, 0)?;
            let b = self.can_link(bottom, profile, append, 0)?;
            let selected = match (a, b) {
                (true, false) => Some(top),
                (false, true) => Some(bottom),
                (false, false) => None,
                (true, true) => {
                    if !self.parts[top]["start_stump"].is_null()
                        && self.parts[top]["start_stump"] == self.parts[bottom]["start_stump"]
                    {
                        return Ok(false);
                    }
                    Some(if right { top } else { bottom })
                }
            };
            if let Some(i) = selected
                && self.link(i, profile, append)?
            {
                linked = true;
            }
        }
        if !linked {
            let h = self
                .all
                .iter()
                .find(|c| c["head"]["head_id"] == head)
                .unwrap();
            let grade = self.context.heads
                [&json!([h["head"]["head_bounds"], h["head"]["shape"]]).to_string()];
            if grade >= 0.3 && profile < 3 {
                return self.sides(head, profile + 1, append);
            }
            for right in [false, true] {
                self.closed.insert((head, right));
            }
            return Ok(false);
        }
        for right in [false, true] {
            if self.linked.contains(&(head, right)) {
                let stem_ids = self
                    .attachments
                    .iter()
                    .filter(|a| {
                        self.graph.id(a.head) == Some(head)
                            && (a.relation["head_side"] == "RIGHT") == right
                    })
                    .map(|a| a.stem_local)
                    .collect::<Vec<_>>();
                for a in &self.attachments {
                    if self.graph.id(a.head) != Some(head) && stem_ids.contains(&a.stem_local) {
                        for side in [false, true] {
                            self.closed.insert((
                                self.graph.id(a.head).ok_or("missing linked head")?,
                                side,
                            ));
                        }
                    }
                }
            }
        }
        Ok(true)
    }
}
/// Image-derived selections register actual cache objects and original head links.
/// Beam side/intermediate builders precede head phases. Full STEMS remains pending.
pub fn continue_linking(mut owned: OwnedItems) -> Result<OwnedItems, String> {
    let report = owned.prepared.report.clone();
    if report["schema"] != "audiveris-native.stem-image-head-items.v1" {
        return Err("fresh actual image items owner required".into());
    }
    let parts = &report["head_chunks_plan"]["head_parts_plan"];
    let graph = owned.graph.snapshot();
    if arr(&graph["inters"])?.iter().any(|v| {
        matches!(
            v["kind"].as_str(),
            Some("SmallBeamInter" | "BeamHookInter" | "StemInter")
        )
    }) {
        return Err("unsupported head-only scheduler beam or existing stem".into());
    }
    let beam_builders = crate::stem_image_beam_builders::prepare(&mut owned)?;
    let raster = GrayImage::decode(&owned.upstream_output_pixels)?;
    let scale = &owned.upstream_report["stem_seeds"]["stem_scaling"]["stem_scale"];
    let thickness = scale["main"].as_u64().ok_or("missing stem thickness")? as usize;
    let max = scale["max"].as_u64().ok_or("missing stem max")? as usize;
    let heads = arr(&graph["inters"])?
        .iter()
        .filter(|h| h["kind"] == "HeadInter")
        .map(|h| {
            Ok((
                json!([h["bounds"], h["shape"]]).to_string(),
                h["grade"].as_f64().ok_or("missing head grade")?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let mut dx = BTreeMap::new();
    for r in arr(&owned.upstream_report["head_registration"]["head_seed_scale"])? {
        let shape = r["shape"].as_str().ok_or("missing DX shape")?.to_owned();
        let slot = if r["side"] == "LEFT" { 0 } else { 1 };
        dx.entry(shape).or_insert([None, None])[slot] = r["dx"].as_f64();
    }
    for v in dx.values_mut() {
        *v = crate::stem_head_reference_context::restored_dx(*v)?;
    }
    let mut systems = Vec::new();
    for (si, sys) in arr(&report["systems"])?.iter().enumerate() {
        let all = arr(&sys["cases"])?;
        let system = sys["system"].as_u64().ok_or("missing system")?;
        let context = Context {
            bank: if owned.catalog.stored_point_size == 51 {
                Bank::load_all_head_coverage("Bravura").map_err(|e| e.to_string())?
            } else {
                owned.catalog.bank()?
            },
            point_size: owned.catalog.stored_point_size,
            profile: 0,
            il: owned.interline,
            thickness,
            dx: dx.clone(),
            heads: heads.clone(),
            all,
        };
        let mut order = arr(&graph["inters"])?
            .iter()
            .filter(|h| {
                h["kind"] == "HeadInter"
                    && h["origin"]["system"] == system
                    && matches!(
                        h["shape"].as_str(),
                        Some("NOTEHEAD_BLACK" | "NOTEHEAD_VOID")
                    )
            })
            .map(|h| {
                (
                    h["id"].as_u64().unwrap() as usize,
                    h["grade"].as_f64().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        order.sort_by(|a, b| b.1.total_cmp(&a.1));
        let mut scheduler = Scheduler {
            registry: &mut owned.prepared.registry,
            cache: owned
                .caches
                .get_mut(&system)
                .ok_or("missing actual cache")?,
            graph: &mut owned.graph,
            all,
            parts: arr(&parts["systems"][si]["cases"])?.to_vec(),
            context,
            raster: &raster,
            max_stem: max,
            system,
            linked: BTreeSet::new(),
            closed: BTreeSet::new(),
            attachments: Vec::new(),
            stems: BTreeMap::new(),
            budget: 10000,
        };
        let mut unlinked = Vec::new();
        let beam_observations = scheduler.beams(&beam_builders[si])?;
        for (h, _) in &order {
            if !scheduler.sides(*h, 0, false)? {
                unlinked.push(*h)
            }
        }
        for h in unlinked {
            scheduler.sides(h, 0, true)?;
        }
        let mut result = json!({"system":system,"registered_stems":scheduler.stems.len(),"head_stem_relations":scheduler.attachments.len(),"glyphs":scheduler.stems.values().map(|s|s.snapshot()).collect::<Result<Vec<_>,_>>()?});
        if !beam_builders[si].is_empty() {
            result["beam_origin_builders"] = beam_observations;
        }
        systems.push(result);
    }
    let beam_finalizer = crate::music_beams::boost_sides(&mut owned.graph)?;
    owned.prepared.report = json!({"schema":"audiveris-native.stem-image-head-linking.v1","corner_count":report["corner_count"],"head_items_plan":report,"systems":systems,"shared_sig":owned.graph.snapshot(),"comparison_scope":"Original XML-loaded beam-free linkStems head phases1/2 plus SIGraph contextual grades; cleaner/reduction/epilog pending","heads_complete":false,"stems_complete":false,"musicxml_complete":false,"epilog_complete":false,"automatic_raster_pipeline_complete":false});
    if beam_builders.iter().any(|s| !s.is_empty()) {
        owned.prepared.report["limited_beam_finalizer"] = beam_finalizer;
        owned.prepared.report["comparison_scope"] = json!(
            "Original HEADS beam-origin builder/expand/createStem, beam-first then head phases1/2 and single-beam Finalizer.boostBeamSides, actual cache/SIG support; full STEMS/reduction/epilog parity pending"
        );
    }
    Ok(owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_side_passes_precede_stumps_and_failed_side_stops_remaining_passes() {
        // Original BeamLinker.linkSides loops both VLinkers before deciding a
        // side failed; StemsRetriever omits linkStumps on that beam afterward.
        let builders = vec![
            json!({"side":"LEFT","vertical":"TOP","n":0}),
            json!({"side":null,"n":1}),
            json!({"side":"RIGHT","n":2}),
            json!({"side":"LEFT","vertical":"BOTTOM","n":3}),
            json!({"side":null,"n":4}),
        ];
        for (failed, expected) in [
            (None, vec![0, 3, 2, 1, 4]),
            (Some("LEFT"), vec![0, 3]),
            (Some("RIGHT"), vec![0, 3, 2]),
        ] {
            let mut trace = Vec::new();
            schedule_beam(&builders, |b| {
                trace.push(b["n"].as_i64().unwrap());
                Ok(json!({"linked":b["side"].as_str()!=failed || failed.is_none()}))
            })
            .unwrap();
            assert_eq!(trace, expected);
        }
    }
}
