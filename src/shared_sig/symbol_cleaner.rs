// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Key};
use std::collections::BTreeSet;
impl Graph {
    pub(crate) fn symbol_cleaner_visits(&self) -> Result<Vec<(Key, bool)>, String> {
        let systems = self
            .vertices
            .iter()
            .filter(|v| !v.removed)
            .map(|v| v.key.system)
            .collect::<BTreeSet<_>>();
        let mut visits = Vec::new();
        for system in systems {
            let mut strong = Vec::new();
            let mut weak = Vec::new();
            for (index, vertex) in self
                .vertices
                .iter()
                .enumerate()
                .filter(|(_, v)| !v.removed && v.key.system == system)
            {
                let kind = vertex.data["kind"]
                    .as_str()
                    .ok_or("music_cleaner_actual_kind_required")?;
                // BeamGroup's visitor has no raster operation or optional pixels.
                if kind == "BeamGroupInter" {
                    continue;
                }
                let frozen = vertex.data["frozen"] == true || vertex.data["is_frozen"] == true;
                let ensemble = self.edges.iter().any(|e| {
                    !e.removed
                        && e.key.kind == "Containment"
                        && e.key.target == index + 1
                        && !self.vertices[e.key.source - 1].removed
                });
                if !frozen && ensemble && !matches!(kind, "BeamInter" | "BeamHookInter") {
                    continue;
                }
                let threshold = match kind {
                    "HeadInter" => 0.6,
                    "StemInter" => 0.7,
                    _ => 0.4,
                };
                let can_hide = frozen
                    || vertex.data["contextual_grade"]
                        .as_f64()
                        .or(vertex.data["grade"].as_f64())
                        .filter(|g| g.is_finite() && (0.0..=1.0).contains(g))
                        .ok_or("music_cleaner_actual_best_grade_required")?
                        >= threshold;
                if can_hide {
                    strong.push(index + 1);
                } else {
                    weak.push(index + 1);
                }
            }
            for (ids, weak) in [(strong, false), (weak, true)] {
                for id in ids {
                    let vertex = &self.vertices[id - 1];
                    if vertex.data["kind"] == "HeadChordInter" {
                        let mut members = Vec::new();
                        for edge in self.edges.iter().filter(|e| {
                            !e.removed && e.key.kind == "Containment" && e.key.source == id
                        }) {
                            self.validate_chord_identity(id, edge.key.target, "Containment")?;
                            let member = &self.vertices[edge.key.target - 1];
                            if member.removed {
                                return Err("music_cleaner_live_chord_member_required".into());
                            }
                            members.push(&member.data);
                        }
                        for member in crate::head_chord_geometry::ordered(&members)? {
                            let id = member["id"]
                                .as_u64()
                                .ok_or("music_cleaner_member_identity_required")?
                                as usize;
                            visits.push((self.vertices[id - 1].key, weak));
                        }
                    } else {
                        visits.push((vertex.key, weak));
                    }
                }
            }
        }
        Ok(visits)
    }
}
