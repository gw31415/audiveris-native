# Initial five-line Staff geometry

`staff-candidates INPUT BINARY.pgm REPORT.json` now runs native preprocessing, scale, sections, curved filament factory, combs, cluster assembly and initial Staff construction. It writes `staff_candidates` with eight stage checkpoints, membership decisions and ordered initial staff descriptors. This is a real translation of Audiveris 5.11.0 commit `9e1e55cd2746037d059345881c53e6a6754bffbd`; Java is test-only.

Translated rules include ancestor connection through comb networks; recursive `LineCluster` pivot/offset inclusion; extrapolated ordinates; expansion with thickness collision tests; cluster matching/merging; weak-end trimming; rejection outside five lines; pair merging and median-relative length rejection; and a second expansion. `LinesRetriever.buildStaves` then applies the official minimum width of 30 interlines, layout ordering, median line endpoints, staff IDs and short-neighbor flags. Preserve upstream stable traversal, ties-to-even scaling, strict tie selection, and the original inclusive-stop hole-length arithmetic. Explicit safety limits (5,000 sections, recursion depth 1,024) are native resource guards, not recognition rules.

Supported processing settings: default five-line staffs, one interline population, horizontal automatic endpoints. A second interline population returns `unsupported` / exit 2; one-line percussion and tablature switches are not exposed. Empty initial Staff output after official minimum-length rejection is a completed geometry stage, not successful whole-page music recognition. Strong curvature and failed SCALE remain blocked with diagnostic exit 2. Later GRID completion, line-end refinement, staff removal, systems/parts, glyph segmentation, musical relationships/rhythm, and MusicXML are not implemented.

## Validation

`scripts/cluster_oracle.py` extracts the complete original `LineCluster` and `FilamentComb` class bodies, and verbatim cluster/buildStaves/short-neighbor methods, into a headless harness. Infrastructure adapters supply sections, scale, sheet metadata and initial Staff data storage. Original method hashes and upstream pin are recorded. Debug checkpoints preserve lazy-cache state. Java receives the native stage's SCALE and horizontal sections; it independently computes filament/cluster/Staff stages. Golden Rust tests consume recorded factory inputs and Java expectations. **This is stage-by-stage numerical/decision parity, not an independently executed image-to-Staff Audiveris application comparison.** Earlier raster-to-section oracles cover their own boundary independently.

Nine fixtures compare all eight checkpoints (cluster membership/order/bounds/true lengths), network survivors/discards, initial staff IDs/endpoints/interlines/flags and spline defining points. Integer/discrete decisions match; float tolerance is `1e-9 + 1e-12*abs(expected)`. No recognition-accuracy or performance percentage is inferred.

| Input | Filtered lines | After network | Created clusters | Five-line survivors | Initial staffs |
|---|---:|---:|---:|---:|---:|
| CC0 An Laura, page 1, 180 DPI | 163 | 121 | 8 | 6 | 6 |
| Long synthetic single staff | 5 | 5 | 1 | 1 | 1 |
| Long synthetic two staffs | 10 | 10 | 2 | 2 | 2 |
| Six earlier short synthetic inputs | 5 or 10 | 5 or 10 | 1 or 2 | 1 or 2 | 0 |

Short fixtures correctly fail the official 30-interline width threshold. The real-page result agrees with the separately recorded human expectation of 2 systems × 3 staffs, but the implementation does not read or encode that expectation. It does not yet calculate those two systems. The source PNG and renderer exit -6 qualification remain in `fixtures/rendered/manifest.json`; 180 DPI is not 300 DPI acceptance.

Reproduce after the earlier filament and curved oracle steps:

```sh
cargo build --locked -j 2
python3 scripts/filament_oracle.py tools/jdk/Contents/Home
python3 scripts/curved_filament_oracle.py tools/jdk/Contents/Home
python3 scripts/cluster_oracle.py tools/jdk/Contents/Home
python3 scripts/staff_overlay.py
cargo test --workspace --offline --locked -j 2
cargo clippy --workspace --offline --all-targets -j 2 -- -D warnings
```

The workspace contains 29 passing tests, including two independent-review regressions: reject grayscale PNG tRNS transparency, and do not print FILAMENTS complete when SCALE blocked it (fix commit `2993ab6`). The original curved checkpoint had 26 tests. Java/source checkpoints are evidence, not production dependencies. See `evidence/cluster-parity.json` and the unchanged-raster side-by-side `evidence/visual/laura-initial-staffs.svg`.
