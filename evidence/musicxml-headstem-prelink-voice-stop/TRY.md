# Stop Voice removal after its first empty owner

```sh
cargo test --offline --locked -j2 --lib first_empty_voice_removal_stops
cargo test --offline --locked -j2 --lib failed_overlapping_empty_voice_prelink
```

The sixth independent original control trims the old Voice's list to chord0,
without changing any Chord.voice/Slot/time/expected references. It then constructs
a new Voice from that same Chord without pruning the old list. Both real lists
contain chord0. Original Measure.removeVoiceChord removes it from the first Voice,
deletes that now-empty Voice from the Measure and stops. The later Voice still
contains chord0 through Do and Undo. On Redo that later Voice becomes empty and
is deleted. Old cached Voice references and their separate slot tables remain.

The earlier native removal first pruned every Voice and then removed all empty
Voices, reproducing no remaining Voices where original retained Voice2. The native
loop now follows actual successful removal and stops on the first empty owner.
It neither purges unrelated already-empty owners nor prunes later lists before
the original stopping point. Retained Voice bindings use the existing actual
identity, including live Chords whose original owner no longer lists them.

Original three setup states, twelve raw preLink task boundaries/thirty listener
records, cached old/new Voice slots and actual reference equality in callbacks
are compared. The five previous original controls are unchanged. Native late
failure restores both real overlapping lists, actual Voice references, Graph/
Chords and expected/slot/termination/excess clocks. The same sequence then runs
Do/Undo/Redo with retained Voice counts1/1/2, matching original lifecycle. This is
native atomic failure policy, not original Java UI exception rollback parity.

General Part/Measure/Voice movement, full normal lists in callbacks and legitimate
reprocessing are still pending. Saved grade/Stem-bounds declarations remain;
automatic source Stem extent parity and public HeadStem CLI admission stay guarded.
All nonUI completion flags remain false; no original functionality is omitted.
