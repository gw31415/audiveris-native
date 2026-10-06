# Original ordinary foundation overlap controls

Original Audiveris5.11.0, source `9e1e55cd2746037d059345881c53e6a6754bffbd`,
JAR SHA256 `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.
Observer: `scripts/java/FoundationOverlapProbe.java`, extending the existing
FoundationOrderProbe/BeamOriginDependencyProbe helpers.

The observer loads the saved original book solely to obtain a real System/Staff,
clears its SIG and constructs the eleven declared object pairs. It invokes
unmodified original private `detectOverlaps`, recording actual glyphs, original
core bounds, both directional predicates, exclusions, repeated insertion and
contextual grades before/after discovery. Exclusions contribute no Support; the
original BeamHead support retains its observed ratios instead of invented values.
These are declared SIG conditions, not an independent raster-recognition result.
The native test recomputes core/area geometry; it never consumes `core_bounds` as
input. Existing full-image MusicXML comparisons are retained separately.

Cases distinguish disconnected ink from shared ink, existing support, the
standard-size head/stem exception with a genuinely positive physical overlap,
compatible beams, head-core border, implicit nodes, V/H runs, barline pixels,
positive ribbon area and boundary-only contact. Angled ribbon controls validate
the Area kernel only; zero-skew image admission is unchanged. Native empty and
inclusive0.05 IOU checks are additional local boundaries, not extra observed
original conditions. No new page fixture or coordinate sweep.
