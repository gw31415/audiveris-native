// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
//! Pinned ClumpPruner SlurEntry/getNonOrphans/selectAmong selection only.
//! Component-sharing groups are a native extraction alternative, not original
//! Skeleton common-trunk clump provenance. This performs no Slur admission.

pub trait PendingLink {
    type Chord;
    fn chord(&self) -> &Self::Chord;
    fn euclidean(&self) -> f64;
}
pub struct Entry<'o, S, L> {
    pub candidate: &'o S,
    pub point_count: usize,
    pub links: [Option<&'o L>; 2],
}
#[derive(Debug, PartialEq)]
pub enum Error {
    MissingPoints,
    MissingLink,
    InvalidDistance,
    MissingComponentOwner,
}
impl<S, L: PendingLink> Entry<'_, S, L> {
    pub fn weighted_distance(&self) -> Result<f64, Error> {
        if self.point_count == 0 {
            return Err(Error::MissingPoints);
        }
        let mut sum = 0.;
        let mut count = 0;
        for link in self.links.iter().flatten() {
            let d = link.euclidean();
            if !d.is_finite() || d < 0. {
                return Err(Error::InvalidDistance);
            }
            sum += d;
            count += 1;
        }
        if count == 0 {
            return Err(Error::MissingLink);
        }
        Ok((sum / count as f64) / self.point_count as f64)
    }
}
/// The returned references preserve pending relation objects and candidate owners.
/// Ordinary orphan legality was already checked by lookupLinkPair; this function
/// merely prefers a complete pair. Its one-sided inputs are not orphan admission.
pub fn select<'a, 'o, S, L: PendingLink>(
    entries: &'a [Entry<'o, S, L>],
    same_chord: impl Fn(&L::Chord, &L::Chord) -> bool,
) -> Result<Option<&'a Entry<'o, S, L>>, Error> {
    let nonorphan = entries.iter().any(|e| e.links.iter().all(Option::is_some));
    let mut pool = entries
        .iter()
        .filter(|e| !nonorphan || e.links.iter().all(Option::is_some))
        .map(|e| e.weighted_distance().map(|d| (e, d)))
        .collect::<Result<Vec<_>, _>>()?;
    // Java Collections.sort is stable; equal scores preserve native enumeration.
    pool.sort_by(|a, b| a.1.total_cmp(&b.1));
    let Some(&(first, first_distance)) = pool.first() else {
        return Ok(None);
    };
    let mut best = first;
    let mut best_distance = first_distance;
    for &(entry, distance) in pool.iter().skip(1) {
        if (0..2).any(|side| match (first.links[side], entry.links[side]) {
            (Some(a), Some(b)) => !same_chord(a.chord(), b.chord()),
            _ => false,
        }) {
            break;
        }
        if distance < best_distance {
            best = entry;
            best_distance = distance;
        }
    }
    Ok(Some(best))
}

pub struct ComponentCandidate<'o, S, G> {
    pub candidate: &'o S,
    pub components: Vec<&'o G>,
}
/// Caller supplies only candidates that passed actual raw run/gap gates. Each
/// component is the retained native component object, never an expected mask.
/// Transitive sharing is explicit and differs from upstream seed/trunk clumps.
pub fn component_groups<'a, 'o, S, G>(
    candidates: &'a [ComponentCandidate<'o, S, G>],
) -> Result<Vec<Vec<&'a ComponentCandidate<'o, S, G>>>, Error> {
    if candidates.iter().any(|c| c.components.is_empty()) {
        return Err(Error::MissingComponentOwner);
    }
    let mut parent: Vec<usize> = (0..candidates.len()).collect();
    fn root(parent: &[usize], mut i: usize) -> usize {
        while parent[i] != i {
            i = parent[i];
        }
        i
    }
    for i in 0..candidates.len() {
        for j in 0..i {
            if candidates[i].components.iter().any(|a| {
                candidates[j]
                    .components
                    .iter()
                    .any(|b| std::ptr::eq(*a, *b))
            }) {
                let ri = root(&parent, i);
                let rj = root(&parent, j);
                parent[ri.max(rj)] = ri.min(rj);
            }
        }
    }
    let mut groups: Vec<(usize, Vec<&ComponentCandidate<'o, S, G>>)> = Vec::new();
    for (i, candidate) in candidates.iter().enumerate() {
        let key = root(&parent, i);
        if let Some((_, group)) = groups.iter_mut().find(|(r, _)| *r == key) {
            group.push(candidate);
        } else {
            groups.push((key, vec![candidate]));
        }
    }
    Ok(groups.into_iter().map(|(_, members)| members).collect())
}
