# Retained selected stack CLI reprocessing

```sh
audiveris-musicxml --reprocess-stacks 2,1,2 \
  fixtures/official-image-musicxml-parallel-voices/parallel-quarters.png \
  /tmp/selected.musicxml /tmp/selected.json
```

The CLI rebuilds actual retained measures in first encounter order, once each,
after any admitted dot correction and before a requested slot query. It uses
the existing guarded Image rhythm path, shared actual mapper and per-stack
system refinement, then binds events to real Voice/Chord/Slot clocks and renders
MusicXML. The standalone OwnedRhythm selected backend is separately compared;
this public path keeps the ordinary Image and correction admission limits.
Report events remain writer metadata, never an ownership reconstruction plan.

Original RhythmsStep/Page order and duplicate suppression are observed at
fixtures/official-voice-selected-rhythm (a4060de,14 original boundaries).
The new API stages Graph, Voices and Measure timing while retaining actual
Chord storage. Success commits only selected timings into original Measures;
untouched Slot storage stays unchanged. A later actual member failure and a
final XML duplicate-event failure both restore original Graph/manual relations,
live/retired Voice references, Slot storage/identities, expected/actual/excess
clocks, report and XML. Repair/retry succeeds. Public provenance refusal keeps
unvalidated correction/report edits guarded. Exact7/64 and short-Voice backend
export remain compared; this option does not widen their public admission.

```sh
cargo test --offline --locked -j2 --lib selected_cli_tests
python3 scripts/check_retained_musicxml_regression.py \
  --native-bin target/debug/audiveris-musicxml --schema-dir /path/to/musicxml4 \
  --reprocess-stacks 1,1 --output /tmp/new-selected-cli-regression
```

The report records requested/processed stack numbers and retains prior admitted
editing impact evidence. No recognition stages or correction callbacks are
rerun by this explicit selected pass. Full nonUI port/controller/PAGE/OMR flags
remain false. Outputs keep the existing create-new paired-file policy.
