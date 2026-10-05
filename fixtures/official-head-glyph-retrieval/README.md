# Original HEADS glyph retrieval and live seed conflicts

Unmodified pinned Audiveris 5.11, 9e1e55cd2746037d059345881c53e6a6754bffbd,
on Linux. No native outputs are used by the observer. For each staff the original
seed process runs first, registering 358 seed heads with actual glyph bounds.
Range proposals remain the certified 2,224 from 118 scanners (407 weak stemless
rejections); aggregation selects 454. Good seed head slices discard 438 range
representatives, leaving 16 range glyphs. Original methods are observed before
staff duplicate/overlap handling, staff note insertion and small-beam purging.

Foreground retrieval reads the full binary raster, not neutralized distance
pixels or HEAD_SPOTS. Only template keypoints with distance zero are selected;
out-of-image coordinates are skipped. All selected pixels form one vertical
run-table glyph, including disconnected pieces. No dilation is used. Canonical
content reuses the sheet glyph registry; glyph bounds replace the slim bounds.
Source shape/score and original slim bounds remain in native metadata. Seed
source templates follow original enum order (void before black), left anchor
before right. Good (grade >= 0.4) seed dx is measured from the actual glyph box.

Sixteen original TemplateSymbol controls cover each of breve, whole, void and
black under clean, erased, clipped-edge and empty images. Empty foreground is
an explicit null retrieval. Native registry/malformed-input controls check reuse
and rejection. The native stage is transactional with respect to the existing
BEAMS/HEADS glyph registry. SIG registration and full HEADS completion are pending.

Pivot controls record original `StaffLine.yAt(int)` (rounded integer ordinate)
and `LineUtil.xAtY` determinant evaluation order as raw double bits. This covers
the half-pixel seed at x=823 and prevents accidental substitution of the double
ordinate overload or algebraically rearranged intersection. The live physical
search uses this original Scanner contract, while the previous saved 418→358
seed diagnostic report remains unchanged.
