# Native candidate clump selection against original

This is a bounded saved-Book selection probe, not image recognition or Slur acceptance. All native candidate inputs originate in actual cleaned native glyph components and raw BINARY foreground refinements. No original Slur coordinates, masks, IDs or expected Head endpoints are read by extraction, raw gating or clump grouping.

`prepare_native_slur_clumps.py` takes every candidate passing the retained original-style raw run/gap gates: **12 candidates**. Transitive sharing of actual native component membership makes **10 groups**. This is an explicit native alternative to original Skeleton seed/common-trunk clump construction, whose equivalence is unproved. Production must use retained component object references; the exploratory fixture's native component indices are only local identity handles.

The original probe receives the existing Rust full-precision generated cubics and their actual foreground point sequences. Original `ClumpPruner.getBounds`, SystemManager routing, `ClumpLinker` Chord box filtering and `selectSlur` execute against real original HeadChord owners from the saved Book. It stops before `prune`'s provisional tie classification, `doLink`, registration or cleanup. It does not invoke image processing or save the Book.

Results:

| Native candidate group | Original selected candidate | Weighted distance |
| --- | --- | --- |
| 30 | none | — |
| 47, 1136 | 1136 | 0.08646256892451985 |
| 60 | 60 | 0.1059172996830022 |
| 64, 1141 | 1141 | 0.09721523480633415 |
| 116 | 116 | 0.11188102926555235 |
| 117 | 117 | 0.09352450579981628 |
| 127 | none | — |
| 135 | none | — |
| 1137 | 1137 | 0.1142962384443199 |
| 152 | none | — |

The selected six have the six previously diagnosed endpoint pairs; that is a diagnostic association, not quality parity or a selector. The raw-gap-failing candidate143 is absent by raw gating. Raw-run-failing48 and unrefined136/137/141 are likewise not admitted because their lookup happened to succeed.

The standalone Rust proposal `/tmp/slur_clump_selection_proposal.rs` compares the same original-computed lookup entries against the source selection result. **18 cases = 10 native groups + 8 source controls; 41 comparisons; 0 mismatches; maximum weighted-distance delta 0**. This is the selection kernel comparison, not a single native production run of all geometry, source Chord filtering and selection. Earlier independent geometry/per-Chord/pair comparisons are preserved separately under `/tmp/slur-native-curve-lookup-original-v1`.

The eight source controls cover empty input, complete-pair priority over a lower-distance orphan, singleton orphan, actual point-count weighting, equal score with both enumeration orders, different actual Chord at equal score, and one-sided mean calculation. The source returns one of the exact input entry objects for all selected controls. No runtime SlurHead relation callback identity claim follows. Two extra Rust-only grouping controls show that separate equal-valued component objects remain distinct and that transitive sharing preserves insertion enumeration. Ordinary orphan legality is outside these controls: singleton-orphan selection does not establish orphan admission.

The Rust proposal borrows candidate and pending-link objects, returning the same chosen entry. Actual links retain their eventual relation owner; the selection kernel allocates no replacement relation. It divides mean LEFT→RIGHT relation Euclidean distance by `SlurInfo.points.size`, stable-sorts, preserves first equal scores and checks actual Chord equality at the source stopping boundary. Native finite/nonempty prerequisites are explicit guards, not invented interpolation or source null fallback.

Remaining recognition gates: actual native Head/Chord/Staff/Stem/Beam ownership integration; proper pre-CURVES source erasure and noncrossable-owner filtering; original/common-trunk extraction timing difference; mirror/ordinary-orphan positive controls; provisional tie/space-clear checks; actual SlurHead relation insertion/callbacks and strong/weak glyph cleanup. Only these completed lookup/selection contracts may be used; counts alone cannot admit the image or claim Audiveris quality.

Original Book SHA before and after is `a5cf0b402687b9448f6a73aad2d82a93a39dcb9001a97c60a32a1909b58f3204`. Exact pinned source, JAR/helper/inputs and commands are recorded in `PROVENANCE.json`, `RUST-PROVENANCE.json`, `native-groups.json` and `MANIFEST.json`. No repository edits, Git operations, full build or recognition rerun occurred.
