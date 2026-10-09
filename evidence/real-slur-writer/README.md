# Retained Slur MusicXML writer — focused checkpoint

The recognized export now visits each actual Voice-BEGIN Head's retained SlurHead
relations and verifies the same Slur/Head/Chord/Measure/Staff/Part and relation Arcs.
Per logical Part, numbers1..16 use the original lowest-free/reuse order. LEFT emits
start/placement and RIGHT stop, using the retained curve and actual Staff line.
The serializer preserves existing articulations while emitting all Slurs on a note.
Unowned, stale, missing or unvisited endpoints refuse output. Tie, orphan, extension
and cross-Staff/System branches remain outside this admitted slice; the legacy
renderer guard remains. Fixture geometry is used only by observation tests.

Five focused targets pass:12 saved-original endpoint arithmetic controls, number
reuse/exhaustion, continuation/even rounding, actual notation serialization, and
existing actual-Chord/Slot/Voice rational7/64 export. The latter is byte-identical to
the saved XML and passes local XSD. fmt/clippy and test compilation pass. The source
review records pinned PartwiseBuilder methods; saved Book/XML observations are not
a new original export execution.

The existing DerWinter test now requires successful output and compares124 serialized
notes and12 attached Slur endpoints. That image target was deliberately not rerun
for this small unit: fresh image output, geometry residuals, XSD/independent-reader
and original/truth quality comparison are pending the next integration boundary.
The prior lifecycle checkpoint proved six native owners and preserved clocks with
output withheld. Its old XML is not a new writer result. Quality remains false.

The image test's immutable Registry snapshot is captured once while retaining every
previous entry and run-allocation assertion. This removes repeated observer work;
no measured speedup is claimed, and the updated observer awaits the shared image run.

Latest user policy: changed-code targets per small commit; representative pages and
63+8 XML/XSD grouped at meaningful recognition connection boundaries. No heavy full
regression, new original image run, Book save, new credential or environment change.
