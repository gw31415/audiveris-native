# Pinned Linux development oracle

This harness runs the **unmodified official Audiveris 5.11.0 JAR** on Linux
x86_64. Java is development instrumentation only. The production CLI and Rust
libraries neither call it nor require a JVM. No Mac or global package install
is used.

## Pins and local dependencies

- Source reference: `9e1e55cd2746037d059345881c53e6a6754bffbd`
- Official release: <https://github.com/Audiveris/audiveris/releases/tag/5.11.0>
- Linux asset: `Audiveris-5.11.0-ubuntu22.04-x86_64.deb`
- Asset SHA-256: `ae714594f40e54b1a4951fc3f914f08ae38fe5d07b7f2283b1a904fdb6e0a318`
- `audiveris.jar`: `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`
- Bundled Linux runtime: Zulu OpenJDK 25.0.3, as recorded by each run
- Observer compiler: official Eclipse Temurin 21.0.12.1+1 Linux x86_64 archive,
  SHA-256 `ce79869e1307ed8ee1e2baa86a412b1eb5b75d10a01006d788a6f968bcfaee94`
- Bravura JAR resource: `res/Bravura.otf`,
  SHA-256 `dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2`

`--bootstrap` downloads only those official pinned archives, checks their
hashes, extracts the Debian package without installation, and unpacks the JDK.
The default dependency home is the repository's sibling `linux-oracle/`;
`--oracle-home` selects another cloud-local folder. Existing archives must
match the pin and are not silently replaced. Python 3.11+ and `dpkg-deb` are
needed; Python 3.12+ provides the safe tar extraction filter used by bootstrap.

```sh
python3 scripts/linux_oracle.py --bootstrap
```

Observers compile against minimal ABI shims for `RunClass` and `Book` using
Java 21. The full-process delegating observer additionally uses ABI-only signatures for
KeyExtractor and related parameter types. All shims are in a distinct
**compile-only** directory which is
never present on the runtime classpath. At runtime the originals from the
verified Java 25 JAR supply these classes. Recognition code is never replaced.

## Reproduction

```sh
python3 scripts/linux_oracle.py --probe clef-candidates
python3 scripts/linux_oracle.py --probe clef-registration
python3 scripts/linux_oracle.py --probe clef-font-metrics
python3 scripts/linux_oracle.py --probe clef-registration-control --fixtures rendered-clef-g_clef
python3 scripts/linux_oracle.py --probe key-preparation
python3 scripts/linux_oracle.py --probe key-preparation-control --fixtures synthetic-system-bars
python3 scripts/linux_oracle.py --probe key-peaks
python3 scripts/linux_oracle.py --probe hilo-control
python3 scripts/linux_oracle.py --probe key-slicing
python3 scripts/linux_oracle.py --probe key-components
python3 scripts/linux_oracle.py --probe key-component-branches --fixtures fixtures/official-grid/damaged-header-*.png
python3 scripts/linux_oracle.py --probe key-fallback --fixtures fixtures/official-grid/fallback-header-*.png
python3 scripts/linux_oracle.py --probe key-neighbor-control --fixtures rendered-header-four-flats
python3 scripts/linux_oracle.py --probe key-clef-control
python3 scripts/linux_oracle.py --probe clef-competition-control
python3 scripts/linux_oracle.py --probe key-process
python3 scripts/linux_oracle.py --probe key-process-plain
python3 scripts/linux_oracle.py --probe key-column
python3 scripts/linux_oracle.py --probe key-column-plain
python3 scripts/linux_header_fixtures.py --kind key-column
python3 scripts/linux_oracle.py --probe key-column --fixtures fixtures/official-grid/key-column-*.png
python3 scripts/linux_oracle.py --probe key-pitch-window-control --fixtures rendered-header-four-flats rendered-header-three-sharps
python3 scripts/linux_oracle.py --probe key-extract-retention-control --fixtures rendered-header-one-flat
python3 scripts/linux_oracle.py --probe time-column
python3 scripts/linux_oracle.py --probe time-column-plain
python3 scripts/linux_oracle.py --probe time-font-metrics
python3 scripts/linux_oracle.py --probe clustering-control
python3 scripts/linux_oracle.py --probe time-selection-control --fixtures key-column-uniform-empty
python3 scripts/linux_oracle.py --probe time-selection-control-plain --fixtures key-column-uniform-empty
python3 scripts/linux_header_fixtures.py --kind time-column
python3 scripts/linux_oracle.py --probe headers
python3 scripts/linux_oracle.py --probe headers-live
python3 scripts/linux_oracle.py --probe stem-seeds --fixtures laura-gray fixtures/official-grid/stem-seeds-*.png
python3 scripts/linux_oracle.py --probe stem-seeds-plain --fixtures laura-gray fixtures/official-grid/stem-seeds-*.png
python3 scripts/linux_oracle.py --probe stem-buffer-control
python3 scripts/linux_oracle.py --probe stem-scale-control --fixtures stem-seeds-clean
python3 scripts/linux_oracle.py --probe stick-factory-control --fixtures stem-seeds-clean
python3 scripts/linux_header_fixtures.py --kind stem-seeds
python3 scripts/linux_oracle.py --probe context-partition-control
```

Each run creates a new timestamped directory under
`evidence/generated/linux-oracle/<probe>/`. An explicit `--output` must not
already exist. Existing fixtures and historical oracle evidence are never
overwritten. `--fixtures` accepts known fixture basenames or explicit PNG
paths. Execution records contain exact arguments, isolated home/preferences
folders, exit code, input/output/log hashes, JAR/resource/dependency hashes,
observer source hashes and compiler/runtime versions. New runs compile retained
source snapshots so concurrent repository edits cannot change their inputs. Failed and successful
runs are separate; a stale output cannot satisfy a later run.

The initial Linux clef-candidate run reproduced all ten preserved official
candidate JSON values exactly. This includes rendered G/F/C positives,
empty/noise negatives and the original Laura score. It is not an exhaustive
recognition accuracy measurement.

### Independent synthetic key rasters

```sh
python3 scripts/linux_header_fixtures.py --output /tmp/audiveris-header-inputs
python3 scripts/linux_oracle.py --probe key-slicing --fixtures /tmp/audiveris-header-inputs/*.png
python3 scripts/linux_oracle.py --probe headers --fixtures /tmp/audiveris-header-inputs/*.png
```

The renderer draws original test artwork with pinned official Bravura. It
makes one/three-sharp, one/four-flat, empty and noise rasters, with a provenance
manifest. No native recognition result is supplied to generation or to the
oracle. Four-flats and three-sharps were independently observed as signatures
−4 and +3 at the pre-classifier slice stage; empty/noise had zero slices.

## Precise observation boundaries

- `clef-candidates`: official raster GRID, header starts, private original
  candidate extraction and best-map evaluation, matching the preserved probe
- `clef-registration`: official `findClefs`, registered theoretical geometry,
  original glyph geometry, intrinsic grade, kind, pitch, stop and exclusions
- `clef-font-metrics`: 768 generic Bravura entries for six allowed shapes and
  interlines 1–128; exact layout, normalized symbol centroid and area-pitch
  offset. Every measured layout had positive width and height. No interpolation
  or per-image answer is used. Centroid font selection reproduces the original
  `getPointSize(sheetInterline)`-as-interline call
- `clef-registration-control`: explicitly supplied shapes and grades on an
  original recognized glyph. The original method preserves equal-grade order
  and inserts all three pairwise OVERLAP exclusions. This is controlled
  algorithm evidence, not naturally detected ambiguity
- `key-preparation`: original clef column and system stop, then original
  `KeyBuilder` constructor, browse geometry and staff-free-pixel projection
- `key-preparation-control`: original connected/unconnected raster bar objects,
  deliberately varied grades around 0.6 and x boundaries; original projection
  with deliberately supplied clamped rectangles
- `key-peaks`: original `retrieveHiLoPeaks`, exact scale-dependent thresholds,
  projection/domain and ranges sorted by main as the original key stage does
- `hilo-control`: exact histogram inputs to the original `HiLoPeakFinder`, with
  17 named and 96 deterministic varied cases. Both returned order and
  main-sorted ranges are retained. Nonpositive gain is excluded from successful
  controls: gain zero caused an original out-of-domain exception, retained in
  separate failed-run evidence. Terminal unclosed derivative runs are not
  flushed by the original
- `key-slicing`: original FLAT/SHARP `ShapeBuilder` methods through
  `allocateSlices`; checkpoints include ranges and peak min/main/max/height/area.
  It stops **before** component extraction/classification
- `key-components`: original `retrieveCandidates` and `retrieveComponents` after
  slice allocation, stopping before hard-slice fallback and clef/key consistency.
  Retained part geometry is observed by replaying the original extraction
  preamble; all component formation, pruning, classification and assignments
  are original method calls. Results distinguish attempted/empty stages,
  post-overlap-purge candidates, part membership, evaluation grades and assigned
  slice alterations with rounded/measured pitch. All 16 current rasters passed:
  34 retained parts, 33 candidates/assigned alterations, one empty allocated
  slice on `cclef-pair` before fallback. Original flat mass offset is 0.65;
  area offsets are FLAT 1.0559375 and SHARP 0. These images do not establish
  exhaustive multipart-combination or overlap-purge coverage. Retained evidence
  is in `evidence/key-components-oracle-validation.json`
- `key-component-branches`: ten explicitly cut/speckled accidental rasters plus
  empty/noise controls. Eight raster cases assigned multi-part alterations and
  six removed overlapping candidates. The observer calls original adapters,
  decomposition and purge methods and verifies that their replay agrees with
  complete original `retrieveCandidates`; it does not supply fake candidates.
  Inputs and exact perturbation rectangles/hashes are retained in
  `evidence/key-component-branches-oracle-validation.json`
- `key-fallback`: original `ShapeBuilder.extractEmptySlices`, before clef/key
  consistency or pitch-window `fillMissingAlters`. A recording list observes
  the original ordered attempts and their before/after assignments. On 16
  initial plus six bridged-key rasters, 11 attempts recovered nine slices and
  left two null; this includes the honest negative `cclef-pair` result. The
  original lower intrinsic threshold is 0.01. Actual natural-case neighbor
  erasure was zero in this bounded set
- `key-neighbor-control`: an explicitly expanded adjacent slice overlaps a
  genuine assigned flat. Original `getSlicePixels` erased exactly 240 pixels
  (485 to 245 foreground pixels). The 44×113 before/after byte buffers are
  retained as Base64 with hashes, rectangle and neighbor runs, independently
  validating pixel erasure without claiming a naturally detected overlap.
  See `evidence/key-fallback-oracle-validation.json`
- `key-clef-control`: explicitly constructed numerical controls using original
  pitch arrays, pitched/key grade methods and the original clef-selection
  decision. Development-only incomplete object allocation is labelled in the
  result, and a recording list stops the original method before graph changes
  or pitch-window fallback. Fields replayed from the source are separately
  labelled; these are not image or complete recognition tests
- `key-process`: calls the complete original per-staff `KeyBuilder.process` and
  records stages after FLAT, SHARP and `selectBestClef`, before KeyColumn
  cross-staff alignment/finalization. Live states retain key members, intrinsic
  and cached contextual grades, clef worklist/SIG survivors, slice evaluation,
  glyph, alteration and stuffed state, and relevant relations. It reads the
  shape builder's key field directly, never the mutating `getBestKeyInter`
  helper. A delegating extractor subclass observes original component/hard
  retry/pitch-window calls; all recognition operations call original `super`
  methods. All stage snapshots on 36 rasters matched `key-process-plain`, which
  disables extractor hooks but retains the same read-only stage snapshots
- Full-process evidence includes three actual `fillMissingAlters` attempts:
  two upper-bridge inputs recovered a missing flat and one thick-bridge input
  stayed null. Two one-item trailing-block inputs failed the original trailing
  space check and destroyed the key/slices. These are raster-derived outcomes,
  distinct from controlled object tests. See
  `evidence/key-process-oracle-validation.json`
- `key-column`: untouched original `KeyColumn.retrieveKeys(maxHeaderWidth)`
  after original GRID/header-start/clef retrieval and system clef-stop propagation.
  It includes original alignment, replication and `finalizeKey`, but stops before
  HeaderBuilder's system key-stop propagation and the time column. Per-system
  output includes returned offset, global offsets, part membership and direct
  staff/builder snapshots, key/clef ranges, stops, members, evaluation/glyph
  state, relations, cached context and frozen inter IDs. Delegating map facades
  record exact original caller stacks and before/after iteration checkpoints;
  observation never calls `getBestKeyInter` or `computeContextualGrade`
- All 51 key-column raster results matched separate `key-column-plain` runs
  exactly for every final system field. This includes the preceding 36 inputs
  and 15 independent Bravura two-staff rasters, whose brace is recognized into
  a genuine multi-staff part. New cases exercise successful missing-key
  replication, flat and sharp last-item shrink, whole-key removal, missing-clef
  preservation, and ink-filled but unrecognized slice retention. Empty, noise,
  uniform and shifted-alignment controls remain retained. Return-value names
  such as `SHRINK` are source-supported interpretations of those observed
  state changes; the probe does not intercept `PartStatus` returns or Gaussian
  internal iterations. All 30 new staff post-process snapshots also equal
  independent `key-process` runs. See `evidence/key-column-oracle-validation.json`
- `key-pitch-window-control`: 21 explicit original `setPitchRect` calls use
  genuine staves/clefs with FLAT/SHARP and heights 39/40/41. They retain the
  original Java integer compound subtraction of the area-pitch offset and
  integer division of height by two. Results are in
  `fixtures/official-grid/key-pitch-window-controls.json`
- `key-extract-retention-control`: three explicitly injected-state tests on a
  genuine one-flat slice. Original `extractAlter` retains a higher prior raw
  evaluation against a separately observed lower candidate; equal evaluation
  and identical registered glyph preserve injected intrinsic/context grades;
  a below-threshold return is null without clearing the existing alteration.
  The equal-score candidate is independently extracted and compared exactly.
  See `evidence/key-extract-retention-oracle-validation.json`
- `clef-competition-control`: explicitly constructed original graph tests call
  original `Staff.getCompetingClefs`; no natural graph-detection claim follows
- `time-column`: original `HeaderTimeColumn.retrieveTime` after original keys
  and system key-stop propagation; observes constructor ROI/projection/space
  state, exact crop attachments, retained parts/links, best WHOLE/NUM/DEN maps,
  before-filter lists, supports/exclusions, pair and system selection, cleanup,
  returned offset and original system time-stop propagation. It stops before
  final clef selection, header bar purge and freezing. All 71 raster outputs
  matched `time-column-plain` exactly for final states and candidate replay
- Candidate replay runs **after all systems finish**: fresh original time
  adapters and `GlyphCluster` enumerate original retained parts, while a proxy
  delegates every method and records actual trial glyphs/part membership and
  a repeated original classifier evaluation. Exact best-map glyph/grade and
  trial-count equality with the primary run is mandatory. This suite retains
  98 trials, with at most three parts per adapter; it does not establish larger
  combination coverage. The 20 independent time inputs include COMMON/CUT,
  supported numeric positives, malformed/absent/noise negatives and multi-staff
  consistency cases. Rendered 2/2 and 12/8 remained honest original negatives.
  See `evidence/time-column-oracle-validation.json`
- `time-font-metrics`: all original WholeTimes/PartialTimes, 21 shapes across
  interlines 1–128 (2,688 entries), reproduced exactly three times. Singles use
  layout `Math.rint` dimensions/half-offsets; combo `TimeWholeInter` uses
  `NumDenSymbol.getDimension` and integer halves instead. Every combo metric
  was checked against original `getSymbolBounds` at generic fixed bounds.
  A null combo layout is explicitly reported, not replaced by invented values
- `clustering-control`: 14 explicit original `Population`/`Clustering.EM`
  controls and 20 `Gaussian.proba` epsilon-boundary controls. Initial means
  come from original populations and sigmas start at 1; final mean, sigma,
  mixture weights and half-even offsets are retained. A deliberate underflow
  input preserves original NaNs as strings and Java integer conversion to 0.
  See `evidence/clustering-oracle-validation.json`
- `time-selection-control`: 37 declared-candidate controls use genuine GRID
  staff geometry with fresh original SIG graphs and the required original
  `SigListener`. They call original `filterCandidates`, `getValueVectors` and
  `checkConsistency`. Vector and selection passes use separate graphs, avoiding
  duplicate pair creation. The extra observation listener records transient
  support/exclusion edges; every other result exactly matches a run without it.
  Inputs explicitly have null glyphs and given bounds/grades, so these are
  bounding-geometry/scoring controls, not raster classifier evidence. They cover
  dx limits, unsupported ratios, ties and ±ULP, ambiguous links, support grades,
  neighbors and incomplete/incompatible columns. An original incomplete pair
  survivor (one member, grade 0, null value) remains represented honestly. See
  `evidence/time-selection-control-validation.json`
- `headers`: read-only observation after official CLI `-step HEADERS`, with
  an explicit check that the step is done. This is a **saved/reloaded** snapshot:
  transient ranges are null and intrinsic/contextual grades are rounded to
  three decimals by original `Jaxb.Double3Adapter`. The expanded 71 results are
  in `fixtures/official-headers-expanded`; the historical 16 remain unchanged
- `headers-live`: original GRID then original `HeaderBuilder.processHeader`
  for each system, captured before persistence, with no graph/collection hooks.
  This includes final clef selection, bar purge and header freezing. All 71
  live/reloaded results match in selected objects, glyphs, bounds, flags,
  rational/key values, staff stops and SIG identities/order/relations after
  explicitly accounting for grade serialization. Reload drops 239 transient
  ranges and rounds 908 grade/context fields in this set, with maximum observed
  difference 0.0004995666341037763. The primary numerical/range oracle is
  `fixtures/official-headers-live`, not the reloaded values. See
  `evidence/headers-expanded-oracle-validation.json`

The full official HEADERS oracle recognized six clefs and six −4 key signatures
on Laura, with three 3/4 times in its first system. Native HEADERS completeness
is a separate claim and is not implied by the availability of this oracle.

An important inheritance detail confirmed by source and observation:
`ClefInter.setBounds` intentionally preserves its existing pitch; it does not
use the pitch-recomputing `AbstractPitchedInter.setBounds` implementation.

## STEM_SEEDS development boundary

Eleven raster cases now have original-method observations through stem scale,
vertical stick retrieval/checking and VERTICAL_SEED free-glyph registration.
All final outputs match untouched `StemSeedsStep.doit`; no StemInter, BEAMS or
LEDGERS result is implied. Full decoded/hashed masks, section/filament geometry,
352 raw check contexts and independent stroke input recipes are retained. The
pinned ImageJ buffer/image-copy behavior means scaler drawing does not modify
its returned NO_STAFF buffer; this is directly tested, not inferred from intent.
See `docs/STEM-SEEDS-DEPENDENCIES.md` and
`evidence/stem-seeds-oracle-validation.json`. The eight new stem renderings
reproduce byte-for-byte.

`context-partition-control` independently exercises original SIG partition
ordering/context evaluation. Its eight cases confirm that maximizing summed
support before computing contextual grades can change an exact floating-point
winner: a lower intrinsic partner can produce a one-ULP higher context. See
`evidence/context-partition-control-validation.json`.

## Limitations

Controlled inputs prove bounded branches only. Curved-key browse geometry and
more unusual clef/key/time combinations still need broader independent cases.
The bundled runtime reports no installed OCR languages; these probes do not
run TEXTS. No OCR, MusicXML, general PDF, cross-font, or full recognition-parity
claim follows from these results. Downloaded upstream distributions remain
separate development tools with their own license terms.

## Perturbed raster reproduction

The additional inputs are transformations of the independent Bravura renderer
outputs, never of native answers. These tools write exact parent/output hashes
and operation rectangles to `perturbations.json`. Compile and invoke their
standalone main methods with the same pinned JDK/runtime:

```sh
JDK=../linux-oracle/jdk-21.0.12.1+1
JAVA=../linux-oracle/official/opt/audiveris/lib/runtime/bin/java
mkdir -p /tmp/audiveris-fixture-classes
"$JDK/bin/javac" --release 21 -d /tmp/audiveris-fixture-classes scripts/java/MakeDamagedHeaderFixtures.java scripts/java/MakeFallbackHeaderFixtures.java scripts/java/MakeKeyProcessFixtures.java
"$JAVA" -Djava.awt.headless=true -cp /tmp/audiveris-fixture-classes MakeDamagedHeaderFixtures fixtures/official-grid /tmp/audiveris-damaged-inputs
"$JAVA" -Djava.awt.headless=true -cp /tmp/audiveris-fixture-classes MakeFallbackHeaderFixtures fixtures/official-grid /tmp/audiveris-fallback-inputs
"$JAVA" -Djava.awt.headless=true -cp /tmp/audiveris-fixture-classes MakeKeyProcessFixtures fixtures/official-grid /tmp/audiveris-key-process-inputs
```

The generic Linux runner accepts those explicit PNG paths. Preserve each
output directory's manifest with its oracle results; do not conflate synthetic
raster evidence with controlled object/list tests or naturally scanned scores.

The two-staff key-column renderer is reproducible through
`python3 scripts/linux_header_fixtures.py --kind key-column`. Its 15 PNG inputs
are generated directly from pinned Bravura glyphs and declared geometry, not
from recognition output. The runner retains the renderer source, exact recipe,
commands, font/JAR pins and input hashes. All 15 retained inputs reproduced
byte-for-byte in a separate render.

Use `python3 scripts/linux_header_fixtures.py --kind time-column` to reproduce
all 20 independent time inputs. Two separate renders matched the retained PNG
bytes exactly. New oracle runs explicitly isolate HOME and all XDG config,
data and cache locations; prior successful receipts preserve their original
arguments and any harmless read-only-home cache warnings.

The isolated pure-Rust `stem_checks` increment matches all 352 original check
contexts with zero observed floating-point delta in debug and release; it has
explicit profile/geometry/pixel-work bounds. This is separate from filament
construction and whole-step integration. See `evidence/stem-checks/validation.json`.
Original histogram controls are indexed by
`evidence/stem-scale-control-validation.json`.

`stick-factory-control` directly constructs original sections and calls original
StickFactory, with four additional direct StraightFilament fits. All 30 controls
reproduce byte-for-byte. Unsorted input and deliberately overlapping neighbors
are labelled explicitly. See `evidence/stick-factory-control-validation.json`.
