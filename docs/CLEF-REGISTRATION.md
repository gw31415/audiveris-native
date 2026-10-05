# Native clef registration and Linux oracle

This checkpoint continues the recovered 2d523da worktree and its unfinished native clef candidate stage. The original dirty work was preserved in local commit 1bbeb68 before further development.

## Native behavior

`page-glyphs` and `staff-mask` now include `clef_registration`. Candidate glyphs still originate from the native NO_STAFF raster, connected glyph-part enumeration, native 110-feature extraction and pinned native classifier inference. Neither test fixtures nor oracle snapshots supply runtime recognition results.

The new stage translates the pinned original `ClefBuilder.registerClefs`:

- Select the first-pass candidate map unless empty, otherwise the second pass
- Stable decreasing intrinsic-grade order, preserving ClefKind enum order on ties
- Exact font-layout bounds with Java ties-to-even rounding and integer glyph-box center
- Horizontal-only glyph centroid positioning, retaining the original vertical position
- Clef pitch is retained from its kind: `ClefInter.setBounds` specifically overrides the pitched superclass and does not recalculate pitch
- First candidate glyph/symbol intersection right edge supplies the staff clef stop
- All candidate pairs receive OVERLAP exclusions in the registration snapshot
- The largest relative clef offset is propagated to each staff's header stop within the same system, matching `HeaderBuilder.setSystemClefStop`

These are candidate registrations, not final selected header clefs. Key recognition can remove clef competitors and change contextual support before final selection. The snapshot marks `clef_selection_complete` and `headers_complete` false.

## Generic font metrics

`assets/clef-font-metrics.json` stores 768 exact numerical layouts: six permitted Bravura header-clef shapes at every integer interline 1–128. This is the full supported candidate-stage scale range, not a table of image-specific placements. Bounds, normalized alpha-weighted centroid offsets and pitch offsets were obtained from the hash-pinned original Java implementation. Runtime uses only the numerical bank; it never invokes Java or a font renderer and never interpolates unsupported entries.

The bank is checked against a compiled SHA-256 plus pinned upstream/JAR/font identity. Bravura attribution and SIL OFL are retained in `assets/BRAVURA-OFL.txt`. The underlying official font remains outside the native runtime assets.

## Independent validation

The additive `scripts/linux_oracle.py` harness downloads checksum-pinned official Linux Audiveris and a development JDK only when explicitly invoked with `--bootstrap`. Compilation shims are excluded from its runtime classpath. Each run writes a new evidence directory and never overwrites preserved fixtures. No Rust runtime code invokes the harness.

The original official Linux distribution reproduced all ten prior clef candidate fixtures exactly. New registration snapshots come from original `findClefs`/`registerClefs` on the source rasters, including rendered G/F/C clefs and empty/noise negatives. Native comparisons cover candidate geometry, grades, kind, pitch, registered symbol bounds and stop; structural and rounding tests cover ties, exclusions, centroid behavior and system-relative stop propagation. Controlled multi-candidate tests are distinguished from natural-image ambiguity evidence.

Pinned implementation copies and hashes are recorded in `evidence/clef-registration-source.json`. Upstream is Audiveris 5.11.0 at `9e1e55cd2746037d059345881c53e6a6754bffbd`; official JAR SHA-256 is `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.

## Boundaries

Single-scale, single-column standard five-line staff support remains explicit. Other fonts, unsupported staff layouts and out-of-budget inputs are not silently approximated. General GRID completion, key/time recognition, final clef selection, STEM_SEEDS/BEAMS/LEDGERS, automatic head search, rhythm and MusicXML remain unfinished. The 4-million-pixel input limit remains; ordinary 300 DPI A4/PDF support is not claimed.
