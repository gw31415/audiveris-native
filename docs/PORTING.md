> この文書は2026-10-02時点の歴史的baselineです。現状・全原版機能の移植目標・欠落・接続順は[PORT-STATUS.md](PORT-STATUS.md)を参照してください。下記の当時の制限は最終機能の削除ではありません。

# Initial baseline and overall port boundary (2026-10-02)

Current status: VerticalFilter streaming, automatic SCALE, GRID run dispatch/sections, the 110→149→149 classifier, and isolated binary glyph → 110 features → classifier are implemented. See [SCALE](SCALE-MILESTONE.md), [GRID boundary](GRID-PREP.md), and the [current integration milestone](GLYPH-INTEGRATION.md). Filament/staff detection and complete page recognition remain unimplemented. The baseline observations below are retained; the current status supersedes the initial implementation list.

## Baseline

Official latest stable release API returned **5.11.0**, published **2026-07-11T21:44:26Z**. The checked-out tag resolves to `9e1e55cd2746037d059345881c53e6a6754bffbd`. This is a release baseline, not moving `master` or `development`. Release response is retained in `evidence/upstream-release.json`. Sources: [repository](https://github.com/Audiveris/audiveris), [release](https://github.com/Audiveris/audiveris/releases/tag/5.11.0), [handbook](https://audiveris.github.io/audiveris/).

## Original CLI and IR

`CLI.java` exposes `-batch` (no GUI), `-transcribe`, `-step`, `-sheets`, `-output`, `-save`, `-export`, and `--` input paths. The production target can retain these concepts without copying its args4j API. Example original invocation: `Audiveris -batch -transcribe -export -output out -- score.pdf`.

`ImageLoading.java` uses ImageIO for raster input and PDFBox for PDF rasterization, default 300 DPI. The `.omr` project is ZIP/XML persistence around Book/Sheet and their images; it is richer than MusicXML. Core chain: pixel source → RunTable → lag Sections → Glyph geometry → Inter hypotheses and SIGraph relations → measure/chord/voice/rhythm and Score → PartwiseBuilder/ProxyMusic 4.0.3 → MusicXML 4.0 (`.mxl` compressed export). Exact `.omr` roundtrip compatibility is a separate milestone, not implied by the new JSON.

## Modules and dependencies

| Upstream module/stage | Native boundary and difficulty |
|---|---|
| LOAD / BINARY (`image`, `run`) | Decoder separate from pure gray buffer. Adaptive binarization and run extraction now ported; RGB/alpha semantics and PDF raster parity still open. |
| SCALE / GRID (`sheet/ScaleBuilder`, `sheet/grid`, `lag`) | Run histograms, peak choice, line thickness/interline, sections/filaments, skew, curved staff and system grouping. SCALE and run dispatch/sections are ported; filament/staff processing remains next. Avoid replacing it with a simple horizontal projection heuristic. |
| HEADERS through STEMS, LEDGERS, BEAMS (`sheet/*`) | Geometric constraints, morphology, head template matching. Font rasterization affects templates; fonts need individual license/provenance review. |
| SYMBOLS (`classifier`, `moments`, `math`) | MixGlyphDescriptor combines ART/geometric moments and normalization; port feature ordering and neural inference together. |
| REDUCTION/LINKS (`sig/inter`, `sig/relation`, `SIGraph`) | Many competing hypotheses and mutable relations; use typed IDs/arena and explicit adjacency, preserving selection rules rather than literal Java object graphs. |
| TEXTS (`text/tesseract`) | Tesseract 5.5.2 + Leptonica 1.87.0 via JavaCPP 1.5.13; replace Java binding with optional native adapter. Language `.traineddata` acquisition/version/license must be pinned separately. |
| MEASURES/CHORDS/RHYTHMS/PAGE / export | Music semantics and cross-system continuity are substantial; golden score tests needed before MusicXML can be called recognition output. |

Pinned `OmrStep.java` order: LOAD, BINARY, SCALE, GRID, HEADERS, STEM_SEEDS, BEAMS, LEDGERS, HEADS, STEMS, REDUCTION, CUE_BEAMS, TEXTS, MEASURES, CHORDS, CURVES, SYMBOLS, LINKS, RHYTHMS, PAGE.

Upstream `app/build.gradle` also includes ImageJ 1.54p, JAI/JAI ImageIO, PDFBox 3.0.6, iText 5.5.13.2, JAXB, JGraphT and Swing/BSAF/JGoodies/JFreeChart. These are **not** native production dependencies. JAI is expressly labeled non-free in the build repository comment, so no assumption that all bundled dependencies can be copied under the root license. Native dependencies are the locked Rust PNG decoder/compression, JSON parsing, and the two imported native crates. No GUI imports or Java process invocation occur in `src/`.

## Model actually inspected

`app/res/basic-classifier.zip` is 396,100 bytes (size is also recorded by the inventory script), with `model.xml`, `means.xml`, `stds.xml`; SHA-256 `327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e`.

Parsed model attributes: 110 inputs, 149 hidden units, 149 outputs; sigmoid shallow network implemented by `math/NeuralNetwork.java`. `BasicClassifier` uses `MixGlyphDescriptor`, subtracts the mean vector, divides by std vector, then runs inference. It uses local `PoorManAlgebra`, not a necessary DL4J/ND4J runtime. The weights are portable data in principle, but feature extraction, label order, bias layout, normalization, and font-related templates must be tested. XML dimensions/hash alone do not prove recognition parity or independently establish the model training-data license. The original and converted model snapshots are now imported locally in the classifier crate with original notices, manifests and explicit unresolved provenance qualifications; this is not redistribution clearance.

## Rust choice

Rust is selected for explicit bounded pixel buffers, predictable allocations, no GC pause requirement, typed hypothesis/relationship IDs, and native library boundaries for optional OCR/PDF. Go is viable for orchestration and would simplify some development, but does not eliminate FFI requirements for OCR/PDF or the geometric/graph rewrite. No Rust-vs-Go speed claim has been measured. A single-threaded core with at most two build jobs is used on this shared Mac.

## What is implemented and tested

- PNG 8-bit grayscale and PGM input, integral-image adaptive binarization, run extraction, binary PGM + JSON output. Native Rust implementation, no Java wrapper.
- Defaults exactly match `AdaptiveDescriptor` (0.7/0.9), `AdaptiveFilter` half-window 18, clipped inclusive neighborhood, `abs` variance and `<=` classification. Padded whole-image sums correspond to `RandomFilter`; production Java usually uses the memory-efficient `VerticalFilter` ring buffer. Both Java variants are tested against each other.
- Full-image run semantics correspond to an accept-all `RunsRetriever` adapter. ROI selection/filtering and compact `RunTable` storage are not ported.
- Test oracle compiles unmodified upstream AdaptiveFilter/RandomFilter/VerticalFilter/RunsRetriever with small infrastructure/data stubs. It does not run the whole Audiveris app and does not test ImageJ/PDFBox decode parity. JVM is test-only, headless.
- `getContext()` upstream uses Population sample variance, unlike actual `isFore()` integral calculation. JSON reports actual classification statistics, **not** UI getContext compatibility.
- Two fixtures: 27,027 binary pixels compared exactly; horizontal and vertical runs exact; 6 threshold probes exact. Rust brute-force tests cover every pixel of 5 additional shapes including degenerate dimensions and clipped windows. PNG pixels equal the PGM fixture.
- No recognition accuracy, large-score performance, or MusicXML fidelity claim. Resource cap currently limits full scans; ring-buffer port and streamed run output were subsequently completed in milestone 2; the input cap is retained.

## License and next acceptance boundary

Direct code translation retains upstream copyright and AGPL-3.0-or-later in source/NOTICE/LICENSE. Upstream model/font/example licenses are distinct audit items; do not claim blanket clearance. Fixtures are original CC0 artwork. Dependency notices are copied under `licenses/`.

Current progress: preprocessing, SCALE, sections, curved filaments and default five-line/single-interline initial Staff geometry are implemented; see STAFF-CLUSTERS.md for the exact boundary. Next: (1) complete remaining GRID/staff-line refinement and system/part grouping; (2) staff removal and glyph candidate grouping, connecting the existing isolated-glyph classifier; (3) templates and basic note/stem associations; (4) single-staff monophonic rhythm/MusicXML; (5) PDF adapter, multi-page/polyphony, OCR, relation conflict reduction and regression corpus. Each milestone exports intermediate values for the original Java oracle. Keep MusicXML tests semantic (pitch/duration/voice/measure) and reject unsupported cases, rather than inventing successful scores.
