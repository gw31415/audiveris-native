# Link and Unlink retained owner tasks (Linux)

```sh
AUDIVERIS_RATIONAL_OWNER_XML=/tmp/headstem-unlink-control.musicxml \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --offline --locked -j1 --lib \
  actual_rational_slots_and_underfilled_voice_termination_match_original
```

The same existing recognized image owner now runs LinkTask do/undo/redo and then
UnlinkTask do/undo/redo. Source publications and callbacks are compared before
legitimate reset; native common selected reprocessing produces original exact
slots/short Voices and8notes after every operation. Generated output matches
`evidence/musicxml-rational-voice-clocks/control.musicxml` and XSD. No HeadStem
CLI option or preLink/automatic correction discovery is enabled.
