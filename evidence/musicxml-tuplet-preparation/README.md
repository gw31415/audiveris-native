# Implicit Tuplet preparation on actual owners

Actual Measure insertion-ordered Tuplet references and the admitted single-part
Stack's unassigned Tuplet references are retained independently of rhythm clocks.
getImplicitTuplets uses Measure members only. Native removal detaches Measure and
Stack first, then marks the actual implicit SIG Tuplet removed before its incoming
ChordTuplet removals, while the vertex is still registered. Manual implicit signs
are removed too. Explicit/manual and Stack-only unassigned implicit signs remain.
Actual source Chords are validated, null/unlinked implicit signs need no fabricated
source/Voice, old relation bodies/IDs are retained and clocks are not cleared.

Two pinned Java short/excess controls compare five boundaries: before removal,
implicit removed, automatic BeamRest clear, complete captured-group detection,
and genuine Measure reset. Six original ChordTuplet removal observations are
AFTER SigListener, with removed=true/registered=true and detached containers.
Native entry/exit keeps actual old Voice/Slot owners and original clocks; only
legitimate reset detaches clocks. Read-only getDuration/getEndTime follow the
first actual ChordTuplet relation and original THREE2/3 or SIX4/6 factor; ordinary
MusicXML duration admission still explicitly guards tuplets. Source duration
changes do not recalculate retained old Voice end/excess behind the callbacks.

The combined pre-pass operation also rolls back late actual RestChord member
failure after implicit/manual removal and automatic-rest clearing. Same physical
Chord, original Tuplet collection storage, Voice/Slot clocks, expected/end/excess,
old registrations and empty discarded trace survive; repair retries same owners.
Initial failure test damaged an unvisited plain candidate; corrected control uses
an actual captured manual RestChord. First source controls remain separate; v2
adds Stack-only null-source implicit sign. Compact frame/clock interning is
lossless and both source/JAR/book and raw/compact hashes are retained.

No new archive/full649/complete selected-profile rerun or auth/settings/environment
change. The recorded automatic Stem5px difference remains separate. General
Tuplet added/explicit removal/extensive ensemble/exclusion callbacks, Stack/Part
multipart assignment, cached factors and implicit generation/merge remain pending.
Complete ordinary selected prep connection must preserve its existing admission
and actual rollback; full nonUI/public flags remain false. See VALIDATION.json.
