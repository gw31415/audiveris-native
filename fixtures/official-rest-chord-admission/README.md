# Original rest admission boundaries

Original5.11.0 source9e1e55cd2746037d059345881c53e6a6754bffbd.
RestChordAdmissionProbe calls unmodified RestInter.createValid on actual physical
glyphs and a typed stem-bound HeadChord on existing ordinary staff/measure.
Five conditions cover head/tail nearness, exact/below minimum dx and boundary-only
intersection. Production native admission now uses the actual chord envelope
and center2D, correcting the former head-only mismatch. The production predicate
is tested against all5 controls; the older PROTOTYPE.json describes its earlier
unconnected state and is retained as historical evidence. No new raster page
or notation admission.
