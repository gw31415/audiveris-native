# Actual new HeadChord ownership inside SIG listeners

```sh
cargo test --offline --locked -j2 --lib pre_link_tests
```

The new HeadChord editing-owner registry is retained in the actual Graph, so
relation callbacks change its members, stem and lazy placement directly. Live
Containment/ChordStem views change before listener entry; invalidation resolves
Measure membership before listener return. AdditionTask redo sets bounds before
listeners but re-registers cached Measure membership after both listener records.

The original new-target control compares eighteen task boundaries and forty-two
listener records, including the actual editing HeadChord's member/stem/bounds,
cached Measure reference and Measure membership at every callback. These are
observations from the typed object, not a reconstruction from report rows.
Late failure restores the whole Graph-owned object with registered relation
identities, plus actual normal Chords, Measures, Voices and expectedDuration;
the same retained sequence can retry/undo/redo.

The existing saved numeric controls and original Part/Staff-warmed target-removal
setup apply. General old Chord/Part/Voice membership inside all callbacks,
ordinary reprocessing of editing Chords, canonical mirrors and the automatic
five-pixel Stem extent difference remain pending. Public CLI guard and global
completion flags remain false. Full historical649 regression was not repeated.
