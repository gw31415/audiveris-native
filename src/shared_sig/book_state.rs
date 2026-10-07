// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Graph;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Book/SheetStub state for the retained single-image editing owner. Initial
/// recognition and persisted project state are not reconstructed from reports.
#[derive(Clone, Debug)]
pub(super) struct BookState {
    modified: bool,
    dirty: bool,
    stub: SheetStub,
}
#[derive(Clone, Debug)]
struct SheetStub {
    number: usize,
    systems: BTreeSet<u64>,
    modified: bool,
}
impl BookState {
    fn new(systems: BTreeSet<u64>) -> Self {
        Self {
            modified: false,
            dirty: false,
            stub: SheetStub {
                number: 1,
                systems,
                modified: false,
            },
        }
    }
    pub(super) fn set_stub_modified(&mut self, modified: bool) {
        self.stub.modified = modified;
        if modified {
            self.set_modified(true);
            self.set_dirty(true);
        }
    }
    pub(super) fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }
    pub(super) fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
    fn snapshot(&self) -> Value {
        json!({"book_modified":self.modified || self.stub.modified,
            "book_modified_field":self.modified,"book_dirty":self.dirty,
            "sheet_modified":self.stub.modified,"sheet_number":self.stub.number,
            "system_ids":self.stub.systems,"sample_repository_present":false,
            "scope":"in-memory retained single-image editing owner; tracking begins after initial recognition/rhythm, not a persisted project baseline",
            "project_persistence_complete":false,"all_dirty_sources_complete":false})
    }
}
impl Graph {
    pub(crate) fn bind_editing_book(&mut self, systems: BTreeSet<u64>) -> Result<(), String> {
        if systems.is_empty()
            || systems.contains(&0)
            || systems.iter().any(|s| {
                !self
                    .vertices
                    .iter()
                    .any(|v| v.key.system == *s && !v.removed)
            })
        {
            return Err("editing_book_actual_system_owner_required".into());
        }
        if let Some(book) = &self.editing_book {
            if book.stub.systems != systems {
                return Err("editing_book_owner_rebinding_pending".into());
            }
        } else {
            self.editing_book = Some(BookState::new(systems));
        }
        Ok(())
    }
    pub fn editing_book_state(&self) -> Option<Value> {
        self.editing_book.as_ref().map(BookState::snapshot)
    }
    pub(super) fn mark_editing_sheet_modified(&mut self, system: u64) {
        if let Some(book) = &mut self.editing_book
            && book.stub.systems.contains(&system)
        {
            book.set_stub_modified(true);
        }
    }
    pub(super) fn mark_controller_sheet_modified(&mut self) {
        if let Some(book) = &mut self.editing_book {
            book.set_stub_modified(true);
        }
    }
    pub(super) fn set_owned_abnormal(&mut self, id: usize, abnormal: bool) {
        let v = &mut self.vertices[id - 1];
        let changed = v.data["abnormal"].as_bool().unwrap_or(false) != abnormal;
        v.data["abnormal"] = json!(abnormal);
        let system = v.key.system;
        if changed {
            self.mark_editing_sheet_modified(system);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_sig::{Key, Phase};
    #[test]
    fn retained_cleaner_callbacks_observe_each_edge_and_dirty_actual_owner() {
        let original: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-head-stem-dirty/observed.json"
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
        let head = nodes
            .iter()
            .find(|v| v["kind"] == "HeadInter" && v["bounds"] == original["head_bounds"])
            .unwrap();
        let hid = head["id"].as_u64().unwrap() as usize;
        let hk: Key = serde_json::from_value(head["origin"].clone()).unwrap();
        // Saved original stems and native glyphs have different extents. Match
        // actual corresponding ownership via heads, without claiming glyph parity.
        let stems = original["before"]["stems"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let owner_bounds = if i == 0 {
                    &original["head_bounds"]
                } else {
                    s["linked_head_bounds"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|b| **b != original["head_bounds"])
                        .unwrap()
                };
                let owner = nodes
                    .iter()
                    .find(|v| v["kind"] == "HeadInter" && v["bounds"] == *owner_bounds)
                    .unwrap();
                let edge = initial["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["kind"] == "HeadStemRelation" && e["source"] == owner["id"])
                    .unwrap();
                nodes.iter().find(|v| v["id"] == edge["target"]).unwrap()
            })
            .collect::<Vec<_>>();
        let ids = stems
            .iter()
            .map(|v| v["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let old = initial["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| {
                e["kind"] == "HeadStemRelation" && e["source"] == hid && e["target"] == ids[0]
            })
            .unwrap();
        let mut extra = old.clone();
        extra["local_relation_id"] = json!(1_000_001);
        extra["grade"] = json!(0.7);
        g.relations(
            hk.system,
            Phase::Stem,
            vec![(
                hk,
                serde_json::from_value(stems[1]["origin"].clone()).unwrap(),
                extra,
            )],
        )
        .unwrap();
        let snapshot = g.snapshot();
        let extra_id = snapshot["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| {
                e["kind"] == "HeadStemRelation" && e["source"] == hid && e["target"] == ids[1]
            })
            .unwrap()["id"]
            .as_u64()
            .unwrap() as usize;
        let compare = |state: &Value, expect: &Value| {
            let node = |id| {
                state["inters"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["id"] == id)
                    .unwrap()
            };
            assert_eq!(node(hid)["abnormal"], expect["head_abnormal"]);
            for (i, id) in ids.iter().enumerate() {
                assert_eq!(node(*id)["abnormal"], expect["stems"][i]["abnormal"]);
            }
            for f in [
                "sheet_modified",
                "book_modified",
                "book_modified_field",
                "book_dirty",
            ] {
                assert_eq!(state["editing_book"][f], expect[f], "{f}");
            }
        };
        compare(&snapshot, &original["before"]);
        g.enable_dot_callback_trace();
        g.remove_head_stem_edges(hk, &[old["id"].as_u64().unwrap() as usize, extra_id])
            .unwrap();
        for (native, java) in g
            .dot_callback_trace()
            .iter()
            .zip(original["callbacks"].as_array().unwrap())
        {
            assert_eq!(native["event"], java["event"]);
            assert_eq!(native["stage"], java["stage"]);
            compare(&native["state"], &java["state"]);
        }
        assert_eq!(
            g.dot_callback_trace().len(),
            original["callbacks"].as_array().unwrap().len()
        );
        let mut after = g.snapshot();
        compare(&after, &original["last"]);
        for v in after["inters"].as_array_mut().unwrap() {
            let id = v["id"].as_u64().unwrap();
            if id == hid as u64 || ids.contains(&(id as usize)) {
                v["abnormal"] = nodes.iter().find(|v| v["id"] == id).unwrap()["abnormal"].clone();
            }
        }
        assert_eq!(after["inters"], initial["inters"]);
        assert_eq!(
            after["augmentation_dot_ownership"],
            initial["augmentation_dot_ownership"]
        );
        let b = g.editing_book.as_mut().unwrap();
        b.set_stub_modified(false);
        b.set_modified(false);
        b.set_dirty(false);
        g.refresh_head_stem_abnormal(&[hid, ids[0], ids[1]]);
        compare(&g.snapshot(), &original["same_value_after_clean"]);
    }
}
