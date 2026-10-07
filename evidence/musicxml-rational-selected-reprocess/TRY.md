# Reproduce the exact selected owner processing (Linux)

```sh
AUDIVERIS_RATIONAL_OWNER_XML=/tmp/rational-selected-owner.musicxml \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --offline --locked -j1 --lib \
  actual_rational_slots_and_underfilled_voice_termination_match_original
```

Existing double-augmentation image recognition provides actual owned input;
an explicit manual alternate HeadStem callback is the control operation. The
same selected analyze/refine/rebind/render helper used by correction processing
computes native candidates/Voices/times. Only its test build exposes the observed
owner scope; no new public CLI option or automatic image admission is claimed.
Original real PageRhythm/Partwise timing and musical stream match; generated
XML matches `evidence/musicxml-rational-voice-clocks/control.musicxml` and XSD.
Public correction provenance, fine-duration/parallel reception and incomplete
Voice guards stay until actual RelationTask/controller comparison is connected.
