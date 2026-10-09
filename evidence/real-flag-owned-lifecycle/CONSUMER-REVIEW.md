# Independent Flag caller and consumer review

Read-only review of the post-Rest checkout on 2026-10-09. No cargo/native image,
new original image, Book save or checkout change was performed for this review.

## Actual production callers

`music_flags::contextualize` is called by `music_links` before Flag overlap/exclusion
reduction. It originally removed weak Flags through `Graph::remove_music_inter`.
`music_flags::competition::reduce_exclusions` independently removed the chosen
Flag through that same generic API. These are the two current ordinary Flag-removal
callers. The Rest/Time reducer selects only RestInter/TimeWholeInter; BasicTimeColumn
only removes Time candidates and explicitly refuses a weaker neighbor callback.
It does not introduce a third Flag-removal caller.

The existing Flag factory must change together with those two callers. Merely
adding a registry/helper would leave source `Flag.remove` behavior skipped. Protect
registered Flags from generic bypass as current registered Rests are protected.
Both ordinary caller wrappers require typed added-boundary ownership; they do not
fall back on missing registration. Low-level unregistered scalar Graph controls
retain their separate generic path.

`reduce_exclusions` already stages the full Graph. `contextualize` must stage its
whole initial-context / frozen weak selection / removals / recontext sequence,
without moving any of those boundaries. A failure on a later Flag must restore the
earlier removal, all legitimate Flag/Head cached-Part mutations and contextual
values. This is native error atomicity, not an observed Java rollback claim.

## Source contract

Pinned original bytecode is already saved in
`/tmp/common-symbol-reducer-unit/simple-Inter-lifecycle.javap`:

- AbstractFlagInter.createValidAdded, lines488–529: construct raw-null Flag,
  lookupLink, addVertex, then Link.applyTo. No setStaff.
- FlagInter.added, lines595–615: nullable System.getStackAt(center),
  Stack.addInter, then abnormal=true.
- FlagInter.getPart, lines638–693: cached Part if present; otherwise iterate
  FlagStem and then HeadStem in actual relation encounter order, returning the
  **first Head.getPart result even when null**. Fallback is AbstractInter.getPart,
  separate from Flag.getStaff.
- FlagInter.remove, lines752–773: nullable fresh Stack lookup, Stack.removeInter,
  then superclass graph lifecycle.

`MeasureStack.java:244` calls inter.getPart before adding to its actual Measure;
`:1744` does the same before removal. The existing retained Part/Stack/Measure
owner graph supplies these objects. Fresh flags are added before their Stem edge,
so null Part means no positive Measure.flags membership is invented. C/K Filler
must not fill Flags later.

`typed-removal-relations.javap:13` observes FlagStem.added checking Flag abnormal
state; `:109` checks isRemoved before recalculating on removal. A dying Flag does
not recompute this flag, clear Chord caches or erase clocks. Edge removal order is
current Graph.incident_edges: incoming then outgoing, each live insertion order.

## Raw-null Staff compatibility

Current musical consumers do not require Flag raw Staff (and
HeadInter/AbstractNoteInter do not override inherited AbstractInter.getPart):

- `music_duration.rs:151` derives flags from actual ChordStem→FlagStem edges and
  Flag.value; it does not use Flag.staff_id.
- `music_xml.rs:606` checks surviving created Flag/Stem identity. The writer at
  `:1229` follows actual Chord Stem to flags and records their IDs.
- `music_time_late.rs:52` reads retained Flag glyph pixels for a Time-neighbor
  overlap; its Staff lookup applies to Time candidates, not Flags.
- Existing duration/container source comparisons use Flag shape/bounds/system,
  actual edges and resulting duration, so no source-equality contract requires
  preserving the current eager closest-Staff assignment.

Retain raw Flag Staff null through these paths. No new lazy Staff call, report
ownership reconstruction or writer filter is needed.

## Evidence limits and integration checks

`flag-saved-observations.json` extracts eighteen actual System2 source Flags with
raw Staff null at the saved before/context barriers, one later weak Flag vertex
callback, and four serialized Measure flags collections. It records no raw Part or
callback Measure flags values. Getter/cache order is source-method evidence;
new native scalar controls must not be represented as an original-runtime replay.

Use existing representative image targets to observe raw null, retained identities
and the actual family-local callback subsequence after adoption. The source common
mixed first-purge timing is still a separate pending dependency. Native tests must
exercise caller behavior (including a late second-Flag failure), not only helpers.
No full common reducer, editor lifecycle or extra image rerun is required to make
this bounded caller integration concrete.

## Coexisting owner proposal

The independent positive-Clef v2 packet also changes `clef.rs`/`shared_sig.rs`.
Merge focused Flag membership/module hunks with its Clef changes; do not replace a
whole file from an older base. The Flag unit does not change C/K Filler or its
completion certificate, and positive Clef admission still needs the common scheduler.
