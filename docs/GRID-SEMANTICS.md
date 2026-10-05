# GRID semantics and automatic staff-mask checkpoint

Follow-up: [GRID context and initial HEADERS stage](GRID-CONTEXT-HEADERS.md) adds
freeze/exclusion checks, a controlled positive line-root oracle, and explicit
brace-fallback refusal. The test counts below describe this earlier checkpoint.

Pinned Audiveris 5.11.0 source: `9e1e55cd2746037d059345881c53e6a6754bffbd`.
The native runtime never invokes Java. `grid_complete:false` remains mandatory.

## Changed behavior

Semantic peak removal now occurs before right-end refinement and before final
StaffLine completion. The final interpretation builder consumes that same
filtered peak/edge state instead of independently repeating the purges after
geometry was fixed. Explicit left/right end flags are retained; equality with a
coordinate is not used as a substitute for a left-end flag. C-clef removal occurs
AFTER right-end refinement, as in the official method order.

The port now includes:

- `purgeLeftOfBraces` and `checkLinesRoot` logic; ordinary left, unaligned and
  extending peak purges; column-wide deletion and edge removal.
- C-clef first/second-peak width, measure-gap, vertical-connection and tail checks.
  The original list-mutation/index skip behavior is preserved.
- Bracket serif ROI selection, exclusion of current/right peak sections,
  connected compounds, centroid-distance ordering and minimum aggregate weight.
  Top/bottom attributes propagate through actual connections to middle portions.
  Candidate sections per serif ROI are capped at 512; excess returns a diagnostic
  failure before quadratic connectivity work.
- Bracket interpretations, connectors and support relations, width-group
  exclusion, nested-level bracket/square/brace group descriptors, and removal
  of true brace groups after creating a multi-staff part.

Bracket interpretation bounds depend on official font layout, not the observed
serif glyph box. Embedded Bravura 1.392 layout measurements cover interlines
**11 and 16 only**. Other bracket sizes return `unsupported_bracket_font_metrics`;
there is no scaling/interpolation/fallback. Font/JAR/measurement hashes and the
actual generation command are in `evidence/bracket-font-provenance.json`.
OFL text remains in `crates/audiveris-template-bank/assets/BRAVURA-OFL.txt`.

## Independent original-image verification

Thirteen original grayscale rasters were run through unchanged official GRID.
No native geometry, memberships, flags or oracle coordinates are fed into the
native pipeline. Phase observation checks surviving peaks, C-clef removal,
endpoints, bracket flags, serif section identities and weights. Final observation
checks all interpretations, relations (mapped by content), glyph pixels,
StaffLine geometry/thickness, horizontal/vertical sections and memberships,
parts, groups and page descriptors. Equality of floating values uses the existing
1e-9 geometry/grade tolerance; pixel/run/identity data are exact.

AnLaura remains 38 interpretations, 30 relations and 50 glyphs. New fixtures:

|Case|Observed behavior|
|---|---|
|cclef-pair|Both C-clef peaks removed|
|cclef-tail|Both peaks and tail removed|
|cclef-too-wide|Historical filename: second width is 5px, exactly the rounded 0.3-interline limit; removed|
|cclef-thick-second|6px second peak exceeds the limit; retained|
|cclef-distant|Pair beyond the near-measure range retained|
|cclef-connected|Connected pair retained|
|right-extending|Overextending final peak removed before right-end refinement; resulting right endpoint 858|
|line-root|Short-root raster; natural path does not trigger `checkLinesRoot` correction|
|bracket-pair|TOP/BOTTOM bracket and connector, 15 interpretations/15 relations|
|bracket-middle|TOP/MIDDLE/BOTTOM, 25 interpretations/30 relations|
|bracket-small-serif|Final bracket column removed; actual intermediate serif selections also compared|

The existing synthetic-system-bars fixture is the thirteenth case. The name
`line-root` is not evidence of a positive root-correction branch: that branch is
ported but still needs a fixture that reaches it. General brace fallback alignment,
nondefault layouts and exclusion/freeze semantics remain incomplete.

Reproduction uses the already installed pinned release and existing compiler:

```
python3 scripts/make_semantic_fixtures.py
python3 scripts/official_semantic_probe.py CONTENTS JDK_HOME
python3 scripts/official_semantic_final_probe.py CONTENTS JDK_HOME
cargo build --offline --locked
python3 scripts/compare_semantic_probe.py
cargo test --workspace --offline --locked -j 2
```

`evidence/grid-semantic-parity.json`, `grid-semantic-final-oracle.json` and
`grid-semantic-validation.json` record actual results. Debug and release passed
**55 tests each**, including all thirteen final GRID/mask comparisons; workspace
clippy with warnings denied, formatting and whitespace checks passed.

## Automatic mask selection, before HEADS dependencies

`staff-mask INPUT MASK.pgm REPORT.json` takes no probe coordinates. It runs the
native image/GRID pipeline, selects actual staff-line glyph pixels, and paints
complete persistent StaffLine curves through crossing symbols with the official
mean-glyph-thickness and rounding rule. `page-glyphs` also reports this preparation.
Mask pixels are 0 for ignored, 255 for retained, directly usable as the optional
mask for `template-probes`.

At GRID, ledgers and vertical seeds do not yet exist. The original Java
`DistancesBuilder.paintLines` was observed through its Painter interface to get
its exact staff mask; the unchanged `buildDistances` independently produced the
distance table. All thirteen native mask hashes, masked-pixel counts and complete
short-distance-table hashes match. This establishes automatic GRID staff-mask
selection, **not a complete HEADS mask or automatic note recognition**.

The report explicitly says `source_stage:GRID`, `heads_mask_complete:false`, and
`note_recognition_complete:false`. No automatic note candidates are emitted.
The 4M-pixel/32MB decoder limits and the 300 DPI PDF blocker remain unchanged.

## Next implementation boundary

Official step order is GRID → HEADERS → STEM_SEEDS → BEAMS → LEDGERS → HEADS.
Before enabling official-style automatic HEADS search, native work still needs
header/competitor data, stem seeds, beam/ledger interpretations and their masks,
measured head width and supported bank-size selection, candidate search,
competition/aggregation and inter creation. A raw component, sampled template
position, or staff-only mask does not satisfy those dependencies. Rhythm and
MusicXML remain later work.
