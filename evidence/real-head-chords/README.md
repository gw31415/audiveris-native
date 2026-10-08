# Actual piano Chord dispatch and default Stem head-end refinement

Original normal reachStep(CHORDS,false) on the unchanged full An Laura image,
before intermediate serialization, observes67 Chords/94 Heads. Native now shares
the stem-Chord collection across the same Part's Staff order, imports each Staff's
recognized header, and dispatches into the same actual Measure geometry by first
containing Part Staff area and first right bar, with the original headLocation
fallback. Measures carry the actual retained layout Arc. The piano is one Part;
no Part/Voice number or ownership is reconstructed from reports. Source getStaff
null for cross-Staff Chords is preserved; this image has zero such null/stemless
cases, so their general recognition is not claimed by this image comparison.

The first strict comparison failed on all67 Stem rectangles while all Head groups
and pitch/Part/Measure ownership matched. Original STEMS then CHORDS observations
show the same Stem/glyph object but modified median: example identity22 retains
glyph bounds[399,381,4,40] and126 pixels, while Stem bounds change40 to33 high.
Pinned ReductionStep calls refineHeadEnd after reduceFoundations by default;
refineTailEnd defaults false. Native now connects that step, holds the same
canonical cache Arcs and physical glyphs, and stages all geometry/locks before
commit. All67 Chord/member/stem/Part/Measure geometry rows now match. Independent
Stem comparison matches glyph bounds/weight/width and median to3.64e-12 pixels.

Original relation consistency is set when its support is created, and refining a
Stem median does not recompute it. A later cache registration initially failed
constructor-only geometry/consistency validation. Internally retained refinement
geometry can now survive synchronization; caller-reported geometry alone cannot
certify it. Exactly retained relation bodies retain their prior consistency.
Retired cache records also preserve refined geometry. Manual relation identities,
cache identity and registry survive a late unknown Head anchor failure. Existing
container/selected/rational checks remain required; prior failed attempts are not
counted as successes.

This is recognition-required assembly, not general editing/API compatibility.
The full-image comparison is a native library path with report GRID ownership
blanked; it is not a full native CLI MusicXML success. SYMBOLS/multi-Staff dot/rest
ownership, common Stack rhythm/voices and real output/quality acceptance remain.
Original export has92 notes; the94 intermediate Heads are not92 correct events,
independent truth agreement, or accepted quality. No oracle output enters native
recognition. VALIDATION records actual successful checks only.
