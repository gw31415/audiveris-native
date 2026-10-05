# Second milestone: fixed contracts

Baseline remains Audiveris 5.11.0 / 9e1e55cd2746037d059345881c53e6a6754bffbd. No custom constant/user scale overrides. CLI input contract remains <=4,000,000 grayscale pixels and <=32,000,000 encoded bytes. No GRID/classifier work in this milestone.

## Memory before and after

Original native RandomFilter-style storage: two u64 padded integral images = `16*(w+1)*(h+1)` bytes, alongside input/output raster and encoded input. Original run JSON serialization retained entire run tables and temporary strings.

New column-major VerticalFilter stores two circular u64 tiles = `16*min(w,38)*h` bytes, with the same clipped 37x37 threshold arithmetic. Only the columns relevant for random access within the current x window are retained; narrowing to min(w,38) is exact when image width <38. Two rasters require `2*w*h` bytes. Encoded input is dropped after decode; run JSON is streamed with an 8 KiB BufWriter. Formula excludes allocator/runtime/decoder overhead and is not an RSS benchmark.

Examples: 2000x2000 integral storage decreases from 64,064,016 to 1,216,000 bytes. Worst tall 1x4,000,000 at the input cap uses 64,000,000 tile bytes (72,000,000 including rasters), avoiding a naive 38-column 2.43GB allocation. PNG decoder allocation limit remains 32 MB. Kept `AdaptiveFilter` is a random-access test/reference API; CLI uses `vertical::binarize` exclusively.

Fixed pixel tests compare all contexts and outputs at 1x1, 1x79, 79x1, 18x37, 37x18, 38x43, 121x87. Existing Java fixture hashes must remain unchanged. Scale fixtures will separately cover clean staves, two staff populations, nearby interline merge, dual line thickness, significant/guessed beams, blank/no-combo/too-low/too-high interline and sparse black cases.

## Renderer availability

MuseScore 4.7.2 is installed. A read-only `--help` invocation with `QT_QPA_PLATFORM=offscreen` and workspace XDG config/cache paths failed: installed Qt platform plugins contain only `cocoa`, no `offscreen`. No Cocoa/GUI invocation was attempted. `pdftoppm` executable is unavailable at the Homebrew path. CC0 MSCX sources remain hash-verified but unrendered. Current image validation is synthetic only; no scanned/real engraving accuracy claim.

## Implemented SCALE path

`audiveris-native scale INPUT BINARY.pgm REPORT.json` runs streaming adaptive binarization, then automatic scale estimation with upstream default constants. The JSON keeps preprocessing v1 fields and adds `scale` (status/error, histogram limits, full black/combo bins, final finder peak lists, selected `[min,main,max]` line/interline ranges, optional smaller interline, measured or extrapolated beam and optional small beam). No classifier or GRID code is added.

Direct translations:

- `ScaleBuilder.HistoKeeper`: max black `rint(height*.08)`, max white `rint(height*.25)`, vertical black and B1+W/W+B2 histograms, foreground density test, interline/line peak selection.
- `HiLoPeakFinder`: derivative hysteresis, stable decreasing-count order, first-maximum ties, 3% gain extension and overlap limits. Preserve no final HiLo flush and derivative threshold zero behavior.
- `IntegerFunction`: ignore out-of-domain histogram additions, local maxima selection and stable ties. Unlike prose comments, `buildCombos` does not independently discard oversized black runs; preserve actual code behavior.
- `ScaleBuilder.computeBeamKeys`: 0.45–0.95 interline search range, quorum, guess and possible two beam populations. Use signed bounds, because the Java range may be inverted on implausible inputs.
- Java `Math.rint` is represented by Rust `round_ties_even`; not ordinary half-up rounding. Finder lists reflect Java `subList` removals, while selected merged ranges are separate.

CLI exit 0 = scale success, 2 = scale rejected with valid retained diagnostic JSON, 1 = input/IO error. Rejection codes are `no_regular_lines`, `too_few_black_pixels`, `no_significant_black_lines`, `interline_too_low`, `interline_too_high`. This is normalization of ScaleBuilder's reason/message; full Java SheetStub ultimately throws generic `Sheet ignored`/`Sheet removed`, which is not copied into the native interface.

No user-specified interline/beam, preexisting Scale reuse, dummy chart mode, sheet/book removal, plot/UI side effects, or persistent `.omr` scale object is implemented. JSON ranges are native IR, not serialized Java classes.

## Differential results

`evidence/scale-parity.json` records **18 fixtures**, full scale JSON equality (every histogram bin, remaining peak triplet, final optional fields and failure code), plus equal SHA-256 of the full binary raster on Java and Rust paths. Cases include clean 16px staff, 16/24 populations, 16/18 merge, distant-peak discard, 2/4px double line, measured/significant/guessed/two beam modes, overlong runs, rint tie, blank, single line, all black, sparse, absent black peak, <11px and >100px interline. All Java-accepted cases exit 0; rejected cases exit 2 with JSON retained.

`fixtures/scale/peak-goldens.txt` contains **128** exact peak-list goldens from unmodified Java HiLoPeakFinder (fixed RNG seed plus tie/terminal/zero-threshold cases). Rust tests also preserve a source quirk: the dominant isolated-black fixture yields line=70/interline=16 and extrapolated beam=48 in both implementations. This demonstrates port parity, not musical validity; adding new validity rules would be a deliberate later behavior change.

The Java harness compiles **unmodified** ScaleBuilder, HiLoPeakFinder, IntegerFunction, Range, VerticalFilter and RunsRetriever. Data objects, constant storage, Sheet/Book/UI/logging/plot/JAXB scaffolding are test-only adapters. The RunTable adapter itself uses original RunsRetriever. Plot stubs throw if executed, no GUI executes. The removal stub throws with the decision reason for comparison. JDK is oracle-only; the native program neither loads nor spawns it. This is algorithm-level parity, not a full Audiveris application run.

Reproduce from a fresh checkout, with the pinned source and a JDK:

```sh
cargo build --locked -j 2
python3 scripts/java_oracle.py /path/to/jdk/Contents/Home
python3 scripts/scale_oracle.py /path/to/jdk/Contents/Home
python3 scripts/peak_goldens.py /path/to/jdk/Contents/Home
cargo test --locked -j 2
cargo clippy --locked --all-targets -j 2 -- -D warnings
```

The scale histogram buffers have `rint(.08*h)+1` and `rint(.08*h)+rint(.25*h)+1` i32 entries, total approximately `1.64*h` bytes plus peak/work vectors. These are allocated after the circular pixel tiles have been released. Bounds and fixture hashes remain fixed; no RSS, speedup or real-score recognition claims are inferred from these formulas.
