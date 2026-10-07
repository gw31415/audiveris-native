# Retained-owner control (not correction CLI admission)

From the existing Linux workspace, using the existing low-debug build settings:

```sh
AUDIVERIS_MANUAL_RETURN_XML=/tmp/manual-return.musicxml \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --offline --locked -j1 --lib retained_manual_owner_reprocess
```

This Rust control recognizes the existing `double-augmentation.png` fixture,
uses actual registered owners for two declared manual additions, tests refused
7/64 reprocessing/rollback and supported returned-owner reprocess/native export.
The saved `control.musicxml` has eight notes and passed MusicXML4 XSD. The original
comparison uses its saved `.omr` owner, real PageRhythm and PartwiseBuilder stream;
it is separate from fresh automatic recognition parity. No Java is needed to
run the native control. Existing image CLI commands in `docs/MUSICXML-LIMITED.md`
remain usable; no manual HeadStem CLI switch is enabled yet.
