# Retained HeadStem tasks (Linux)

```sh
AUDIVERIS_RATIONAL_OWNER_XML=/tmp/headstem-task-control.musicxml \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --offline --locked -j1 --lib \
  actual_rational_slots_and_underfilled_voice_termination_match_original
```

The existing native-recognized double-augmentation owner receives an explicit
manual alternate HeadStem task. Real do/undo/redo callbacks, retained caches,
publication and dirty notification are compared with original5.11 saved-owner
`HeadStemTaskProbe`. Common selected processing produces the same8notes/div16/
backup36 XML as `evidence/musicxml-rational-voice-clocks/control.musicxml` after
each operation. It derives the Voice plan and exact clocks, not supplied output.
No HeadStem CLI option, preLink discovery or new automatic image admission is
claimed. Existing CLI examples, automatic13/16 refusal and full flags stay.
