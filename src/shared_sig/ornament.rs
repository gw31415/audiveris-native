// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Graph, Phase};
use serde_json::Value;

impl Graph {
    pub(super) fn validate_chord_ornament(
        &self,
        source: usize,
        target: usize,
        data: &Value,
    ) -> Result<(), String> {
        let chord = &self.vertices[source - 1];
        let ornament = &self.vertices[target - 1];
        if chord.key.phase != Phase::Chord
            || chord.data["kind"] != "HeadChordInter"
            || ornament.key.phase != Phase::Symbol
            || ornament.data["kind"] != "OrnamentInter"
            || chord.key.system != ornament.key.system
            || data["source_ratio"] != 1.0
            || !data["grade"]
                .as_f64()
                .is_some_and(|g| (0.0..=1.0).contains(&g))
            || !data["target_ratio"]
                .as_f64()
                .is_some_and(|r| (r - (1. + 0.5 * data["grade"].as_f64().unwrap())).abs() < 1e-12)
        {
            return Err("music_ornament_typed_chord_support_required".into());
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn retained_ornament_relation(&self, id: usize) -> Result<(&Value, bool), String> {
        let edge = self
            .edges
            .get(
                id.checked_sub(1)
                    .ok_or("music_ornament_relation_required")?,
            )
            .ok_or("music_ornament_relation_required")?;
        if edge.key.kind != "ChordOrnamentRelation" {
            return Err("music_ornament_relation_required".into());
        }
        Ok((&edge.data, edge.removed))
    }
}
