# Next original rhythm dependency

Original source9e1e55cd2746037d059345881c53e6a6754bffbd, app/src/main/java.
Connected bounded units: ordered SlotsRetriever narrow/wide graph -> original
compound partition/member lookup -> resolved current intrinsic duration -> real
starting Voice/setVoice group/classified-tie -> setAndPushTime(0). Current later
rookies still use the admitted single-voice continuation; not general rhythm.

The next connected unit is `MeasureRhythm.SlotMapper`:

1. `readSlotTimes`: collect known chord times per narrow member. One time fixes
   the slot; conflicting times go to original analyzeTimes, not silent overwrite.
   `purgeRookiesSet`: already-owned beam/tie chords are not candidates for mapping.
2. `retrieveActives`: normal Voices only; last chord/end must allow remaining
   measure duration (with original implicit-tuplet exception); last chord slot
   must not be later than current compound; extinct Voice kept only for explicit
   NextInVoice. Requires actual temporary candidate slot/Voice identities.
3. `ChordsMapper.processNext` preassignment, Next/SeparateVoice priority and
   `VoiceDistance.Separated` -> `InjectionSolver` strict cost improvement and
   ascending-range tie behavior. This is not an invented nearest/greedy matcher.
   Current one-part/one-staff bounded path uses Separated, while Merged remains
   required for original merged parts. Pitch distance is original highest-note
   absolute diatonic pitch; rest pairs use head-note area center, not stem tail.
4. `checkSlotTime`/mapRookies candidate validation, end beyond expected duration,
   blacklist retries and applyMapping(setVoice -> setAndPushTime(active.end)).
   Unmapped rookies require original new-Voice/sibling-time/previous-slot merge,
   not unconditional serial assignment. Implicit-tuples shrink/retry, multiple
   voices/parts/stacks and full Rational remain necessary dependencies.

Source audit read original ChordsMapper.java, VoiceDistance.java and
math/InjectionSolver.java from the pinned original public tree through the
existing permitted route. No implementation claim follows from that read.
`processAll` is explicitly disabled in original mapRookies ("Feature not yet OK")
so it does not replace the active original process() path.

The original all-rest divisions=0/XSD failure remains a recorded negative
boundary. Do not rerun it or expand image/font/scan/skew admission to manufacture
positive parity. Existing same-image MusicXML examples and global false flags
remain. No planned nonUI feature omission.
