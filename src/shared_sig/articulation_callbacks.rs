// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
impl Graph {
    pub(super) fn validate_chord_articulation(
        &self,
        source: usize,
        target: usize,
        data: &Value,
    ) -> Result<(), String> {
        let chord = &self.vertices[source - 1];
        let articulation = &self.vertices[target - 1];
        if chord.key.phase != Phase::Chord
            || chord.data["kind"] != "HeadChordInter"
            || articulation.key.phase != Phase::Symbol
            || articulation.data["kind"] != "ArticulationInter"
            || chord.key.system != articulation.key.system
            || data["source_ratio"] != 1.0
            || !data["grade"]
                .as_f64()
                .is_some_and(|g| (0.0..=1.0).contains(&g))
            || !data["target_ratio"]
                .as_f64()
                .is_some_and(|r| (r - (1. + 2.5 * data["grade"].as_f64().unwrap())).abs() < 1e-12)
        {
            return Err("music_articulation_typed_chord_support_required".into());
        }
        Ok(())
    }
    pub(super) fn refresh_articulation_abnormal(&mut self, target: usize) {
        if !self.vertices[target - 1].removed
            && self.vertices[target - 1].data["kind"] == "ArticulationInter"
        {
            let linked = self.edges.iter().any(|e| {
                !e.removed && e.key.kind == "ChordArticulationRelation" && e.key.target == target
            });
            self.vertices[target - 1].data["abnormal"] = json!(!linked);
        }
    }
    pub(super) fn remove_articulation_edges_ordered(&mut self, ids: &[usize]) {
        for &id in ids {
            let target = self.edges[id - 1].key.target;
            self.edges[id - 1].removed = true;
            self.refresh_articulation_abnormal(target);
        }
    }
    #[cfg(test)]
    pub(crate) fn remove_articulation_links_observed(&mut self, key: Key) -> Vec<Value> {
        let target = self.id(key).unwrap();
        let ids = self
            .live_edges()
            .iter()
            .filter(|e| e.key.kind == "ChordArticulationRelation" && e.key.target == target)
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let mut states = Vec::new();
        for id in ids {
            self.remove_articulation_edges_ordered(&[id]);
            states.push(self.vertices[target - 1].data.clone());
        }
        states
    }
}
