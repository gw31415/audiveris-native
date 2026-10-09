# Original Rest removal callback and clock controls

Two original-runtime declared controls passed. They load the unchanged saved SYMBOLS An Laura Book, execute the original MeasureFiller and ordered SymbolsLinker component methods, select the actual observed singleton QUARTER_REST4436/RestChord4485, and remove that same Rest through `remove(true)`. No image recognition, `reachStep`, native execution, Book save, new authentication, or persistent configuration change is performed. Each control uses its own fresh Java process and private temporary preferences/cache/home directories.

The original Rest, Containment, RestChord, Staff4, Part1, Measure, and Stack objects are retained. The original pre-control Chord Voice/Slot/timeOffset and Stack expectedDuration are null; no existing clocks or owners are cleared. The declaration sets Rest/Chord/Containment manual flags through their original setters, constructs actual original Voice/Slot/SlotVoice instances, registers them through original APIs and the actual Stack slot list, then sets Chord and Slot timeOffset to 7/64. `Voice.checkDuration()` produces:

| expectedDuration | quarter-rest end | termination | Voice/Stack excess |
| --- | --- | --- | --- |
| 3/4 | 23/64 | -25/64 | null |
| 1/4 | 23/64 | 7/64 | 7/64 |

Both controls produced 21 observation rows with no observer errors. The five native-comparable boundaries are Staff.removeNote return → Measure.removeInter return → Containment edgeRemoved → RestChord vertexRemoved → Rest vertexRemoved. At Staff return the Rest is not yet marked removed. At Measure return the Rest is removed but registered, the Chord is not yet marked removed, and the actual Voice is already removed from the Measure. At the Containment callback both Rest and Chord are marked removed and remain registered; the relation has already left the graph. Containment.removed entry/return states are identical and no Chord.invalidateCache callback occurs because the Chord is already removed.

At all 21 phases the actual Chord's cached bounds, manual flag, Measure, Voice, Slot, and 7/64 clock references remain intact. The retired actual Voice retains its number, Slot table and SlotVoice reference, termination, and excess. The actual Slot retains its clock, incoming Chord and Stack membership. The actual Staff/Part/Measure/Stack chain and expectedDuration object are retained; raw cached Rest/Chord Part fields remain null. The source readback therefore does not materialize a new Part cache to report ownership.

`RestRemovalRewriter.java` adds method-entry/ordinary-return observation calls to seven original classes. `VerifyRestOverlay.java` removes only those exact four-instruction calls for comparison and verifies original classfile headers plus every original instruction, branch, exception/frame table, method, field, and annotation. The audit passes with 22 inserted calls and zero other differences. Original major69 headers are restored before the original packaged JVM runs. The temporary major65 adjustment is solely for the existing JDK21 ASM parser; it is not an executed classfile change.

`run.py` records exact compiler, rewrite, audit, and two runtime commands; original Book/JAR SHA256 before and after; helper/compiler/runtime hashes; exit codes and elapsed times. `extract.py` checks every observed state and writes the bounded `summary.json`. Raw observations remain immutable historical output; the summary is a projection, not a replacement. No original observed values are used as native recognition input.

Qualification: these are explicitly declared in-memory original-method controls on reloaded owners, not fresh-image chronology, ordinary automatic LINKS nonzero-clock evidence, or proof that a manual Rest is selected by automatic weak purge. The source API is deliberately called with `extensive=true`. Native rollback remains an additional Rust transaction contract; Java rollback is not claimed. The native scalar uses its own geometry and `Measure.geometry=None`, so compare phases, relative membership, clock values, manual flags, and retained native Voice/Slot Arcs, not source/native IDs, absolute geometry, Part/Stack object coverage, or Java SlotVoice-object identity. The existing same-image Rest test separately covers native actual owner geometry with the original null clocks.

Reproduce without modifying the source Book:

```sh
python3 /tmp/rest-removal-clock-original-v1/run.py
python3 /tmp/rest-removal-clock-original-v1/extract.py
```

Preserve this evidence directory before rerunning; use a copied helper directory for a fresh historical observation.

## Saved native comparison

`compare_native.py` is a read-only saved-observation comparator. Its Python syntax check passed; no native/source rerun is required. Run it only after root confirms the native clock target passed, while preserving the enclosing batch result separately:

```sh
python3 /tmp/rest-removal-clock-original-v1/compare_native.py \
  --native /tmp/rest-owned-native-scalar-observed.json \
  --source /tmp/rest-removal-clock-original-v1/summary.json \
  --output /tmp/rest-removal-clock-original-v1/native-comparison-v2.json
```

The comparator requires exactly the declared underfill/excess controls and the five source-observed callback roles in order. It compares 29 common fields plus two explicitly qualified value/container mappings at prepared, five callbacks, and after. It separately validates the native saved boundary chain, same-Arc/value invariants, slot BEGIN role, and queued already-removed Chord no-op. Full source-only physical owner identities and callback entry hooks are not silently counted as native equivalence. The specific native target's pass and enclosing runner's status remain external evidence; merely creating its observation marker is insufficient.


The first comparator output (`native-comparison-v1-null-assumption.json`, with its saved comparator) rejected14 native-only assertions because the draft had assumed Graph raw bounds remained null. Actual prepared native observations already contained `[20,10,6,20]`, preserved throughout; all29 common fields and two mapped contracts already agreed. Version2 corrects only that observer/comparator assumption and compares native raw bounds to the actual prepared value. This is no source/native implementation or rerun change. The first failure is retained and must not be described as a recognition mismatch.


Saved comparison version2 passed: two controls,14 states including10 callback aftermath states,434 common/mapped field comparisons,588 checks including native-only preservation assertions, zero mismatches. The exact Rest clock test passed in `/tmp/rest-time-owners-v2/scalar.log` using binary `ffd9c2a6b4f7806c0965aa60fa5dbc502374d352d25a462133519d4ce2b96597`. Its enclosing batch reported16 passed/1 failed (exit101): the older music_dots assertion still expected a later refresh to remove a Chord that the new typed lifecycle already removed. Parent is validating that separate test-only correction and pending image targets; they are not claimed as passed here. See `NATIVE-COMPARISON-PROVENANCE.json` for exact log/marker/comparator hashes. No second native/source execution was used to produce either comparison.
