// SPDX-License-Identifier: AGPL-3.0-or-later
//! Slur lookup borrows the live recognition Head/Chord/Measure path.
//! Report projections cannot supply these owners or substitute equal scalar IDs.
use super::{Graph, Key};
use crate::{
    grid_ir::{StaffGeometry, SystemGeometry},
    music_measures::{Chord, ChordKind, Measure, OwnedMusic},
    music_slurs::lookup as kernel,
    stem_cache_sig::CachedStem,
};
use serde_json::Value;
use std::sync::Arc;

pub(crate) struct NativeHead<'m> {
    pub key: Key,
    pub body: &'m Value,
    pub bounds: [i64; 4],
    pub staff: Arc<StaffGeometry>,
}
pub(crate) struct NativeChord<'m> {
    pub owner: &'m Chord,
    pub measure: &'m Measure,
    pub bounds: [i64; 4],
    pub members: Vec<NativeHead<'m>>,
    pub stem: Option<Arc<CachedStem>>,
    pub stem_direction: i32,
    pub mirror_key: Option<Key>,
    // Each entry comes from a distinct live BeamStem relation; do not deduplicate its count.
    pub beams: Option<Vec<(Key, &'m Value)>>,
}
pub(crate) struct NativeOwners<'m> {
    pub chords: Vec<NativeChord<'m>>,
}
impl Graph {
    fn slur_members(&self, chord: Key) -> Result<Vec<(Key, &Value)>, String> {
        let id = self.endpoint(chord)?;
        if self.vertices[id - 1].data["kind"] != "HeadChordInter" {
            return Err("slur_actual_HeadChord_required".into());
        }
        let mut members = Vec::new();
        for eid in self.incident_edges(id) {
            let edge = &self.edges[eid - 1];
            if edge.key.source == id && edge.key.kind == "Containment" {
                self.validate_chord_identity(id, edge.key.target, &edge.key.kind)?;
                let head = &self.vertices[edge.key.target - 1];
                if head.removed || head.data["kind"] != "HeadInter" {
                    return Err("slur_actual_live_Head_required".into());
                }
                members.push((head.key, &head.data));
            }
        }
        let bounds = |h: &Value| -> Result<[i64; 4], String> {
            serde_json::from_value(h["bounds"].clone())
                .map_err(|_| "slur_actual_head_bounds_required".into())
        };
        for (_, body) in &members {
            bounds(body)?;
        }
        members.sort_by_key(|(_, body)| {
            let b = bounds(body).unwrap();
            std::cmp::Reverse(b[1] + b[3] / 2)
        });
        Ok(members)
    }
    fn slur_mirror_chord(&self, chord: Key) -> Result<Option<Key>, String> {
        for (key, _) in self.slur_members(chord)? {
            let head = self.endpoint(key)?;
            for eid in self.incident_edges(head) {
                let edge = &self.edges[eid - 1];
                if edge.key.kind != "MirrorRelation" {
                    continue;
                }
                let other = if edge.key.source == head {
                    edge.key.target
                } else {
                    edge.key.source
                };
                let mirror = &self.vertices[other - 1];
                if mirror.removed
                    || mirror.key.system != chord.system
                    || mirror.data["kind"] != "HeadInter"
                {
                    return Err("slur_actual_live_mirror_Head_required".into());
                }
                for mid in self.incident_edges(other) {
                    let membership = &self.edges[mid - 1];
                    if membership.key.target == other && membership.key.kind == "Containment" {
                        let ensemble = &self.vertices[membership.key.source - 1];
                        if ensemble.removed || ensemble.data["kind"] != "HeadChordInter" {
                            return Err("slur_actual_mirror_HeadChord_required".into());
                        }
                        return Ok(Some(ensemble.key));
                    }
                }
                return Err("slur_actual_mirror_HeadChord_membership_required".into());
            }
        }
        Ok(None)
    }
    fn slur_beam_stem_owners(&self, stem: Key) -> Result<Vec<(Key, &Value)>, String> {
        let sid = self.endpoint(stem)?;
        let mut beams = Vec::new();
        for eid in self.incident_edges(sid) {
            let edge = &self.edges[eid - 1];
            if edge.key.kind != "BeamStemRelation" {
                continue;
            }
            let bid = if edge.key.source == sid {
                edge.key.target
            } else {
                edge.key.source
            };
            let beam = &self.vertices[bid - 1];
            if beam.removed
                || beam.key.system != stem.system
                || !matches!(
                    beam.data["kind"].as_str(),
                    Some("BeamInter" | "BeamHookInter")
                )
            {
                return Err("slur_actual_live_Beam_required".into());
            }
            beams.push((beam.key, &beam.data));
        }
        Ok(beams)
    }
}
impl<'m> NativeOwners<'m> {
    pub(crate) fn borrow(
        music: &'m OwnedMusic,
        graph: &'m mut Graph,
        system: &Arc<SystemGeometry>,
    ) -> Result<Self, String> {
        if !music.owner.systems.iter().any(|s| Arc::ptr_eq(s, system)) {
            return Err("slur_actual_System_owner_required".into());
        }
        let keys = graph.live_inter_keys(system.id, "HeadChordInter");
        for key in &keys {
            if music
                .chords
                .iter()
                .filter(|c| c.kind == ChordKind::Head && c.key == *key)
                .count()
                != 1
            {
                return Err("slur_actual_retained_HeadChord_required".into());
            }
        }
        // Existing live geometry and warm retained caches are both owned by this Graph.
        // Chord/Measure/Staff/Stem references stay borrowed; no report reconstruction.
        let reads = keys
            .iter()
            .map(|key| {
                graph
                    .retained_or_live_chord_getters(*key)
                    .map(|r| (*key, r))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let requested_stems = reads
            .iter()
            .filter_map(|(_, read)| read.stem.map(|key| key.local))
            .collect::<Vec<_>>();
        let stems = if requested_stems.is_empty() {
            Vec::new()
        } else {
            music
                .owner
                .caches
                .get(&system.id)
                .ok_or("slur_actual_Stem_cache_required")?
                .registered_stems(graph, &requested_stems)?
        };
        let mut stems = stems.into_iter();
        let mut chords = Vec::new();
        for (key, read) in reads {
            let owner = music
                .chords
                .iter()
                .find(|c| c.kind == ChordKind::Head && c.key == key)
                .unwrap();
            let measure = music
                .measures
                .get(owner.measure)
                .ok_or("slur_actual_Measure_required")?;
            let layout = measure
                .geometry
                .as_ref()
                .ok_or("slur_actual_MeasureLayout_required")?;
            if layout.part.system != system.id
                || !music
                    .owner
                    .parts
                    .iter()
                    .any(|p| Arc::ptr_eq(p, &layout.part))
            {
                return Err("slur_actual_Part_owner_required".into());
            }
            let mut members = Vec::new();
            for (key, body) in graph.slur_members(owner.key)? {
                let staff_id = body["staff_id"]
                    .as_u64()
                    .ok_or("slur_actual_Head_Staff_required")?;
                let staff = music
                    .owner
                    .staffs
                    .iter()
                    .find(|s| s.system == system.id && s.id == staff_id)
                    .ok_or("slur_actual_Staff_owner_required")?;
                if !system.staffs.iter().any(|s| Arc::ptr_eq(s, staff))
                    || !layout.part.staffs.iter().any(|s| Arc::ptr_eq(s, staff))
                {
                    return Err("slur_actual_System_Part_Staff_membership_required".into());
                }
                let bounds = serde_json::from_value(body["bounds"].clone())
                    .map_err(|_| "slur_actual_Head_bounds_required")?;
                members.push(NativeHead {
                    key,
                    body,
                    bounds,
                    staff: Arc::clone(staff),
                });
            }
            if members.is_empty() {
                return Err("slur_nonempty_actual_HeadChord_required".into());
            }
            let stem = if read.stem.is_some() {
                Some(stems.next().ok_or("slur_actual_Stem_batch_required")?)
            } else {
                None
            };
            if read.stem != owner.stem {
                return Err("slur_retained_Chord_Stem_changed".into());
            }
            let stem_direction = if stem.is_some() {
                let head = read
                    .head
                    .ok_or("slur_actual_chord_head_location_required")?;
                let tail = read
                    .tail
                    .ok_or("slur_actual_chord_tail_location_required")?;
                (tail[1] - head[1]).signum() as i32
            } else {
                0
            };
            let beams = read
                .stem
                .map(|s| graph.slur_beam_stem_owners(s))
                .transpose()?;
            chords.push(NativeChord {
                owner,
                measure,
                bounds: read.bounds.ok_or("slur_actual_Chord_bounds_required")?,
                members,
                stem,
                stem_direction,
                mirror_key: graph.slur_mirror_chord(key)?,
                beams,
            });
        }
        Ok(Self { chords })
    }
}
impl<'m> kernel::ChordOwners for NativeOwners<'m> {
    type Chord = NativeChord<'m>;
    type Head = NativeHead<'m>;
    fn chord_bounds(&self, c: &Self::Chord) -> Result<kernel::Rect, String> {
        Ok(c.bounds)
    }
    fn head_bounds(&self, h: &Self::Head) -> Result<kernel::Rect, String> {
        Ok(h.bounds)
    }
    fn notes<'a>(&'a self, c: &'a Self::Chord) -> Result<Vec<&'a Self::Head>, String> {
        Ok(c.members.iter().collect())
    }
}
impl<'m> kernel::PairOwners for NativeOwners<'m> {
    type Staff = Arc<StaffGeometry>;
    fn same_chord(&self, a: &Self::Chord, b: &Self::Chord) -> bool {
        std::ptr::eq(a.owner, b.owner)
    }
    fn chord_center_2d(&self, c: &Self::Chord) -> Result<kernel::Point, String> {
        Ok([
            c.bounds[0] as f64 + c.bounds[2] as f64 / 2.,
            c.bounds[1] as f64 + c.bounds[3] as f64 / 2.,
        ])
    }
    fn mirror<'a>(&'a self, c: &'a Self::Chord) -> Result<Option<&'a Self::Chord>, String> {
        c.mirror_key
            .map(|key| {
                self.chords
                    .iter()
                    .find(|m| m.owner.key == key)
                    .ok_or_else(|| "slur_actual_mirror_Chord_outside_owner_view".into())
            })
            .transpose()
    }
    fn top_staff<'a>(&'a self, c: &'a Self::Chord) -> Result<&'a Self::Staff, String> {
        c.members
            .last()
            .map(|h| &h.staff)
            .ok_or_else(|| "slur_actual_top_Staff_required".into())
    }
    fn same_staff(&self, a: &Self::Staff, b: &Self::Staff) -> bool {
        Arc::ptr_eq(a, b)
    }
    fn staff_id_compare(&self, a: &Self::Staff, b: &Self::Staff) -> i32 {
        match a.id.cmp(&b.id) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
    fn stem_direction(&self, c: &Self::Chord) -> Result<i32, String> {
        Ok(c.stem_direction)
    }
    fn beam_stem_relation_count(&self, c: &Self::Chord) -> Result<usize, String> {
        c.beams
            .as_ref()
            .map(Vec::len)
            .ok_or_else(|| "slur_mirror_BeamStem_count_needs_actual_Stem".into())
    }
    fn can_be_orphan(&self, _: kernel::Curve, _: kernel::Side) -> Result<bool, String> {
        // Shared typed HeaderStop construction is separate; ordinary orphan/tablature
        // admission is still uncompared. Never invent a legal orphan from missing owners.
        Err("slur_actual_orphan_Staff_headerStop_tablature_Measure_boundary_pending".into())
    }
}
