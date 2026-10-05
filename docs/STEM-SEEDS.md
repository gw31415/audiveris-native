# Native STEM_SEEDS

Pinned Audiveris 5.11.0: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

The CLI now runs native GRID and HEADERS, estimates stem width, assembles vertical filaments from native vertical sections plus one-pixel-wide horizontal stickers, applies the seven seed checks, and returns accepted VERTICAL_SEED glyphs. These are seed candidates, not final StemInter or recognized musical notes.

## Verified behavior

- 11 complete native image pipelines match original accepted seed membership and glyph pixels, candidate membership/geometry, closest staff/header skips, black/white/gap counts and acceptance
- 352 original raw StemChecker contexts independently match exactly, including all floating check results when using original geometry
- 16 direct-original histogram cases cover run-count area, peaks, fallback, terminal behavior, half-even rounding and counts above 32767
- 26 direct-original StickFactory controls cover core length/thickness, stable ties, side-growth ratio 0.4, stickers and explicit constructed overlapping/unsorted cases
- Four direct StraightFilament fits preserve the actual sampler’s ceil/floor boundaries

End-to-end floating geometry/grades are compared with absolute tolerance1e-8. Pixel runs and sample counts are exact. Accepted seed membership is compared as a set because the original observer canonicalizes that collection; candidate order is compared directly.

Laura produces 300 accepted seeds. Independent clean/headed/mixed-width stem renderings each produce 3, broken stems 6, and empty/noise/short/sloped renderings 0. Earlier empty-header and four-flat renderings each produce one bar-like seed in the original, so they are not labelled negative seed inputs.

## Pinned-runtime subtleties

StemScaler paints a detached BufferedImage: its returned ByteProcessor remains byte-identical to NO_STAFF. This is verified by three mask hashes per raster and a separate 5×4 ImageJ control. The port preserves this actual behavior rather than implementing the erasure suggested by comments.

StickFactory preserves supplied vertical-section order; SystemInfo sorting is performed at the GRID adapter boundary. SectionTally’s recorded starts, mutable neighbor pruning, coordinate-tied TreeSets and delayed compound ownership are retained. BasicLine accumulation includes section pixels, and glyph assembly independently deduplicates pixels.

Staff-area containment precedes the original truncated Java-int distance comparison. SystemInfo.getRight is left+width, not left+width−1. Profile 1 is the pipeline default; the isolated checker also supports profile 0.

## Limits

The pipeline explicitly requires the current single-column, five-line, full-width-area native GRID path and profile 1. It preserves the four-million-pixel cap. Unsupported profiles, malformed geometry/identity, oversized lag/spline/graph inputs, Java-int overflow and undefined sampling fail explicitly. Java is used only by development oracles; normal CLI execution does not invoke it.

Additional source downloading was stopped after a cancelled network approval. Subsequent work used only cached pinned Java files and bytecode inspection of the already-authorized official local JAR. No alternate network retrieval was attempted.

BEAMS pixel/spot preparation is now connected; see [the separate partial milestone](BEAM-PREPARATION.md). Full BEAMS interpretation, LEDGERS, automatic HEADS/stem attachment, rhythm, MusicXML and bounded-memory 300 DPI PDF processing remain open.
