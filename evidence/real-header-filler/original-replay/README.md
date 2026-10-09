# Original declared MeasureFiller replay evidence

Original Audiveris pin: `9e1e55cd2746037d059345881c53e6a6754bffbd`.

One raw replay observation is stored at `fixtures/official-measure-filler-headers/declared-first-add-observed.json` (216,183 bytes, SHA256 `94f58e2bafcf069b024cf7635b54a687aec8e5c7136ceba360d769719cd71a63`). The raw observation is not duplicated in this evidence directory. Source Book, Jar, classes, private runtime caches and preferences are excluded.

The existing original saved AnLaura SYMBOLS Book was loaded read-only. Loading runs `Measure.afterReload`, which already populates selected header Clef/Key memberships. The observer declares removal of only those actual selected-header references from the retained collection objects, preserving the nonheader SYMBOLS Clef. It performs original `SIGraph.inters(FILLING_CLASSES)`, then, for each actual Stack, completes original `Stack.filter` before the sequential original `Inter.getPart`, `Stack.getMeasureAt` and `Measure.addInter` calls. All actual peer Measure lists and collection identities are recorded at each before/after add. A final original `MeasureFiller.process()` call proves idempotence of that completed sequence.

This is a declared actual-owner replay after reload, not fresh first-fill recognition or internal callback instrumentation. Source Systems have zero Voices and Slots. An existing actual HeadChord time `7/64` and its shared Stack expected `1/2` are declared controls; original references and all original memberships/raw Part caches are restored exactly. Every source add preserves the held graph/manual identities, recorded Chord/owner caches and declared clocks. These controls do not establish nonempty Voice/Slot behavior or same-input native clock parity.

Both Systems succeeded: 6 and 7 adds. Twelve selected-header adds are compared in the B prerequisite. Source nonheader Clef 4226 remains observed and retained; its factory/common reduction is outside this header subset. Source JVM IDs are run-local comparisons, never native recognition or ownership inputs.

The initial v1 failed before the first declared reset: reflection requested `contextualGrade`, but the original field is `ctxGrade`. Its compact observed/error/command records are retained under `failed-v1/`. It produced no successful System replay. The frozen v1 Java source remains at `/tmp/measure-filler-first-add-replay-v1/MeasureFillerFirstAddReplayProbe.java`; its SHA is listed in COPY-PLAN. Successful v2 corrects the field. Both runs' Book/Jar before/after hashes are unchanged in `unchanged-source-hashes.json`.

`RUN-COMMAND.json` records the actual successful command, which used the previously compiled reflection superclass from `/tmp/measure-filler-clef-key-book-v1`. The exact Java superclass source used is included in `scripts/java`. Its compiled dependency SHA is recorded in COPY-PLAN but the class binary is excluded. `scripts/replay_header_filler.py` compiles these sources into a new isolated output directory and accepts the existing Book path; it has not been executed, avoiding a duplicate replay. It performs no image recognition, reachStep, Book save or Jar write.

The comparator `scripts/compare_header_filler_replay.py` requires actual all-peer native observations; it never reconstructs peer state from target-only reports. Source-only equality against the prior source fixture and declared comparator mutation checks passed. Native comparison is pending the existing required image target and will be recorded separately after completion. No whole-page recognition or MusicXML quality acceptance is implied.

From the checkout root, once that existing target has completed:

```
python3 scripts/compare_header_filler_replay.py \
  --source fixtures/official-measure-filler-headers/declared-first-add-observed.json \
  --native /tmp/real-header-filler-native-control.json \
  --links /tmp/real-header-filler-native-LINKS.json \
  --output /tmp/real-header-filler-original-replay-comparison.json
```
