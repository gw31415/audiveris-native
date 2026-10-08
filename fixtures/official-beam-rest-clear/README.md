# Original pre-reset BeamRest clear and RestChord beam cache

Pinned original9e1e55c and JAR9b35257e; reused original parallel-beam saved book.
Six independent short/excess x automatic/manual/mixed controls compare30 method
boundaries. Source rests are actual newly registered RestInter/RestChord members
of the first real Measure; the third actual rest belongs to the unselected
Measure and remains unlinked/null-Voice. No new Voice number is supplied.
The actual private clearInterleavedRests method is invoked before Measure reset.
Real original relation objects, cached beam-list objects, old Voice bodies/slots,
termination/excess, cached expected/actual and actual Chords are observed.

Automatic BeamRest edges are removed; manual edges retain their original objects.
The removed callback checks only beam abnormality. RestChord beam lists stay
cached through removal, repeated clear and legitimate Measure.resetRhythm.
Explicit RestChord.invalidateCache supplies a fresh beam-list object, reflecting
remaining manual relations. Original old Voice objects and stack timing survive
Measure reset while Chord clock/Slot/Voice references reset legitimately.
This controlled pass does not claim automatic detection or raster recognition
of the declared rests, general controller/task correction or full rhythm parity.

Raw original observation and logs remain at /tmp/beam-rest-clear-bounded-original.
The retained observed.json uses shared relation arrays to remove repeated rows;
expansion was checked equal to every raw original field. Provenance separately
records the raw original hash and retained lossless fixture hash. No old oracle
observation or ce785c9 was replaced. Initial observer attempts chose nonbeamed
source/target chords and failed at null groups; the final declared control uses
an actual beamed source Voice chord and an actual unlinked target rest.

Source: https://raw.githubusercontent.com/Audiveris/audiveris/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sig/inter/RestChordInter.java
