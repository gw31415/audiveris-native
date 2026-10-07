# Reproduce the owned mapper comparison (Linux)

```sh
AUDIVERIS_RATIONAL_OWNER_XML=/tmp/rational-mapped-owner.musicxml \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --offline --locked -j1 --lib \
  actual_rational_slots_and_underfilled_voice_termination_match_original
```

The input is the existing native-recognized double-augmentation image owner plus
an explicit manual alternate HeadStem callback. Native geometry/candidate order,
starting construction, mapper and extinction choose all Voice/Chord clocks;
no original Voice plan or output times are supplied. Compare real original
saved-owner PageRhythm/Partwise timing and musical stream. Output matches
`evidence/musicxml-rational-voice-clocks/control.musicxml` and XSD.
Public CLI examples remain unchanged; the remaining selected processing and
RelationTask/controller dependencies still guard manual fine-duration admission.
