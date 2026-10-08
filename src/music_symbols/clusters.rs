// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(super) struct Cluster {
    pub(super) all_part_count: usize,
    pub(super) parts: Vec<GlyphPixels>,
    neighbors: Vec<Vec<usize>>,
    #[cfg(test)]
    graph_neighbors: Vec<Vec<usize>>,
}
fn too_large(g: &GlyphPixels, il: usize, left: i32) -> bool {
    g.width > 4 * il || (g.left as f64 + g.width as f64 / 2. >= left as f64 && g.height > 10 * il)
}
pub(super) fn eligible(g: &GlyphPixels, il: usize, fine_boxes: &[[i64; 4]]) -> bool {
    let weight = crate::clef_candidates::weight(g);
    weight >= (0.03 * (il * il) as f64).round_ties_even() as usize
        || (weight >= (0.006 * (il * il) as f64).round_ties_even() as usize
            && fine_boxes.iter().any(|b| {
                (g.left as i64) < b[0] + b[2]
                    && b[0] < g.left as i64 + g.width as i64
                    && (g.top as i64) < b[1] + b[3]
                    && b[1] < g.top as i64 + g.height as i64
            }))
}
pub(super) fn components(
    glyphs: &[GlyphPixels],
    il: usize,
    fine_boxes: &[[i64; 4]],
) -> Result<Vec<Cluster>, String> {
    let mut parts = glyphs
        .iter()
        .filter(|g| eligible(g, il, fine_boxes))
        .cloned()
        .collect::<Vec<_>>();
    parts.sort_by_key(|g| g.left);
    let mut neighbors = vec![Vec::new(); parts.len()];
    let gap = (0.8 * il as f64).ceil() as i64;
    for (i, a) in parts.iter().enumerate() {
        for (j, b) in parts.iter().enumerate().skip(i + 1) {
            if b.left as i64 >= a.left as i64 + a.width as i64 + gap {
                break;
            }
            if b.top as i64 >= a.top as i64 + a.height as i64 + gap
                || a.top as i64 - gap >= b.top as i64 + b.height as i64
            {
                continue;
            }
            let links =
                crate::clef_candidates::links_at_distance(&[a.clone(), b.clone()], 0.8 * il as f64)
                    .map_err(|e| e.to_string())?;
            if links[0] != 0 {
                neighbors[i].push(j);
                neighbors[j].push(i);
            }
        }
    }
    let mut visited = vec![false; parts.len()];
    let mut clusters = Vec::new();
    for start in 0..parts.len() {
        if visited[start] {
            continue;
        }
        let mut component = vec![start];
        visited[start] = true;
        let mut at = 0;
        while at < component.len() {
            for &j in &neighbors[component[at]] {
                if !visited[j] {
                    visited[j] = true;
                    component.push(j);
                }
            }
            at += 1;
        }
        let all_part_count = component.len();
        if all_part_count > 7 {
            component
                .sort_by_key(|&i| std::cmp::Reverse(crate::clef_candidates::weight(&parts[i])));
            component.truncate(7);
        }
        let graph_neighbors = component
            .iter()
            .map(|&i| {
                neighbors[i]
                    .iter()
                    .filter_map(|j| component.iter().position(|p| p == j))
                    .collect()
            })
            .collect();
        clusters.push(Cluster::ordered(
            all_part_count,
            component.into_iter().map(|i| parts[i].clone()).collect(),
            graph_neighbors,
        ));
    }
    Ok(clusters)
}
impl Cluster {
    fn ordered(
        all_part_count: usize,
        parts: Vec<GlyphPixels>,
        graph_neighbors: Vec<Vec<usize>>,
    ) -> Self {
        // getSubGraph inserts edges in selected vertex encounter order. This
        // changes recursive neighbor order even though connectivity is unchanged.
        let mut edges = BTreeSet::new();
        let mut neighbors = vec![Vec::new(); parts.len()];
        for (i, adjacent) in graph_neighbors.iter().enumerate() {
            for &j in adjacent {
                if edges.insert((i.min(j), i.max(j))) {
                    neighbors[i].push(j);
                    neighbors[j].push(i);
                }
            }
        }
        Self {
            all_part_count,
            parts,
            neighbors,
            #[cfg(test)]
            graph_neighbors,
        }
    }
    #[cfg(test)]
    pub(super) fn in_observed_part_order(&self, order: &[usize]) -> Self {
        let parts = order.iter().map(|&i| self.parts[i].clone()).collect();
        let graph_neighbors = order
            .iter()
            .map(|&i| {
                self.graph_neighbors[i]
                    .iter()
                    .map(|j| order.iter().position(|p| p == j).unwrap())
                    .collect()
            })
            .collect();
        Self::ordered(self.all_part_count, parts, graph_neighbors)
    }

    fn process(
        &self,
        members: Vec<usize>,
        seen: u16,
        il: usize,
        left: i32,
        trials: &mut Vec<Vec<usize>>,
    ) {
        let mask = members.iter().fold(0, |m, &i| m | 1 << i);
        let glyph = crate::clef_candidates::compound(&self.parts, mask);
        if too_large(&glyph, il, left) {
            return;
        }
        if crate::clef_candidates::weight(&glyph) as f64 / (il * il) as f64 >= 0.04 {
            trials.push(members.clone());
        }
        let mut outliers = Vec::new();
        for &i in &members {
            for &j in &self.neighbors[i] {
                if seen & (1 << j) == 0 && !members.contains(&j) && !outliers.contains(&j) {
                    outliers.push(j);
                }
            }
        }
        // Siblings share the growing considered set; resetting it enumerates
        // different compounds and changes Inter creation/exclusion order.
        let mut considered = seen;
        for j in outliers {
            considered |= 1 << j;
            let mut larger = members.clone();
            larger.push(j);
            self.process(larger, considered, il, left, trials);
        }
    }
    pub(super) fn trials(&self, il: usize, left: i32) -> Vec<Vec<usize>> {
        if self.all_part_count == 1 {
            return if crate::clef_candidates::weight(&self.parts[0]) as f64 / (il * il) as f64
                >= 0.04
            {
                vec![vec![0]]
            } else {
                Vec::new()
            };
        }
        let mut seeds = (0..self.parts.len()).collect::<Vec<_>>();
        seeds.sort_by_key(|&i| std::cmp::Reverse(crate::clef_candidates::weight(&self.parts[i])));
        let mut considered = 0;
        let mut trials = Vec::new();
        for seed in seeds {
            considered |= 1 << seed;
            self.process(vec![seed], considered, il, left, &mut trials);
        }
        trials
    }
}
pub(super) fn candidates(
    glyphs: &[GlyphPixels],
    il: usize,
    left: i32,
    fine_boxes: &[[i64; 4]],
) -> Result<Vec<(GlyphPixels, Vec<GlyphPixels>)>, String> {
    let mut out = Vec::new();
    for cluster in components(glyphs, il, fine_boxes)? {
        for members in cluster.trials(il, left) {
            let mask = members.iter().fold(0, |m, &i| m | 1 << i);
            out.push((
                crate::clef_candidates::compound(&cluster.parts, mask),
                members
                    .into_iter()
                    .map(|i| cluster.parts[i].clone())
                    .collect(),
            ));
        }
    }
    Ok(out)
}
