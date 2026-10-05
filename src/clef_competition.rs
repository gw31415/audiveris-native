// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// Staff.getCompetingClefs / SIGraph.getExclusions at
// 9e1e55cd2746037d059345881c53e6a6754bffbd. No graph mutation or clef selection.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Live ClefInter nodes, in the SIG vertex insertion order. Other inter types
/// need not be included; exclusions to their absent IDs are ignored.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ClefNode {
    pub id: u64,
    pub staff_id: u64,
    pub left: i64,
}

/// Exclusions in edge insertion order. Direction must be retained, even though
/// their semantic incompatibility is symmetric.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ExclusionEdge {
    pub source: u64,
    pub target: u64,
}

/// Return the original ordered candidate IDs at an x location. Only the initial
/// anchor must satisfy left < x; its exclusion partners are not x-filtered.
pub fn competing_clef_order(
    staff_id: u64,
    x: i64,
    clefs: &[ClefNode],
    exclusions: &[ExclusionEdge],
) -> Result<Vec<u64>, String> {
    if clefs.len() > 4096 || exclusions.len() > 16_384 {
        return Err("unsupported_clef_competition_budget".into());
    }
    if !(-4_000_000..=4_000_000).contains(&x)
        || clefs
            .iter()
            .any(|c| !(-4_000_000..=4_000_000).contains(&c.left))
    {
        return Err("unsupported_clef_competition_coordinate".into());
    }
    let mut by_id = BTreeMap::new();
    for clef in clefs {
        if by_id.insert(clef.id, clef).is_some() {
            return Err("duplicate_clef_competition_id".into());
        }
    }
    let mut staff_clefs = clefs
        .iter()
        .filter(|c| c.staff_id == staff_id)
        .collect::<Vec<_>>();
    // Stable sort reproduces Collections.sort ties over SIG.inters insertion order.
    staff_clefs.sort_by_key(|c| c.left);
    let Some(anchor) = staff_clefs.iter().rev().find(|c| c.left < x) else {
        return Ok(vec![]);
    };
    let anchor_id = anchor.id;
    let mut result = vec![anchor_id];
    let mut seen = BTreeSet::from([anchor_id]);
    // The pinned directed graph enumerates incoming edges before outgoing edges.
    for incoming in [true, false] {
        for edge in exclusions {
            let other = if incoming && edge.target == anchor_id {
                edge.source
            } else if !incoming && edge.source == anchor_id {
                edge.target
            } else {
                continue;
            };
            if let Some(clef) = by_id.get(&other)
                && clef.staff_id == staff_id
                && seen.insert(other)
            {
                result.push(other);
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn nodes() -> Vec<ClefNode> {
        vec![
            ClefNode {
                id: 1,
                staff_id: 1,
                left: 10,
            },
            ClefNode {
                id: 2,
                staff_id: 1,
                left: 10,
            },
            ClefNode {
                id: 3,
                staff_id: 1,
                left: 0,
            },
            ClefNode {
                id: 4,
                staff_id: 2,
                left: 30,
            },
        ]
    }
    fn edges() -> Vec<ExclusionEdge> {
        vec![
            ExclusionEdge {
                source: 2,
                target: 3,
            },
            ExclusionEdge {
                source: 1,
                target: 2,
            },
            ExclusionEdge {
                source: 2,
                target: 4,
            },
            ExclusionEdge {
                source: 1,
                target: 2,
            },
            ExclusionEdge {
                source: 2,
                target: 2,
            },
            ExclusionEdge {
                source: 2,
                target: 99,
            },
        ]
    }
    #[test]
    fn stable_left_ties_anchor_last_then_incoming_precedes_outgoing() {
        assert_eq!(
            competing_clef_order(1, 11, &nodes(), &edges()).unwrap(),
            vec![2, 1, 3]
        );
        let mut reversed = nodes();
        reversed.swap(0, 1);
        assert_eq!(
            competing_clef_order(1, 11, &reversed, &edges()).unwrap(),
            vec![1, 2]
        );
    }
    #[test]
    fn x_is_strict_only_for_anchor_and_foreign_or_duplicate_partners_are_skipped() {
        assert_eq!(
            competing_clef_order(1, 10, &nodes(), &edges()).unwrap(),
            vec![3, 2]
        );
        assert!(
            competing_clef_order(1, 0, &nodes(), &edges())
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            competing_clef_order(2, 31, &nodes(), &edges()).unwrap(),
            vec![4]
        );
    }
    #[test]
    fn duplicate_ids_and_budget_violations_are_explicit() {
        let mut n = nodes();
        n.push(n[0]);
        assert!(competing_clef_order(1, 20, &n, &edges()).is_err());
        assert!(competing_clef_order(1, i64::MAX, &nodes(), &edges()).is_err());
        assert!(
            competing_clef_order(
                1,
                20,
                &[],
                &vec![
                    ExclusionEdge {
                        source: 1,
                        target: 2
                    };
                    16_385
                ]
            )
            .is_err()
        );
    }
}
