# Bounded original PAGE owner replay

Original commit: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

This successful observation loads the existing post-PAGE Der Winter Book, retains its actual objects, and replays selected original duration/classification methods on two held Stacks. It does not rerun image recognition, RHYTHMS, whole MeasureFixer.process, or the rest of PAGE. The Book was never stored. Native recognition output, source XML, truth scores, and native reports were not supplied to the Java observer.

The original Book SHA-256 before and after execution is `a5cf0b402687b9448f6a73aad2d82a93a39dcb9001a97c60a32a1909b58f3204`. Runtime exit 0; 2.991 seconds. Exact Java/compiler arguments and hashes are in PROVENANCE.json. The Java runtime's home, temporary, preferences, JavaCPP cache and XDG directories are isolated under this directory.

## Files

- `observed.json`: full ordered before/after frames with actual owner, Chord, member, Voice and Slot object tokens and musical values.
- `summary.json`: compact derivation, generated solely by `extract.py`.
- `PageMeasureFixerReplayProbe.java`: new observer.
- `MixedRestStartingProbe.java`: common reflection/object-token/geometry helpers; its main is not called.
- `run.py`: exact successful invocation. Preserve existing observations if making another run; create a new output directory and adjust the runner's fixed output path.
- `MANIFEST.json`: supplementary hashes, including common helper and pinned method sources.
- `compare_native.py`: offline comparison to the existing native image target's PAGE report. Does not build or recognize an image.

## Calls and scope

The observer loads the Book through Book.loadBook and the first Stub's ordinary getSheet restoration. It retains System 1's actual `getStacks().get(0)` and `getStacks().get(2)`; their one-based physical stack indices are 1 and 3. Existing printed IDs are 0 and 2.

For each selected Stack:

1. Observe the saved/restored owner state.
2. Call original `MeasureStack.resetSpecial()`.
3. Iterate actual Stack Measures and each Measure's actual Voices in original collection order, matching `MeasureStack.checkDuration` → `Measure.checkDuration`. Call the actual `Voice.checkDuration()` separately, so each before/after state and all peer Measure views can be observed.
4. Instantiate original MeasureFixer. Reflection sets its `stack` field to the held actual Stack; call its original private `getStackTermination`, place that exact returned Rational in `stackTermination`, then call its original private `isPickup`.
5. Only for the first Stack, call original `setPickup` and the original private MeasureFixer `setId(0)`, matching the initial pickup branch where lastId and prevSystemLastId are null.

This deliberately selects the pickup branch after its real predicate, and does not execute the other process branches such as empty, multi-rest, repeat halves, merging, or computeRepeats. The second Stack's ID remains its persisted ID; its ordinary numbering branch is not runtime-observed here.

## Observed results

| Actual Stack | Expected / actual | Ordered final Voice terminations | Common termination | Result |
|---|---|---|---|---|
| System 1, physical 1 | 1/2 / 1/8 | −3/8, −3/8, −3/8 | −3/8 | isPickup true; selected branch ID 0, PICKUP/implicit true |
| System 1, physical 3 | 1/2 / 5/8 | 0, 0, +1/8 | null | isPickup false; shared excess remains 1/8 |

All restored Voice terminations begin null. Voice `kind` and `startingStaff` are also null after this Book reload. These are persistence/transient observations, not instructions to clear the live native post-RHYTHMS state. The actual positive-excess Voice already has excess 1/8 after reload. The native comparison must compare resulting duration contracts while showing its already-computed prior terminations separately.

`resetSpecial` clears PICKUP only; expected, actual, excess, Chord clocks, Voice handles and Slots remain intact. Per-Voice frames demonstrate that each Voice's termination changes only at its own check. Object-token structures are compared in the Java runtime at every frame, including actual Measure/Part/Stack/Chord/member/Voice/Slot references and graph vertex/relation identity, endpoints and manual status. These checks retain the actual objects; no owner is reconstructed from a report. Manual status was preserved as observed; no positive manual mutation control is claimed. Tokens have meaning only inside this execution, not across JVM/native runtimes.

## Declared controls on the actual excess Voice

These controls run after the two replay sequences. They mutate explicitly named in-memory fields on the held actual Part 2 Voice 5 and its actual last Chord 4206, then call the original method. They are not natural image states.

- Null last Chord offset: old termination 1/8, Voice excess 1/8 and shared Stack excess 1/8 survive. The actual Measure was already abnormal, so this control does not observe a false→true abnormal transition.
- Last Chord offset 0 with duration 1/8: termination becomes −3/8 while the old Voice and shared Stack excesses remain 1/8.
- Original last offset 1/2, shared Stack excess declared 1/64: one Voice check publishes 1/8, immediately visible through both actual peer Measures' `getStack()` references.

Between controls and at the end, the exact held old offset/termination/excess/abnormal values are restored. Full serialized selected-Stack state value equality and actual identity structure equality are asserted after restoration. Slots/clocks/owners and relation/manual identity remain intact. Declared controls are evidence only and are not fed to native recognition.

## Native comparison and current coverage

Run after the already-running native target has saved its complete PAGE observation:

```
python3 /tmp/real-rest-page-fixer-replay-original-v3/compare_native.py \
  --native /tmp/real-rest-native-internal-rhythm-export.json \
  --output /tmp/real-rest-page-fixer-replay-original-v3/native-comparison.json
```

Exit 0 means all scoped comparisons match; 1 means differences; 2 means the observation is incomplete. The script requires PAGE success/identity/stale-generation/rollback fields, compares the two Stack final duration contracts and four peer Measure/six Voice states, and compares the first pickup numbering branch. It checks native Graph and Stack-clock/Slot projections are unchanged across PAGE. It does not mistake an older pre-PAGE report for a match. `native-comparison-before-PAGE.json` records the deliberate incomplete result against the earlier report.

The original has per-Voice intermediate frames. The current native PAGE report has aggregate Stack rows and actual identity assertions; this script does not claim to compare each native intermediate callback. Source-declared branch controls are not run on native objects by this script.

Existing native `music_voices::group_tests::original_saved_group_getters_and_declared_beam_rest_follow_actual_ensemble` checks complete→shortened with no old excess, a positive-excess update, then null-offset retention, against `fixtures/official-voice-duration/controls.json`. It does not independently cover the new old-positive-excess→underflow control or the two-peer intermediate shared-excess publication. The present native PAGE implementation publishes the new shared excess to each staged Measure after each Voice check; source replay supplies the concrete constraint for that ordering.

No quality-equivalence or public CLI admission claim is made by this bounded evidence. Existing rhythm, export and image-quality comparisons remain separate.

## Retained failed observer attempts

- v1 compiled, but attempted to dereference the legitimate null startingStaff after reload. It failed before producing successful evidence. v3 preserves the null.
- v2 compiled and reached controls, but its restoration check used Java Map equality with nested int[] bounds, causing reference-array equality to reject equal state values. v3 compares canonical JSON state hashes and independently compares the actual identity structure.

Both directories remain present. Both Book hashes were unchanged. Neither failed run is counted as successful original evidence.
