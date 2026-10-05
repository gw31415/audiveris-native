# Native BEAMS spot preparation

Current CLI contract: [BEAMS epilog](BEAMS-EPILOG.md). The stage reports below describe the named historical checkpoint; current supported single-staff inputs can reach BEAMS.EPILOG. Phase snapshots retain their earlier completion flags.

This pixel milestone is now followed by [native BEAMS prolog](BEAM-PROLOG.md), which adds head sizing and system dispatch. The historical v15 evidence below remains unchanged.

Pinned Audiveris 5.11.0: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

`beam-spots INPUT SPOTS.pgm REPORT.json` now runs the native image pipeline through GRID, HEADERS and STEM_SEEDS, then writes the binary beam-spot image and canonical unassigned spot glyphs. Java is a development oracle only; the executable does not launch Java or other tools, load expected fixtures, or use a network service.

This is an explicitly partial BEAMS milestone. `last_completed_stage` remains `STEM_SEEDS`, the original v15 `last_completed_substage` was `BEAMS.SPOT_PREPARATION` (current successful prolog reports `BEAMS.PROLOG`), and `beams_complete`/`musicxml_complete` remain false. Spot glyphs can contain noteheads and noise as well as beams. They are not recognized musical beams or notes.

## Translated behavior

- NO_STAFF horizontal run filtering, with inclusive `length >= maxStem`
- Picture's radius-1 median and two-pass Gaussian gray filters, including byte rounding between passes and Java float accumulation
- SpotsBuilder header removal, using the system area bounds and outer staff curves at the first header's stop, with a 2-interline margin and half-even integer coordinates
- Exact grayscale closing with the original circular support, float radius, exterior values and threshold equality
- Separate beam threshold 140 and head-spot threshold 170 images
- Original vertical GlyphFactory extraction order, four-connected pixels, canonical runs, weight and half-even pixel centroid

The JSON records full-image SHA-256 and zero-valued pixel counts for each intermediate buffer. The new command's PGM is the final beam threshold image. `page-glyphs` and `staff-mask` also add the preparation report; if its additional bounds reject an input, those older commands retain their earlier outputs and explicitly report the failed substage instead of falsely marking it complete. `beam-spots` requires the new substage to succeed and writes no output on such failure.

## Independent validation

- 13 full native pipelines (12 independently rendered synthetic rasters plus Laura) match original scale/header rectangles, every intermediate image hash, both threshold images, and all 483 unassigned spot glyphs in original order with exact pixels, weights and centroids
- Direct-original controls cover 56 grayscale inputs, 464 morphology outputs, 120 horizontal filters, 10 circular supports, 24 extractions/224 glyphs, 14 ROI controls and 42 stored-curve integer samples; see `tests/beam_spots_oracle.rs` and its provenance files
- The unmodified original `BeamsStep.doit` was also run on all 13 rasters. HEAD_SPOTS, unchanged NO_STAFF, head scale, and beam vertices/relations agree with the method-boundary observer
- The 12 synthetic full final snapshots agree exactly. Laura's original plain step retains one grouped spot per system after epilog while the explicit boundary observer leaves none. This historical observer artifact is now explained and repaired in [atomic prolog observation](BEAM-OBSERVER-IDENTITY.md); native full BEAMS is still not claimed
- `beam-spots` is tested with an empty environment and `PATH=/nonexistent`; existing output paths are refused and preserved

Reproduce development-only original comparisons with the already-installed local oracle dependencies:

```sh
python3 scripts/linux_oracle.py --probe beams --fixtures fixtures/official-grid/beams-*.png laura-gray
python3 scripts/linux_oracle.py --probe beams-plain --fixtures fixtures/official-grid/beams-*.png laura-gray
python3 scripts/linux_oracle.py --probe beam-filters-control --fixtures stem-seeds-clean
cargo test --locked --test beam_preparation --test beam_spots --test beam_spots_oracle
```

## Supported boundary

The input boundary remains opaque 8-bit grayscale PNG or PGM, four million pixels and 32 MB. This is not yet bounded-memory 300 DPI PDF processing. The adapter requires the current single-column, five-line, full-width native GRID path and the default `smallBeams=false` setting. A measured small-beam scale still lowers the closing diameter. It rejects inconsistent dependency identities, oversized spline/graph inputs and invalid geometry.

Additional native computational bounds are explicit: effective beam thickness up to 128, circular radius up to 64, and at most one billion morphology sample probes. These are native resource safeguards, not claimed upstream restrictions. Integer-overflowing header rectangles are rejected, including one upper-edge case that original ImageJ silently ignores.

BlackHeadSizer and glyph registration/tagging/system dispatch are now connected in the subsequent prolog milestone. BeamStructure, BeamInter/beam groups, hooks and multiple-measure rests remain downstream. LEDGERS, automatic HEADS/stem attachment, rhythm and MusicXML are also unfinished. Retained original full-BEAMS snapshots are future research evidence, not runtime dependencies or a claim of native implementation.
