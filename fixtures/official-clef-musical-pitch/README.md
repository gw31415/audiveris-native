Original Audiveris5.11 ClefInter.noteStepOf/octaveOf controls (180), including
half-integer ties-even pitch conversion, G/F/C clefs and octave-transposing shapes.
ClefInter.java is reused from ../clef-registration-source, not duplicated.
AbstractNoteInter.getStep/getOctave ordinarily resolve Measure.getClefBefore;
the native image path currently uses only its recognized staff header clef.
This fixture validates arithmetic, not page-wide clef/accidental/duration/rhythm
recognition, special staff support or MusicXML completion. No oracle is used
by the production Rust image path.
