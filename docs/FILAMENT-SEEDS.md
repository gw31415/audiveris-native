# Initial filament seeds and restricted staff-comb candidates

This milestone ports actual Audiveris 5.11.0 algorithms from commit `9e1e55cd2746037d059345881c53e6a6754bffbd`. It adds `filament-seeds INPUT BINARY.pgm REPORT.json` to the native CLI. Java is only an oracle.

Implemented: `FilamentFactory.createInitialFilaments` section-length gate, `isSectionFat` (mean thickness, aspect, BasicLine regression, two quarter-position pixel probes), `BasicLine` horizontal/vertical least-squares choice and normalization, and `ClustersRetriever.retrieveCombs`/`retrieveFilamentsAtX` sampling and consecutive ordinate-gap grouping. Default scale conversions preserve `Math.rint` ties-to-even, exclusive Rectangle edges, and inclusive filament endpoints. Numerical line accumulation uses the original BasicSection increasing-x point order.

**Boundary:** these are initial one-section filament seeds, before official merge/expand. Only rectangular, constant-thickness horizontal seeds enter the comb adapter; their centroid ordinate is constant and does not need a curve approximation. Nonrectangular seeds retain fitted section-line coefficients for inspection but are excluded from combs. This is not a substitute for `StaffFilament` curve interpolation. The remaining `canMerge`, `mergeFilaments`, `expandFilaments`, NaturalSpline/CurvedFilament processing, slope/curvature purges, network following and `LineCluster` operations are not implemented.

`five_line_seed_groups` is a native diagnostic deduplication of exactly five-member combs by section IDs. It is **not an upstream Staff or LineCluster result**. Broken/noisy staves can produce multiple overlapping groups, and spurious musical/text fragments can form groups. The JSON names the limitations explicitly: `staff_detection:not_complete`, `merge_expand:not_implemented`, `curve_interpolation:not_implemented`, `comb_input_scope:rectangular_unmerged_seeds_only`. No recognition accuracy or staff count is claimed.

## Oracle evidence

`scripts/filament_oracle.py` extracts the original Java numerical/selection/comb methods verbatim and records each method hash in `evidence/filament-parity.json`. The harness supplies small horizontal Section/Rectangle/PointsCollector adapters and constant-ordinate rectangular-filament adapters. Thus the oracle validates the explicitly restricted stage; it does not run Audiveris GRID end to end. Its input is the section table previously verified independently against original Java SectionFactory.

Eight populated fixtures (seven synthetic plus CC0 An Laura) match decisions and every sampled comb exactly. BasicLine coefficient error is zero in these cases. Blank input is blocked by SCALE and does not fabricate filaments. Persistent Java goldens in `fixtures/filament-golden.json` cover 2,702 sections; the Rust regression test compares all decisions, seed geometry and combs. All 23 workspace tests, formatting, and clippy with warnings denied pass.

| Fixture | Seeds | Rectangular seeds used by combs | Combs | Diagnostic five-member groups |
|---|---:|---:|---:|---:|
| straight | 5 | 5 | 19 | 1 |
| two-staves | 10 | 10 | 38 | 2 |
| broken | 15 | 15 | 17 | 3 |
| sloped | 15 | 10 | 4 | 2 |
| curved | 50 | 30 | 40 | 4 |
| noisy | 11 | 11 | 19 | 7 |
| crossings | 20 | 20 | 18 | 4 |
| CC0 An Laura | 727 | 652 | 712 | 87 |

The real score's 87 diagnostic groups demonstrate why these are not staff detections. Completing merge/curve/network logic is necessary before connecting this stage to staff removal or the existing isolated-glyph classifier.

## Review artifacts

`evidence/visual/` contains six standalone SVGs, with the unmodified original image on the left and a separate debug layer on the right. Cyan outlines rectangular seeds, orange other seeds, and magenta marks sampled combs. They embed their input PNGs and do not modify source image files. Java/native equality is documented numerically, not inferred from the overlay.

`fixtures/rendered/laura-180dpi.png` is the original 1489×2104 RGBA PNG, page 1 at 180 DPI, SHA-256 `958288a4fcefefa8ea899cbd58dd6d77598dd592974072fff8baa3f5566243df`. Its chunk CRCs, decompressed raster and normalized grayscale pixels were read successfully. The source MSCX and CC0 notice remain in `fixtures/openscore-lieder/`. **MuseScore exited -6 after writing this PNG under a write-restricted sandbox. A successful renderer process, 300 DPI, other pages, or reproducible full conversion is not claimed.** The manifest preserves this qualification and the source/PNG/PGM hashes.

Reproduce after building:

```sh
cargo run --locked -- filament-seeds fixtures/sections/straight.pgm /tmp/straight-binary.pgm /tmp/straight-report.json
python3 scripts/filament_oracle.py tools/jdk/Contents/Home
python3 scripts/filament_overlay.py
cargo test --workspace --locked -j 2
```

Original upstream sources and a test-only JDK are prerequisites for regenerating Java goldens; ordinary Rust tests use committed goldens and need no Java. Existing output paths are refused.

Next: port real filament geometry and merge decisions, retaining section membership and numerical checkpoints; then official comb network/LineCluster pruning and staff creation. The independent rational/time crate (`2cc72301ba61d77e2743391205142f729e124c68`) is intentionally not imported yet: rhythm is a later stage, and its Java-compatible versus exact arithmetic contracts must remain distinct. PDF/page input is also being developed independently. MusicXML and music-relationship inference remain unfinished.
