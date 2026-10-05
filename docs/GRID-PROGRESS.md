# Raster → bar sticks → initial systems (full GRID pending)

`grid-candidates INPUT BINARY.pgm REPORT.json` extends native Rust processing through initial system construction. Official source pin remains Audiveris 5.11.0 / `9e1e55cd2746037d059345881c53e6a6754bffbd`. It does not execute Java. New default-path translations are:

- `StaffProjector`: staff-height column projection, elite derivative threshold, thickness-derived line/blank/chunk thresholds, blank selection, derivative peak splitting, peak-side/chunk refinement, vertical white gaps and weighted grade acceptance.
- `LinesRetriever.addShortSections`, `BarFilamentBuilder` and `BarFilamentFactory`: select narrow horizontal/vertical sections, populate vertical cores, expand by actual section contact, compute one-interline vertical spline geometry, and flag curvature-compatible brace candidates.
- `PeakGraph`: alignment tests, actual pixel connections with `AreaUtil.verticalCore`, grade-weighted conflicting-edge pruning, `getSystemTops` and initial system membership.

Supported scope is five-line, single-interline, single-column layout without merged peaks requiring splitting. Multi-column staff neighbors or a detected split requirement return explicit `unsupported`; they are not silently approximated. The graph includes the original split-eligibility check but not the mutation that splits/rebuilds peak filaments. `system_candidates.grid_complete` is always false. `grid-candidates` exits 2 if no initial systems or a required stage is unsupported/failed. The older `staff-candidates` command retains its earlier initial-Staff success contract and additionally emits the downstream diagnostics.

## Independent evidence

The official full-app reference from task-6 is copied under `fixtures/official-grid`: immutable original raster inputs, official BINARY images and golden LOAD/BINARY/SCALE/GRID results. A new observer uses the official `-run` extension after original-raster LOAD/BINARY/SCALE, then calls **unchanged classes from the official jar** for initial staff creation, projection, bar sticks and graph stages. No Rust SCALE, sections or other IR enters Java.

Audiveris jar SHA-256 is checked before running: `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`. Java25 runtime comes from the official distribution already obtained by the independent task. The local compiler is Java21: two compile-only `Book`/`RunClass` ABI shims permit compiling the reflection observer; their directory is **excluded from runtime classpath**, whose official jar supplies the real classes. There is no replacement recognition implementation in this observer. Its source/hash, run arguments, input hashes and output hashes are retained in `probe-provenance.json` and `*-probe-execution.json`.

| Fixture | BINARY / SCALE | Initial Staff geometry | Raw peaks / retained bar sticks | Initial systems |
|---|---|---|---:|---|
| CC0 An Laura, 180 DPI | exact | 6 staffs / 30 lines, all defining points exact | 55 / 55 | 2: [1,2,3], [4,5,6] |
| Original synthetic system with bars | exact | 2 staffs / 10 lines, all defining points exact | 6 / 6 | 1: [1,2] |
| Original short two-staff image without bars | exact | no initial staffs after minimum-width gate | 0 / 0 | none; native exit 2 and official GRID failure |

All projection bins, thresholds, blanks, peak bounds, bar-stick section IDs/orientations/bounds/weights, defining spline points and brace flags match. The four graph checkpoints match official edge endpoints/types/order, with grades compared at `1e-9 + 1e-12*abs(expected)`. Final system counts and staff membership also match the independent completed GRID result. These fixtures do not cover all graph split/layout branches or establish recognition accuracy.

The full workspace now has 31 tests, including an end-to-end original-raster comparison and no-system CLI regression. Formatting, clippy with warnings denied, and release build pass. Evidence: `evidence/official-grid-comparison.json` and `grid-progress-*` logs.

## What still separates this from completed GRID

The final official lines differ from initial geometry: An Laura left endpoints are one pixel farther left, right endpoints are one or four pixels farther right. Final StaffLine storage adds Y+0.5, rounds XML to one decimal and simplifies points. Thirty-nine of the forty final comparison lines have two stored points; An Laura's final bottom line retains seventeen. Counts and system membership do not prove those shape/endpoint differences resolved.

Remaining order: `BarsRetriever` columns/start columns, brace/bracket interpretation, false-peak cleanup, final staff ends, bars/groups/parts; then `LinesRetriever.completeLines` endpoint definition, discarded-filament and thick/thin/sticker section inclusion, hole filling and curvature polishing; then `StaffLineCleaner` glyph registration, line simplification, staff removal, HLAG rebuild and system population. Do not skip to MusicXML or claim full GRID until these are compared with the independent final outputs.

## Pixel-membership handoff

Initial staff lines keep `section_ids`; the referenced `grid_preparation.horizontal_sections` retains `first_pos`, orientation and every `[start,length]` run. Bar-stick members now also export **their own first_pos and complete run arrays**, including newly created short horizontal sections. Bounds alone are not used as a substitute for pixel membership.

These are **current** memberships, not final staff-line glyph masks: completeLines will add/discard sections and must produce a final union before invoking the independently developed staff-removal module. Rendering a stroke from curve/thickness would not reproduce the official removal. The task-6 `musicxml-oracle/STAFF-REMOVAL-BOUNDARY.md` agrees with this boundary; its reported downstream MusicXML and rhythm inconsistencies are a future observation baseline, not musical ground truth.

## Reproduction

```sh
cargo build --offline --locked -j 2
python3 scripts/official_grid_probe.py /path/to/official/Audiveris.app/Contents
python3 scripts/compare_official_grid.py
cargo test --workspace --offline --locked -j 2
```

Only regenerating the oracle requires the existing official distribution and Java; saved golden tests and native production execution need neither. No new software download, GUI, authentication, or external publication was performed in this milestone.
