# Fixed-template page evaluation checkpoint

The root native Rust CLI now integrates the independent distance/template core
and authenticated Bravura template bank. This is an explicit-position evaluation
API, **not an automatic notehead detector or a completed HEADS stage**.
`grid_complete:false` and `note_recognition_complete:false` remain in its report.

```
cargo run --locked -- template-probes INPUT.png PROBES.json REPORT.json [IGNORE_MASK.pgm]
```

The request is an array of 1..4096 objects, at most 1 MiB:

```json
[{"shape":"NOTEHEAD_BLACK","point_size":51,"x":200,"y":500,"anchor":"MIDDLE_LEFT"}]
```

Shapes are BREVE, WHOLE_NOTE, NOTEHEAD_VOID, NOTEHEAD_BLACK; point size is exactly
51. Null or omitted anchor means template origin; named anchors must exist on
that shape. Unknown fields, shape/size, malformed numbers or absent anchors fail
explicitly. No nearest-size interpolation, Java fallback, or font rasterization
occurs. Coordinates are caller-specified; clipping and no-reference score
sentinels follow the official core. Bank width calibration APIs remain available
but this command does not estimate head width or choose point size automatically.

Original grayscale input is binarized by the existing native VerticalFilter.
The optional mask must match its dimensions and be binary: 0 ignored, 255 kept.
Ignored pixels are whitened before propagation, then set to -1 afterwards.
Without a mask, all page pixels participate. This is **not** the official
DistancesBuilder staff/ledger/vertical-seed selection. NO_STAFF residuals alone
are not that stage's input. The API can later accept a correctly selected mask.

Output records original binary SHA256, full distance-table SHA256 (row-major
signed i16 big endian), JavaShort overflow status, mask source, exact scores,
hole-white ratios and usable point counts. Existing output paths are refused;
validation errors create no success report. An IO error during writing can still
leave a partial output file, as with existing commands.

## Independently rerun official comparison

`scripts/verify_template_page.py CONTENTS JAVAC` uses the pinned official release
JAR SHA256 `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.
`TemplatePageProbe.java` reads original grayscale PNGs independently and invokes
unchanged official VerticalFilter, ChamferDistance.Short, TemplateFactory and
Template. Native binary pixels, distances, templates and intermediate GRID
memberships are not inputs to the Java oracle. Both sides receive the same
explicit probe coordinates and diagnostic mask rule.

Four cases passed: AnLaura (3,132,856 pixels) and synthetic-system-bars (268,800
pixels), each with no mask and with a synthetic diagnostic mask. Each full binary
and full distance-table hash matches. All 5,296 template evaluations and 5,296
hole ratios match exactly, including page boundaries and completely clipped
positions. None of these cases overflowed JavaShort. The mask is a test pattern,
not evidence of automatic staff/ledger/seed selection. Coordinates sample the
whole page, not labeled note detections. The synthetic page uses an explicitly
requested 51pt bank; this does not validate automatic font sizing on that page.

`evidence/template-page-parity.json` records commands, hashes, counts and actual
Java exits. `fixtures/template-page` stores independent expected hashes/scores
for Rust regression tests. Existing GRID/glyph comparisons also remain tested.

## Import and validation

`evidence/template-import.json` identifies clean predecessor commits and exact
copy differences. Core commit is 593d1c5d4cddfed4085371ef6a25447151a923d0; bank
commit is 2b6a2e15f5612c4bffcd8669116a809ee2405d65. All tracked source, fixtures,
provenance and licenses were retained. Bank serde is pinned to existing workspace
1.0.228 rather than 1.0.229 to resolve Cargo's exact-version conflict; its complete
strict-input test groups run under the workspace version. Assets are unchanged.

## Resource and completion boundary

The root decoder still limits input to 32 MB and 4,000,000 pixels. Ordinary
2480×3508 A4 at 300 DPI has 8,699,840 pixels and is still unsupported. This
checkpoint does not lower DPI, silently tile the image or remove the cap.
The template core's larger internal limit does not bypass the root decoder.
The new stage uses an N-byte binary, optional N-byte mask, N-byte ignore vector
and 2N-byte short distance table, plus bounded request/bank/results. Decoded
source is dropped before distance computation. These are buffer bounds, not a
hard RSS guarantee; runtime/decoder allocations also exist. The command does
not materialize distance values as a JSON array.

PDF integration must still meet `PDF-INTEGRATION-ACCEPTANCE.md`: dimension-aware
300 DPI budget, adversarial run/section/output bounds, peak memory measurements
and sequential-page lifecycle. Existing GRID complexity caps remain unchanged.

Remaining native-port work: brackets/C-clef filtering, semantic purge/endpoint
ordering and other pending GRID branches; automatic head-width calibration,
semantic mask selection, candidate search/competition/relations, rhythm, and
MusicXML export. The matching checkpoint does not remove those blockers.

The checkpoint rerun passed 54 tests in both debug and release, workspace clippy
with warnings denied, formatting and diff whitespace checks. A release AnLaura
raw-probe process took 0.395 seconds and had measured maximum RSS 21,102,592 bytes
using Python `getrusage(RUSAGE_CHILDREN)` for one spawned process. This measures
only this page/probe command, not worst-case input, GRID or 300 DPI PDF. macOS
`time -l` failed on a restricted sysctl and was not used as RSS evidence.
See `evidence/template-validation.json` and retained logs.

## Subsequent semantic/mask checkpoint

[GRID-SEMANTICS.md](GRID-SEMANTICS.md) supersedes the earlier pending bracket,
C-clef and endpoint-order statements for its explicitly supported cases. It also
adds the automatic GRID staff mask. Full GRID/HEADS and MusicXML remain incomplete.
