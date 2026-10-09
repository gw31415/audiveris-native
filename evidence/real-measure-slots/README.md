# Actual shared Stack geometry during preliminary slots

Original SlotsRetriever compares Chord Stack x-offsets. An integer Measure left
boundary loses the fractional physical bar center and cannot interpolate across
actual Staff curves. Retain measured sheet skew in existing MeasureLayout Arcs;
read the physical right-bar center2D (or staff-start/midline point), find actual
surrounding Staffs and their same held Stack Measures, and use original one-Staff
deskew or two-Staff midline interpolation. Geometry is not reconstructed from a
report; foreign same-ID Measure Arcs remain explicit failures.

All67 actual Chord offsets match source within1e-9. Original normal LINKS's
8 actual Measures have narrow/wide/compound counts7/5/8/5/6/6/6/6. Full member
bounds and order match all three retrieval stages, and the full graph remains
unchanged. Existing integer abscissae keep their own original contract; center2D
is consumed only by the Stack geometric calculation.

This is preliminary geometry before Voice clocks, not final shared rhythm or
XML quality. Next connect actual shared Stack slots and3/4 rational clocks to
normal multi-Part output using the already compared actual Voice families and
separated distance. Preserve Source null/abnormal/excess intermediate states
and current selected reprocessing callback/rollback semantics.
