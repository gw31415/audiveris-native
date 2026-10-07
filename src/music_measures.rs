// SPDX-License-Identifier: AGPL-3.0-or-later
//! MEASURES/head CHORDS continuation, before SYMBOLS and rhythm.
//! Translation of MeasuresBuilder.buildPartMeasures and ChordsBuilder.connectHead/
//! getStemChords/dispatchChords for single-staff parts and single-stem heads.
use crate::{
    shared_sig::{Key, Phase},
    stem_image_owned::OwnedItems,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub struct OwnedMusic {
    pub owner: OwnedItems,
    pub(crate) measures: Vec<Measure>,
    pub(crate) chords: Vec<Chord>,
    pub(crate) voices: Vec<crate::music_voices::Voice>,
    pub(crate) editing_chords: Vec<crate::shared_sig::chord_tasks::EditingHeadChord>,
}
impl OwnedMusic {
    /// Apply the compared callback to an existing ordinary retained HeadChord.
    /// This is a declared correction stage, not recognition, publication or
    /// rhythm/export admission; the caller must use a validated reprocess path.
    pub fn apply_manual_head_stem_callback(
        &mut self,
        head: Key,
        stem: Key,
        relation: Value,
    ) -> Result<Key, String> {
        let (index, key) = self.manual_head_stem_owner(head, stem)?;
        let graph = &self.owner.graph;
        let chord = &self.chords[index];
        let cache = self
            .owner
            .caches
            .get(&stem.system)
            .ok_or("manual_head_stem_actual_cache_required")?;
        let canonical = cache.registered_stem(graph, stem.local)?;
        let mut next = graph.clone();
        if next.retained_chord_stem(chord.key)? != chord.stem {
            return Err("manual_head_stem_retained_stem_mismatch".into());
        }
        next.add_owned_manual_head_stem(chord.key, head, stem, relation)?;
        let read = next.retained_chord_getters(chord.key)?;
        if read.stem != Some(stem)
            || !std::sync::Arc::ptr_eq(&canonical, &cache.registered_stem(&next, stem.local)?)
        {
            return Err("manual_head_stem_canonical_owner_changed".into());
        }
        let bounds = read
            .bounds
            .ok_or("manual_head_stem_chord_bounds_required")?;
        self.owner.graph = next;
        // Keep the same retained chord/member/measure/voice objects. The live
        // identity relation and cache getters supply this continuation's stem.
        self.chords[index].stem = read.stem;
        self.chords[index].bounds = bounds;
        Ok(key)
    }
    pub(crate) fn manual_head_stem_owner(
        &self,
        head: Key,
        stem: Key,
    ) -> Result<(usize, Key), String> {
        let graph = &self.owner.graph;
        let hid = graph
            .id(head)
            .ok_or("manual_head_stem_live_head_required")?;
        let owners = self
            .chords
            .iter()
            .enumerate()
            .filter(|(_, c)| c.members.iter().any(|h| h["id"] == hid))
            .collect::<Vec<_>>();
        if owners.len() != 1 {
            return Err("manual_head_stem_unique_retained_chord_required".into());
        }
        let (index, chord) = owners[0];
        if chord.kind != ChordKind::Head
            || chord.members.is_empty()
            || head.phase != Phase::Head
            || stem.phase != Phase::Stem
            || stem.system != chord.key.system
        {
            return Err("manual_head_stem_ordinary_retained_owner_required".into());
        }
        Ok((index, chord.key))
    }
    /// Read-only original nearest-slot selection in the admitted one-staff,
    /// zero-skew measure. Its staff fixes the reference for getXOffset(point,
    /// staff); general point-to-staff selection/interpolation remains pending.
    /// Measure number is the report's global one-based number, not a SIG ID.
    pub fn closest_slot_at(&self, measure_number: usize, x: f64) -> Result<Value, String> {
        if !x.is_finite() {
            return Err("music_slot_finite_abscissa_required".into());
        }
        let measure = measure_number
            .checked_sub(1)
            .and_then(|i| self.measures.get(i))
            .ok_or("music_slot_owned_measure_required")?;
        let slot =
            crate::music_voices::closest_slot(&measure.timing.slots, x - measure.left as f64);
        Ok(
            json!({"measure_number":measure_number,"system_id":measure.system,"part_id":measure.part,"staff_id":measure.staff,"x":x,
            "slot":slot.map(|s|json!({"id":s.id,"x_offset":s.x_offset,"time_offset":s.time_offset.map(crate::music_time::format_fraction),"incoming_chord_ids":s.incoming_ids}))}),
        )
    }
    /// Layout read from the live continuation, before meter or rhythm is set.
    pub fn layout(&self) -> Value {
        let row = |c: &Chord| json!({"id":c.id,"origin":c.key,"stem_origin":c.stem,"bounds":c.bounds,"measure_index":self.measures[c.measure].index,"member_ids":c.members.iter().map(|h|h["id"].clone()).collect::<Vec<_>>(),"duration":null,"voice":null});
        json!({"measures":self.measures.iter().map(|m|json!({"system_id":m.system,"part_id":m.part,"staff_id":m.staff,"index":m.index,"left":m.left,"right":m.right})).collect::<Vec<_>>(),"head_chords":self.chords.iter().filter(|c|c.kind == ChordKind::Head).map(|c|{let mut v=row(c);v["head_ids"]=v["member_ids"].take();v}).collect::<Vec<_>>(),"rest_chords":self.chords.iter().filter(|c|c.kind == ChordKind::Rest).map(row).collect::<Vec<_>>()})
    }
}
#[derive(Clone)]
pub(crate) struct Measure {
    pub system: u64,
    pub part: u64,
    pub staff: u64,
    pub index: usize,
    pub left: i64,
    pub right: i64,
    pub timing: crate::music_voices::MeasureTiming,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChordKind {
    Head,
    Rest,
}
#[derive(Clone, Debug)]
pub(crate) struct Chord {
    pub kind: ChordKind,
    pub key: Key,
    pub id: usize,
    pub stem: Option<Key>,
    pub members: Vec<Value>,
    pub measure: usize,
    pub bounds: [i64; 4],
}
fn center(n: &Value) -> Result<(i64, i64), String> {
    let b: [i64; 4] =
        serde_json::from_value(n["bounds"].clone()).map_err(|_| "music_bounds_required")?;
    Ok((b[0] + b[2] / 2, b[1] + b[3] / 2))
}
/// Only consumes the actual retained image owner. No selected IDs/serialized SIG input.
pub fn continue_measures(mut owner: OwnedItems) -> Result<OwnedMusic, String> {
    if owner.prepared.report["schema"] != "audiveris-native.stem-head-foundations.v1"
        || owner.prepared.report["shared_sig"] != owner.graph.snapshot()
    {
        return Err("retained_reduced_image_owner_required".into());
    }
    let initial = owner.graph.snapshot();
    let nodes = initial["inters"].as_array().ok_or("missing_music_inters")?;
    let edges = initial["relations"]
        .as_array()
        .ok_or("missing_music_relations")?;
    if nodes.iter().any(|n| {
        !matches!(
            n["kind"].as_str(),
            Some(
                "BarlineInter"
                    | "HeadInter"
                    | "StemInter"
                    | "LedgerInter"
                    | "BeamInter"
                    | "BeamHookInter"
                    | "BeamGroupInter"
            )
        )
    }) || edges.iter().any(|e| e["kind"] == "BarGroupRelation")
    {
        return Err("music_single_barlines_head_only_scope_required".into());
    }
    let grid = &owner.upstream_report["grid_ir"];
    let systems = grid["systems"].as_array().ok_or("missing_music_systems")?;
    let staffs = grid["staffs"].as_array().ok_or("missing_music_staffs")?;
    let mut graph = owner.graph.clone();
    let mut measures = Vec::new();
    let mut chords = Vec::new();
    let mut bar_rows = Vec::new();
    let mut clef_rows = Vec::new();
    let mut key_rows = Vec::new();
    let mut time_rows = Vec::new();
    for system in systems {
        let sid = system["id"].as_u64().ok_or("invalid_music_system")?;
        let parts = system["parts"].as_array().ok_or("missing_music_parts")?;
        for part in parts {
            let pid = part["id"].as_u64().ok_or("invalid_music_part")?;
            let staff_ids = part["staff_ids"]
                .as_array()
                .ok_or("missing_music_part_staffs")?;
            if staff_ids.len() != 1 {
                return Err("music_multistaff_parts_pending".into());
            }
            let staff_id = staff_ids[0].as_u64().ok_or("invalid_music_staff")?;
            let staff = staffs
                .iter()
                .find(|s| s["id"] == staff_id && s["system_id"] == sid)
                .ok_or("missing_music_staff")?;
            if staff["lines"].as_array().is_none_or(|l| l.len() != 5) {
                return Err("music_special_staff_pending".into());
            }
            let left = staff["left"]
                .as_f64()
                .ok_or("missing_staff_left")?
                .round_ties_even() as i64;
            let right = staff["right"]
                .as_f64()
                .ok_or("missing_staff_right")?
                .round_ties_even() as i64;
            let owned_bars = graph.staff_barlines(sid, staff_id);
            let mut bars = owned_bars.iter().collect::<Vec<_>>();
            bars.sort_by_key(|n| center(n).map(|p| p.0).unwrap_or(i64::MIN));
            let mut previous = left;
            let mut index = 0;
            for (i, bar) in bars.iter().enumerate() {
                let x = center(bar)?.0;
                if i > 0 && x <= center(bars[i - 1])?.0 {
                    return Err("music_duplicate_barline_pending_grouping".into());
                }
                // StaffBarlineInter.isStaffEnd: scale.toPixels(maxShift=1.0).
                let leading = (x - left).abs() <= owner.interline as i64;
                let key = Key {
                    system: sid,
                    phase: Phase::StaffBarline,
                    local: bar["id"].as_u64().ok_or("missing_bar_id")? as usize,
                };
                let id = graph.vertex(key, json!({"kind":"StaffBarlineInter","staff_id":staff_id,"bounds":bar["bounds"],"member_ids":[bar["id"]],"style":"REGULAR","leading":leading}), false)?;
                let member: Key = serde_json::from_value(bar["origin"].clone())
                    .map_err(|_| "invalid_bar_origin")?;
                graph.relations(
                    sid,
                    Phase::StaffBarline,
                    vec![(key, member, json!({"kind":"Containment"}))],
                )?;
                bar_rows.push(json!({"id":id,"origin":key,"physical_barline_id":bar["id"],"staff_id":staff_id,"x":x,"leading":leading}));
                if !leading {
                    if x <= previous {
                        return Err("invalid_music_measure_width".into());
                    }
                    index += 1;
                    measures.push(Measure {
                        system: sid,
                        part: pid,
                        staff: staff_id,
                        index,
                        left: previous,
                        right: x,
                        timing: Default::default(),
                    });
                }
                previous = x;
            }
            if bars.is_empty()
                || (center(bars[bars.len() - 1])?.0 - right).abs() > owner.interline as i64
            {
                if right <= previous {
                    return Err("invalid_trailing_music_measure".into());
                }
                measures.push(Measure {
                    system: sid,
                    part: pid,
                    staff: staff_id,
                    index: index + 1,
                    left: previous,
                    right,
                    timing: Default::default(),
                });
            }
            let header = owner.upstream_report["headers"]["staffs"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|h| h["staff_id"] == staff_id && h["system_id"] == sid)
                .ok_or("music_staff_header_required")?;
            let mut clef = header["clef"].clone();
            if clef["shape"].as_str().is_none() {
                return Err("music_native_clef_required".into());
            }
            let key = Key {
                system: sid,
                phase: Phase::MusicHeader,
                local: staff_id as usize,
            };
            clef["kind"] = json!("ClefInter");
            clef["source_header_id"] = clef["id"].clone();
            clef["staff_id"] = json!(staff_id);
            let id = graph.vertex(key, clef, false)?;
            clef_rows.push(json!({"id":id,"origin":key,"staff_id":staff_id}));
            let relations = owner.upstream_report["headers"]["systems"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|s| s["system_id"] == sid)
                .and_then(|s| s["relations"].as_array())
                .ok_or("music_header_relations_required")?;
            let imported = crate::music_headers::import_key(
                &mut graph,
                &mut owner.prepared.registry,
                header,
                key,
                relations,
            )?;
            if !imported.is_null() {
                key_rows.push(imported);
            }
            let imported = crate::music_headers::import_time(
                &mut graph,
                &mut owner.prepared.registry,
                header,
                sid,
                relations,
            )?;
            if !imported.is_null() {
                time_rows.push(imported);
            }

            let mut heads = nodes
                .iter()
                .filter(|n| {
                    n["kind"] == "HeadInter"
                        && n["origin"]["system"] == sid
                        && n["staff_id"] == staff_id
                })
                .collect::<Vec<_>>();
            heads.sort_by_key(|n| center(n).map(|p| p.0).unwrap_or(i64::MIN));
            let mut by_stem = BTreeMap::<Key, Vec<Value>>::new();
            let mut stem_order = Vec::new();
            for head in heads {
                let rels = edges
                    .iter()
                    .filter(|e| e["kind"] == "HeadStemRelation" && e["source"] == head["id"])
                    .collect::<Vec<_>>();
                if rels.len() != 1 {
                    return Err("music_shared_or_stemless_head_chords_pending".into());
                }
                let stem = nodes
                    .iter()
                    .find(|n| n["kind"] == "StemInter" && n["id"] == rels[0]["target"])
                    .ok_or("missing_chord_stem")?;
                let key: Key = serde_json::from_value(stem["origin"].clone())
                    .map_err(|_| "invalid_chord_stem_origin")?;
                // Confirm this is still the canonical registered object in the live cache.
                if owner
                    .caches
                    .get(&sid)
                    .ok_or("missing_music_cache")?
                    .registered_stem(&owner.graph, key.local)?
                    .snapshot()?["id"]
                    != stem["id"]
                {
                    return Err("stale_music_cache_stem".into());
                }
                if !by_stem.contains_key(&key) {
                    stem_order.push(key);
                }
                by_stem.entry(key).or_default().push(head.clone());
            }
            for stem in stem_order {
                let members = by_stem.remove(&stem).unwrap();
                let member_refs: Vec<_> = members.iter().collect();
                let stem_node = nodes
                    .iter()
                    .find(|n| n["id"] == graph.id(stem).unwrap())
                    .ok_or("missing_music_chord_stem")?;
                let geometry =
                    crate::head_chord_geometry::calculate(&member_refs, Some(stem_node))?;
                let bounds = geometry.bounds.ok_or("music_chord_bounds_required")?;
                let members: Vec<_> = crate::head_chord_geometry::ordered(&member_refs)?
                    .into_iter()
                    .cloned()
                    .collect();
                let cx = bounds[0] + bounds[2] / 2;
                let measure = measures
                    .iter()
                    .position(|m| {
                        m.system == sid
                            && m.part == pid
                            && m.staff == staff_id
                            && m.left < cx
                            && cx <= m.right
                    })
                    .ok_or("chord_outside_native_measure")?;
                let key = Key {
                    system: sid,
                    phase: Phase::Chord,
                    local: chords
                        .iter()
                        .filter(|c: &&Chord| c.key.system == sid)
                        .count()
                        + 1,
                };
                let id = graph.vertex(key,json!({"kind":"HeadChordInter","bounds":bounds,"staff_id":staff_id,"part_id":pid,"measure_index":measures[measure].index,"stem_id":graph.id(stem),"member_ids":members.iter().map(|h|h["id"].clone()).collect::<Vec<_>>(),"leading_note_id":geometry.leading_id,"head_location":geometry.head_location,"tail_location":geometry.tail_location,"duration":null,"voice":null}),false)?;
                let mut relations = vec![(key, stem, json!({"kind":"ChordStemRelation"}))];
                for head in &members {
                    let hkey: Key = serde_json::from_value(head["origin"].clone())
                        .map_err(|_| "invalid_chord_head_origin")?;
                    relations.push((key, hkey, json!({"kind":"Containment"})));
                }
                graph.relations(sid, Phase::Chord, relations)?;
                chords.push(Chord {
                    kind: ChordKind::Head,
                    key,
                    id,
                    stem: Some(stem),
                    members,
                    measure,
                    bounds,
                });
            }
        }
    }
    // Verify the same actual Containment/ChordStem geometry used by generic
    // overlap before committing the CHORDS continuation to its retained owner.
    let snapshot = graph.snapshot();
    crate::stem_overlap_ownership::Ownership::new(&snapshot)?;
    owner.graph = graph;
    let reduction = std::mem::take(&mut owner.prepared.report);
    owner.prepared.report = json!({"schema":"audiveris-native.music-measures-head-chords.v1","reduction":reduction,"staff_barlines":bar_rows,"registered_native_clefs":clef_rows,"registered_native_keys":key_rows,"registered_native_times":time_rows,"measures":measures.iter().map(|m|json!({"system_id":m.system,"part_id":m.part,"staff_id":m.staff,"index":m.index,"left":m.left,"right":m.right})).collect::<Vec<_>>(),"head_chords":chords.iter().map(|c|json!({"id":c.id,"origin":c.key,"stem_origin":c.stem,"bounds":c.bounds,"measure_index":measures[c.measure].index,"head_ids":c.members.iter().map(|h|h["id"].clone()).collect::<Vec<_>>(),"duration":null,"voice":null})).collect::<Vec<_>>(),"shared_sig":owner.graph.snapshot(),"scope":"native single-barline, single-staff parts, single-stem heads; canonical cache and actual SIG retained; original head union/stem tail/leading note geometry; no duration/meter assumptions","pending":["bar groups/repeats/multi-staff consistency/shared or stemless heads/courtesy measures","SYMBOLS/rest chords/LINKS/rhythm/export","full reduction/finalizeStems/cleaner/epilog","nonzero skew/real scans/special staff/full24shape pages/32sources"],"heads_complete":false,"stems_complete":false,"musicxml_complete":false,"epilog_complete":false,"automatic_raster_pipeline_complete":false});
    Ok(OwnedMusic {
        owner,
        measures,
        chords,
        voices: Vec::new(),
        editing_chords: Vec::new(),
    })
}
