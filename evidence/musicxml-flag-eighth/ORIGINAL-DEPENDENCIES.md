# Ordinary single flag → eighth-note image export

Reference: unmodified Audiveris5.11.0, Java source
`9e1e55cd2746037d059345881c53e6a6754bffbd`; original runtime jar SHA256
`9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`.
The renderer and reflection probe are development tools. Neither Java nor
original recognition/XML is used by the native CLI.

| Original dependency | Native connection and bounded verification |
| --- | --- |
| `SymbolsFilter`, `InterFactory`, `AbstractFlagInter.createValidAdded/lookupLink` | Classify the actual residual glyph, search actual owned stems in abscissa order, register canonical flag pixels and a `FlagStemRelation`; no caller-selected IDs. |
| `StemInter.computeDirection`, `HeadStemRelation.getStemPortion` | Determine polarity from the actual single HeadStem relation and extended stem endpoints. Multiple-head stems remain pending. |
| `AbstractConnection.setInOutGaps`, `FlagStemRelation` | Preserve different lookup profile1 and intrinsic relation profile0, signed in/out gap weights, minimum relation grade0.1, support coefficient3 at both ends. |
| `SIGraph.computeContextualGrade`, `SigReducer.contextualizeAndPurge` | Reuse contextual support/exclusion partitions; add HeadStem/FlagStem support, preserve non-Support Containment/ChordStem distinction, remove weak flags below0.5 with actual incident tombstones. General supported symbol conflicts remain pending. |
| `AbstractChordInter.getBeamsOrFlagsNumber/getDurationSansDotOrTuplet` | An actual ordinary value1 flag on a black-head chord halves its base quarter duration. No flag inferred from note position. Positive beams and higher/small flags remain pending. |
| `Voice`, `MeasureRhythm`, `PartwiseBuilder` | Exact eighth-of-whole integer ticks supply onsets and measure sums. XML divisions2 is selected when needed; former quarter/half-only output retains divisions1. This is the current single-voice complete-measure subset, not full slot/voice mapping. |

Same-input full-image oracle success: Bravura44/interline11, ordinary five-line
treble/C-major4/4, two measures, nine events. The first two are B4 eighths;
remaining seven are quarters. MusicXML pitch, duration, onset and voice agree.
The original has **no beam tags** for these standalone flagged notes; native
also emits none. A separate beam image is not implied by this validation.

Saved original flag bounds/runs, intrinsic/contextual three-decimal grades,
relation gaps/grades/extensions agree on this input. Native stem endpoints are
not identical to the saved original final STEMS endpoints; whole STEMS/SIG
parity is not claimed. Four lookup and seven grade controls are reflection
kernel comparisons, not eleven additional successful pages.

Erase only the actual flag wing pixels in the test: the notes become quarters
and the five-quarter measure refuses. Wrong polarity/system, empty input,
lookup overlap and relation gap thresholds also have targeted checks. Positive
beams, finer durations, tuplets, polyphony, arbitrary scans/fonts/skew and all
original-stage completion flags remain pending/false.
