# Pinned STEM_SEEDS dependency boundary

Reference: Audiveris 5.11.0, source commit
`9e1e55cd2746037d059345881c53e6a6754bffbd`, JAR SHA-256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.
This is the reference dependency audit. The bounded native implementation is now
connected and validated; see [STEM-SEEDS.md](STEM-SEEDS.md) for its scope.

## Original execution

1. Complete live `HeaderBuilder.processHeader` on every system
2. `StemSeedsStep.doProlog` preserves a user-assigned stem scale, otherwise calls
   `StemScaler.retrieveStemWidth`
3. Build a histogram of horizontal foreground run lengths 0 through sheet
   interline, using original `HistoKeeper`; runs longer than interline do not
   contribute. Original HiLo thresholds use minimum value 1, quorum
   `rint(area * 0.1)`, derivative `rint(area * 0.05)` and gain 0.1. Select the
   first returned peak; absent peak falls back to foreground main/max, ratio 1
4. For each system, `VerticalsBuilder.retrieveCandidates` filters vertical
   sections by strict `system.left < areaCenter.x < system.right`; it also
   supplies horizontal sections whose horizontal length is exactly one pixel
5. Original `StickFactory` constructs linked-section adjacency, core sticks,
   thickens them and adds adjacent/opposite stickers. Its configured thickness
   is `maxStem`, core length is `rint(interline * 1.5)` at default profile, and
   minimum side ratio is 0.4
6. For each retrieved filament, skip absent/tablature closest staff and centers
   strictly left of `staff.headerStop`. Register its original glyph, evaluate
   `StemChecker`, and accept grade >= 0.2 at the tested default profile. Accepted
   glyphs gain `VERTICAL_SEED` and enter system free glyphs. No `StemInter` is
   created at this stage

## Critical pinned-runtime buffer behavior

The source paints a `BufferedImage` to hide bars/connectors, headers and pixels
outside core staff paths. However, the pinned ImageJ `ByteProcessor.getBufferedImage`
returns a copy: those drawing operations do **not** change the returned
ByteProcessor used by `StemScaler`'s histogram. In every one of the 11 observed
rasters, the returned buffer and NO_STAFF before/after bytes are identical.

A separate 5×4 original API control paints the entire image white while the
ByteProcessor bytes remain unchanged. Preserve this observed behavior rather
than silently implementing the intended mask as a destructive erasure. The
oracle exports full compressed byte buffers, dimensions, black counts and
SHA-256 hashes; all encoded buffers were independently decoded and checked.

## What the present native inputs already provide

- GRID section runs/orientations and per-system horizontal/vertical memberships
- Sheet interline and foreground main/max scale
- Exact NO_STAFF pixels and image bounds
- Full live HEADERS stops, staff geometry and SIG state
- A bounded original HiLo translation from the key-peak stage

## Native implementation boundaries

- Stem-specific histogram/scale selection; preassigned scale overrides are outside the fresh CLI scope
- Vertical `StickFactory` linked-section graph, stable core selection,
  thickening/sticker rules and processing flags
- Orientation-aware multi-section `StraightFilament` line/endpoints and mean
  distance. Existing `filaments.rs` explicitly handles the horizontal staff
  factory and one-section line fitting; it is not this vertical stick factory
- Profile-dependent seed check suites and context side effects: slope,
  straightness, length, clean length, black length, black ratio and gap. Clean
  checking supplies black/white/largest-gap values used by later checks
- Per-system accepted VERTICAL_SEED membership, without inventing a final
  stem interpretation or claiming arbitrary Java/global numeric glyph IDs

The detailed oracle retains candidate order, member sections with orientation
and IDs, full glyph runs, line/endpoints/mean distance, nearest staff, skip flag,
check raw values/thresholds/weights/impacts and acceptance. The final accepted
seed list is canonicalized by glyph geometry for comparison; it is not a claim
about arbitrary global glyph IDs. Candidate acceptance order remains visible.

## Coverage and limits

`fixtures/official-stem-seeds` has Laura, two existing header rasters and eight
independent Bravura/stroke renderings. Clean, headed and mixed-width strokes
produce three seeds each; empty/noise/short/sloped controls produce zero;
broken strokes produce six fragment seeds. Laura produces 300 seeds. The two
older header rasters each have one interior-bar seed and are not negative seed
cases. All final scale, section, buffer, staff-input and accepted-glyph outputs
match untouched original `StemSeedsStep.doit` runs. There are 352 replayed check
contexts. Non-default profiles, tablature and preassigned stem scales remain
untested. No BEAMS, LEDGERS or final-stem recognition claim follows.

Receipts: `evidence/stem-seeds-oracle-validation.json`,
`evidence/stem-seeds-source.json` and `docs/LINUX-ORACLE.md`.
Four Java source files were fetched successfully. The subsequent official
GitHub source batch was cancelled at network approval and was not retried.
Additional `.javap.txt` files are local bytecode inspections of the already
verified JAR, explicitly distinguished from fetched Java source.

## Isolated native checker increment

`src/stem_checks.rs` now implements profile 0/1 check scoring and pixel sampling
without Java or fixture lookup. All 352 observed class-profile-1 contexts match
raw values, impacts, weights, black/white/gap counts, grade and acceptance with
zero floating-point delta in debug and release. Source-derived units additionally
cover profile 0, gaps, forced-black endpoints, attached ink, explicit rejection
bounds, and valid ImageJ linear-index side-belt wrapping. Unsupported profiles,
actual out-of-array sampling and the near-horizontal sanity/NaN path fail
explicitly. See `evidence/stem-checks/validation.json`.

The original histogram supplement has 16 declared-bin controls, including empty,
terminal Hi/Lo, half-even threshold areas, stable equal peaks and counts above
32767. It calls original HistoKeeper/computeStem; see
`evidence/stem-scale-control-validation.json`.

## Direct stick/fitting controls

`fixtures/official-grid/stick-factory-controls.json` contains 26 direct-original
factory cases and four original multi-section line fits, reproduced byte-for-byte
in a second process. They cover core length/thickness limits, stable equal-length
ordering, the exact 0.4 thickening ratio, one-column stickers, opposite-sticker
intersection and non-axis-aligned fitting/sample rounding. A reversed unsorted
list and two intentionally overlapping coordinate-tied neighbor cases are marked
explicitly. They do not claim to be natural GRID segmentation output.

These controls expose that original `retrieveSticks` preserves the supplied
vertical-section sequence for stable core ties. The system's normal section
sorting belongs to its caller, not an extra sort inside the original factory.
See `evidence/stick-factory-control-validation.json` for pins, hashes and receipts.
