# Actual raw Chord duration before Tuplet base caching

Pinned original9e1e55cd2746037d059345881c53e6a6754bffbd provides560 raw getter rows
on retained actual timed Chords: all34 Shape.getNoteDuration aliases plus G_CLEF
null, dots0/1/2/9, with/without a real beam, short/excess clocks. Raw duration uses
the stable bottom-up first actual note, ignores dots/Tuplets and the measure-rest
wrapper, takes rest shape duration directly, and divides non-rest duration for
each actual Beam plus live flag value. LONG4, BREVE2, WHOLE1 and finer rest values
are valid raw getter results without widening ordinary image/XML admission.

Ten additional original boundaries hold the same Chord/Voice/Slot and actual
manual Mirror/Flag relations. Only exact NOTEHEAD_VOID with a mirror and positive
Beam/Flag count promotes to black; SMALL_VOID remains half. Real Beam+flag2+flag1
are summed, producing raw1/32 and1/64 under ignored dots9. Native holds the actual
relation objects/manual bodies through all additions. Test-only dependency restore
uses its held actual Graph: it does not claim Flag/Mirror removal callback or task
admission. Source cleanup is separately recorded in the original fixture.

Four original fresh base observations show dots9 ignored before first getBase
and the same cached base reference after dots restored. Six additional controls
show source partial caching: known then failing raw leaves1/4; known then null
leaves1/8 before compare throws; null then known returns1/8. Retry returns the same
cached reference without re-reading the bad source. Native primitive preserves
these partial states; the surrounding retained owner transaction restores cold
cache/Graph/real Chord/Slot state on failure. These are distinct contracts; the
original primitive is not described as rolling back its cache.

Inherited getPart reads the EXISTING Staff field, without triggering virtual
getStaff. Even after links/base reads, direct getPart is null until explicit lazy
getStaff finds actual owner Staff; then Part1 remains cached after Staff is cleared.
This corrects the unobserved standalone Part path before fresh Stack.addInter.
The previous combined getter compared Staff before Part, and its20 observed
boundaries still pass. Native Part remains the real-bound owner numeric projection,
not a claim of a new full Part object model. Actual Staff/Curve Arcs are retained.

Three native targets compare these source contracts, cached Arc identity, real
physical Chords and Slot storage, old Voice/end/excess, manual relations and held
report. No owner is rebuilt from a report. Shape/dot variation is an explicit
private-field kernel control; original has no public setShape workflow. Source
fixture frames/relations/rows are losslessly interned and whole-inflation verified:
51,717+12,065+13,109 bytes. Native outputs do not supply fixture content.

The initial raw observer used a fully-qualified name with a helper that already
prefixes the package; correcting to the relative name fixed it. Native initial
test used an absent Voice getter; it now reads the actual owner Voice identity.
Raw attempts remain separate. See VALIDATION.json for final target/regression
checks. Ordinary dot-coupled/special/mirrored getDuration/export, recognition and
Flag/Mirror callbacks remain their own dependencies. Fresh Tuplet allocation,
force/inspect/merge/second pass and full nonUI completion remain unfinished.

Primary source: [Shape.getNoteDuration](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/glyph/Shape.java),
[AbstractChordInter](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sig/inter/AbstractChordInter.java),
[TupletInter](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sig/inter/TupletInter.java),
[AbstractInter.getPart](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sig/inter/AbstractInter.java).
