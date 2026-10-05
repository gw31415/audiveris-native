# GRID preparation milestone

Implemented scope is **run dispatch → sections**, not GRID completion or detected staves. `grid-prep INPUT BINARY.pgm REPORT.json` runs native binarization and SCALE, then emits `grid_preparation`. A SCALE rejection leaves `status=blocked,error=scale_failed` and exit2; resource rejection leaves the reason in valid JSON and exit2. Input/IO failures remain exit1.

## Ported contracts

- `LagManager.dispatchRuns`: separate vertical runs at `1+rint(maxLine*1.2)`; retain short vertical pixels and re-extract their horizontal runs.
- `LinesRetriever.createBothLags`: defer horizontal runs shorter than `rint(interline*.25)`; count these separately.
- Vertical SectionFactory uses JunctionShiftPolicy `rint(interline*.05)`. Horizontal SectionFactory uses JunctionRatioPolicy 1.25 (inclusive reciprocal-to-max).
- Original SectionFactory one-shot order, IDs, overlap, run extension, split/merge termination, gaps. Ratio denominator is **integer** `weight/runCount`, matching BasicSection.getMeanRunLength.
- Each section has orientation-local `SectionId`, first position, ordered runs, foreground weight, absolute `[x,y,width,height]` bounds. IDs are not globally unique across orientations.

`sections::{Orientation,SectionId,Bounds,RunTable,Section,GlyphCandidate}` is the public integration boundary. `Section::as_glyph_candidate()` returns an absolute crop origin, local row-major grayscale mask (0 foreground/255 background) and contributing orientation/section refs. It does **not** assign a music label, infer that each section is a valid complete glyph, normalize interline, or fabricate the classifier's 110 features. The next glyph grouping/feature extractor can consume this representation explicitly.

Resource additions for section commands: max dimension 16,384, max 250,000 accepted runs per table, max 100,000 sections per orientation. These are native safety limits, not claimed to be Audiveris limits. Total pixel/encoded input caps remain 4M/32MB. Normal binarize/scale contracts are unchanged. No graph adjacency, GUI index, UI events, or `.omr` serialization compatibility is implied by the word LAG.

## Java comparison

`evidence/section-parity.json` compares every section ID/order, run tuple, first position, weight, integer mean run length, bounds and table dispatch count. 8 synthetic cases cover straight, slope, curve, noise, multiple staves, gaps, crossings and blank. Blank is explicitly blocked by SCALE. Real-score fixture additionally yields **2576 horizontal and 3673 vertical sections** in both implementations.

Oracle uses unmodified SectionFactory and JunctionPolicy classes. LagManager dispatch/build methods and LinesRetriever.createBothLags are copied **verbatim** into minimal wrappers; source/method hashes are recorded. DynamicSection, BasicSection, RunTable, Lag, Sheet, constants and logging are infrastructure/data adapters. Original RunsRetriever supplies run extraction, original ScaleBuilder supplies scale. It is not a full Java app/GRID run. Geometry in the adapter is derived from the raw section runs; the exact partition/order itself comes from original SectionFactory.

```sh
cargo build --locked -j 2
python3 scripts/java_oracle.py /path/to/jdk/Contents/Home
python3 scripts/scale_oracle.py /path/to/jdk/Contents/Home
python3 scripts/section_oracle.py /path/to/jdk/Contents/Home
cargo test --locked -j 2
```

## Real-score rendering evidence

OpenScore/Lieder `lc6019054.mscx` is CC0; fixed source/hash/license already held in `fixtures/openscore-lieder/`. Existing MuseScore **4.7.2 / 260525085** Cocoa CLI was invoked in converter mode at **180dpi**, with no resizing. Legacy documented `-c`, `-s`, `-m` flags were rejected by this installed version; actual `--help` was checked. A child sandbox denied network and writes outside this workspace/tmp, avoiding changes to user settings. Converter wrote `laura-1.png` then terminated with signal6 during restricted logging/crash handling. We explicitly do not call that a successful process run.

The resulting PNG was fully decoded, its CRC checked by the PNG decoder, and visually inspected as the complete An Laura page. An isolated test adapter composites RGBA over white and uses integer BT.601 luma; both Java and native algorithms receive the same resulting PGM pixels. `fixtures/rendered/manifest.json` preserves renderer, DPI, dimensions **1489x2104**, source/PNG/PGM hashes and exit status. This is a useful 180dpi engraving fixture, not a scan, not 300dpi, and not evidence of OMR accuracy. No unrelated window was manipulated, no software was installed.

## Remaining route to staff candidates

1. `FilamentFactory.createInitialFilaments`: length >= `rint(interline*.5)` then `isSectionFat` mean-thickness/aspect and local thickness probes. Need BasicLine/PointsCollector parity; do not substitute bounding-box thickness for its sloped sampling.
2. `FilamentFactory.retrieveFilaments`: merge/expand compatible sections with overlap/gap/curvature/slope constraints and StaffFilament geometry.
3. `LinesRetriever`: curved-filament purge, global slope, sloped-filament purge.
4. `ClustersRetriever`: regular interline patterns, optional smaller staff population, and buildStaves. Only here should the result be called detected staff groups.

Next oracle should freeze filament membership IDs, endpoints/line fit/probe counts, then cluster membership and final staff geometry. Current curved fixture yields 50 horizontal sections, broken fixture 15, noisy fixture 11: these are fragments requiring later assembly, not five successfully detected lines.

## Classifier integration handoff

After this GRID-preparation commit, classifier source commit `185b0992172197a41cbfa5ec85f335471b01df7b` was imported under `crates/audiveris-classifier` via a two-parent local merge (`ee5f3ff`), retaining its actual history. Its 53-file artifact manifest verifies unchanged. The root workspace links its library; `classify-features MODEL.json FEATURES.json` accepts an explicit 110-number vector and returns 149 labels/sigmoid scores. The imported standalone CLI remains available, but the root command calls the Rust library directly, not a subprocess.

Root CLI verification covers all 136 Java score vectors/top5 order plus invalid shape, NaN syntax, null and overflow. Imported crate tests cover normalized and hidden intermediates too (55,488 values). Model/provenance/license notices are retained; no relicense or external publication occurs. `GlyphCandidate` above remains the future feature-extractor boundary: there is no automatic section-mask →110 features →class prediction connection in this milestone.
