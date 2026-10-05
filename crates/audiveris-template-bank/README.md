# Audiveris official template bank

Production Rust loader for a small versioned bank from **Audiveris 5.11.0**, commit
`9e1e55cd2746037d059345881c53e6a6754bffbd`. Uses `audiveris-template-match`.
Runtime has **no Java, font rasterizer, subprocess, network or template resampling**.

## Confirmed coverage

The CC0 OpenScore An Laura PNG was rerun through the unmodified official release,
LOAD through HEADS with default settings. The post-step reflection probe obtained
the actual sheet candidate shape sets and staff sizes. Reproduction run: exit 0, 6.63 seconds.
Command, log, source input hash and loaded Book class provenance: `oracle/run/`.

- Bravura; all 6 staves non-drum/non-tablature, specific interline 11.
- Mean black-head width 15px; MusicFontScale 51pt; every staff requests 51pt.
- All candidates: BREVE, WHOLE_NOTE, NOTEHEAD_VOID, NOTEHEAD_BLACK.
- Stem candidates: VOID/BLACK. Hollow candidates: BREVE/WHOLE/VOID.

These are candidate sets, not just recognized output labels. `NoteHeadsBuilder`
uses All for non-drum staves, Stem/Hollow subsets and an explicit VOID test. This
is a **post-step probe plus source-path verification, not a bytecode call trace**.
Official Catalog eagerly builds unused head templates too; this bank covers the
matching requirements and omits that unused catalog population.

|Shape|Size|Canvas|Final keypoints|Anchors|
|---|---:|---:|---:|---:|
|BREVE|51|35×20|476|3|
|WHOLE_NOTE|51|26×17|285|3|
|NOTEHEAD_VOID|51|20×17|198|9|
|NOTEHEAD_BLACK|51|20×17|198|9|

Stores final keypoints, float anchors, fat/slim bounds and dimensions. 4 assets,
1,157 points; payload 28,585 bytes + manifest 1,062 bytes = **29,647 bytes**.
Joint gzip estimate 3,545 bytes; loader reads uncompressed files only.
Font/model binaries are unnecessary at runtime and are not included here.

Coverage is the only limitation related to this input. Runtime has **no AnLaura
recognition rules**, note positions, staff coordinates or special classifier
outcomes. The reference PNG hash is generation provenance; it is not compared to
pages being recognized. Other pages can use the same exact shape/size assets.

## Calibration and unsupported inputs

Pinned AWT NOTEHEAD_BLACK widths at sizes 50/51/54 are 14.75/15.046875/15.9375px.
Official `computePointSize` for width 15 uses `v1=rint(15*3.3)=50`, `dv=4`, `v2=54`,
then the original width-to-size formula gives 51. Native code uses the same
arithmetic and ties-to-even rounding. `staff_point_size` implements the stored
font-scale ratio. This is official sizing arithmetic, not template interpolation.

Absent AWT metrics return `UnsupportedCalibration`; absent shape/size assets
return `UnsupportedTemplate`. A computed size must always go through `get`.
No nearest-size fallback, resampling, synthesized metrics, Java call or fallback
font. The no-MusicFontScale fallback is outside this API. Adding coverage requires
new oracle assets/metrics and an explicitly updated trusted release manifest.

## Integration

Place this crate beside `audiveris-template-match`, or update the path dependency
on import. Numerical core revision: `593d1c5d4cddfed4085371ef6a25447151a923d0`.

```rust
let bank = audiveris_template_bank::Bank::load_builtin()?;
let global_size = bank.point_size_for_width(measured_head_width)?;
let size = audiveris_template_bank::Bank::staff_point_size(
    staff_interline, main_interline, global_size)?;
let asset = bank.get("NOTEHEAD_BLACK", size)?;
let result = asset.template().evaluate(
    &distance_table, x, y, Some(asset.anchor("MIDDLE_LEFT")?),
    audiveris_template_match::Weights::default())?;
```

`load_builtin` embeds authenticated data. `load_dir(path, trust)` reads exactly
`manifest.json` and `templates.json`; `from_bytes` validates caller-owned bytes.
Asset access is immutable. Obtain `Trust` from trusted application release code,
**never from the untrusted directory's own hashes**. Hashes provide release
integrity, not a remote signing/key-management system.

## Loader guarantees

- Hard bounds: manifest 64KiB, payload 1MiB, 128 assets, dimensions 1..2048,
  65,536 points/asset, 131,072 total points, 256 metrics, point size 1..1024.
  File length checked before allocation; max+1 read cap also catches growth.
  Final path components must be regular files; Unix uses O_NOFOLLOW/O_NONBLOCK
  and rechecks the opened file. The bank directory is caller-selected.
- Trusted manifest SHA256 before payload read; payload SHA256 before decode.
  Verify schema, release/commit, family/font version/hash, classifier name/hash
  and reference-input provenance hash. The caller's expected model identity
  should match the rest of the pipeline; this loader does not load that model.
- Typed JSON rejects unknown/duplicate fields and uses serde's recursion limit.
  Canonical uppercase ASCII keys; no case folding. Exact manifest/payload coverage,
  duplicate key/anchor/point rejection, finite values, valid bounds/coordinates,
  nonempty keypoints and required base anchors. Anchor count must be 3 or 9.
  No payload path fields exist, so keys never become filesystem paths.
- Missing, malformed, oversized or unauthenticated banks fail explicitly.
  No data synthesis or runtime recovery fallback.

Built and tested on macOS arm64. Other platform builds were not run here.

## Checks and evidence

Six test groups cover official scores/calibration/anchors, unsupported sizes,
integrity/identity, duplicate fields/keys, case collisions, missing files/assets,
invalid dimensions/bounds/points/anchors, NaN/Infinity/overflowing JSON numbers,
oversized byte/file input and symlinks. Semantic tests re-sign mutated data with
test-only hashes so the integrity gate does not hide structural-validation bugs.
The four asset self-image scores match official Java exactly.

```sh
cargo test --offline
cargo test --offline --release
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --check
```

The prior 5-shape × 3-size comparison and 15 official assets are included under
`evidence/font-comparison/`, as evidence **outside runtime coverage**. It found
6/15 exact native bitmaps and 45/45 frozen-asset score matches. Its archived README
refers to the complete earlier audit bundle; this evidence subset contains the
report, comparison tables, manifest and official asset JSON.

## Generation and license

`CoverageProbe` runs through official `-run` after HEADS. Compile-only Book/RunClass
signatures under `oracle/stubs/` are excluded from the runtime classpath; the observed Book class source
is the official JAR. `BankAssetOracle` invokes its unchanged private Builder only
for the four required 51pt shapes and collects three AWT widths. All Java is
development-time tooling. No large bank generation was performed.

After oracle generation, set `AUDIVERIS_CONTENTS` to the pinned app's Contents and
run `python3 oracle/build_bank.py`. It verifies JAR/font hashes and exact candidate
coverage, then emits payload, manifest and builtin trusted identity. Review the
release diff and run tests. It is never called by the runtime crate.
For the complete regeneration, set `JAVAC` to JDK21's compiler and run
`python3 oracle/reproduce.py /path/to/laura-original-rgba.png`. This checks the
exact oracle JAR and reference-input hashes before running the bounded probe.

AGPL-3.0-or-later applies to the translation/loader and original source copies;
see LICENSE/NOTICE. Bravura-derived assets retain copyright and SIL OFL 1.1 in
`assets/BRAVURA-OFL.txt`. CC0 input provenance is in `evidence/input/`. Dependency
licenses/texts are included. No publication, new authentication or lead workspace
write occurred.

Pinned primary sources are copied under `oracle/upstream/` with their notices:
[MusicFont](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/ui/symbol/MusicFont.java),
[ShapeSet](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/glyph/ShapeSet.java),
[NoteHeadsBuilder](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sheet/note/NoteHeadsBuilder.java),
[TemplateFactory](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/image/TemplateFactory.java).
