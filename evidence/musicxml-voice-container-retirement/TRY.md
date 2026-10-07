# Normal container removal retains actual Voice owners

```sh
cargo test --offline --locked -j2 --lib music_voices::lifecycle_tests
cargo test --offline --locked -j2 --lib music_containers::
```

Two new independent original controls compare twelve timed actual Measure/
Voice ownership boundaries. Consecutive genuine Measure.removeInter calls
remove stored Chords from the Voice and finally remove its live registration.
Original cached Chord references still identify that actual Voice, whose four
slot entries, signed termination-5/8 or+1/2/excess1/2 and manual seed remain.
Stack/expected/actual clocks are unchanged and repeated removal is idempotent.
Original SIG removal is separately covered by the existing exclusive/shared
head-chord controls; this new observer isolates its Measure callback delegate.
No full original/native graph or inside-listener equality is claimed.

Source3699bc2 loses that actual owner after normal last-Chord removal. Normal
music_containers now hands the moved Voice to the same actual Graph owner used
by task removal, refine purge and Measure reset. Live and removed Chord bindings
stay on the existing identity; first-empty stop is unchanged. Refresh validates
all real Chords before altering actual Voice ownership. A bad later Chord after
removed-Chord detection leaves actual Graph, Chords, live Voice/identity,
expected/manual/slots/termination/excess intact; repair/retry retains one owner.
This is native atomic refresh policy, not Java UI/controller exception parity.

Reproduce the new original control in a fresh output directory:

```sh
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/double-augmentation.omr \
  --probe VoiceRemovalLifecycleProbe --output /tmp/new-normal-removal-control
```

Previous24 original lifecycle boundaries and existing CLI XML/XSD remain in
regression. Public/global guards stay false/guarded. General Part/Measure
callback integration, cached retired Voice getters and rhythm/history remain
pending; no report owner reconstruction or callback-before clock erase.
Historical649 and original task/environment are untouched. ce785c9 is neither
obtained nor regenerated/replaced. See VALIDATION.json and NEXT-DEPENDENCY.md.
