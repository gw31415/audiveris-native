// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{EdgeKey, Graph, Key, Phase};
use crate::music_font_symbols::TupletShape;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn target_coefficient(shape: TupletShape) -> f64 {
    match shape {
        TupletShape::Three => 2.0 * 0.33,
        TupletShape::Six => 2.0 * 0.17,
    }
}

impl Graph {
    #[cfg(test)]
    pub(crate) fn retained_explicit_tuplet_relation(
        &self,
        id: usize,
    ) -> Result<(&Value, bool), String> {
        let edge = self
            .edges
            .get(
                id.checked_sub(1)
                    .ok_or("music_actual_chord_tuplet_relation_required")?,
            )
            .ok_or("music_actual_chord_tuplet_relation_required")?;
        if edge.key.kind != "ChordTupletRelation" {
            return Err("music_actual_chord_tuplet_relation_required".into());
        }
        self.retained_tuplet_status(self.vertices[edge.key.target - 1].key)?;
        Ok((&edge.data, edge.removed))
    }

    fn explicit_tuplet_owner(&self, key: Key) -> Result<(usize, TupletShape), String> {
        let id = self.endpoint(key)?;
        if self.tuplet_status(key)?.0
            || !self.vertices[id - 1]
                .tuplet_cache
                .as_ref()
                .is_some_and(|cache| cache.sig_attached)
        {
            return Err("music_explicit_tuplet_attached_owner_required".into());
        }
        let shape = match self.vertices[id - 1].data["shape"].as_str() {
            Some("TUPLET_THREE") => TupletShape::Three,
            Some("TUPLET_SIX") => TupletShape::Six,
            _ => return Err("music_explicit_tuplet_shape_required".into()),
        };
        Ok((id, shape))
    }

    fn explicit_tuplet_chord(&self, chord: Key, tuplet: Key) -> Result<usize, String> {
        let id = self.endpoint(chord)?;
        let vertex = &self.vertices[id - 1];
        if chord.phase != Phase::Chord
            || chord.system != tuplet.system
            || !matches!(
                vertex.data["kind"].as_str(),
                Some("HeadChordInter" | "RestChordInter")
            )
        {
            return Err("music_actual_chord_tuplet_relation_required".into());
        }
        Ok(id)
    }

    pub(crate) fn add_explicit_chord_tuplet<F>(
        &mut self,
        chord: Key,
        tuplet: Key,
        shape: TupletShape,
        callback: F,
    ) -> Result<usize, String>
    where
        F: FnOnce(&mut Graph, Key) -> Result<(), String>,
    {
        let (target, actual_shape) = self.explicit_tuplet_owner(tuplet)?;
        let source = self.explicit_tuplet_chord(chord, tuplet)?;
        if shape != actual_shape {
            return Err("music_explicit_tuplet_relation_shape_required".into());
        }
        let ordinal = self
            .edge_keys
            .keys()
            .filter(|edge| {
                edge.system == tuplet.system
                    && edge.phase == Phase::Symbol
                    && edge.source == source
                    && edge.target == target
                    && edge.kind == "ChordTupletRelation"
            })
            .map(|edge| edge.ordinal)
            .max()
            .map(|n| n.checked_add(1).ok_or("shared_sig_relation_order_budget"))
            .transpose()?
            .unwrap_or(0);
        let mut next = self.clone();
        next.relations(
            tuplet.system,
            Phase::Symbol,
            vec![(
                chord,
                tuplet,
                json!({"kind":"ChordTupletRelation","manual":false,"grade":1.0,
                    "source_ratio":1.0,"target_ratio":1.0 + target_coefficient(shape),
                    "_editing_registration_ordinal":ordinal}),
            )],
        )?;
        let id = *next
            .edge_keys
            .get(&EdgeKey {
                system: tuplet.system,
                phase: Phase::Symbol,
                source,
                target,
                kind: "ChordTupletRelation".into(),
                ordinal,
            })
            .ok_or("music_explicit_tuplet_relation_registration_required")?;
        // The callback must see this edge and only the previously added edges.
        // Reading duration/factor or clearing clocks here changes that boundary.
        callback(&mut next, tuplet)?;
        *self = next;
        Ok(id)
    }

    pub(crate) fn contextualize_explicit_tuplet(&mut self, key: Key) -> Result<Value, String> {
        let (id, shape) = self.explicit_tuplet_owner(key)?;
        let grade = self.vertices[id - 1].data["grade"]
            .as_f64()
            .filter(|g| (0.0..=1.0).contains(g))
            .ok_or("music_explicit_tuplet_intrinsic_grade_required")?;
        let mut partners = Vec::new();
        let mut grades = BTreeMap::new();
        let mut contributions = BTreeMap::new();
        let mut sources = Vec::new();
        for eid in self.incident_edges(id) {
            let edge = &self.edges[eid - 1];
            match edge.key.kind.as_str() {
                "Exclusion" | "NoExclusion" => continue,
                "ChordTupletRelation" => {}
                _ => return Err("music_explicit_tuplet_context_support_pending".into()),
            }
            if edge.key.target != id {
                return Err("music_actual_chord_tuplet_relation_required".into());
            }
            let partner = &self.vertices[edge.key.source - 1];
            self.explicit_tuplet_chord(partner.key, key)?;
            let relation_grade = edge.data["grade"]
                .as_f64()
                .filter(|g| (0.0..=1.0).contains(g))
                .ok_or("music_explicit_tuplet_support_grade_required")?;
            let ratio = edge.data["target_ratio"]
                .as_f64()
                .filter(|r| r.is_finite() && *r >= 1.0)
                .ok_or("music_explicit_tuplet_support_ratio_required")?;
            if edge.data["source_ratio"] != 1.0
                || ratio != 1.0 + target_coefficient(shape) * relation_grade
            {
                return Err("music_explicit_tuplet_shape_support_required".into());
            }
            // SIGraph skips a null intrinsic partner grade. A contextual grade
            // or a freshly averaged member grade is not a substitute.
            let intrinsic = &partner.data["grade"];
            let partner_grade = if intrinsic.is_null() {
                None
            } else {
                Some(
                    intrinsic
                        .as_f64()
                        .filter(|g| (0.0..=1.0).contains(g))
                        .ok_or("music_explicit_tuplet_cached_Chord_grade_required")?,
                )
            };
            let contribution = partner_grade
                .filter(|_| ratio > 1.0)
                .map(|g| g * (ratio - 1.0));
            if let Some(value) = contribution {
                partners.push(edge.key.source);
                grades.insert(edge.key.source, partner_grade.unwrap());
                contributions.insert(edge.key.source, value);
            }
            sources.push(json!({"relation_id":eid,"partner_id":edge.key.source,
                "partner_bounds":partner.data["bounds"],"partner_grade":partner_grade,
                "relation_grade":relation_grade,"target_ratio":ratio,"contribution":contribution}));
        }
        if partners.len() > 16 {
            return Err("music_explicit_tuplet_context_partner_limit_16".into());
        }
        // Preserve duplicate partner entries and last contribution per partner,
        // as in original List + Map, before its stable reverse-grade ordering.
        partners.sort_by(|a, b| grades[b].total_cmp(&grades[a]));
        let mut conflicts = vec![Vec::new(); partners.len()];
        for (i, partner) in partners.iter().enumerate() {
            for eid in self.incident_edges(*partner) {
                let edge = &self.edges[eid - 1];
                if edge.key.kind != "Exclusion" {
                    continue;
                }
                let other = if edge.key.source == *partner {
                    edge.key.target
                } else {
                    edge.key.source
                };
                if let Some(j) = partners.iter().position(|p| *p == other)
                    && j > i
                    && !conflicts[i].contains(&j)
                {
                    conflicts[i].push(j);
                }
            }
        }
        let mut contextual = 0.0_f64;
        let mut partitions = Vec::new();
        for partition in crate::sig_context::compatible_partitions(&conflicts) {
            let ids = partition.iter().map(|i| partners[*i]).collect::<Vec<_>>();
            let contribution = ids.iter().map(|p| contributions[p]).sum::<f64>();
            let cg = ((1.0 + contribution) * grade) / (1.0 + contribution * grade);
            contextual = contextual.max(cg);
            partitions
                .push(json!({"partner_ids":ids,"contribution":contribution,"contextual_grade":cg}));
        }
        self.vertices[id - 1].data["contextual_grade"] = json!(contextual);
        Ok(
            json!({"id":id,"bounds":self.vertices[id - 1].data["bounds"],
            "shape":shape.as_str(),"grade":grade,"contextual_grade":contextual,
            "sources":sources,"partner_order":partners,"partitions":partitions}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_supports() -> (Graph, Key, Vec<Key>) {
        let source: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-linked-explicit-tuplets/support-inputs-observed.json"
        ))
        .unwrap();
        let row = &source["rows"][0];
        let mut graph = Graph::default();
        let tuplet = Key {
            system: 2,
            phase: Phase::Symbol,
            local: 1,
        };
        graph
            .vertex(
                tuplet,
                json!({"kind":"TupletInter","shape":"TUPLET_SIX",
            "bounds":row["tuplet_before"]["bounds"],"grade":row["tuplet_before"]["grade"],
            "implicit":false,"manual":false,"abnormal":true,"staff_id":null}),
                false,
            )
            .unwrap();
        let mut chords = Vec::new();
        for (index, link) in row["actual_lookup_links"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let actual = row["around"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["id"] == link["partner"])
                .unwrap();
            let chord = Key {
                system: 2,
                phase: Phase::Chord,
                local: index + 1,
            };
            // A different current contextual grade must not replace the actual
            // intrinsic grade captured by the independent original observer.
            graph
                .vertex(
                    chord,
                    json!({"kind":"HeadChordInter","bounds":actual["bounds"],
                "grade":actual["intrinsic_grade"],"contextual_grade":0.01,
                        "time_offset":crate::music_time::format_fraction(crate::music_time::Fraction::new(7,64).unwrap()),"manual":true}),
                    false,
                )
                .unwrap();
            chords.push(chord);
        }
        (graph, tuplet, chords)
    }

    #[test]
    fn actual_cached_chord_support_matches_original_six_context() {
        let (mut graph, tuplet, chords) = source_supports();
        let exact = crate::music_time::Fraction::new(7, 64).unwrap();
        for chord in &chords {
            assert_eq!(graph.actual_chord_time_offset(*chord).unwrap(), Some(exact));
        }
        let before = chords
            .iter()
            .map(|c| graph.retained_inter(*c).unwrap().0.clone())
            .collect::<Vec<_>>();
        let cache = graph.tuplet_cache_observation(tuplet);
        for chord in &chords {
            graph
                .add_explicit_chord_tuplet(*chord, tuplet, TupletShape::Six, |_, _| Ok(()))
                .unwrap();
        }
        let observation = graph.contextualize_explicit_tuplet(tuplet).unwrap();
        let original: Value = serde_json::from_str(include_str!(
            "../../fixtures/official-linked-explicit-tuplets/stages-observed.json"
        ))
        .unwrap();
        let expected = original["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["event"] == "edgeRemoved")
            .unwrap()["retained_tuplets"][0]["contextual_grade"]
            .as_f64()
            .unwrap();
        assert!((observation["contextual_grade"].as_f64().unwrap() - expected).abs() < 1e-15);
        assert!(
            (observation["partitions"][0]["contribution"]
                .as_f64()
                .unwrap()
                - 0.9641948300543761)
                .abs()
                < 1e-15
        );
        assert_eq!(
            observation["partner_order"],
            json!([
                graph.id(chords[1]).unwrap(),
                graph.id(chords[2]).unwrap(),
                graph.id(chords[0]).unwrap()
            ])
        );
        assert_eq!(graph.tuplet_cache_observation(tuplet), cache);
        for (chord, old) in chords.iter().zip(before) {
            assert_eq!(graph.retained_inter(*chord).unwrap().0, &old);
            assert_eq!(graph.actual_chord_time_offset(*chord).unwrap(), Some(exact));
        }
    }

    #[test]
    fn callback_failure_keeps_registry_and_exclusions_limit_support() {
        let (mut graph, tuplet, chords) = source_supports();
        let before = graph.snapshot();
        let edge_clock = graph.edge_clock;
        let fail =
            graph.add_explicit_chord_tuplet(chords[0], tuplet, TupletShape::Six, |next, key| {
                assert_eq!(next.tuplet_chords(key)?.len(), 1);
                next.set_tuplet_abnormal(key, false)?;
                Err("declared_callback_failure".into())
            });
        assert_eq!(fail.unwrap_err(), "declared_callback_failure");
        assert_eq!(graph.snapshot(), before);
        assert_eq!(graph.edge_clock, edge_clock);
        assert!(graph.edge_keys.is_empty());
        for chord in &chords {
            assert_eq!(
                graph.actual_chord_time_offset(*chord).unwrap(),
                Some(crate::music_time::Fraction::new(7, 64).unwrap())
            );
        }
        let first = graph
            .add_explicit_chord_tuplet(chords[0], tuplet, TupletShape::Six, |_, _| Ok(()))
            .unwrap();
        graph
            .add_explicit_chord_tuplet(chords[1], tuplet, TupletShape::Six, |_, _| Ok(()))
            .unwrap();
        graph
            .relations(
                2,
                Phase::Symbol,
                vec![(chords[0], chords[1], json!({"kind":"Exclusion"}))],
            )
            .unwrap();
        let observation = graph.contextualize_explicit_tuplet(tuplet).unwrap();
        assert!(
            observation["partitions"]
                .as_array()
                .unwrap()
                .iter()
                .all(|p| p["partner_ids"].as_array().unwrap().len() == 1)
        );
        let best = observation["sources"][1]["contribution"].as_f64().unwrap();
        let grade = observation["grade"].as_f64().unwrap();
        let expected = ((1.0 + best) * grade) / (1.0 + best * grade);
        assert_eq!(observation["contextual_grade"], json!(expected));
        let held = graph
            .retained_explicit_tuplet_relation(first)
            .unwrap()
            .0
            .clone();
        graph.remove_owned_tuplet(tuplet, &[], &[]).unwrap();
        let (body, removed) = graph.retained_explicit_tuplet_relation(first).unwrap();
        assert!(removed);
        assert_eq!(body, &held);
    }
}
