# Next connected rhythm unit

Original5.11 source9e1e55cd2746037d059345881c53e6a6754bffbd.
The all-rest input is an original invalid-export boundary, not a positive
MusicXML parity fixture. Do not repeat its full recognition or search positions.
Current native recognition/Voice/slot units and successful CLI examples remain.

1. `SlotsRetriever(measure,false/true).buildChordRelationships`: original order
   standard chords without measure rests; SeparateSlots; explicit SameSlot;
   beam groups; mirrors; root stems; intermediate SameSlot extension;
   adjacency; close and distant geometry. `getStemDir` prerequisite is connected.
   Original BEFORE/AFTER/EQUAL/CLOSE relation graph and precedence must be retained,
   using actual chord/typed SIG references, not chosen fixture IDs.
2. `SlotsRetriever.buildSlots`: original relation-sorted candidate list; every
   candidate must be EQUAL/CLOSE to all current members, otherwise start a new
   MeasureSlot. Obtain narrow and wide slots from the same owned image chords.
3. `MeasureRhythm.buildCompoundSlots`: partition narrow slots by intersection
   with wide slots. `processStartingChords`: whole-measure rests and first-slot
   chords in centerY order -> real Voice construction and setAndPushTime(0).
4. `SlotMapper`/`ChordsMapper`/`VoiceDistance`: active/rookie assignment, original
   NextInVoice constraints, inferred clock consistency and fallback/new voices.
   This replaces current single-voice sequential assignment when connected;
   do not simply remove the current polyphony/overlap admission guards.
5. Connect shared multi-part MeasureStack ownership and full rational/tuplet
   timing in dependency order. Currently per-measure timing only represents the
   admitted one-part/one-staff stack. General lyrics, automatic slur/tie,
   correction persistence and complete cleaner/epilog remain partial.

Existing20-slot saved-state/final collection equality is downstream of solved
voices. It is not geometric SlotsRetriever/voice matching parity. Likewise,
existing declared tie propagation does not demonstrate automatic curve/tie
recognition. Positive same-image MusicXML evidence remains distinct from these
saved-state and method-control comparisons. No planned nonUI omission.
