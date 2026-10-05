# GRID context and initial HEADERS stage

Local-only continuation of `4cd1ce4`, on top of the historical receipt commit
`c26a820`. No Library update or external publication is part of this checkpoint.
The runtime remains native Rust, with no Java invocation or fallback.

## Verified changes

- `BarConnectorInter.isGood` uses its own inclusive intrinsic-grade threshold
  0.65. Such connectors freeze themselves and the connected column of peaks.
  The traversal follows every surviving alignment, including non-connection
  edges. Bracket connectors do not trigger this rule. All final GRID inters now
  expose `frozen`; 13 independent original-image official runs match this flag,
  as well as the previously checked geometry, relations, glyphs and masks.
  These images cover frozen and unfrozen bars/connectors/brackets; they do not
  establish exhaustive threshold-edge or every graph-topology coverage.
- The bounded GRID SIG contextualizer uses official partner-grade ordering,
  direction-specific support enumeration (incoming before outgoing), repeated
  partner/map overwrite behavior, and exclusion partitions. Seven constructed
  graphs made from original JAR objects match every contextual grade bit for bit.
  These are controlled graph tests, **not image-derived exclusion detections**.
  Non-GRID interpretation/support types are rejected. A maximum of 16 support
  partner entries bounds the exponential partition storage; excess is rejected
  without partially updating grades. No HeadInter/StemInter special case is
  claimed. The GRID pipeline currently generates no Exclusion relations itself.
- The positive `checkLinesRoot` branch is proven on the original line-root
  raster's projector with only the start attribute and initial left boundary
  deliberately supplied: original and native move left from 121 to 100 and
  clear the start flag. The unchanged no-start case remains 121. This is a
  controlled stage test; no natural `detectStartColumns` trigger is claimed.
- If brace assembly encounters a missing detected portion and would need the
  ordinary-bar fallback, native processing now fails with
  `unsupported_brace_fallback_alignment`, including system/staff identifiers.
  It no longer emits a partial brace/part result for that branch. A test-local
  raster with the first AnLaura piano brace's lower portion removed verifies
  this refusal. This intentionally conservative boundary can reject cases the
  official fallback would merely decline. Fallback geometry remains unported.
- `header_preparation::from_grid` ports `HeaderBuilder.computeHeaderStarts`:
  staff left side-bar selection, directed bar-group traversal, right edge + 1,
  and required dummy-bar geometry. All 13 original-image official stage results
  match header starts/stops, last-group bounds and dummy descriptors exactly.
  `page-glyphs` and `staff-mask` reports include this initial-stage snapshot.
  Dummy descriptors are separate from the unchanged GRID graph; later header
  recognition must attach them to its own working graph.

## Evidence and reproduction

Workspace debug and release each passed **59 tests**. Clippy
with warnings denied, formatting and whitespace checks passed. Actual commands,
input/output hashes and test-log hashes are retained in
`evidence/grid-context-validation.json`.

The original official JAR remains pinned at SHA-256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`, source commit
`9e1e55cd2746037d059345881c53e6a6754bffbd` (Audiveris 5.11.0).
`evidence/grid-context-source.json` records the exact preserved AGPL source
copies. Existing Bravura font provenance and OFL license remain unchanged.

```
python3 scripts/official_semantic_final_probe.py CONTENTS JDK_HOME
python3 scripts/official_context_probe.py CONTENTS JDK_HOME
python3 scripts/official_line_root_probe.py CONTENTS JDK_HOME
cargo test --workspace --offline --locked -j 2
cargo test --workspace --offline --locked --release -j 2
cargo clippy --workspace --offline --locked --all-targets -j 2 -- -D warnings
cargo fmt --all --check
```

The test-only observer calls official `computeHeaderStarts` after collecting
GRID inter/section/glyph outputs. Header mutations are not included in the GRID
snapshot. The observer never replaces the official recognition algorithms.
Compile-only ABI shims remain excluded from the runtime classpath.

## Remaining work and limits

`grid_complete`, `headers_complete`, `heads_mask_complete`, and
`note_recognition_complete` remain false. Clef/key/time recognition and column
selection are the next HEADERS work. Then STEM_SEEDS → BEAMS → LEDGERS must
supply actual seed/ledger inters and masks before faithful automatic HEADS
candidate search. Measured head sizing, template selection, competition and
aggregation, rhythm and MusicXML are still missing.

Brace fallback and nondefault GRID layouts remain unsupported. Bracket metrics
remain limited to Bravura interlines 11/16 and head templates to the existing
Bravura 51pt four-shape bank. Unknown font/size input is not interpolated.
The decoder still rejects more than 4M pixels (32MB encoded input cap); ordinary
300 DPI A4 (~8.7M pixels) and PDF integration remain blockers. No increased cap,
bounded-memory high-resolution integration, or complete image/PDF→MusicXML path
is claimed by this checkpoint.
