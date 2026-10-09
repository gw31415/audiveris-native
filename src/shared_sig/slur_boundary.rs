// SPDX-License-Identifier: AGPL-3.0-or-later
use super::Graph;

impl Graph {
    pub(crate) fn has_live_slur_material(&self) -> bool {
        self.vertices
            .iter()
            .any(|vertex| !vertex.removed && vertex.data["kind"] == "SlurInter")
            || self
                .edges
                .iter()
                .any(|edge| !edge.removed && edge.key.kind == "SlurHeadRelation")
    }
}
