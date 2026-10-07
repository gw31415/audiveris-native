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
