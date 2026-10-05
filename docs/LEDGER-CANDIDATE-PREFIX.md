# Native ledger candidate prefix

Ports `LedgersBuilder.getCandidateFilaments` from original Audiveris commit
`9e1e55cd2746037d059345881c53e6a6754bffbd` (authorized JAR SHA-256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`).

The original order is horizontal StickFactory retrieval with no opposite
sections, beam-midpoint overlap purge, then maximum-width purge. Maximum
thickness is the smaller of ties-to-even rounded 3.25 foreground thickness and
0.4 main interline; minimum core length is one main interline, side ratio 0.9,
and maximum ledger width 20 main interlines (equality retained). Horizontal
glyphs retain horizontal run tables. Existing vertical callers retain their
0.4 side ratio and vertical run tables, without transposing member coordinates.

`ledger_candidates::retrieve_candidates` takes declared horizontal sections
and already-selected good full beam areas in original left-abscissa order.
Beam areas and bounds must come from the same beam; caller selection of good
full beams remains a prerequisite. No grade heuristic or synthetic ledger
registration is substituted for that prerequisite.

The independent Java observer constructs real original DynamicSections, scales,
good full BeamInters and a LedgersBuilder on a real original GRID system. It
invokes the untouched private candidate method directly. Its 93 supplied
controls include core-length/thickness boundaries, odd interline rounding,
0.9 side overlap, exact maximum length and sloped beam midpoint overlap:
54 produce candidates and 39 produce none. Native checks compare member order,
bounds, endpoints, fitted line/coefficients/distance, weight and glyph runs.
The existing 26 vertical factory controls, 12 section banks and four direct
line fits are also retained as regression controls.

This is candidate discovery only. Native GRID ledger dispatch, good full beam
selection, signed staff-index lookup, original ledger check suites, reduction,
glyph/graph registration and Staff ledger maps remain unfinished. No accepted
LedgerInter or positive real-score rest conversion is claimed. Original stage
order places BEAMS before LEDGERS; this prefix does not retroactively complete
the earlier BEAMS rest stage. Full OMR/MusicXML remain unfinished.

The initial independent comparison found vertical glyph serialization on a
horizontal filament; it was corrected before the passing focused checkpoint.
The first all-target check also caught an internal test call needing the new
orientation parameter; its corrected original line-fit controls are checked.
Full suites are unverified at this source. The prior full baseline remains
321 debug and 321 release tests on source
`7bc4213cea7ed27ee8355e319447dda3b9b7074f`.
