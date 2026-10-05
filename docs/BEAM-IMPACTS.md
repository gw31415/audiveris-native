# Native BeamItem mask evaluation and impacts

Scope: direct translation of the pinned Audiveris 5.11.0 item methods at
`9e1e55cd2746037d059345881c53e6a6754bffbd`, using the already-local official JAR.
This is a method-level component, not a completed BEAMS builder.

`beam_impacts::Evaluator::new(&no_staff, interline)` validates and borrows the
unmodified NO_STAFF raster. `compute_beam_impacts(item, top, bottom, dist, p)`
and `compute_hook_impacts(item, dist, p)` return counts, method rejection and
optional impacts. Hooks always sample both vertical belt sides. Beam callers
provide the outer-line side flags; this module does not choose them.

## Original behavior preserved

- Core: horizontal parallelogram through the item median, full item height
- Belt: original `LineUtil.intersectionAtX` arithmetic expands the median by
  the rounded horizontal margin; shifted enlarged parallelogram minus core
- Original `Area.subtract` normalization is required. Simple outer-minus-core
  point booleans differ at exact integer boundaries for sloped items
- The cached Rectangle2D bounds retain contour-order rounded accumulation;
  direct extrema can change even an integer row of a horizontal beam
- Sampling uses integer `(x,y)` points, not pixel centers. Only byte value 0 is
  foreground; the input is never thresholded or changed
- Out-of-sheet points count in the denominator but never as foreground
- Width is ties-to-even `rint(x2-x1+1)`, separate from raw median span
- Gates reject width below low, height below minimum or above maximum, core
  ratio below 0.7, belt ratio above 0.4. Equality passes these method gates
- Width, minimum/maximum height, core, belt and supplied distance/jitter impacts
  are clamped to [0,1]. Weights are `[0.5,1,1,2,2,2]`; grade is their weighted
  geometric mean times the intrinsic ratio 0.8
- Both inherited minimum grades are `0.8*0.1 = 0.08000000000000002`. The 0.35
  good-beam threshold is a different concept and is not used here
- A non-null impact result does not mean an accepted candidate. Exact boundary
  ratios/heights can yield zero grade. `accepted_as_beam/hook` performs `>=`
- Empty sampled masks preserve original 0/0 NaN impacts and failed acceptance,
  separately from method rejection. A later zero jitter resets the original
  grade to finite zero while the NaN impact remains: `non_finite` covers the
  impacts as well as the grade. JSON represents nonfinite numeric entries
  as null together with the explicit `non_finite: true` flag

The pure Rust runtime neither starts Java/subprocesses nor reads fixture data.
The separately licensed `audiveris-area::LinearArea` retains OpenJDK
GPL-2.0-only WITH Classpath-exception-2.0 attribution. The BeamItem/BeamsBuilder
adapter retains Audiveris AGPL-3.0-or-later terms.

## Independent verification

`fixtures/beam-impacts-controls.json` holds direct calls into the unmodified
original methods, using independent rectangles, rasters, and supplied medians.
There are 307 cases, 158,989 integer mask points and 120 returned-impact cases.
The suite compares every sampled coordinate, foreground/total count, rounded
width, impact presence/value, grade and threshold acceptance. Finite grades and
impacts matched exactly on this cloud platform, including near-minimum-grade
controls. Degenerate NaNs are compared by classification, never as an accepted
candidate. A declared 1e-15 numeric comparison limit for finite grades is only
for platform pow differences; count, mask, threshold and impact tests are exact.

Controls cover top/bottom side flags; ties-to-even scale margins and widths;
minimum/maximum heights; core and belt ratio equalities and immediate neighbors;
out-of-sheet masks; nonzero-gray foreground exclusion; sloped and subpixel
boundaries; noisy/sparse and dense sloped rasters; distance clamp/grade boundary;
and zero-area masks. Additional tests reject malformed images, nonfinite
geometry/parameters, vertical/reversed median, and exhausted sampling budgets.

Reproduce with the cached local binaries, with no downloads or Mac usage:

```
python3 scripts/run_beam_impacts_controls.py --output /NEW/EVIDENCE/DIRECTORY
cargo test --locked --test beam_impacts -- --nocapture
cargo test --locked --release --test beam_impacts -- --nocapture
```

The observer allocates only the enclosing builder with Unsafe and injects its
original Parameters and raster; original BeamItem, AreaMask, private item
methods, GradeImpacts and minimum-grade methods execute unchanged. No builder
prechecks or registration decisions are replayed as if they were original calls.
Source, input/output, runtime/compiler hashes and executed commands are under
`evidence/beam-impacts/original-controls/provenance.json`.

Native sampling bounds are 8,000,000 combined core/belt bounding-box cells per
item by default. Geometry normalization caps input edges and sweep work; resource
errors are native limits, not claimed original rejections. Callers remain
responsible for page-level aggregate budgets and candidate orchestration.

Still outside this component: glyph/line selection, jitter computation and
policy, candidate registration/order, glyph retrieval, overlaps, extensions,
hook discovery, cue aggregation, beam groups, MultipleRests and downstream
notes/rhythm/MusicXML. The parent builder integration is verified separately.

## Independent bounds review

A second review found and corrected cached Rectangle2D rounding in the line-only
adapter. The 576 new actual-parallelogram cases compare 424,201 integer core/belt
queries and exact bounds. Before the fix seven bounds differed and three core
counts differed; one horizontal ULP-neighbor item had 128 instead of the
original 112 sampled points. The confined correction retains contour-order
move-to/line bounds accumulation, integer conversion and the original bounds
containment precheck. It does not broaden CAG functionality.

Three additional direct-original controls use a sheet-edge beam and explicit
core-ratio parameter overrides to test the changed denominator's effect on
method rejection, finite grades and acceptance. Eight additional empty-mask
controls cover positive/negative zero and negative jitter, including six cases
with NaN impacts but finite zero grade. Native tests, the minimized reproduction,
source hashes, bytecode identity checks and run logs are under
`evidence/beam-impacts-review/`; run `cargo test --locked --test beam_impacts_review`.
