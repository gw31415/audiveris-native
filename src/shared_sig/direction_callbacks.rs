// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
impl Graph {
    pub(super) fn validate_chord_direction(
        &self,
        source: usize,
        target: usize,
        data: &Value,
    ) -> Result<(), String> {
        let chord = &self.vertices[source - 1];
        let direction = &self.vertices[target - 1];
        let dynamics = data["kind"] == "ChordDynamicsRelation";
        let expected = if dynamics {
            "DynamicsInter"
        } else {
            "PedalInter"
        };
        let coeff = if dynamics { 2.5 } else { 0. };
        if chord.key.phase != Phase::Chord
            || !matches!(
                chord.data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
            || direction.key.phase != Phase::Symbol
            || direction.data["kind"] != expected
            || chord.key.system != direction.key.system
            || data["source_ratio"] != 1.
            || !data["grade"]
                .as_f64()
                .is_some_and(|g| (0.0..=1.0).contains(&g))
            || !data["target_ratio"]
                .as_f64()
                .is_some_and(|v| (v - (1. + coeff * data["grade"].as_f64().unwrap())).abs() < 1e-12)
        {
            return Err("music_direction_typed_Chord_support_required".into());
        }
        Ok(())
    }
    pub(super) fn refresh_pedal_abnormal(&mut self, target: usize) {
        if !self.vertices[target - 1].removed
            && self.vertices[target - 1].data["kind"] == "PedalInter"
        {
            let linked = self.edges.iter().any(|e| {
                !e.removed && e.key.kind == "ChordPedalRelation" && e.key.target == target
            });
            self.vertices[target - 1].data["abnormal"] = json!(!linked);
        }
    }
    pub(super) fn remove_pedal_edges_ordered(&mut self, ids: &[usize]) {
        for &id in ids {
            let target = self.edges[id - 1].key.target;
            self.edges[id - 1].removed = true;
            self.refresh_pedal_abnormal(target);
        }
    }
}

#[cfg(test)]
impl Graph {
    pub(crate) fn observe_actual_pedal_callbacks(&self) -> Result<Value, String> {
        let mut g = self.clone();
        let edge = g
            .live_edges()
            .into_iter()
            .find(|e| e.key.kind == "ChordPedalRelation")
            .ok_or("test_actual_pedal_relation_required")?
            .clone();
        let cid = edge.key.source;
        let pid = edge.key.target;
        let relation_id = edge.data["id"].as_u64().unwrap() as usize;
        let chord = g.vertices[cid - 1].key;
        let pedal = g.vertices[pid - 1].key;
        let baseline = g.vertices[cid - 1].data.clone();
        let voice = g.vertices[cid - 1].voice_owner.clone();
        let slot = g.vertices[cid - 1].slot_owner.clone();
        let read = |g: &Graph| json!({"abnormal":g.vertices[pid-1].data["abnormal"],"manual":g.vertices[pid-1].data["manual"],"relation_manual":g.edges[relation_id-1].data["manual"],"relation_present":!g.edges[relation_id-1].removed,"same_Chord_owner":g.edges.iter().any(|e|!e.removed&&e.key.kind=="ChordPedalRelation"&&e.key.source==cid&&e.key.target==pid),"Chord_voice":g.vertices[cid-1].data["voice"],"Chord_time":g.vertices[cid-1].data["time_offset"],"Chord_duration":g.vertices[cid-1].data["duration"]});
        let mut stages = vec![read(&g)];
        g.remove_pedal_edges_ordered(&[relation_id]);
        stages.push(read(&g));
        let mut body = edge.data.clone();
        body["manual"] = json!(true);
        g.relations(chord.system, edge.key.phase, vec![(chord, pedal, body)])?;
        stages.push(read(&g));
        if baseline != g.vertices[cid - 1].data
            || voice
                .as_ref()
                .zip(g.vertices[cid - 1].voice_owner.clone().as_ref())
                .is_none_or(|(a, b)| !std::sync::Arc::ptr_eq(a, b))
            || slot
                .as_ref()
                .zip(g.vertices[cid - 1].slot_owner.clone().as_ref())
                .is_none_or(|(a, b)| !std::sync::Arc::ptr_eq(a, b))
            || g.edge_keys.get(&edge.key) != Some(&relation_id)
        {
            return Err("test_actual_pedal_owner_identity_changed".into());
        }
        let before = g.snapshot();
        let head = g
            .vertices
            .iter()
            .find(|n| !n.removed && n.key.system == chord.system && n.key.phase == Phase::Head)
            .unwrap()
            .key;
        let body = g.edges[relation_id - 1].data.clone();
        assert!(
            g.relations(
                chord.system,
                Phase::Symbol,
                vec![(chord, pedal, body.clone()), (head, pedal, body)]
            )
            .is_err()
        );
        assert_eq!(g.snapshot(), before);
        // Extensive Chord removal invokes removed only after its dying source;
        // the surviving Pedal must become abnormal and keep its own identity.
        g.remove_music_inter(chord)?;
        assert_eq!(g.vertices[pid - 1].data["abnormal"], true);
        assert_eq!(g.vertices[pid - 1].data["manual"], false);
        Ok(
            json!({"stages":stages,"actual_identity_retained":true,"invalid_late_typed_support_rolled_back_graph_and_clocks":true,"surviving_Pedal_abnormal_after_Chord_removal":true}),
        )
    }
}
