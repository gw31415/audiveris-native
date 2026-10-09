// SPDX-License-Identifier: AGPL-3.0-or-later
//! Existing AnLaura target only; original raw Part and edge callbacks were not observed.
use super::*;
use serde_json::{Value, json};

impl Graph {
    pub(crate) fn assert_owned_flag_family_removal(
        &self,
        before: &Graph,
        report: &Value,
        weak_pass: bool,
    ) -> Value {
        let before_flags = before
            .vertices
            .iter()
            .filter(|v| !v.removed && v.data["kind"] == "FlagInter")
            .collect::<Vec<_>>();
        assert!(!before_flags.is_empty());
        for vertex in &before_flags {
            assert!(vertex.data["staff_id"].is_null());
            assert!(before.requires_owned_flag_removal(vertex.key));
            assert!(self.requires_owned_flag_removal(vertex.key));
            assert!(self.retained_inter(vertex.key).unwrap().0["staff_id"].is_null());
        }
        let removed = report[if weak_pass {
            "removed"
        } else {
            "removed_conflicting_flags"
        }]
        .as_array()
        .unwrap();
        let events = report["lifecycle"].as_array().unwrap();
        let mut observed_edges = BTreeSet::new();
        let mut offset = 0;
        let mut rows = Vec::new();
        for id in removed {
            let id = id.as_u64().unwrap() as usize;
            let original = &before.vertices[id - 1];
            let key = original.key;
            assert_eq!(original.data["kind"], "FlagInter");
            let expected = before
                .incident_edges(id)
                .into_iter()
                .filter(|eid| !observed_edges.contains(eid))
                .collect::<Vec<_>>();
            let callbacks = &events[offset..offset + expected.len() + 2];
            assert_eq!(callbacks[0]["stage"], "ContainerLookupComplete");
            assert_eq!(callbacks.last().unwrap()["stage"], "VertexRemoved");
            assert!(
                callbacks
                    .iter()
                    .all(|e| e["key"] == json!(key) && e["inter_id"] == id)
            );
            assert!(callbacks[0]["relation_id"].is_null());
            assert!(callbacks.last().unwrap()["relation_id"].is_null());
            for (event, eid) in callbacks[1..callbacks.len() - 1].iter().zip(&expected) {
                assert_eq!(event["stage"], "EdgeRemoved");
                assert_eq!(event["relation_id"], *eid);
                assert!(self.edges[*eid - 1].removed);
                assert_eq!(self.edges[*eid - 1].data, before.edges[*eid - 1].data);
                observed_edges.insert(*eid);
            }
            let (body, removed) = self.retained_inter(key).unwrap();
            assert!(removed);
            assert_eq!(body["manual"], original.data["manual"]);
            assert_eq!(body["abnormal"], original.data["abnormal"]);
            rows.push(
                json!({"system":key.system,"shape":body["shape"],"bounds":body["bounds"],
                "raw_staff":body["staff_id"],"removed":true,"registered":false,
                "native_inter_id":id,"native_relation_ids":expected}),
            );
            offset += callbacks.len();
        }
        assert_eq!(offset, events.len());
        // Collect actual native state before looking at any original observation.
        let descriptor = |body: &Value| json!({"shape":body["shape"],"bounds":body["bounds"],"raw_staff":body["staff_id"]});
        let mut native_before = before_flags
            .iter()
            .filter(|v| v.key.system == 2)
            .map(|v| descriptor(&v.data))
            .collect::<Vec<_>>();
        native_before.sort_by_key(Value::to_string);
        let native_weak=rows.iter().filter(|r|r["system"]==2).map(|r|json!({"shape":r["shape"],"bounds":r["bounds"],"raw_staff":r["raw_staff"],"removed":r["removed"],"registered":r["registered"]})).collect::<Vec<_>>();
        let source: Value = serde_json::from_str(include_str!(
            "../../../../fixtures/official-flag-owned-lifecycle/saved-observations.json"
        ))
        .unwrap();
        let stage = if weak_pass {
            "before_reducer"
        } else {
            "first_weak_purge"
        };
        let frame = source["frames"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["stage"] == stage)
            .unwrap();
        let mut source_before = frame["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| json!({"shape":r["shape"],"bounds":r["bounds"],"raw_staff":r["raw_staff"]}))
            .collect::<Vec<_>>();
        source_before.sort_by_key(Value::to_string);
        assert_eq!(native_before, source_before);
        if weak_pass {
            let observed=source["events"].as_array().unwrap().iter().map(|e| {
                assert_eq!(e["event"],"vertexRemoved");
                let r=&e["inter"];
                json!({"shape":r["shape"],"bounds":r["bounds"],"raw_staff":r["raw_staff"],"removed":r["removed"],"registered":r["registered"]})
            }).collect::<Vec<_>>();
            assert_eq!(native_weak, observed);
        }
        json!({"family_pass":if weak_pass {"weak"} else {"exclusion"},"native_removed":rows,
            "actual_factory_registration_and_raw_null_Staff_retained":true,
            "actual_incident_edge_order_and_manual_relation_identity_retained":true,
            "source_System2_raw_Staff_membership_matches":true,
            "source_weak_vertex_subsequence_compared":weak_pass,
            "qualification":"Existing Flag-family scheduling only. Saved source records raw Staff and weak vertex removal; Part getter and Measure/edge callback order are bytecode plus declared native controls, not observed source callback parity."})
    }
}
