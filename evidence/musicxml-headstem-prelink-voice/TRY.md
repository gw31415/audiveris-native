# Last-Chord Voice retirement control

```sh
cargo test --offline --locked -j2 --lib pre_link_tests
```

The third original control has twelve task boundaries and thirty listener records.
Declared setup uses old Voice.removeChord, chord.justAssignVoice(null), new
Voice(chord,Measure), Measure.addVoice and the original existing SlotVoice entry.
It changes Voice assignment explicitly; Slot/time/expected clocks are untouched.

On last-Chord removal, the actual empty Voice moves out of Measure.voices into a
retained native Graph owner. Cached Chords hold its stable owner key, separate from
the musical voice number. Its old BEGIN slot still references the actual removed
Chord. Undo re-adds Chord and relations, but does not put that Voice in Measure's
list. Do/Undo/Redo reuses the same retained Voice key and slot entry. The detached
object is observed inside subsequent callbacks, with live Voice setup checked at
task boundaries. Late failure restores actual Voices/Chords/Graph and clocks;
the same sequence can retry/undo/redo without allocating another retained Voice.

Native atomic rollback is not original UI exception-handler parity. Saved original
numeric controls apply; no new CLI admission or automatic Stem extent claim.
General Voice reassignment/reused musical numbers, Part/Measure migration,
ordinary reprocessing, mirrors/controller/epilog/history/PAGE/persistence remain
pending. All global completion flags remain false. Historical649 was not rerun.
