// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
impl Graph {
    pub(super) fn validate_alter_head(
        &self,
        source: usize,
        target: usize,
        data: &Value,
    ) -> Result<(), String> {
        let alter = &self.vertices[source - 1];
        let head = &self.vertices[target - 1];
        if alter.key.phase != Phase::Symbol
            || alter.data["kind"] != "AlterInter"
            || head.key.phase != Phase::Head
            || head.data["kind"] != "HeadInter"
            || alter.key.system != head.key.system
            || data["target_ratio"] != 1.0
            || !data["grade"]
                .as_f64()
                .is_some_and(|g| (0.0..=1.0).contains(&g))
            || !data["source_ratio"]
                .as_f64()
                .is_some_and(|r| (r - (1. + 3. * data["grade"].as_f64().unwrap())).abs() < 1e-12)
        {
            return Err("music_alter_typed_head_support_required".into());
        }
        Ok(())
    }
    pub(super) fn refresh_alter_abnormal(&mut self, sources: &[usize]) {
        for &source in sources {
            if !self.vertices[source - 1].removed
                && self.vertices[source - 1].data["kind"] == "AlterInter"
            {
                let linked = self.edges.iter().any(|e| {
                    !e.removed && e.key.kind == "AlterHeadRelation" && e.key.source == source
                });
                self.vertices[source - 1].data["abnormal"] = json!(!linked);
            }
        }
    }
    pub(super) fn remove_alter_edges_ordered(&mut self, ids: &[usize]) {
        for &id in ids {
            let source = self.edges[id - 1].key.source;
            self.edges[id - 1].removed = true;
            self.refresh_alter_abnormal(&[source]);
        }
    }
    #[cfg(test)]
    pub(crate) fn remove_alter_links_observed(&mut self, key: Key) -> Vec<Value> {
        let source = self.id(key).unwrap();
        let ids = self
            .live_edges()
            .iter()
            .filter(|e| e.key.source == source && e.key.kind == "AlterHeadRelation")
            .map(|e| e.data["id"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let mut states = Vec::new();
        for id in ids {
            self.remove_alter_edges_ordered(&[id]);
            states.push(self.vertices[source - 1].data.clone());
        }
        states
    }
}
