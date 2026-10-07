# Live Voice reassignment and number exchange

```sh
cargo test --offline --locked -j2 --lib live_voice_swap_preserves_reference
cargo test --offline --locked -j2 --lib failed_live_voice_swap_prelink
```

The fifth independent original control constructs a new Voice from chord0 without
pruning the old Voice's list. Actual Chord.voice changes to the new object, while
both lists contain the Chord. Original Measure.swapVoiceId then exchanges their
musical numbers. Voice.setId changes only the Voice object; Chords read the number
through their actual reference, regardless of stale list entries. The prior native
list-based scalar update reproduced chord0=2 where original chord0=1.

Native retained Voices now have a stable identity shared by their Graph bindings
and staging/rollback copies. Assignment's recursion guard uses that actual binding;
renaming updates bound Chords, including cached references absent from a Voice's
list, without assigning them through old member lists. Empty-Voice retirement
matches those same bindings. Native bounds/grade refresh retains the binding;
explicit setVoice replaces it. No ownership is rebuilt from a report.

Original setup before/rebind/swap, twelve raw preLink task boundaries and thirty
listener records are compared. Actual original Voice reference equality is
observed inside all callbacks in this case. The final original reuse of the
retired Voice's number by the surviving live Voice is a separate observation;
old/new objects and slots remain separate. The four prior cases are unchanged.
Normal live Voice lists/slots are compared at task/setup boundaries, and cached
retired Voice state is compared in callbacks; full normal Part/Chord/Voice lists
inside callbacks remain pending. Saved grade/Stem-bounds controls still apply;
this is not automatic image parity or public CLI admission.

A late native task failure after member/empty-Chord/Voice removal restores actual
Voice identities, old overlapping member lists, slot/termination/excess state,
Graph, real Chords and expectedDuration. Repairing the same sequence then executes
Do/Undo/Redo while retaining exactly one removed Voice and the same references.
This is native atomic policy, not original Java UI exception handling parity.
