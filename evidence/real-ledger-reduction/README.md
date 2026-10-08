# Actual ordinary multi-Staff ledger pruning

Pinned source9e1e55cd SigReducer.checkLedgers calls shared-ledger correction first,
then captures live system Heads once and visits actual Staff order, sorted map
indices and member order repeatedly. Its support predicate uses the containing
map index, rather than Ledger.getIndex(), and the next line's actual references,
including removed bodies. An overlapping head may belong to another Staff;
rectangle boundary contacts do not count. Original Ledger.remove detaches only
its current Staff's first matching reference, so duplicate retired references may
remain in other lists.

The native image path now applies that traversal to constructor-retained maps
and canonical graph bodies, retaining actual Staff/Curve owners. A genuinely
shared upper+1/lower-most-negative object still reports a specific reassignment
boundary; the full An Laura HEADS state has no such shared objects. Special Staff
geometry remains explicit. Late predicate/removal failure commits neither the
staged graph nor map changes. Report projections are observations, not ownership
sources.

The declared controls load the retained original book, not native recognition
output. A separate original image RunClass observes fresh HEADS objects directly
before serialization. Four declared SIG controls isolate orphan chains with a
repeated reference, a different body/map index plus another Staff's Head, a mere
boundary contact, and a removed next-line reference. The fresh image observer separately invokes the
private original reduction on the unmodified full image HEADS state:26 bodies,
six Staffs, zero shared cases and10 orphan retirements. That comparison boundary
is before STEMS; production foundation reduction occurs in its existing original
consistency order. It does not claim all later pipeline states match.

fixtures/official-ledger-reduction contains the original observations and hash
provenance. VALIDATION records focused targets, existing63+correction8 and exact
7/64 XML byte/XSD checks, plus the full native-image execution/frontier. Quality
acceptance remains false until image-origin MusicXML and independent semantic
comparison meet docs/IMAGE-MUSICXML-ACCEPTANCE.md.

The preliminary reloaded-Book diagnostic in observed.json has null transient
Ledger body indices after original deserialization. It is not the fresh image
comparison. image-observed.json/image-provenance.json retain the direct image
observations, including original non-null indices before and after retirement.
Initial comparison and missing-fixture compile failures are retained in /tmp;
only final successful checks count as validation.
