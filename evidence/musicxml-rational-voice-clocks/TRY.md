# Reproduce the declared owner control (Linux, no Java runtime required)

From a normal source checkout:

```sh
AUDIVERIS_RATIONAL_OWNER_XML=/tmp/rational-owner.musicxml \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --offline --locked -j1 --lib \
  actual_rational_slots_and_underfilled_voice_termination_match_original
```

This recognizes the existing double-augmentation image, applies a declared manual
HeadStem callback to the real owned Chord, then runs actual Voice/slot/check/export
algorithms with an explicitly declared original Voice plan. The saved native
`control.musicxml` contains8 notes and two underfilled Voices in Measure1; it is
not a public HeadStem correction option or new automatic image acceptance.
The original Java control uses the same saved-page owner and actual
PageRhythm.reprocessStack/PartwiseBuilder; original XML is not reused.
Existing public image/correction examples in `docs/MUSICXML-LIMITED.md` remain.

Historical sourcec357b75 supplied the Voice plan. Current main runs native
candidate/start/mapper/extinction instead; see the later
`evidence/musicxml-rational-mapper/TRY.md`. The old validation/readback records
remain source-specific.
