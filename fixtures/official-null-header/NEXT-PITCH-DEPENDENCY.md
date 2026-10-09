# Next unit: nullable pitch and exported header state

Status: source observations prepared; native nullable pitch, mapping and export
changes are **not implemented or tested** in this unit. The completed change is
only empty-header admission through native MEASURES/head CHORDS.

`pitch-export-original.json` observes the unchanged PNG through fresh original
LINKS and PAGE, followed by an actual `PartwiseBuilder` on its real Score.
All 20 Staff 6 heads retain null physical clef/key lookup. `ClefInter` still
computes their step, octave and absolute pitch using its static G/TREBLE default.
That default is not registered in the Staff or SIG.

The builder emits C1/F4 clefs and a global key +1 only at the initial measure.
Its final logical Part 2 key cache still contains +1 for both staff indices.
The emitted pitch/header model matches the saved original baseline for every
part, measure and note. This is not a raw MusicXML byte comparison.

Two actual F5 heads, bounds `[186,1276,15,12]` and `[635,1276,15,12]`, distinguish
the key contexts: declared `getAlteration(null)` returns 0, declared
`getAlteration(+1)` returns 1, and the real builder exports alter 1. These declared
calls are controls; the real builder output and cache are observed separately.

Implementation boundaries:

1. In `music_xml/note_pitch.rs`, retain all actual Head/Chord/Measure/Staff checks.
   Make the actual clef identity optional. An absent same-physical-Staff clef uses
   G/TREBLE numeric conversion without creating a vertex or borrowing a previous
   system's clef object. Keep ordinary measure-local key reads unchanged. Add an
   explicit export read taking optional exported-cache fifths for alteration,
   including recursive tied-head lookup; preserve the distinction between an
   actual key ID and a carried export key value.
2. In `music_rookie_mapping.rs`, retain actual highest Head and Staff selection,
   then implement `ClefInter.absolutePitchOf(null)` directly. Do not infer absolute
   pitch from musical step/octave: source `absolutePitchOf` uses
   `28 - rint(clef.pitch) - headPitch` for C clefs, whereas step/octave conversion
   uses the opposite clef-pitch sign. Current native mapping's conversion differs
   by 8 on a C1 clef at pitch 4. The 268 fresh direct-JAR controls in
   `absolute-control-original.json` certify this API result, null/default and
   half-even/adjacent-value boundaries independently of step/octave. Its effect
   on current image Voice decisions is not yet measured.
3. In `music_xml/recognized_export.rs`, track emitted clef/key state by logical
   Part staff index independently of physical Staff ownership. Emit only actual
   clef events and retain preceding XML F4 when Staff 6 has none. Feed the retained
   key +1 to note alteration before writing events. `processKey(null)` is a no-op;
   the separate `processKeyVoid` case resets a nonzero carried key only when a
   system-start measure has no keys at all. Here Staff 5 has a real key +1, so that
   whole-Part void case does not apply. Do not turn missing Staff 6 key into an
   artificial natural-key event.
4. Preserve the existing explicit mid-measure-clef guard and other unverified
   cases. `stack_rhythm.rs` also requires a Part's leading Staff clef; Der Winter
   Part 2's leading Staff 5 has a real C clef, so that guard is not this dependency.
   A completely clefless Part remains a separate admission case.

Target validation should distinguish source behavior from score truth. The
original baseline has 20 second-system lower-staff pitch errors, plus two earlier
timing/duration errors. Matching its null-clef behavior demonstrates port fidelity;
it does not establish recognition quality. Reuse the existing actual-image owner
test path where possible, compare null IDs and stable graph/owner clocks, then
compare exported clef/key events and pitches with the source and independent
score truth. Do not insert original observations into native recognition.
