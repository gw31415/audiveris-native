# Actual stale-predecessor Tuplet merge

Pinned Java 9e1e55cd2746037d059345881c53e6a6754bffbd controls invoke
private MeasureRhythm.mergeTuplets with the actual original TupletGenerator,
retained Voices/BeamGroups/old signs/manual relations and explicitly declared
cached geometry. Eight controls/16 frames/41 graph callbacks cover same, different
and NULL actual BeamGroups, two/three successive signs, overlapping groups,
explicit predecessor, reuse of a later sign and a late registration failure.

Native keeps each captured original current group and previous Tuplet reference
after generation; a removed predecessor is passed as that same retained object on
the next iteration. It removes both actual references before extension/generation
and does not replace a predecessor by a reused/generated sign or numeric ID.
Removed sign shape/bounds are observed from the retained object body, not the
public tombstone projection. Raw unassigned ownership is captured at callbacks.

Callback order is observed independently on actual native operations. Original
registration precedes SIG attachment and owner membership, removal callbacks hold
the old Chord/Voice/Slot/end/excess/expected/manual state. The source listener
failure leaves a partially registered sign after earlier removals; native outer
rollback separately restores actual old graph/cache/container bodies and storage,
physical Chords/Slots and manual relations, then retries to the source success.

Fixture observed.json is losslessly interned only from original Java output;
provenance records original JAR/book/observer/source hashes. There is no report
owner reconstruction, source-fed callback ordering or callback clock clearing.
VALIDATION.json records focused targets, fmt/clippy/build, retained63+correction8
XML byte/XSD regression and exact7/64 XML. No whole649/profile rerun.

This is an explicitly controlled ordinary single-staff owner boundary; actual
ordinary Stub/font/time signature/switch binding and whole rhythm admission are
still pending. Full/public nonUI completion flags remain false.
