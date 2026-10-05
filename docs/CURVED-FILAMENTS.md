# Curved filament factory milestone

`filaments INPUT BINARY.pgm REPORT.json` extends the pinned Audiveris 5.11.0 Rust port beyond rectangular seeds. It performs automatic horizontal `StaffFilament` geometry, both `FilamentFactory.mergeFilaments` passes with the intervening `expandFilaments` pass, the default compatibility conditions, line curvature/slope filtering, and comb sampling on all surviving curves. The old `filament-seeds` command remains a reproducible earlier checkpoint.

Translated geometry includes `CurvedFilament.computeLine/getPositionAt`, 0.5-interline centroid probe windows, 4-interline segment spacing, minimum probe weight, automatic endpoint centroids, `NaturalSpline` line/quadratic/cubic cases and natural derivatives, and `GeoPath.yAtX`. Upstream evaluates Bezier ordinates using the normalized endpoint-x parameter; the port preserves this behavior instead of substituting a different spline solver. Explicitly assigned endpoints and vertical-factory paths are outside this stage.

Translated factory conditions include coordinate/ordinate gaps, thickness and overlap space at up to three probes, thickness consistency, zero-space expansion contact, slope compatibility for long lines, stable descending-length traversal, ancestor/membership updates, and the deliberately frozen expansion search box. Section membership retains upstream abscissa/ordinate/id ordering. Touching at a diagonal corner alone is excluded by the original `GeoUtil.touch` rule.

Filtering includes `LinesRetriever.purgeCurvedFilaments`, central rotation >0.1, global slope from the longest 10% (ties-to-even count, minimum one), the 0.0002 slope floor, and 0.025 slope-difference/short-line exceptions. If curvature filtering leaves too few lines, diagnostics are retained and CLI exits 2. Empty lists produce a null global slope instead of continuing into upstream's removal/interactive control flow. More than 5,000 horizontal sections is an explicit native resource-limit failure, with exit 2 and no partial factory result. These guards are not upstream recognition rules.

## Evidence and limits

The oracle compiles unchanged `NaturalSpline.java` and `GeoPath.java`, and verbatim numerical/factory/filter/comb methods from the pinned Java source. Explicit lightweight horizontal Section/compound/scale/logger adapters replace infrastructure. The harness does not run the complete Audiveris app or its UI removal decisions. Source/method hashes are recorded in `evidence/curved-filament-parity.json`. This validates the stated horizontal automatic-endpoint path, not every configurable upstream branch.

For seven synthetic fixtures and the CC0 An Laura raster, final factory membership, accepted merge/expand event order, filter decisions, all defining spline points, nine ordinate samples per final factory filament and all comb samples match Java. Maximum measured geometry error is zero on these fixtures (fixed test tolerance `1e-9 + 1e-12*abs(expected)`). The golden suite and a corner-contact regression plus a resource-budget regression bring workspace tests to 26; clippy and formatting pass. Numerical parity is not recognition accuracy.

| Fixture | Factory filaments | Merge/expand events | Retained after line filters | Combs |
|---|---:|---:|---:|---:|
| broken | 5 | 10 | 5 | 19 |
| crossings | 5 | 15 | 5 | 19 |
| curved | 5 | 45 | 0 | 0 |
| noisy | 5 | 6 | 5 | 19 |
| sloped | 5 | 10 | 5 | 19 |
| straight | 5 | 0 | 5 | 19 |
| two-staves | 10 | 0 | 10 | 38 |
| CC0 An Laura | 207 | 720 | 163 | 703 |

The intentionally strongly curved fixture is rejected by the official curvature condition; it is not reported as a successful staff detection. Six `*-curved-merged.svg` files show original images beside sampled curve/comb debug overlays. Original image pixels remain unchanged.

**Historical boundary of the `filaments` command (26-test checkpoint).** The later `staff-candidates` command adds default initial Staff construction: see [current scope](STAFF-CLUSTERS.md). At this earlier checkpoint formal Staff creation was not implemented. `ClustersRetriever.followCombsNetwork`, recursive `LineCluster` assembly, cluster expansion/merge/collision checks, trimming, length/consistency rejection, smaller-interline second pass, and `LinesRetriever.buildStaves` remain next. The 163 real-page line candidates and 703 combs must not be interpreted as staff counts. Parent visual inspection suggests 2 systems × 3 staffs = 6 staffs; the separate JSON annotation is a future acceptance expectation, not an algorithm result or an implementation constant. The earlier 87 five-member diagnostic groups are also not Staffs.

The CC0 input remains the original 1489×2104, 180-DPI page-1 PNG and its verified grayscale normalization. MuseScore wrote the complete decoded/CRC-verified PNG then exited -6 under the restricted sandbox. That renderer qualification and all hashes remain in `fixtures/rendered/manifest.json`.

Reproduce after building:

```sh
cargo run --locked -- filaments fixtures/sections/sloped.pgm /tmp/sloped-binary.pgm /tmp/sloped-report.json
python3 scripts/filament_oracle.py tools/jdk/Contents/Home
python3 scripts/curved_filament_oracle.py tools/jdk/Contents/Home
python3 scripts/curved_overlay.py
cargo test --workspace --locked -j 2
```

The new oracle uses the preceding oracle's generated BasicLine/Section scaffold; run them in that order. Rust golden tests need no Java. [PDF acceptance requirements](PDF-INTEGRATION-ACCEPTANCE.md) are recorded separately; no input adapter, rhythm or MusicXML integration is claimed by this milestone.

A single local debug-build 180-DPI timing observation is retained in `evidence/curved-native-profile.txt`; it is not a general performance claim, hard memory guarantee or 300-DPI acceptance. Full preprocessing/filament JSON output size was recorded alongside it. RSS was unavailable because the timing tool could not read `kern.clockrate` in the sandbox; no RSS value or memory success is claimed.

Oracle independence: filament Java harnesses receive native SCALE and horizontal sections. Their calculations are independent within that stage, not a separate whole-image Audiveris run.
