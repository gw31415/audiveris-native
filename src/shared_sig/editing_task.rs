// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, NoteOwnerEdit, Phase};
use serde_json::{Value, json};

const STEPS: [&str; 20] = [
    "LOAD",
    "BINARY",
    "SCALE",
    "GRID",
    "HEADERS",
    "STEM_SEEDS",
    "BEAMS",
    "LEDGERS",
    "HEADS",
    "STEMS",
    "REDUCTION",
    "CUE_BEAMS",
    "TEXTS",
    "MEASURES",
    "CHORDS",
    "CURVES",
    "SYMBOLS",
    "LINKS",
    "RHYTHMS",
    "PAGE",
];

#[derive(Clone, Debug)]
pub struct EditingLink {
    partner: Key,
    outgoing: bool,
    relation: Value,
    edge: Option<usize>,
}
impl EditingLink {
    pub fn new(partner: Key, outgoing: bool, relation: Value) -> Self {
        Self {
            partner,
            outgoing,
            relation,
            edge: None,
        }
    }
    pub fn from_registered(graph: &Graph, inter: Key, id: usize) -> Result<Self, String> {
        let focus = graph.id(inter).ok_or("editing_inter_identity_required")?;
        let edge = graph
            .edges
            .get(
                id.checked_sub(1)
                    .ok_or("editing_relation_identity_required")?,
            )
            .ok_or("editing_relation_identity_required")?;
        if edge.key.phase != Phase::Symbol
            || !matches!(
                edge.key.kind.as_str(),
                "AugmentationRelation" | "DoubleDotRelation"
            )
        {
            return Err("editing_relation_callback_pending".into());
        }
        let outgoing = edge.key.source == focus;
        if !outgoing && edge.key.target != focus {
            return Err("editing_relation_focus_required".into());
        }
        let partner = if outgoing {
            edge.key.target
        } else {
            edge.key.source
        };
        Ok(Self {
            partner: graph.vertices[partner - 1].key,
            outgoing,
            relation: edge.data.clone(),
            edge: Some(id),
        })
    }
    fn endpoints(&self, graph: &Graph, inter: Key) -> Result<(usize, usize), String> {
        let a = graph.id(inter).ok_or("editing_inter_identity_required")?;
        let b = graph
            .id(self.partner)
            .ok_or("editing_partner_identity_required")?;
        if inter.system != self.partner.system {
            return Err("editing_same_system_required".into());
        }
        Ok(if self.outgoing { (a, b) } else { (b, a) })
    }
    fn apply(&mut self, graph: &mut Graph, inter: Key) -> Result<bool, String> {
        self.apply_relation(graph, inter, true)
    }
    fn apply_relation(
        &mut self,
        graph: &mut Graph,
        inter: Key,
        search_link: bool,
    ) -> Result<bool, String> {
        let (source, target) = self.endpoints(graph, inter)?;
        if search_link && (graph.vertices[source - 1].removed || graph.vertices[target - 1].removed)
        {
            return Ok(false);
        }
        if !search_link {
            graph.endpoint(inter)?;
            graph.endpoint(self.partner)?;
            if let Some(id) = self.edge {
                let retained = &graph.edges[id - 1];
                // Raw addEdge on the same live object does not re-register it;
                // task references share any edits made to the retained relation.
                if !retained.removed {
                    return Ok(false);
                }
                self.relation = retained.data.clone();
            }
        }
        let kind = self.relation["kind"]
            .as_str()
            .ok_or("editing_relation_kind_required")?;
        if !matches!(kind, "AugmentationRelation" | "DoubleDotRelation") {
            return Err("editing_relation_callback_pending".into());
        }
        if search_link
            && graph
                .live_edges()
                .iter()
                .any(|e| e.key.source == source && e.key.target == target && e.key.kind == kind)
        {
            return Ok(false);
        }
        let ordinal = self
            .edge
            .map(|i| graph.edges[i - 1].key.ordinal)
            .unwrap_or(graph.edges.len() + 1);
        let mut data = self.relation.clone();
        data["_editing_registration_ordinal"] = json!(ordinal);
        graph.relations(
            inter.system,
            Phase::Symbol,
            vec![(
                graph.vertices[source - 1].key,
                graph.vertices[target - 1].key,
                data,
            )],
        )?;
        self.edge = graph
            .edges
            .iter()
            .position(|e| {
                e.key.source == source
                    && e.key.target == target
                    && e.key.kind == kind
                    && e.key.ordinal == ordinal
            })
            .map(|i| i + 1);
        Ok(true)
    }
    fn remove(&self, graph: &mut Graph, inter: Key) -> Result<bool, String> {
        let (source, target) = self.endpoints(graph, inter)?;
        if graph.vertices[source - 1].removed || graph.vertices[target - 1].removed {
            return Ok(false);
        }
        let Some(id) = self.edge else {
            return Ok(false);
        };
        if graph.edges[id - 1].removed {
            return Ok(false);
        };
        graph.remove_augmentation_edge(id)?;
        Ok(true)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EditingOperation {
    Do,
    Undo,
    Redo,
}
impl EditingOperation {
    fn name(self) -> &'static str {
        match self {
            Self::Do => "do",
            Self::Undo => "undo",
            Self::Redo => "redo",
        }
    }
}

/// Retained DefaultEditor/EditingTask boundary, not controller preEdit/impact.
#[derive(Clone, Debug)]
pub struct DefaultEditingTask {
    inter: Key,
    original_bounds: [i64; 4],
    latest_bounds: [i64; 4],
    original_glyph: Option<(Value, Value)>,
    links: Vec<EditingLink>,
    unlinks: Vec<EditingLink>,
}
impl DefaultEditingTask {
    pub fn new(
        graph: &Graph,
        inter: Key,
        latest_bounds: [i64; 4],
        links: Vec<EditingLink>,
        unlinks: Vec<EditingLink>,
    ) -> Result<Self, String> {
        let id = graph.id(inter).ok_or("editing_inter_identity_required")?;
        let v = &graph.vertices[id - 1];
        if v.removed || v.key.phase != Phase::Symbol || v.data["kind"] != "AugmentationDotInter" {
            return Err("editing_default_editor_kind_pending".into());
        }
        if graph
            .edges
            .iter()
            .any(|e| !e.removed && e.key.kind == "Containment" && e.key.target == id)
        {
            return Err("editing_ensemble_bounds_invalidation_pending".into());
        }
        let original_bounds = serde_json::from_value(v.data["bounds"].clone())
            .map_err(|_| "editing_bounds_required")?;
        Ok(Self {
            inter,
            original_bounds,
            latest_bounds,
            original_glyph: None,
            links,
            unlinks,
        })
    }
    pub fn retained_links(&self) -> usize {
        self.links.len()
    }
    pub fn perform(
        &mut self,
        graph: &mut Graph,
        operation: EditingOperation,
    ) -> Result<Value, String> {
        let mut next = graph.clone();
        let mut task = self.clone();
        let id = next
            .id(task.inter)
            .ok_or("editing_inter_identity_required")?;
        if next.vertices[id - 1].data["frozen"] == true {
            return Err("music_removal_frozen_inter".into());
        }
        if matches!(operation, EditingOperation::Undo) {
            next.edit_note_owner(task.inter, NoteOwnerEdit::Bounds(task.original_bounds))?;
            if let Some((glyph, gid)) = &task.original_glyph {
                next.vertices[id - 1].data["glyph"] = glyph.clone();
                next.vertices[id - 1].data["glyph_id"] = gid.clone();
            }
            if !next.vertices[id - 1].data["attachments"].is_null() {
                next.vertices[id - 1].data["attachments"] = json!({});
            }
            for link in &mut task.unlinks {
                link.apply(&mut next, task.inter)?;
            }
            for link in &task.links {
                link.remove(&mut next, task.inter)?;
            }
        } else {
            next.edit_note_owner(task.inter, NoteOwnerEdit::Bounds(task.latest_bounds))?;
            let n = &mut next.vertices[id - 1].data;
            if !n["glyph_id"].is_null() || !n["glyph"].is_null() {
                task.original_glyph = Some((n["glyph"].clone(), n["glyph_id"].clone()));
            }
            n["glyph"] = Value::Null;
            n["glyph_id"] = Value::Null;
            let mut applied = Vec::new();
            for mut link in task.links {
                if link.apply(&mut next, task.inter)? {
                    applied.push(link);
                }
            }
            task.links = applied;
            for link in &task.unlinks {
                link.remove(&mut next, task.inter)?;
            }
        }
        // Neither raw editor nor task reconstructs Measure membership. Controller
        // impact belongs after task publication, and remains a separate boundary.
        let publication = json!({"inter_id":id,"operation":operation.name(),"phase":"after_task","actual_dirty_state":next.editing_book_state(),"controller_impact_complete":false});
        next.edit_publications.push(publication.clone());
        #[cfg(test)]
        next.dot_callback_event("publish", id, "delivered");
        *graph = next;
        *self = task;
        Ok(publication)
    }
}

/// Actual RelationTask identity, separate from search Link.applyTo deduplication.
#[derive(Clone, Debug)]
struct RetainedHeadStem {
    chord: Key,
    stem: std::sync::Arc<crate::stem_cache_sig::CachedStem>,
}
#[derive(Clone, Debug)]
pub struct RelationTask {
    source: Key,
    link: EditingLink,
    unlink: bool,
    owned_chord: Option<RetainedHeadStem>,
}
impl RelationTask {
    pub fn link(graph: &Graph, source: Key, target: Key, relation: Value) -> Result<Self, String> {
        graph.endpoint(source)?;
        graph.endpoint(target)?;
        if source.system != target.system
            || !matches!(
                relation["kind"].as_str(),
                Some("AugmentationRelation" | "DoubleDotRelation")
            )
        {
            return Err("editing_relation_callback_pending".into());
        }
        Ok(Self {
            source,
            link: EditingLink::new(target, true, relation),
            unlink: false,
            owned_chord: None,
        })
    }
    pub fn unlink(graph: &Graph, id: usize) -> Result<Self, String> {
        let edge = graph
            .edges
            .get(
                id.checked_sub(1)
                    .ok_or("editing_relation_identity_required")?,
            )
            .filter(|e| !e.removed)
            .ok_or("editing_live_relation_required")?;
        let source = graph.vertices[edge.key.source - 1].key;
        Ok(Self {
            source,
            link: EditingLink::from_registered(graph, source, id)?,
            unlink: true,
            owned_chord: None,
        })
    }
    /// Explicit retained-owner task; this does not run controller preLink or
    /// admit a new recognition/correction CLI input.
    pub fn manual_head_stem(
        owner: &crate::music_measures::OwnedMusic,
        head: Key,
        stem: Key,
        mut relation: Value,
    ) -> Result<Self, String> {
        let (_, chord) = owner.manual_head_stem_owner(head, stem)?;
        let graph = &owner.owner.graph;
        graph.endpoint(head)?;
        graph.endpoint(stem)?;
        let canonical = owner
            .owner
            .caches
            .get(&stem.system)
            .ok_or("manual_head_stem_actual_cache_required")?
            .registered_stem(graph, stem.local)?;
        if relation["kind"] != "HeadStemRelation" || relation["manual"] != true {
            return Err("manual_head_stem_declared_relation_properties_required".into());
        }
        relation
            .as_object_mut()
            .ok_or("invalid_shared_sig_relation")?
            .remove("local_relation_id");
        Ok(Self {
            source: head,
            link: EditingLink::new(stem, true, relation),
            unlink: false,
            owned_chord: Some(RetainedHeadStem {
                chord,
                stem: canonical,
            }),
        })
    }
    fn perform_owned(
        &mut self,
        owner: &mut crate::music_measures::OwnedMusic,
        operation: EditingOperation,
    ) -> Result<(), String> {
        let Some(binding) = self.owned_chord.as_ref() else {
            return self.perform(&mut owner.owner.graph, operation);
        };
        let (_, current) = owner.manual_head_stem_owner(self.source, self.link.partner)?;
        let canonical = owner
            .owner
            .caches
            .get(&self.link.partner.system)
            .ok_or("manual_head_stem_actual_cache_required")?
            .registered_stem(&owner.owner.graph, self.link.partner.local)?;
        if current != binding.chord || !std::sync::Arc::ptr_eq(&canonical, &binding.stem) {
            return Err("manual_head_stem_retained_owner_changed".into());
        }
        let adding = self.unlink == matches!(operation, EditingOperation::Undo);
        if adding {
            let retained = self.link.edge.map(|id| &owner.owner.graph.edges[id - 1]);
            if retained.is_none_or(|e| e.removed) {
                if let Some(edge) = retained {
                    self.link.relation = edge.data.clone();
                }
                if self.link.edge.is_none() {
                    let ordinal = owner
                        .owner
                        .graph
                        .edges
                        .iter()
                        .filter(|e| e.key.phase == Phase::Stem)
                        .map(|e| e.key.ordinal)
                        .max()
                        .unwrap_or(0)
                        .checked_add(1)
                        .ok_or("shared_sig_relation_order_budget")?;
                    self.link.relation["local_relation_id"] = json!(ordinal);
                }
                owner.apply_manual_head_stem_callback(
                    self.source,
                    self.link.partner,
                    self.link.relation.clone(),
                )?;
                let graph = &owner.owner.graph;
                let source = graph.endpoint(self.source)?;
                let target = graph.endpoint(self.link.partner)?;
                let ordinal = self.link.relation["local_relation_id"]
                    .as_u64()
                    .ok_or("editing_relation_identity_required")?
                    as usize;
                self.link.edge = graph
                    .edges
                    .iter()
                    .position(|e| {
                        e.key.phase == Phase::Stem
                            && e.key.kind == "HeadStemRelation"
                            && e.key.source == source
                            && e.key.target == target
                            && e.key.ordinal == ordinal
                    })
                    .map(|i| i + 1);
                if self.link.edge.is_none() {
                    return Err("editing_relation_identity_required".into());
                }
            }
            self.publish(&mut owner.owner.graph, operation)?;
        } else {
            let id = self.link.edge.ok_or("editing_live_relation_required")?;
            // HeadStemRelation.removed checks endpoints. It does not restore
            // the ChordStem removed by the earlier nested added callback.
            owner
                .owner
                .graph
                .remove_head_stem_edges(self.source, &[id])?;
        }
        Ok(())
    }
    fn publish(&self, graph: &mut Graph, operation: EditingOperation) -> Result<(), String> {
        let id = graph.endpoint(self.source)?;
        graph
            .edit_publications
            .push(json!({"inter_id":id,"operation":operation.name(),
            "phase":"after_task","task":if self.unlink {"UnlinkTask"}else{"LinkTask"},
            "actual_dirty_state":graph.editing_book_state(),"controller_impact_complete":false}));
        #[cfg(test)]
        graph.dot_callback_event("publish", id, "delivered");
        Ok(())
    }
    fn perform(&mut self, graph: &mut Graph, operation: EditingOperation) -> Result<(), String> {
        if self.owned_chord.is_some() {
            return Err("editing_actual_music_owner_required".into());
        }
        let adding = self.unlink == matches!(operation, EditingOperation::Undo);
        if adding {
            self.link.apply_relation(graph, self.source, false)?;
            self.publish(graph, operation)?;
        } else {
            let id = self.link.edge.ok_or("editing_live_relation_required")?;
            if graph.edges[id - 1].removed {
                return Err("editing_live_relation_required".into());
            }
            self.link.remove(graph, self.source)?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub enum EditingTask {
    Inter(DefaultEditingTask),
    Relation(RelationTask),
}
impl EditingTask {
    fn focus(&self) -> Key {
        match self {
            Self::Inter(t) => t.inter,
            Self::Relation(t) => t.source,
        }
    }
    fn supports(&self) -> Vec<&EditingLink> {
        match self {
            Self::Inter(t) => t.links.iter().chain(&t.unlinks).collect(),
            Self::Relation(t) => vec![&t.link],
        }
    }
    fn impact_inters(&self) -> Vec<Key> {
        match self {
            Self::Inter(t) => vec![t.inter],
            Self::Relation(t) => vec![t.source, t.link.partner],
        }
    }
    fn perform(&mut self, graph: &mut Graph, operation: EditingOperation) -> Result<(), String> {
        match self {
            Self::Inter(t) => {
                t.perform(graph, operation)?;
            }
            Self::Relation(t) => t.perform(graph, operation)?,
        }
        Ok(())
    }
}

/// UITaskList order plus CtrlTask impact notification for the retained dot tasks.
/// A notification is not evidence that RhythmStep/PageStep reprocessing ran.
#[derive(Clone, Debug)]
pub struct EditingSequence {
    tasks: Vec<EditingTask>,
}
impl EditingSequence {
    pub fn new(tasks: Vec<DefaultEditingTask>) -> Self {
        Self::from_tasks(tasks.into_iter().map(EditingTask::Inter).collect())
    }
    pub fn from_tasks(tasks: Vec<EditingTask>) -> Self {
        Self { tasks }
    }
    pub fn prepend(&mut self, task: EditingTask) {
        self.tasks.insert(0, task);
    }
    pub(crate) fn require_ordinary_reprocess(&self, graph: &Graph) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Err("music_reprocess_timed_support_edit_pending".into());
        }
        for task in &self.tasks {
            let supports = task.supports();
            if supports.is_empty() {
                continue;
            }
            if let EditingTask::Inter(t) = task
                && (t.links.len() > 1 || t.unlinks.len() > 1)
            {
                return Err("music_reprocess_timed_support_edit_pending".into());
            }
            let focus = graph.endpoint(task.focus())?;
            let dot = &graph.vertices[focus - 1];
            let target = graph
                .dot_edit_stack_target(task.focus())?
                .ok_or("editing_owned_stack_selection_required")?;
            for link in supports {
                if !link.outgoing || link.relation["kind"] != "AugmentationRelation" {
                    return Err("music_reprocess_timed_support_edit_pending".into());
                }
                let (_, id) = link.endpoints(graph, task.focus())?;
                let note = &graph.vertices[id - 1];
                let owners = graph
                    .live_edges()
                    .into_iter()
                    .filter(|e| e.key.kind == "Containment" && e.key.target == id)
                    .map(|e| e.key.source)
                    .collect::<Vec<_>>();
                if note.removed
                    || note.key.phase != Phase::Head
                    || note.data["kind"] != "HeadInter"
                    || note.data["shape"] != "NOTEHEAD_BLACK"
                    || owners.len() != 1
                {
                    return Err("music_reprocess_timed_support_edit_pending".into());
                }
                let chord = &graph.vertices[owners[0] - 1];
                let members = graph
                    .live_edges()
                    .into_iter()
                    .filter(|e| e.key.kind == "Containment" && e.key.source == owners[0])
                    .count();
                if chord.removed
                    || chord.data["kind"] != "HeadChordInter"
                    || members != 1
                    || chord.key.system != task.focus().system
                    || chord.data["measure_index"] != target["measure_index"]
                    || chord.data["part_id"] != dot.cached_part.map_or(Value::Null, |p| json!(p))
                    || chord.data["staff_id"] != dot.data["staff_id"]
                    || chord.data["duration"].as_str().is_none()
                    || chord.data["time_offset"].as_str().is_none()
                    || chord.data["voice"].as_u64().is_none()
                    || chord.data["slot_id"].as_u64().is_none()
                {
                    return Err("music_reprocess_timed_support_edit_pending".into());
                }
                // Exact getters do not imply admission to the rhythm grid.
                crate::music_duration::live(&graph.snapshot(), owners[0])?;
            }
        }
        Ok(())
    }
    pub(crate) fn require_single_support(&self, graph: &Graph) -> Result<(), String> {
        for task in &self.tasks {
            let id = graph.endpoint(task.focus())?;
            if graph
                .live_edges()
                .into_iter()
                .filter(|e| e.key.kind == "AugmentationRelation" && e.key.source == id)
                .count()
                > 1
            {
                return Err("music_reprocess_shared_dot_support_edit_pending".into());
            }
        }
        Ok(())
    }
    pub(crate) fn require_same_stack(&self, before: &Graph, after: &Graph) -> Result<(), String> {
        for task in &self.tasks {
            if before.dot_edit_stack_target(task.focus())?
                != after.dot_edit_stack_target(task.focus())?
            {
                return Err("music_reprocess_cross_stack_owner_edit_pending".into());
            }
        }
        Ok(())
    }
    pub fn perform(
        &mut self,
        graph: &mut Graph,
        operation: EditingOperation,
        latest_step: &str,
    ) -> Result<Value, String> {
        let latest = STEPS
            .iter()
            .position(|s| *s == latest_step)
            .ok_or("editing_latest_step_required")?;
        let mut next = graph.clone();
        let mut sequence = self.clone();
        if matches!(operation, EditingOperation::Undo) {
            for task in sequence.tasks.iter_mut().rev() {
                task.perform(&mut next, operation)?;
            }
        } else {
            for task in &mut sequence.tasks {
                task.perform(&mut next, operation)?;
            }
        }
        let notification = sequence.notify_impact(&mut next, operation, latest_step, latest)?;
        *graph = next;
        *self = sequence;
        Ok(notification)
    }
    /// Run real tasks against retained Chords as well as SIG. The notification
    /// is still planned impact; the caller must separately reprocess its stacks.
    pub fn perform_owned(
        &mut self,
        owner: &mut crate::music_measures::OwnedMusic,
        operation: EditingOperation,
        latest_step: &str,
    ) -> Result<Value, String> {
        let latest = STEPS
            .iter()
            .position(|s| *s == latest_step)
            .ok_or("editing_latest_step_required")?;
        let graph = owner.owner.graph.clone();
        let chords = owner.chords.clone();
        let mut sequence = self.clone();
        let result = (|| {
            let tasks: Box<dyn Iterator<Item = &mut EditingTask> + '_> =
                if matches!(operation, EditingOperation::Undo) {
                    Box::new(sequence.tasks.iter_mut().rev())
                } else {
                    Box::new(sequence.tasks.iter_mut())
                };
            for task in tasks {
                match task {
                    EditingTask::Relation(t) => t.perform_owned(owner, operation)?,
                    EditingTask::Inter(t) => {
                        t.perform(&mut owner.owner.graph, operation)?;
                    }
                }
            }
            sequence.notify_impact(&mut owner.owner.graph, operation, latest_step, latest)
        })();
        match result {
            Ok(notification) => {
                *self = sequence;
                Ok(notification)
            }
            Err(error) => {
                owner.owner.graph = graph;
                owner.chords.clone_from(&chords);
                Err(error)
            }
        }
    }
    fn notify_impact(
        &self,
        next: &mut Graph,
        operation: EditingOperation,
        latest_step: &str,
        latest: usize,
    ) -> Result<Value, String> {
        if matches!(operation, EditingOperation::Do) {
            next.mark_controller_sheet_modified();
        }
        // Compared dot tasks and ordinary HeadStem LinkTasks first affect RHYTHMS.
        // Other relation classes need their original impact boundary.
        let first =
            (!self.tasks.is_empty()).then(|| STEPS.iter().position(|s| *s == "RHYTHMS").unwrap());
        let planned = first
            .filter(|first| *first <= latest)
            .map_or_else(Vec::new, |first| STEPS[first..=latest].to_vec());
        let mut stack_targets = Vec::new();
        if planned.contains(&"RHYTHMS") {
            for task in &self.tasks {
                // Impact scans the original task list even on undo; relations
                // insert source then target into a LinkedHashSet.
                for inter in task.impact_inters() {
                    if let Some(target) = next.inter_edit_stack_target(inter)?
                        && !stack_targets.contains(&target)
                    {
                        stack_targets.push(target);
                    }
                }
            }
        }
        let notification = json!({
            "operation":operation.name(), "stage":"after_sequence_publications",
            "inter_ids":self.tasks.iter().map(|t|next.id(t.focus()).unwrap()).collect::<Vec<_>>(),
            "mark_sheet_modified":matches!(operation,EditingOperation::Do),
            "first_impacted_step":first.map(|i|STEPS[i]), "latest_completed_step":latest_step,
            "planned_steps":planned, "rhythm_stack_targets":stack_targets, "reprocessed_steps":[],
            "actual_dirty_state":next.editing_book_state(),"project_persistence_complete":false,
            "controller_impact_complete":false
        });
        next.edit_impacts.push(notification.clone());
        Ok(notification)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_book_dirty_callbacks_and_controller_requests_match_original_intermediates() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-editing-book-dirty/observed.json"
        ))
        .unwrap();
        let image = crate::GrayImage::decode(include_bytes!(
            "../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let export = crate::music_xml::recognize(&image).unwrap();
        let mut graph = export.owner.owner.graph.clone();
        let initial = graph.snapshot();
        let dot = initial["inters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "AugmentationDotInter")
            .min_by_key(|n| n["bounds"][0].as_i64())
            .unwrap();
        let id = dot["id"].as_u64().unwrap() as usize;
        let key: Key = serde_json::from_value(dot["origin"].clone()).unwrap();
        let head = initial["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == oracle["target_chord_bounds"])
            .unwrap()["member_ids"][0]
            .as_u64()
            .unwrap() as usize;
        let clean = |g: &mut Graph| {
            let b = g.editing_book.as_mut().unwrap();
            b.set_stub_modified(false);
            b.set_modified(false);
            b.set_dirty(false);
        };
        let compare = |g: &Graph, expected: &Value| {
            let actual = g.editing_book_state().unwrap();
            for field in [
                "sheet_modified",
                "book_modified",
                "book_modified_field",
                "book_dirty",
            ] {
                assert_eq!(actual[field], expected[field], "{field}");
            }
            assert_eq!(
                g.vertices[id - 1].data["abnormal"],
                expected["dot_abnormal"]
            );
            assert_eq!(actual["system_ids"], json!([key.system]));
            assert_eq!(actual["project_persistence_complete"], false);
        };
        for step in oracle["setters"].as_array().unwrap() {
            let b = graph.editing_book.as_mut().unwrap();
            match step["operation"].as_str().unwrap() {
                "clean" => clean(&mut graph),
                "stub_true" => b.set_stub_modified(true),
                "stub_false" => b.set_stub_modified(false),
                "book_false" => b.set_modified(false),
                "dirty_false" => b.set_dirty(false),
                "book_false_with_stub_true" => {
                    b.set_stub_modified(true);
                    b.set_modified(false);
                }
                _ => unreachable!(),
            }
            compare(&graph, &step["state"]);
        }
        let mut bounds: [i64; 4] = serde_json::from_value(dot["bounds"].clone()).unwrap();
        bounds[0] += 3;
        let mut sequence = export.dot_relation_transfer(id, head, true).unwrap();
        sequence.prepend(EditingTask::Inter(
            DefaultEditingTask::new(&graph, key, bounds, vec![], vec![]).unwrap(),
        ));
        for (i, op) in [
            EditingOperation::Do,
            EditingOperation::Undo,
            EditingOperation::Redo,
        ]
        .into_iter()
        .enumerate()
        {
            clean(&mut graph);
            graph.enable_dot_callback_trace();
            let original = &oracle["operations"][i];
            compare(&graph, &original["states"][0]["state"]);
            let indexes = if matches!(op, EditingOperation::Undo) {
                vec![2, 1, 0]
            } else {
                vec![0, 1, 2]
            };
            for (j, index) in indexes.into_iter().enumerate() {
                sequence.tasks[index].perform(&mut graph, op).unwrap();
                compare(&graph, &original["states"][j + 1]["state"]);
            }
            let callbacks = graph
                .dot_callback_trace()
                .iter()
                .filter(|e| e["event"] != "publish")
                .collect::<Vec<_>>();
            assert_eq!(
                callbacks.len(),
                original["callbacks"].as_array().unwrap().len()
            );
            for (a, b) in callbacks
                .into_iter()
                .zip(original["callbacks"].as_array().unwrap())
            {
                for f in ["event", "stage"] {
                    assert_eq!(a[f], b[f]);
                }
                for f in [
                    "sheet_modified",
                    "book_modified",
                    "book_modified_field",
                    "book_dirty",
                ] {
                    assert_eq!(a["state"]["editing_book"][f], b["state"][f]);
                }
            }
            let notice = sequence
                .notify_impact(&mut graph, op, "RHYTHMS", 18)
                .unwrap();
            compare(&graph, &original["states"][4]["state"]);
            assert_eq!(
                notice["mark_sheet_modified"],
                matches!(op, EditingOperation::Do)
            );
            assert_eq!(notice["actual_dirty_state"]["sheet_modified"], true);
            assert_eq!(notice["controller_impact_complete"], false);
        }
        clean(&mut graph);
        graph.set_owned_abnormal(id, false);
        compare(&graph, &oracle["stable"][0]["state"]);
        sequence.tasks[2]
            .perform(&mut graph, EditingOperation::Do)
            .unwrap();
        compare(&graph, &oracle["stable"][1]["state"]);
        let notice = sequence
            .notify_impact(&mut graph, EditingOperation::Do, "HEADS", 8)
            .unwrap();
        compare(&graph, &oracle["stable"][2]["state"]);
        assert_eq!(notice["planned_steps"], json!([]));
    }
    #[test]
    fn actual_relation_task_replay_publishes_without_callback_and_missing_remove_refuses() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-relation-tasks/replay.json"
        ))
        .unwrap();
        let image = crate::GrayImage::decode(include_bytes!(
            "../../fixtures/official-image-musicxml-augmentation/double-augmentation.png"
        ))
        .unwrap();
        let mut export = crate::music_xml::recognize(&image).unwrap();
        let state = export.owner.owner.graph.snapshot();
        let edge = state["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "AugmentationRelation")
            .unwrap();
        let dot = edge["source"].as_u64().unwrap() as usize;
        let target = state["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "HeadChordInter" && n["bounds"] == oracle["target_chord_bounds"])
            .unwrap()["member_ids"][0]
            .as_u64()
            .unwrap() as usize;
        let mut sequence = export.dot_relation_transfer(dot, target, true).unwrap();
        export
            .apply_editing_sequence(&mut sequence, EditingOperation::Do)
            .unwrap();
        let xml = export.xml.clone();
        let mut repeated = EditingSequence::from_tasks(vec![sequence.tasks[1].clone()]);
        export.owner.owner.graph.enable_dot_callback_trace();
        export
            .apply_editing_sequence(&mut repeated, EditingOperation::Do)
            .unwrap();
        assert_eq!(export.xml, xml);
        let graph = &export.owner.owner.graph;
        assert_eq!(graph.dot_callback_trace().len(), 1);
        assert_eq!(graph.dot_callback_trace()[0]["event"], "publish");
        assert_eq!(oracle["repeat_link_do"]["callback_events"], 0);
        assert_eq!(oracle["repeat_link_do"]["publication_count"], 1);
        assert_eq!(oracle["repeat_link_do"]["same_relation_registered"], true);
        assert_eq!(oracle["repeat_link_do"]["same_task_relation"], true);
        let before = graph.snapshot();
        let mut missing = EditingSequence::from_tasks(vec![sequence.tasks[0].clone()]);
        assert_eq!(
            export
                .apply_editing_sequence(&mut missing, EditingOperation::Do)
                .unwrap_err(),
            "editing_live_relation_required"
        );
        assert!(
            oracle["repeat_unlink_do"]["original_exception"]
                .as_str()
                .unwrap()
                .starts_with("IllegalArgumentException:no such edge in graph:")
        );
        assert_eq!(export.owner.owner.graph.snapshot(), before);
        assert_eq!(export.xml, xml);
        // Declared relation setter control on actual retained objects, not a
        // report/provenance change or a new image correction admission.
        let mut graph = export.owner.owner.graph.clone();
        let old = edge["id"].as_u64().unwrap() as usize;
        let new = match &sequence.tasks[1] {
            EditingTask::Relation(t) => t.link.edge.unwrap(),
            _ => unreachable!(),
        };
        graph.edges[old - 1].data["grade"] = json!(0.31);
        graph.edges[new - 1].data["grade"] = json!(0.81);
        repeated
            .perform(&mut graph, EditingOperation::Do, "PAGE")
            .unwrap();
        assert_eq!(
            graph.edges[new - 1].data["grade"],
            oracle["repeat_link_do"]["live_grade"]
        );
        sequence
            .perform(&mut graph, EditingOperation::Undo, "PAGE")
            .unwrap();
        assert_eq!(
            graph.edges[old - 1].data["grade"],
            oracle["undo"]["old_grade"]
        );
        assert_ne!(graph.edges[old - 1].data["manual"], true);
        let source = graph.vertices[graph.edges[old - 1].key.target - 1].key;
        let owner = graph
            .live_edges()
            .into_iter()
            .find(|e| e.key.kind == "Containment" && graph.vertices[e.key.target - 1].key == source)
            .unwrap()
            .key
            .source;
        assert_eq!(
            graph.chord_duration(graph.vertices[owner - 1].key).unwrap(),
            Some("1/4".into())
        );
    }
}
