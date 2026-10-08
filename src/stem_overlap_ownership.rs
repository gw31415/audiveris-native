// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original typed Head.getMirror/getChord, chord members and SIG.noSupport.
//! Stem-bound chord geometry is shared with the live MEASURES/CHORDS owner.
//! Other ensemble classes and complete removal callbacks remain separate units.
use crate::shared_sig::{Key, Phase};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct Ownership<'a> {
    pub nodes: BTreeMap<u64, &'a Value>,
    pub members: BTreeMap<u64, Vec<u64>>,
    pub head_chords: BTreeMap<u64, u64>,
    pub mirrors: BTreeMap<u64, u64>,
    edges: &'a [Value],
}
impl<'a> Ownership<'a> {
    pub fn new(snapshot: &'a Value) -> Result<Self, String> {
        let rows = snapshot["inters"]
            .as_array()
            .ok_or("overlap_owned_nodes_required")?;
        let edges = snapshot["relations"]
            .as_array()
            .ok_or("overlap_owned_edges_required")?;
        let mut nodes = BTreeMap::new();
        for n in rows {
            let id = n["id"]
                .as_u64()
                .filter(|id| *id > 0)
                .ok_or("invalid_overlap_owned_id")?;
            if nodes.insert(id, n).is_some() {
                return Err("duplicate_overlap_owned_id".into());
            }
        }
        let mut members: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        let mut head_chords = BTreeMap::new();
        let mut mirrors = BTreeMap::new();
        let mut stems = BTreeMap::new();
        for e in edges {
            if !matches!(
                e["kind"].as_str(),
                Some("MirrorRelation" | "Containment" | "ChordStemRelation")
            ) {
                continue;
            }
            let source = e["source"].as_u64().ok_or("invalid_overlap_owned_edge")?;
            let target = e["target"].as_u64().ok_or("invalid_overlap_owned_edge")?;
            let a = nodes.get(&source).ok_or("missing_overlap_owned_endpoint")?;
            let b = nodes.get(&target).ok_or("missing_overlap_owned_endpoint")?;
            let ak: Key = serde_json::from_value(a["origin"].clone())
                .map_err(|_| "invalid_overlap_owned_origin")?;
            let bk: Key = serde_json::from_value(b["origin"].clone())
                .map_err(|_| "invalid_overlap_owned_origin")?;
            if ak.system != bk.system || source == target {
                return Err("invalid_overlap_owned_edge_system".into());
            }
            if e["kind"] == "MirrorRelation" {
                if a["kind"] != "HeadInter"
                    || b["kind"] != "HeadInter"
                    || ak.phase != Phase::Head
                    || bk.phase != Phase::Head
                {
                    return Err("overlap_head_mirror_types_required".into());
                }
                // AbstractInter.getMirror takes the first incident typed edge.
                mirrors.entry(source).or_insert(target);
                mirrors.entry(target).or_insert(source);
            } else if a["kind"] == "HeadChordInter" {
                if e["kind"] == "ChordStemRelation" {
                    if b["kind"] != "StemInter"
                        || ak.phase != Phase::Chord
                        || bk.phase != Phase::Stem
                    {
                        return Err("overlap_chord_stem_types_required".into());
                    }
                    // HeadChord.getStem uses its first typed ChordStem edge.
                    stems.entry(source).or_insert(target);
                    continue;
                }
                if b["kind"] != "HeadInter" || ak.phase != Phase::Chord || bk.phase != Phase::Head {
                    return Err("overlap_chord_head_containment_required".into());
                }
                let ms = members.entry(source).or_default();
                if !ms.contains(&target) {
                    ms.push(target);
                }
                // AbstractInter.getEnsemble returns the first Containment.
                head_chords.entry(target).or_insert(source);
            }
        }
        for n in rows.iter().filter(|n| n["kind"] == "HeadChordInter") {
            let id = n["id"].as_u64().unwrap();
            let ms = members.entry(id).or_default();
            if !n["member_ids"].is_null() {
                let declared: BTreeSet<_> = n["member_ids"]
                    .as_array()
                    .ok_or("invalid_overlap_chord_members")?
                    .iter()
                    .map(|id| id.as_u64().ok_or("invalid_overlap_chord_member"))
                    .collect::<Result<_, _>>()?;
                if declared != ms.iter().copied().collect() {
                    return Err("stale_overlap_chord_containment".into());
                }
            }
            let heads: Vec<_> = ms.iter().map(|m| nodes[m]).collect();
            let stem = stems.get(&id).map(|s| nodes[s]);
            if !n["stem_id"].is_null() && n["stem_id"].as_u64() != stems.get(&id).copied() {
                return Err("stale_overlap_chord_stem".into());
            }
            let geometry = crate::head_chord_geometry::calculate(&heads, stem)?;
            let actual: [i64; 4] = serde_json::from_value(n["bounds"].clone())
                .map_err(|_| "overlap_chord_bounds_required")?;
            if Some(actual) != geometry.bounds {
                return Err("stale_overlap_chord_bounds".into());
            }
            *ms = crate::head_chord_geometry::ordered(&heads)?
                .iter()
                .map(|h| h["id"].as_u64().unwrap())
                .collect();
        }
        Ok(Self {
            nodes,
            members,
            head_chords,
            mirrors,
            edges,
        })
    }
    pub fn mirror(&self, id: u64) -> Option<u64> {
        if self.nodes[&id]["kind"] == "HeadChordInter" {
            for member in &self.members[&id] {
                if let Some(mirror) = self.mirrors.get(member) {
                    // Original returns immediately, including a null chord.
                    return self.head_chords.get(mirror).copied();
                }
            }
            None
        } else {
            self.mirrors.get(&id).copied()
        }
    }
    pub fn mirror_entities(&self, id: u64) -> BTreeSet<u64> {
        let mut result = BTreeSet::new();
        if let Some(mirror) = self.mirror(id) {
            let chord = if self.nodes[&id]["kind"] == "HeadInter" {
                result.insert(mirror);
                self.head_chords.get(&mirror).copied()
            } else {
                Some(mirror)
            };
            if let Some(chord) = chord {
                result.insert(chord);
                result.extend(&self.members[&chord]);
            }
        }
        result
    }
    pub fn no_support(&self, a: u64, b: u64) -> Result<bool, String> {
        for e in self.edges.iter().filter(|e| {
            (e["source"] == a && e["target"] == b) || (e["source"] == b && e["target"] == a)
        }) {
            match e["kind"].as_str() {
                Some(
                    "HeadStemRelation"
                    | "HeadHeadRelation"
                    | "BeamStemRelation"
                    | "BeamHeadRelation"
                    | "BeamBeamRelation"
                    | "MirrorRelation"
                    | "NoExclusion"
                    | "BarConnectionRelation",
                ) => return Ok(false),
                Some("Containment" | "ChordStemRelation" | "Exclusion" | "BarGroupRelation") => {}
                _ => return Err("overlap_owned_support_kind_pending".into()),
            }
        }
        Ok(true)
    }
}
