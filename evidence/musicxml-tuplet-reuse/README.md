# Actual Tuplet reuse and group extension

The existing-object phase of pinned TupletGenerator.generateOneTuplet walks the
supplied actual Chords, then each Chord's incoming-first/live encounter relations,
and immediately returns the first actual opposite Tuplet. Explicit/manual and
Stack-only signs are eligible. Later null members are not read after a winner;
a null before a winner is an original NullPointerException and a native actual
owner error. The phase's None means allocation is needed, not the return value
of the original complete generateOneTuplet; fresh creation remains next.

Group extension follows each actual cached first beam's actual group, extends
through its real HeadChord/RestChord members, deduplicates only beam members and
stably sorts actual integer centers. Supplied output entries and lone duplicates
remain. The first different actual group object is the only split pivot; the
second group can contain more than one beam group without recursive splitting.
A selected group getter reads only its incident Chords, avoiding unrelated bounds
reads. No report constructs Chord/Measure/Voice/Slot owners and no relation or
Tuplet is allocated in these reuse comparisons.

Two pinned short/excess controls observe22 scenarios: supplied/incident encounter,
manual explicit/implicit objects, actual Stack-only reuse, early return/null,
actual beam extension, initial/lone duplicates and first-pivot split. Original
GraphListener records zero graph changes in every scenario; retained Voice/Slot,
ending/excess and Chord duration/end remain unchanged. Native late actual physical
Chord mismatch after earlier beam extension rolls back staged getters and retains
real owners for same-owner retry. Existing automatic Stem5px discrepancy remains
separate; these controls use actual automatic geometry without declared coercion.

Raw original output is preserved separately and fixtures losslessly intern only
two state frames. Source hashes pin TupletGenerator/TupletInter/ChordTupletRelation
at9e1e55cd2746037d059345881c53e6a6754bffbd. No Java executes in native runtime,
new archive/full649/complete profile/newauth/other environment is used. The
ordinary fresh Tuplet/XML admission and all full/public completion remain false.
See VALIDATION.json for focused targets and existing byte/XSD regressions.
