# Native line completion and experimental page components

Baseline: Audiveris 5.11.0, `9e1e55cd2746037d059345881c53e6a6754bffbd`, AGPL-3.0-or-later. This checkpoint is **not complete GRID** and is not MusicXML recognition.

`line_completion.rs` translates BarsRetriever column assembly/start-column selection/right-end refinement, LinesRetriever endpoint extrapolation and StaffPattern search, discarded filament inclusion, thick/thin section inclusion, StaffFilament hole filling, CurvedFilament curvature polishing, sticker selection and inclusion. `curves.rs` adds NaturalSpline derivatives and explicit endpoint interpolation. Section membership becomes integer glyph-local horizontal runs, preserving the endpoint-enlarged Java Rectangle bounds without applying StaffLine's geometric Y+0.5 to pixels.

All 9 completion checkpoints on the original AnLaura raster and synthetic two-staff system are compared against calls into the unmodified official jar. AnLaura section totals are 569 → 569 → 569 → 619 → 781 → 781 → 781 → 818 → 818; every section ID and curve point matches (floating tolerance 1e-9). All six final staff end pairs are now exact (234..1416 for first three, 72..1415 for last three), computed from peaks/blanks rather than fixture constants.

The 30 native glyph run tables, integer boxes and origins match the separately captured full official GRID output. Their union removes 54,466 foreground pixels. The residual is 135,311 foreground pixels, raw SHA-256 `26146ee6b8cd7d198ba3dd8b409a354c5aa7f93e2d66439fa357932a65b0b6ff`; every residual and removal-mask pixel is tested.

Integrated `audiveris-staff-removal` from independent local commit `988d3c7fb01b3577ce242aff20446d37d6e5a717` with its tests, oracle, fixtures and license notices. Only two clone-to-slice expressions in tests were adapted for this workspace's Clippy. Production dependencies are std-only for this crate.

```sh
cargo run --offline --locked -- page-glyphs \
  fixtures/official-grid/laura-gray.png /tmp/no-staff.pgm /tmp/page-glyphs.json \
  crates/audiveris-classifier/models/audiveris-5.11.0.json
```

`page-glyphs` is explicitly experimental. Optional model argument adds 149 labeled sigmoid scores per component; without it, 110 raw features are still emitted. Native processing yields 604 four-connected components. These are raw components, not necessarily symbols; no accuracy or speed claim follows from matching computational outputs. No Java, shell subprocess, or reference membership JSON is used by the production CLI.

## Remaining boundary

- BarsRetriever peak purges, brace/bracket recognition, line-root checks, barline interpretations, groups, parts and contextualization are not fully ported. Endpoint geometry matches both fixtures, but general layouts may differ where these omitted filters change peak selection. JSON explicitly keeps `grid_complete:false` and lists pending work.
- Native glyph pixel buffers are emitted; canonical shared glyph-ID registry, simplified StaffLine persistence, horizontal LAG rebuilding and system population remain pending.
- Multiple interline/multicolumn/merged-peak splitting remain unsupported upstream boundaries. Nontrivial discarded-filament acceptance and curvature removal need independent targeted fixtures; in AnLaura these operations are inert, while their surrounding point/membership checkpoints match.
- Beam/stem/note grouping, contextual symbol interpretation, OCR, rhythm, score assembly, PDF rendering and MusicXML export remain unimplemented. Model provenance/license caveats in the classifier crate remain applicable.

Reproduce official checkpoints with `scripts/official_completion_probe.py <Audiveris.app/Contents>` using the separately installed official runtime. The script validates jar SHA, compiles reflection observers with test-only ABI shims, then executes the real jar from original images. Its observers call methods, not rewritten reference algorithms. The production tests consume saved goldens and run offline.
