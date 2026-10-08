// SPDX-License-Identifier: AGPL-3.0-or-later
//! SlotsRetriever relationship priority and preliminary narrow/wide MeasureSlots.
//! These are geometric candidates before Voice/time assignment, not stack Slots.
//! The image adapter uses actual chords/SIG in the existing zero-skew one-staff
//! admission. General staff interpolation, dotted-note acquisition, cue chords
//! and ChordsMapper remain separate dependencies, not planned omissions.
use crate::music_time::Fraction;
use crate::{
    music_measures::{Chord, ChordKind, Measure},
    shared_sig::Graph,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Deserialize)]
struct Fact {
    id: usize,
    bounds: [i64; 4],
    dot_bounds: [i64; 4],
    x: f64,
    head: [f64; 2],
    rest: bool,
    stem: Option<usize>,
    stem_dir: i64,
    has_beams: bool,
    pitches: Vec<i64>,
    leading_pitch: i64,
    time_offset: Option<Fraction>,
}
#[derive(Deserialize)]
struct Explicit {
    kind: String,
    a: usize,
    b: usize,
}
#[derive(Deserialize)]
struct Input {
    chords: Vec<Fact>,
    explicit: Vec<Explicit>,
    mirrors: Vec<[usize; 2]>,
    aligned: Vec<[usize; 2]>,
    groups: Vec<Vec<usize>>,
    interline: usize,
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum Rel {
    Before,
    After,
    Equal,
    Close,
}
#[derive(Serialize)]
struct Relation {
    a: usize,
    b: usize,
    rel: Option<Rel>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct MeasureSlot {
    #[serde(skip)]
    pub(crate) identity: std::sync::Arc<()>,
    pub id: usize,
    pub chord_ids: Vec<usize>,
}
#[derive(Serialize)]
pub(crate) struct Retrieval {
    relations: Vec<Relation>,
    pub slots: Vec<MeasureSlot>,
}
impl Retrieval {
    pub(crate) fn equal(&self, a: usize, b: usize) -> bool {
        self.relations
            .iter()
            .any(|r| r.a == a && r.b == b && r.rel == Some(Rel::Equal))
    }
    pub(crate) fn ordered_siblings(
        &self,
        slot: &CompoundSlot,
        rookie: usize,
        centers: &BTreeMap<usize, i64>,
    ) -> Result<Vec<usize>, String> {
        let x = *centers
            .get(&rookie)
            .ok_or("music_sibling_owned_center_required")?;
        let mut siblings = slot
            .chord_ids
            .iter()
            .copied()
            .filter(|id| *id != rookie)
            .collect::<Vec<_>>();
        if siblings.iter().any(|id| !centers.contains_key(id)) {
            return Err("music_sibling_owned_center_required".into());
        }
        let equal = |id| self.equal(rookie, id);
        siblings.sort_by(|a, b| match (equal(*a), equal(*b)) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => (centers[a] - x).abs().cmp(&(centers[b] - x).abs()),
        });
        Ok(siblings)
    }
}
struct Matrix {
    edges: BTreeMap<(usize, usize), Rel>,
    outgoing: BTreeMap<usize, Vec<usize>>,
}
impl Matrix {
    fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
            outgoing: BTreeMap::new(),
        }
    }
    fn get(&self, a: usize, b: usize) -> Option<Rel> {
        self.edges.get(&(a, b)).copied()
    }
    fn set(&mut self, a: usize, b: usize, rel: Rel) {
        if self.edges.insert((a, b), rel).is_none() {
            self.outgoing.entry(a).or_default().push(b);
        }
    }
    fn pair(&mut self, a: usize, b: usize, rel: Rel) {
        self.set(a, b, rel);
        self.set(b, a, if rel == Rel::Before { Rel::After } else { rel });
    }
    fn equals(&self, a: usize) -> Vec<usize> {
        self.outgoing
            .get(&a)
            .into_iter()
            .flatten()
            .copied()
            .filter(|b| self.get(a, *b) == Some(Rel::Equal))
            .collect()
    }
    fn propagate(&mut self, pair: [usize; 2], chords: &[Fact]) {
        for c in chords.iter().filter(|c| !pair.contains(&c.id)) {
            for i in 0..2 {
                let (one, two) = (pair[i], pair[(i + 1) % 2]);
                if let Some(rel) = self.get(c.id, one)
                    && self.get(c.id, two).is_none()
                {
                    self.set(c.id, two, rel);
                }
                if let Some(rel) = self.get(one, c.id)
                    && self.get(two, c.id).is_none()
                {
                    self.set(two, c.id, rel);
                }
            }
        }
    }
}
fn gap(a: [i64; 4], b: [i64; 4]) -> i64 {
    a[0].max(b[0]) - (a[0] + a[2]).min(b[0] + b[2])
}
fn overlap(a: [i64; 4], b: [i64; 4]) -> i64 {
    (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1])
}
fn adjacent(a: &Fact, b: &Fact, adj_gap: i64, low: i64) -> bool {
    if a.rest || b.rest {
        return false;
    }
    let xgap = gap(a.dot_bounds, b.dot_bounds);
    if xgap > adj_gap {
        return false;
    }
    if a.stem.is_some() && b.stem.is_some() {
        if a.stem == b.stem {
            return true;
        }
        if a.stem_dir == b.stem_dir {
            return false;
        }
        if xgap < 0 {
            return true;
        }
        if a.has_beams && b.has_beams {
            return false;
        }
        if a.pitches
            .iter()
            .any(|p| b.pitches.iter().any(|q| (p - q).abs() <= 2))
        {
            return true;
        }
    } else if (a.leading_pitch - b.leading_pitch).abs() <= 2 {
        return true;
    }
    (a.head[0] - b.head[0]).abs() <= low as f64
}
fn retrieve(input: &Input, wide: bool) -> Result<Retrieval, String> {
    let ids: BTreeMap<_, _> = input.chords.iter().map(|f| (f.id, f)).collect();
    if ids.len() != input.chords.len()
        || input.interline == 0
        || input.chords.iter().any(|f| !f.x.is_finite())
    {
        return Err("music_measure_slot_valid_owned_facts_required".into());
    }
    let get = |id| {
        ids.get(&id)
            .copied()
            .ok_or("music_measure_slot_relation_chord_required")
    };
    let pixels = |v: f64| (v * input.interline as f64).round_ties_even() as i64;
    let low = pixels(0.5);
    let maxdx = if wide { pixels(1.1) } else { low };
    let mut matrix = Matrix::new();
    let mut standard = input.chords.iter().collect::<Vec<_>>();
    standard.sort_by_key(|f| f.bounds[0] + f.bounds[2] / 2);
    // Original inspection order is semantic: explicit separate, same, beam,
    // mirrors, root alignment, same intermediates, adjacency, close, distant.
    for e in input.explicit.iter().filter(|e| {
        matches!(
            e.kind.as_str(),
            "SeparateTimeRelation" | "NextInVoiceRelation"
        )
    }) {
        let (a, b) = (get(e.a)?, get(e.b)?);
        if a.x < b.x {
            matrix.pair(a.id, b.id, Rel::Before);
        } else {
            matrix.pair(b.id, a.id, Rel::Before);
        }
    }
    for e in input
        .explicit
        .iter()
        .filter(|e| e.kind == "SameTimeRelation")
    {
        get(e.a)?;
        get(e.b)?;
        matrix.pair(e.a, e.b, Rel::Equal);
    }
    for group in &input.groups {
        for (i, &a) in group.iter().enumerate() {
            get(a)?;
            for &b in &group[i + 1..] {
                get(b)?;
                matrix.pair(a, b, Rel::Before);
            }
        }
    }
    for (phase, pairs) in [&input.mirrors, &input.aligned].into_iter().enumerate() {
        for &pair in pairs {
            get(pair[0])?;
            get(pair[1])?;
            if phase == 0 && matrix.get(pair[0], pair[1]).is_some() {
                continue;
            }
            matrix.propagate(pair, &input.chords);
            matrix.pair(pair[0], pair[1], Rel::Equal);
        }
    }
    for e in input
        .explicit
        .iter()
        .filter(|e| e.kind == "SameTimeRelation")
    {
        let a = standard.iter().position(|c| c.id == e.a).unwrap();
        let b = standard.iter().position(|c| c.id == e.b).unwrap();
        for i in a.min(b)..a.max(b) {
            for j in i + 1..=a.max(b) {
                let (a, b) = (standard[i].id, standard[j].id);
                if matrix.get(a, b).is_none() {
                    matrix.set(a, b, Rel::Equal);
                }
                if matrix.get(b, a).is_none() {
                    matrix.set(b, a, Rel::Equal);
                }
            }
        }
    }
    for (i, a) in standard.iter().enumerate() {
        for b in &standard[i + 1..] {
            if matrix.get(a.id, b.id).is_none()
                && overlap(a.bounds, b.bounds) > pixels(0.25)
                && adjacent(a, b, pixels(0.4), low)
            {
                matrix.pair(a.id, b.id, Rel::Equal);
            }
        }
    }
    for (i, a) in standard.iter().enumerate() {
        for b in &standard[i + 1..] {
            if matrix.get(a.id, b.id).is_none() && (a.x - b.x).abs() <= maxdx as f64 {
                matrix.pair(a.id, b.id, Rel::Close);
            }
        }
    }
    let separate = |a, b| {
        input.explicit.iter().any(|e| {
            matches!(
                e.kind.as_str(),
                "SeparateTimeRelation" | "NextInVoiceRelation"
            ) && ((e.a == a && e.b == b) || (e.a == b && e.b == a))
        })
    };
    for (i, a) in standard.iter().enumerate() {
        for b in &standard[i + 1..] {
            if matrix.get(a.id, b.id).is_some() {
                continue;
            }
            let mut compatible = false;
            for equal in matrix.equals(b.id) {
                if !separate(a.id, equal) && (a.x - get(equal)?.x).abs() <= maxdx as f64 {
                    compatible = true;
                    break;
                }
            }
            if compatible {
                matrix.pair(a.id, b.id, Rel::Close);
            } else if a.x < b.x {
                matrix.pair(a.id, b.id, Rel::Before);
            } else {
                matrix.pair(b.id, a.id, Rel::Before);
            }
        }
    }
    // Stable relation comparator retains standard collection order on unknown
    // or EQUAL/CLOSE clocks. Before Voice assignment all offsets are null.
    let mut candidates = input.chords.iter().collect::<Vec<_>>();
    candidates.sort_by(|a, b| match matrix.get(a.id, b.id) {
        Some(Rel::Before) => Ordering::Less,
        Some(Rel::After) => Ordering::Greater,
        _ => a
            .time_offset
            .zip(b.time_offset)
            .map_or(Ordering::Equal, |(a, b)| a.cmp(&b)),
    });
    let mut slots: Vec<MeasureSlot> = Vec::new();
    for c in candidates {
        if slots.last().is_some_and(|s| {
            s.chord_ids
                .iter()
                .all(|&id| matches!(matrix.get(id, c.id), Some(Rel::Equal | Rel::Close)))
        }) {
            slots.last_mut().unwrap().chord_ids.push(c.id);
        } else {
            slots.push(MeasureSlot {
                identity: std::sync::Arc::new(()),
                id: slots.len() + 1,
                chord_ids: vec![c.id],
            });
        }
    }
    // Slot/MeasureSlot constructor sorts incoming chord centers by ordinate.
    for slot in &mut slots {
        slot.chord_ids
            .sort_by_key(|id| ids[id].bounds[1] + ids[id].bounds[3] / 2);
    }
    let mut relations = Vec::new();
    for a in &input.chords {
        for b in &input.chords {
            if a.id != b.id {
                relations.push(Relation {
                    a: a.id,
                    b: b.id,
                    rel: matrix.get(a.id, b.id),
                });
            }
        }
    }
    Ok(Retrieval { relations, slots })
}
/// Slot.CompoundSlot retains its actual narrow members. Its chord list is
/// concatenated and stably sorted by center ordinate; aliases are not silently
/// removed. getNarrowSlot takes the first containing member, as in Java.
#[derive(Clone)]
pub(crate) struct CompoundSlot {
    pub identity: std::sync::Arc<()>,
    // Merge can renumber slots; references keep the constructor identity.
    pub key: usize,
    pub id: usize,
    pub members: Vec<MeasureSlot>,
    pub chord_ids: Vec<usize>,
}
impl CompoundSlot {
    pub(crate) fn narrow_owner(&self, chord: usize) -> Option<&MeasureSlot> {
        self.members.iter().find(|s| s.chord_ids.contains(&chord))
    }
    pub(crate) fn narrow_slot(&self, chord: usize) -> Option<usize> {
        self.narrow_owner(chord).map(|s| s.id)
    }
    pub(crate) fn projection(&self) -> Value {
        serde_json::json!({"id":self.id,"narrow_slot_ids":self.members.iter().map(|s|s.id).collect::<Vec<_>>(),"chord_ids":self.chord_ids,"lookup":self.chord_ids.iter().map(|id|serde_json::json!({"chord_id":id,"narrow_slot_id":self.narrow_slot(*id)})).collect::<Vec<_>>()})
    }
}
/// Ordered removal must retain objects still referenced by a chord.
pub(crate) struct CandidateOrder {
    objects: BTreeMap<usize, CompoundSlot>,
    order: Vec<usize>,
    offsets: BTreeMap<usize, i64>,
}
impl CandidateOrder {
    pub(crate) fn new(
        slots: &[CompoundSlot],
        references: &CandidateReferences,
    ) -> Result<Self, String> {
        let objects = slots
            .iter()
            .map(|s| (s.key, s.clone()))
            .collect::<BTreeMap<_, _>>();
        if objects.len() != slots.len() {
            return Err("music_candidate_unique_slot_identity_required".into());
        }
        Ok(Self {
            objects,
            order: slots.iter().map(|s| s.key).collect(),
            offsets: slots
                .iter()
                .map(|s| references.x_offset(s.key).map(|x| (s.key, x)))
                .collect::<Result<_, _>>()?,
        })
    }
    pub(crate) fn get(&self, key: usize) -> Result<&CompoundSlot, String> {
        self.objects
            .get(&key)
            .ok_or_else(|| "music_candidate_retained_slot_required".into())
    }
    pub(crate) fn previous(&self, key: usize) -> Result<Option<&CompoundSlot>, String> {
        self.get(key)?
            .id
            .checked_sub(2)
            .map(|i| {
                self.order
                    .get(i)
                    .ok_or_else(|| "music_candidate_previous_slot_index_required".to_owned())
                    .and_then(|k| self.get(*k))
            })
            .transpose()
    }
    pub(crate) fn merge_previous(&mut self, key: usize, rookie: usize) -> Result<(), String> {
        let id = self.get(key)?.id;
        let previous = self
            .previous(key)?
            .ok_or("music_candidate_previous_slot_index_required")?
            .key;
        if id > self.order.len() {
            return Err("music_candidate_ordered_slot_index_required".into());
        }
        let removal = self
            .order
            .iter()
            .position(|k| self.offsets[k] == self.offsets[&key])
            .ok_or("music_candidate_ordered_slot_required")?;
        self.objects
            .get_mut(&previous)
            .unwrap()
            .chord_ids
            .push(rookie);
        for k in &self.order[id..] {
            self.objects.get_mut(k).unwrap().id -= 1;
        }
        // Java List.remove uses Slot.equals (cached abscissa), not identity.
        self.order.remove(removal);
        Ok(())
    }
    pub(crate) fn at(&self, index: usize) -> Option<&CompoundSlot> {
        self.order.get(index).map(|k| &self.objects[k])
    }
    pub(crate) fn contains(&self, key: usize) -> bool {
        self.order
            .iter()
            .any(|k| self.offsets[k] == self.offsets[&key])
    }
    pub(crate) fn ordered(&self) -> impl Iterator<Item = &CompoundSlot> {
        self.order.iter().map(|k| &self.objects[k])
    }
}
// Original caches the constructor position; member mutations must not move it.
pub(crate) struct CandidateReferences {
    bindings: BTreeMap<usize, usize>,
    offsets: BTreeMap<usize, i64>,
}
impl CandidateReferences {
    pub(crate) fn new(
        graph: &Graph,
        measure: &Measure,
        compounds: &[CompoundSlot],
    ) -> Result<Self, String> {
        let snapshot = graph.snapshot();
        let nodes = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| (n["id"].as_u64().unwrap() as usize, n))
            .collect::<BTreeMap<_, _>>();
        let mut bindings = BTreeMap::new();
        let mut offsets = BTreeMap::new();
        for slot in compounds {
            if slot.chord_ids.is_empty() || offsets.contains_key(&slot.key) {
                return Err("music_candidate_unique_nonempty_slot_required".into());
            }
            let mut sum = 0_i64;
            for &id in &slot.chord_ids {
                let n = nodes
                    .get(&id)
                    .ok_or("music_candidate_live_chord_required")?;
                if !matches!(
                    n["kind"].as_str(),
                    Some("HeadChordInter" | "RestChordInter")
                ) || n["origin"]["system"] != measure.system
                    || n["part_id"] != measure.part
                    || n["measure_index"] != measure.index
                {
                    return Err("music_candidate_owned_measure_chord_required".into());
                }
                let b: [i64; 4] = serde_json::from_value(n["bounds"].clone())
                    .map_err(|_| "music_candidate_bounds_required")?;
                sum += b[0] + b[2] / 2;
                bindings.insert(id, slot.key);
            }
            let x = (sum as f64 / slot.chord_ids.len() as f64 - measure.left as f64)
                .round_ties_even() as i64;
            offsets.insert(slot.key, x);
        }
        Ok(Self { bindings, offsets })
    }
    pub(crate) fn reference(&self, chord: usize) -> Option<usize> {
        self.bindings.get(&chord).copied()
    }
    pub(crate) fn x_offset(&self, key: usize) -> Result<i64, String> {
        self.offsets
            .get(&key)
            .copied()
            .ok_or_else(|| "music_candidate_retained_slot_required".into())
    }
}
/// Measure-scoped transient narrow-slot objects retain their own nullable clock.
/// Shared compound members address one clock through the actual narrow owner;
/// reused numbers on distinct objects must not combine their clocks.
/// These clocks differ from the later timed MeasureStack slots.
#[derive(Clone)]
pub(crate) struct ClockState {
    members: Vec<NarrowClock>,
}
#[derive(Clone)]
struct NarrowClock {
    identity: std::sync::Arc<()>,
    #[cfg(test)]
    id: usize,
    chords: Vec<usize>,
    time: Option<Fraction>,
}
impl ClockState {
    pub(crate) fn new(compounds: &[CompoundSlot]) -> Result<Self, String> {
        let mut members: Vec<NarrowClock> = Vec::new();
        for narrow in compounds.iter().flat_map(|c| &c.members) {
            if let Some(owner) = members
                .iter()
                .find(|m| std::sync::Arc::ptr_eq(&m.identity, &narrow.identity))
            {
                if owner.chords != narrow.chord_ids {
                    return Err("music_candidate_narrow_identity_required".into());
                }
            } else {
                members.push(NarrowClock {
                    identity: narrow.identity.clone(),
                    #[cfg(test)]
                    id: narrow.id,
                    chords: narrow.chord_ids.clone(),
                    time: None,
                });
            }
        }
        Ok(Self { members })
    }
    pub(crate) fn get_owner(&self, owner: &MeasureSlot) -> Result<Option<Fraction>, String> {
        self.members
            .iter()
            .find(|m| std::sync::Arc::ptr_eq(&m.identity, &owner.identity))
            .map(|m| m.time)
            .ok_or_else(|| "music_candidate_owned_narrow_required".into())
    }
    pub(crate) fn set_owner(
        &mut self,
        owner: &MeasureSlot,
        time: Option<Fraction>,
    ) -> Result<(), String> {
        self.members
            .iter_mut()
            .find(|m| std::sync::Arc::ptr_eq(&m.identity, &owner.identity))
            .ok_or("music_candidate_owned_narrow_required")?
            .time = time;
        Ok(())
    }
    #[cfg(test)]
    fn unique_number(&self, id: usize) -> Result<usize, String> {
        let mut owners = self.members.iter().enumerate().filter(|(_, m)| m.id == id);
        let (i, _) = owners
            .next()
            .ok_or("music_candidate_owned_narrow_required")?;
        if owners.next().is_some() {
            return Err("music_candidate_ambiguous_narrow_number".into());
        }
        Ok(i)
    }
    #[cfg(test)]
    pub(crate) fn get(&self, id: usize) -> Result<Option<Fraction>, String> {
        Ok(self.members[self.unique_number(id)?].time)
    }
    #[cfg(test)]
    pub(crate) fn set(&mut self, id: usize, time: Option<Fraction>) -> Result<(), String> {
        let i = self.unique_number(id)?;
        self.members[i].time = time;
        Ok(())
    }
    pub(crate) fn first_time(&self, slot: &CompoundSlot) -> Result<Option<Fraction>, String> {
        self.get_owner(
            slot.members
                .first()
                .ok_or("music_candidate_first_narrow_required")?,
        )
    }
    pub(crate) fn projection(&self, slot: &CompoundSlot) -> Result<Value, String> {
        slot.members.iter().map(|s| self.get_owner(s).map(|t| serde_json::json!({"id":s.id,"time_offset":t.map(crate::music_time::format_fraction)}))).collect::<Result<Vec<_>,_>>().map(|v|serde_json::json!(v))
    }
}
pub(crate) fn compound(
    id: usize,
    members: &[MeasureSlot],
    centers: &BTreeMap<usize, i64>,
) -> Result<CompoundSlot, String> {
    let mut chord_ids = members
        .iter()
        .flat_map(|s| s.chord_ids.iter())
        .copied()
        .collect::<Vec<_>>();
    if chord_ids.iter().any(|id| !centers.contains_key(id)) {
        return Err("music_compound_slot_owned_chord_required".into());
    }
    chord_ids.sort_by_key(|id| centers[id]);
    Ok(CompoundSlot {
        identity: std::sync::Arc::new(()),
        key: id - 1,
        id,
        members: members.to_vec(),
        chord_ids,
    })
}
/// MeasureRhythm.buildCompoundSlots. Retain the original last-range cursor
/// behavior even for declared overlapping wide lists; do not invent a repair.
pub(crate) fn build_compounds(
    narrow: &[MeasureSlot],
    wide: &[MeasureSlot],
    centers: &BTreeMap<usize, i64>,
) -> Result<Vec<CompoundSlot>, String> {
    let mut result = Vec::new();
    let mut start = 0;
    for wide in wide {
        let mut next = false;
        for (i, slot) in narrow.iter().enumerate().skip(start) {
            let intersection = slot.chord_ids.iter().any(|id| wide.chord_ids.contains(id));
            if !intersection && i > start {
                result.push(compound(result.len() + 1, &narrow[start..i], centers)?);
                start = i;
                next = true;
                break;
            }
        }
        if !next {
            result.push(compound(result.len() + 1, &narrow[start..], centers)?);
        }
    }
    Ok(result)
}
/// Prepare getters from live typed SIG, not serialized oracle facts. Current
/// includes actual augmentation-dot bounds in the original proximity test.
pub(crate) fn from_owned(
    graph: &Graph,
    measure: &Measure,
    chords: &[&Chord],
    interline: usize,
) -> Result<(Retrieval, Retrieval), String> {
    let snapshot = graph.snapshot();
    let nodes: BTreeMap<_, _> = snapshot["inters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_u64().unwrap() as usize, n))
        .collect();
    let edges = snapshot["relations"].as_array().unwrap();
    let mut facts = Vec::new();
    let mut stems = BTreeMap::new();
    let mut notes = BTreeMap::new();
    for &chord in chords {
        if graph.id(chord.key) != Some(chord.id) || chord.key.system != measure.system {
            return Err("music_measure_slot_actual_owner_required".into());
        }
        let rest = chord.kind == ChordKind::Rest;
        if rest
            && chord
                .members
                .first()
                .and_then(|n| n["shape"].as_str())
                .is_some_and(|s| crate::music_voices::is_measure_rest_shape(s, false))
        {
            continue;
        }
        let members = chord
            .members
            .iter()
            .map(|n| {
                nodes
                    .get(&(n["id"].as_u64().unwrap() as usize))
                    .copied()
                    .ok_or("music_measure_slot_live_member_required")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let stem = chord
            .stem
            .map(|k| graph.id(k).ok_or("music_measure_slot_live_stem_required"))
            .transpose()?;
        let geometry = if rest {
            crate::head_chord_geometry::calculate_rest(&members)?
        } else {
            crate::head_chord_geometry::calculate(&members, stem.map(|id| nodes[&id]))?
        };
        let bounds = geometry
            .bounds
            .ok_or("music_measure_slot_chord_bounds_required")?;
        let pitch = |n: &Value| {
            n["pitch"]
                .as_f64()
                .filter(|p| p.is_finite())
                .map(|p| p.round_ties_even() as i64)
                .ok_or("music_measure_slot_note_pitch_required")
        };
        let pitches = members
            .iter()
            .map(|n| pitch(n))
            .collect::<Result<Vec<_>, _>>()?;
        let leading = nodes[&(geometry
            .leading_id
            .ok_or("music_measure_slot_leading_note_required")?
            as usize)];
        let has_beams = !crate::music_voices::chord_beams(&snapshot, chord.id)?.is_empty();
        facts.push(Fact {
            id: chord.id,
            bounds,
            dot_bounds: crate::music_dots::bounds_with_dots(&snapshot, chord, bounds)?,
            x: (bounds[0] + bounds[2] / 2 - measure.left) as f64,
            head: geometry
                .head_location
                .ok_or("music_measure_slot_head_location_required")?
                .map(|x| x as f64),
            rest,
            stem,
            stem_dir: geometry.stem_dir(stem.is_some())?,
            has_beams,
            pitches,
            leading_pitch: pitch(leading)?,
            time_offset: nodes[&chord.id]["time_offset"]
                .as_str()
                .map(Fraction::parse)
                .transpose()?,
        });
        stems.insert(chord.id, stem);
        notes.insert(
            chord.id,
            members
                .iter()
                .map(|n| n["id"].as_u64().unwrap())
                .collect::<Vec<_>>(),
        );
    }
    let ids: BTreeSet<_> = facts.iter().map(|f| f.id).collect();
    let explicit = edges
        .iter()
        .filter(|e| {
            matches!(
                e["kind"].as_str(),
                Some("SameTimeRelation" | "SeparateTimeRelation" | "NextInVoiceRelation")
            ) && ids.contains(&(e["source"].as_u64().unwrap() as usize))
                && ids.contains(&(e["target"].as_u64().unwrap() as usize))
        })
        .map(|e| Explicit {
            kind: e["kind"].as_str().unwrap().into(),
            a: e["source"].as_u64().unwrap() as usize,
            b: e["target"].as_u64().unwrap() as usize,
        })
        .collect();
    let ownership = crate::stem_overlap_ownership::Ownership::new(&snapshot)?;
    let mut mirrors = Vec::new();
    let mut aligned = Vec::new();
    for (i, a) in facts.iter().enumerate().filter(|(_, f)| !f.rest) {
        for b in facts[i + 1..].iter().filter(|f| !f.rest) {
            if notes[&a.id].iter().any(|n| {
                ownership
                    .mirrors
                    .get(n)
                    .is_some_and(|m| notes[&b.id].contains(m))
            }) {
                mirrors.push([a.id, b.id]);
            }
            if let (Some(sa), Some(sb)) = (stems[&a.id], stems[&b.id])
                && edges.iter().any(|e| {
                    e["kind"] == "StemAlignmentRelation"
                        && ((e["source"] == sa && e["target"] == sb)
                            || (e["source"] == sb && e["target"] == sa))
                })
            {
                aligned.push([a.id, b.id]);
            }
        }
    }
    // BeamGroup.getChords already translates each typed beam's head/rest links,
    // stable center -> left sorting and aliases, without querying group Voice.
    let groups = crate::music_voices::beam_group_chords(graph)?;
    // Measure.getBeamGroups follows each standard chord's actual first beam
    // ensemble, retaining first encounter order. Other groups in the same
    // system must not override this measure's relationships.
    let mut selected = Vec::new();
    for fact in &facts {
        if let Some(gid) = crate::music_voices::chord_group(&snapshot, fact.id)?
            && !selected.contains(&gid)
        {
            selected.push(gid);
        }
    }
    let groups = selected
        .into_iter()
        .map(|gid| {
            let group = groups
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["id"] == gid)
                .ok_or("music_measure_slot_selected_group_required")?;
            Ok(group["chords"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["id"].as_u64().unwrap() as usize)
                .filter(|id| ids.contains(id))
                .collect())
        })
        .collect::<Result<Vec<_>, String>>()?;
    let input = Input {
        chords: facts,
        explicit,
        mirrors,
        aligned,
        groups,
        interline,
    };
    Ok((retrieve(&input, false)?, retrieve(&input, true)?))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn constructor_bindings_and_cached_positions_survive_membership_changes() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-candidate-slot-reference/observed.json"
        ))
        .unwrap();
        let image = crate::GrayImage::decode(include_bytes!(
            "../fixtures/official-image-musicxml-cross-system/beam-two-systems.png"
        ))
        .unwrap();
        let owner = crate::music_xml::recognize(&image).unwrap().owner;
        let mi = 1;
        let mut chords = owner
            .chords
            .iter()
            .filter(|c| c.measure == mi)
            .collect::<Vec<_>>();
        chords.sort_by_key(|c| c.bounds[0] + c.bounds[2] / 2);
        for case in oracle["controls"].as_array().unwrap() {
            assert_eq!(
                json!(chords.iter().map(|c| c.bounds).collect::<Vec<_>>()),
                case["input"]["chord_bounds"]
            );
            assert_eq!(
                owner.measures[mi].left as f64,
                case["input"]["left"].as_f64().unwrap()
            );
            let centers = chords
                .iter()
                .map(|c| (c.id, c.bounds[1] + c.bounds[3] / 2))
                .collect();
            let mut objects = Vec::new();
            for constructor in case["input"]["constructors"].as_array().unwrap() {
                let ordinals: Vec<usize> =
                    serde_json::from_value(constructor["chord_ids"].clone()).unwrap();
                let member = MeasureSlot {
                    identity: std::sync::Arc::new(()),
                    id: constructor["id"].as_u64().unwrap() as usize,
                    chord_ids: ordinals.iter().map(|&i| chords[i].id).collect(),
                };
                objects.push(compound(member.id, &[member], &centers).unwrap());
            }
            let references =
                CandidateReferences::new(&owner.owner.graph, &owner.measures[mi], &objects)
                    .unwrap();
            let mut ordered = objects.iter().map(|s| s.key).collect::<Vec<_>>();
            if let Some(i) = case["input"]["append_to_first"].as_u64() {
                objects[0].chord_ids.push(chords[i as usize].id);
            }
            if let Some(key) = case["input"]["remove_ordered"].as_u64() {
                ordered.retain(|&k| k != key as usize);
            }
            let first = references.x_offset(objects[0].key).unwrap();
            let rows = case["output"]["references"].as_array().unwrap().iter().map(|row| {
                let i = row["chord"].as_u64().unwrap() as usize;
                let key = references.reference(chords[i].id).unwrap();
                let x = references.x_offset(key).unwrap();
                json!({"chord":i,"object_key":key,"x_offset":x,"compare_to_first":(x-first).signum()})
            }).collect::<Vec<_>>();
            assert_eq!(
                json!({"ordered_keys":ordered,"first_x_offset":first,"references":rows}),
                case["output"],
                "{}",
                case["name"]
            );
        }
    }
    #[test]
    fn original_compound_partition_ordinate_lookup_cursor_and_empty() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../fixtures/official-compound-slots/observed.json"
        ))
        .unwrap();
        for page in oracle["pages"].as_array().unwrap() {
            for measure in page["measures"].as_array().unwrap() {
                let centers = measure["chord_bounds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(id, b)| (id, b[1].as_i64().unwrap() + b[3].as_i64().unwrap() / 2))
                    .collect();
                for case in measure["cases"].as_array().unwrap() {
                    let narrow: Vec<MeasureSlot> =
                        serde_json::from_value(case["narrow"].clone()).unwrap();
                    let wide: Vec<MeasureSlot> =
                        serde_json::from_value(case["wide"].clone()).unwrap();
                    let compounds = build_compounds(&narrow, &wide, &centers).unwrap();
                    assert_eq!(
                        serde_json::json!(
                            compounds
                                .iter()
                                .map(CompoundSlot::projection)
                                .collect::<Vec<_>>()
                        ),
                        case["compounds"],
                        "{}",
                        case["name"]
                    );
                    for c in compounds {
                        assert!(c.narrow_slot(usize::MAX).is_none());
                    }
                }
            }
        }
    }
    #[test]
    fn original_saved_and_declared_measure_slot_priority_and_all_member_compatibility() {
        let original: Value = serde_json::from_str(include_str!(
            "../fixtures/official-measure-slot-retrieval/observed.json"
        ))
        .unwrap();
        let mut observations = Vec::new();
        for p in original["pages"].as_array().unwrap() {
            observations.extend(p["measures"].as_array().unwrap().iter());
        }
        observations.extend(
            original["controls"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| &c["observation"]),
        );
        for row in observations {
            let input: Input = serde_json::from_value(row.clone()).unwrap();
            for (name, wide) in [("narrow", false), ("wide", true)] {
                assert_eq!(
                    serde_json::to_value(retrieve(&input, wide).unwrap()).unwrap(),
                    row[name]
                );
            }
        }
        let empty = Input {
            chords: vec![],
            explicit: vec![],
            mirrors: vec![],
            aligned: vec![],
            groups: vec![],
            interline: 11,
        };
        assert!(retrieve(&empty, false).unwrap().slots.is_empty());
    }
}
