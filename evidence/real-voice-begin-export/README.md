# Actual recognized Voice/BEGIN multi-Part export

The internal recognized writer traverses retained Voice/BEGIN tables and
canonical shared Stack Slots. It reads live note pitch/accidentals and actual
Chord duration/dots/stem/beams/articulation, with registered Measure/Part/Staff
Arcs. It keeps actual null-clock Chords and emits omitted-BEGIN diagnostics.
The old admitted one-Part CLI writer still preserves its XML bytes.

This is an internal unchanged-image-to-actual-owner export comparison. The
public multi-Part image admission remains guarded: Dynamics/Pedal factories,
LINK lookup/support/weak reduction and corresponding directions are still
missing. The internal observation marks required_directions_connected=false
and quality_accepted=false. Rest-only Staff header reads and mid-measure clef
changes have explicit unresolved guards. This page has no rests/ties/graces;
its success cannot prove those categories or general recognition quality.

Compare canonical notes (including exact onset/duration, Staff, pitch, voice,
chord, dots/accidental/stem/beam/articulation), effective meter/key/clef per
Measure, backup/forward including actual top Staff, and within-Measure voice
groups against original XML. The separate independent truth metric retains
all original/native errors. The comparison reader does not feed recognition.

One initially started validation run was stopped before writer observations to
include the original's four multi-Staff forward Staff fields before the
completed run. Its child outlived the first termination and produced an output,
which is excluded from completed validation. The second completed run includes
all four original forward Staff fields. No success is attributed to that cancelled run. Source artifacts
and the saved b272380 checkpoint remain intact.
