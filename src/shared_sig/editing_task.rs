// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key, NoteOwnerEdit, Phase};
use serde_json::{Value, json};

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
        if graph.vertices[source - 1].removed || graph.vertices[target - 1].removed {
            return Ok(false);
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
        let publication = json!({"inter_id":id,"operation":operation.name(),"phase":"after_task","controller_impact_complete":false});
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
pub struct RelationTask {
    source: Key,
    link: EditingLink,
    unlink: bool,
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
        })
    }
    fn perform(&mut self, graph: &mut Graph, operation: EditingOperation) -> Result<(), String> {
        let adding = self.unlink == matches!(operation, EditingOperation::Undo);
        if adding {
            self.link.apply_relation(graph, self.source, false)?;
            let id = graph.endpoint(self.source)?;
            graph
                .edit_publications
                .push(json!({"inter_id":id,"operation":operation.name(),
                "phase":"after_task","task":if self.unlink {"UnlinkTask"}else{"LinkTask"},
                "controller_impact_complete":false}));
            #[cfg(test)]
            graph.dot_callback_event("publish", id, "delivered");
        } else {
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
                graph.chord_duration(chord.key)?;
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
        // Dot InterTasks and retained augmentation RelationTasks first affect
        // RHYTHMS. Other relation classes need their original impact boundary.
        let first = (!sequence.tasks.is_empty())
            .then(|| STEPS.iter().position(|s| *s == "RHYTHMS").unwrap());
        let planned = first
            .filter(|first| *first <= latest)
            .map_or_else(Vec::new, |first| STEPS[first..=latest].to_vec());
        let mut stack_targets = Vec::new();
        if planned.contains(&"RHYTHMS") {
            for task in &sequence.tasks {
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
            "inter_ids":sequence.tasks.iter().map(|t|next.id(t.focus()).unwrap()).collect::<Vec<_>>(),
            "mark_sheet_modified":matches!(operation,EditingOperation::Do),
            "first_impacted_step":first.map(|i|STEPS[i]), "latest_completed_step":latest_step,
            "planned_steps":planned, "rhythm_stack_targets":stack_targets, "reprocessed_steps":[],
            "controller_impact_complete":false
        });
        next.edit_impacts.push(notification.clone());
        *graph = next;
        *self = sequence;
        Ok(notification)
    }
}
