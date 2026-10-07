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
        if graph
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
