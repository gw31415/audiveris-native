# Reused musical Voice number with distinct actual owners

```sh
cargo test --offline --locked -j2 --lib same_number_reassigned_voice
cargo test --offline --locked -j2 --lib failed_reused_number_voice_edit
```

The fourth independent original control adds a new live Voice2 after the earlier
solo Voice2 was retired and the old Chord restored by undo. Actual setVoice(new2)
changes the Chord's object binding despite the equal musical number, adds it to
new2's real member list and keeps its Slot/time/expected clocks. Old2 retains its
slot1 entry; new2 retains slot2 without inheriting old2's slot1. Subsequent redo
removes the old Chord from new2's list while keeping new2 alive with its other
Chord. Twelve task boundaries/thirty listener records and explicit rebind
before/after states are compared. Earlier three original cases are unchanged.

The new observation reproduced the native number-only no-op before the fix.
The live assignment now considers detached ownership and replaces the binding
explicitly; generic bounds/grade updates still preserve cached ownership.
Actual live/retired Voice objects remain distinct, with existing relation/manual
identities and callback clocks preserved. Late failed redo restores actual
normal/retired Voices, their member/slot/termination/excess fields, actual Chords,
Graph and expectedDuration; the same sequence can retry.

New live Voice membership/slot state is compared at task/rebind boundaries, while
old retired ownership is observed in callbacks. General multi-owner/Part/Measure
callback/reprocessing integration and other Voice purge/rename/history identity
paths remain pending. Saved numeric controls apply; automatic Stem extent parity
and public CLI admission remain guarded. All global completion flags stay false.
