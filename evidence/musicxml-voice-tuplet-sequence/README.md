# Actual Voice Tuplet lookup and retained-list boundaries

Pinned original5 mixed/explicit-first/implicit-first/explicit-only short/excess
cases observe25 actual list/purge/restoration states and50 Voice getter rows,
including350 actual ChordAfter queries. Initial Tuplet/ChordTuplet objects are
real declared SIG/Measure/Stack objects with manual flags and supplied encounter
order; no native output supplies the losslessly interned65,501 byte fixture.

Voice.getTuplets visits its actual ordered Chords, reads only the first encountered
relation on each, and deduplicates actual Tuplet objects in encounter order. An
explicit first relation hides a later implicit one for getVoicesWithImplicit,
which visits the current actual Measure Voice list. getLastChordWithTuplet instead
counts any ChordTupletRelation, including explicit/manual. No Voice numbers or
projected reports supply these lists.

getChordAfter follows source indexOf: absent actual Chord or null input returns
the first owned Chord; last input returns null. An allocated empty list returns
empty Tuplet list/null successors. A reset null list throws in getTuplets but
returns null for last-with-Tuplet and successors. Pure Measure.purgeVoices does not
renumber remaining Voices or erase old backrefs/slot/end/excess/manual status.
Native retains the old object identity for direct getters. Adding the same old
Voice back changes current Measure encounter order, as observed in source.

Native pure purge is shared by existing refinement before its later sort/rename.
Reader errors leave Graph/physical Chord/Slot storage/expected/report unchanged.
These controls precede inspectVoicesEnd/shrink/merge and second-pass integration;
actual font/signature/switch ownership is not inferred from the evidence. Full
nonUI/public fresh Tuplet automatic admission remain false. See VALIDATION.json.
