# Native key projection and geometric slices

Pinned Audiveris 5.11.0 source: `9e1e55cd2746037d059345881c53e6a6754bffbd`.
The runtime is Rust. Java observers are development-only independent oracles.

## Implemented boundaries

`key_preparation` consumes the native NO_STAFF raster, canonical GRID and registered
clefs. It implements the initial part of `KeyColumn.retrieveKeys`,
`Staff.getBrowseStop`, `KeyBuilder.getBrowseRect` and `KeyExtractor.getProjection`.

- Header stops already include the system's maximum **relative** clef offset
- Browse start is individual clef stop + 1, or system-propagated header stop + 1
- Default maximum x is header start + 15 sheet interlines
- Only a barline with intrinsic grade >= 0.6 and an incident
  `BarConnectionRelation` clips browsing. Barline iteration follows integer-center
  order, and the bar's left boundary must be strictly beyond browse start
- The upstream browse rectangle uses first/last line ordinates at browse start
  throughout its loop. It extends 2 specific staff interlines above and 1 below
- Pixel lookup clamps the rectangle to the image. The projection starts at the
  earlier of measure start and clamped rectangle x; it includes pre-browse columns
- Counts are actual zero-valued NO_STAFF pixels, with inclusive x and y endpoints

The shared `staff_inputs` parser requires explicit per-staff interline metadata.
The canonical GRID producer currently emits its supported single-scale value.
Missing metadata and mixed specific interlines are explicitly refused.

`key_peaks` ports the bounded generic HiLoPeakFinder: derivative hysteresis,
first-maximum ties, descending-height processing, neighbor bounds, gain-ratio
widening, quorum and the original unfinished-terminal behavior. KeyBuilder's
defaults are min value rint(0.4*IL)+1, derivative rint(0.35*IL), quorum
rint(1.25*IL) and gain ratio 0.2. Absolute function/subdomain coordinates are
preserved. The older SCALE peak finder is unchanged. Nonpositive gain ratios are
explicitly unsupported because the original routine can overrun that domain.

`key_slices` consumes the same typed staff input, native projection and native
HiLo peaks. It implements each `KeyBuilder.ShapeBuilder` through `allocateSlices`:

1. Browse spaces and peaks, testing actual image rows for a stem-like feature
2. Refine the initial boundary with a narrower staff-height pixel projection
3. Merge adjacent peaks and purge weak peaks near another peak
4. Infer a geometric sharp/flat count; refine trailing space and inter-item spacing
5. Compute starts and allocate slice rectangles

FLAT runs before SHARP, matching the upstream `EnumMap` declaration order. Range
start and explicit stop remain nullable; a null explicit stop is not replaced with
the browse stop when applying `shrinkStop`. Original intermediate range/peak states
are exposed for comparison.

The reported `signature` is a geometric hypothesis, **not a recognized key**.
No alteration classifier, hard-slice fallback, clef/key compatibility, cross-staff
key replication, final key selection or freezing is included in this stage.
`key_recognition_complete` and `headers_complete` remain false.

## Evidence

- Projection: 10 independent original-image runs, 17 staffs and 3,647 integer
  projection entries, plus all ROI/header fields, match exactly
- Controlled original-object observer: 36 bar threshold/connection/x-boundary
  cases, including 0.6 +/- one ULP, and 3 clamped projections match exactly
- HiLo peaks: all 10 original-image ranges and 113 named/fixed-seed controlled
  original peak cases match; these controlled inputs are not natural image ambiguities
- Slices: 16 independent original-image runs match every recorded intermediate
  range/peak state, inferred/refined count, start and final rectangle
- Positive independent slice fixtures include one/three sharps and one/four flats;
  empty and noise fixtures exercise negative results
- Six source-derived unit tests cover nullable stops, inclusive stem-row quorum,
  strict light-peak distance, odd-sharp removal, interval truncation, first-minimum
  tie behavior, empty results and malformed-input rejection

The controlled slice unit tests are not independent Java branch-edge oracles.
Curved-line/xMin behavior has source-derived unit coverage and natural image
coverage; exhaustive curvature and every shape-slicing branch are not claimed.

Sources and hashes are in `evidence/key-preparation-source.json` and
`evidence/key-peaks-source.json`, plus `evidence/key-slices-source.json`.
Oracle commands, distribution/JAR hashes, inputs
and output hashes are retained under `evidence/key-preparation/` and
`evidence/key-slices/`; fixtures are in `fixtures/official-grid/`.

Reproduce the focused checks:

```sh
cargo test --offline --locked --test key_preparation --test key_slices
cargo test --offline --locked --release --test key_preparation --test key_slices
cargo test --offline --locked --lib key_slices::tests
cargo clippy --offline --locked --test key_preparation --test key_slices --lib -- -D warnings
```

## Explicit support limits

Five-line staves, binary NO_STAFF inputs and pinned default constants are required.
Projection is limited to 8,192 columns and 1,048,576 sampled pixels, with no Java
short overflow. Slice preparation accepts at most 128 ordered non-overlapping
HiLo peaks. Unsupported/empty clamped regions, invalid curve geometry, too-short
stem windows, out-of-domain projections and negative slice widths fail explicitly.
No partial key result is returned on these failures.

Existing GRID, font, image-size, PDF and downstream notation limits still apply.
The 4M-pixel input cap remains. Time recognition, final clef selection, header
freezing, STEM_SEEDS, BEAMS, LEDGERS, automatic heads, rhythm and MusicXML are still
open. This milestone does not complete image/PDF-to-MusicXML.
