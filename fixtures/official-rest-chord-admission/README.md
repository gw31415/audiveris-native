# Original next-unit rest admission (CLI connection pending)

Original5.11.0 source9e1e55cd2746037d059345881c53e6a6754bffbd.
RestChordAdmissionProbe calls unmodified RestInter.createValid on actual physical
glyphs and a typed stem-bound HeadChord on existing ordinary staff/measure.
Five conditions cover head/tail nearness, exact/below minimum dx and boundary-only
intersection. It uses chord envelope and center2D; the current native head-only
admission differs on tail/below-threshold controls and will be corrected next.
A standalone Rust predicate prototype agrees with all5 controls, but is not yet
connected to production image CLI. No new raster page or notation admission.
