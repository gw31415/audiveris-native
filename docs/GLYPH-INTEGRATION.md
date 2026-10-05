# Native integration milestone — 2026-10-02

The root CLI now exposes two independently verified native paths:

1. Grayscale page → adaptive binary → runs → SCALE → GRID run dispatch and LAG sections.
2. Explicit isolated binary glyph + interline → 110 features → normalization → 149 hidden values → 149 labeled sigmoid scores.

They are not yet an automatic page-to-score pipeline. Section masks are available as public `GlyphCandidate` values with absolute bounds, pixels and source section IDs. `glyph::from_candidate` preserves that geometry and extracts features, but the caller must choose/group the sections. A section is not necessarily one musical symbol. Staff removal, candidate segmentation and musical relationship/rhythm inference are still missing; there is no MusicXML export.

## Imported implementation and provenance

- Classifier source commit `185b0992172197a41cbfa5ec85f335471b01df7b`; history import `ee5f3ff`, root integration `6328648`.
- Glyph features source commit `225d081c888086e8617be920e3f3ca1bc1f9e4b2`; history import `bbb6d91`.
- Imports use local fetch, a two-parent history-preserving merge, and `read-tree` under `crates/`. The classifier's 53 and features' 72 manifest entries retain original bytes. Neither original workspace was edited.
- Both translate official Audiveris 5.11.0 commit `9e1e55cd2746037d059345881c53e6a6754bffbd` under AGPL-3.0-or-later. Model/training-data provenance remains separately qualified in the classifier NOTICE. The feature test font retains its SIL OFL-1.1 notice.

## Verification

`cargo test --workspace --offline -j 2`: 22 tests pass. `cargo clippy --workspace --offline --all-targets -j 2 -- -D warnings`, formatting and build also pass.

Root CLI feature-vector parity: 136 official Java cases, 20,264 output scores; maximum absolute error `2.220446049250313e-16`, identical top-5 order. Four invalid input cases reject without successful JSON.

Root CLI image-to-classifier parity: 96 fixed Java cases (synthetic and three OFL Bravura glyphs), both run orientations, multiple scales/origins/padding. 49,728 numerical values compared. Maximum absolute errors: raw `3.0531133177191805e-16`, normalized `7.993605777301127e-15`, hidden `3.885780586188048e-15`, scores `1.4432899320127035e-15`; top-5 order identical. Fixed tolerance is `1e-12 + 1e-10 * abs(expected)`. Gray-valued input, empty glyph, zero interline and invalid orientation reject with exit 1 and no stdout JSON. These establish numerical parity, not recognition accuracy.

Earlier GRID preparation parity covers eight synthetic fixtures plus one CC0 OpenScore score raster, including 2,576 horizontal and 3,673 vertical sections for the real page. The MuseScore converter produced a complete PNG but exited abnormally under the filesystem restriction; the validated artifact, CRC/dimensions/hashes and exact qualification are recorded in `fixtures/rendered/manifest.json`. This is not a successful-process or renderer reproducibility claim. See `GRID-PREP.md` for source/oracle boundaries.

## Next implementation boundary

Implement `FilamentFactory` length/fatness gates and line fitting/merge/expand using original numerical rules, then curved line selection and `ClustersRetriever` staff grouping. Retain section IDs and absolute geometry so a subsequent staff-removal/candidate-grouping stage can use the existing glyph API. Compare geometry/intermediate decisions to pinned Java before claiming staff detection. Keep separate stages for note/stem relationships, rhythm and semantic MusicXML tests.

No GUI or Java runtime is a production dependency. Java is a test oracle only. No external publication, remote repository or credentials were created.
