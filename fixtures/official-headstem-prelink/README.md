# Original preLink task lifecycle controls

Pinned original: `9e1e55cd2746037d059345881c53e6a6754bffbd` (5.11.0).
`HeadStemPreLinkProbe` calls the unmodified `HeadStemRelation.preLink` on a real
saved, timed double-augmentation owner. It observes each original raw task and
SigListener entry/return, without resetting clocks before callbacks. The final
manual LinkTask is explicitly appended; InterController conflicting-relation
discovery, epilog, PAGE and persistence are separate dependencies.

The existing-target case has twelve task executions and thirty callback records.
It moves actual Containment, removes the now-empty old Chord, and restores the
same original relation instances on undo. Removed Chords retain their Part,
Measure, Voice, Slot and timeOffset references. Measure membership disappears;
Voice's Chord list loses the old Chord and undo does not restore that list.
`SIGraph.addVertex` fires both listener boundaries before `Inter.added` clears
the removed flag and appends the Chord to its Measure.

Native starts with its real recognized image owner. Its saved-owner comparison
explicitly rounds grade/context to original saved three-decimal precision and
declares original Stem `setBounds` values on the same canonical cached Stems.
Their glyphs, medians, registry/SIG identities and actual owners are retained.
This declaration is needed because the current native source Stem bounds extend
five pixels into the head beyond original saved bounds. Full-image stem extent
parity is pending; this control does not certify it or enable public CLI input.

The new-target case is original evidence for the next AdditionTask dependency.
It warms the retained target Part/Staff getters, then removes only the target
Chord as declared setup, retaining its real stem and head. Existing-target
observations are unchanged from the first source checkpoint. Its initial empty new Chord has no Part or Measure; Containment later
places it. Redo can reuse Part/Measure references cached during its first run.
Native now compares eighteen task executions and forty-two listener records
for this branch, with real typed editing HeadChord placement at each task
boundary. General Part/Measure/Voice integration, ordinary reprocessing and
canonical mirror sharing remain guarded. The following callback-owner checkpoint also observes the new typed editing
HeadChord members/stem/bounds, cached Measure and membership inside all forty-two
listener records. General old-Chord/Part/Voice callback membership and ordinary
reprocessing remain pending.

Reproduce with a saved original double-augmentation book:

```sh
python3 scripts/run_headstem_prelink_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/double-augmentation.omr \
  --output /tmp/new-prelink-original-observation
```

The checked-in observation was generated in this Cloud from a fresh original
PAGE/save run of the committed PNG. It is new evidence, not a replacement or
recreation of the unpushed `ce785c9` validation from the previous task.


The third original control declares a single-Chord Voice using the real Voice
constructor and retained original SlotVoice entry; its Slot/time/expected clocks
are unchanged. Removing that Chord deletes the empty Voice from Measure.voices,
but the same Chord still references the detached Voice and its slot information.
Undo does not restore the Voice to the Measure list. Native retains the moved
actual Voice in the Graph and binds cached Chords to a stable retained-owner key;
no report-based recreation is used. Twelve task boundaries and thirty listener
records are compared, including the detached Voice after deletion. General Voice
reassignment and musical-id reuse/history across Part/Measure owners remain pending.
The first two cases are unchanged from their earlier saved observations.


The fourth original control constructs a new live Voice2 after the old solo Voice2
has retired and the old Chord has been restored by undo. Actual setVoice(new2)
rebinds that Chord even though its musical number stays2. Old2 keeps slot1; new2
keeps slot2 and gains the Chord in its member list. Subsequent redo preserves new2
with its remaining Chord. Native reproduced the number-only no-op before the fix,
then matches original explicit rebind/task states while retaining old2 separately.
General purge/rename/history identity paths and full owner reprocessing remain
pending. The preceding three original controls are unchanged.
