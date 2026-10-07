# Retained new-target preLink control

```sh
cargo test --offline --locked -j2 --lib pre_link_tests
python3 scripts/check_retained_musicxml_regression.py \
  --native-bin target/debug/audiveris-musicxml \
  --schema-dir /path/to/pinned-musicxml4-schema --output /tmp/new-cli-controls
```

Original new-target raw preLink has six tasks: UnlinkTask, RemovalTask,
AdditionTask, ChordStem LinkTask, Containment LinkTask, manual HeadStem LinkTask.
Do/Undo/Redo compares eighteen boundaries and forty-two listener records.
The original target Part/Staff is explicitly warmed before declared remove(false).
The same canonical Stems and saved numeric controls are used as the first unit.
No clocks are erased before listeners and no owners are rebuilt from reports.

A new actual typed editing HeadChord has nullable bounds and no Measure initially.
Containment places it; undo keeps cached placement and redo registers after SIG
listeners. Typed placement/members are checked at task boundaries. Callback
Graph/getters are checked separately; full owner membership within every callback
and ordinary Part/Measure/Voice integration remain pending. Native late failure
rollback and reuse are checked without claiming original UI exceptions rollback.
All global flags and CLI admission stay false/guarded.
