# Original previous-staff seed retirement contract

Pinned unmodified Audiveris 5.11.0 on Linux, original NoteHeadsBuilder duplicate
purge followed by Inters.intersectedInters, getCompetitorsSlice and seed filtering.
The observer reads no native output. Eight constructor-only conditions include
empty input, singleton, equal-grade tolerance preserving seed dx, higher/lower
seed grade, weak seed, separated and out-of-slice seeds.

The decisive boundary removes a seed of grade 0.7 in favor of a seed of grade
0.699995 carrying dx. The range head lies between these seed grades at the 0.9
conflict threshold. Retaining the removed reference incorrectly vetoes the next
staff's range head. The original live slice skips it and accepts the head.

The physical registration path now retires each staff's duplicates, overlaps
and tally before slicing competitors for the next staff. The earlier standalone
glyph report is preserved as the pre-purge diagnostic. Both paths retain all
118 Laura scanners and 358 seed / 438 conflict / 16 range glyphs, and the full
registration/epilog still matches 352 original heads and 513 exclusions.

HEADS writes to the existing common shared Graph and canonical sheet glyph
registry. A later GRID, BEAMS or LEDGERS-only refresh of a graph containing live
HEADS is rejected transactionally; callers must provide complete phase state.
Legacy report and earlier phase identities remain unchanged. Overall HEADS
completion stays false: broader font/size and special staff catalogs are still
outside the certified Bravura 51pt, ordinary five-line staff/ledger subset.
