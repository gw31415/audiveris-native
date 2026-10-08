# Actual Tuplet lazy getter owners

Compare pinned original9e1e55cd2746037d059345881c53e6a6754bffbd on two retained
short/excess controls, three actual implicit Tuplets and ten boundaries each.
DurationFactor retains its actual numerator/denominator and same cached object:
SIX is4/6, although its rational arithmetic reduces to2/3. The first non-null base
and Staff are retained; an initially null base can warm when links are added.
A later faster Chord does not replace a cached base. AbstractInter.setStaff(null)
leaves its cached Part intact. Real Note/Chord Staff changes do not relocate the
real Chord from its Measure. Tuplet Staff holds actual owner Staff/Curve Arcs.

Voice follows the first incoming actual relation in encounter order, including
null. It is independent of the sorted left-bounds getChords sequence. Removing
that implicit relation retains its object/manual body and cached factor/base/
Staff/Part while the next actual incoming source becomes first. Genuine Measure
reset leaves old held Voice/Slot/end/excess references available; construction of
one fresh Voice reuses musical number1 with a distinct object. A later source's
fresh Voice never substitutes for a first source's null Voice.

Native staged getter reads retain cache objects and actual physical Chords;
late second Stem bounds failure after earlier getters restores cold caches and
Graph, actual Chord/Slot storage. Warm cache/Staff references remain the same after
an owner mismatch and repair. No report is used to rebuild physical ownership.
Part remains the bound real-owner numeric projection; this does not claim a new
complete native Part object model. Ordinary mutable/null Staff admission is still
guarded: the test frame reads held actual BeamGroups to observe the legitimate
null Staff boundary, without calling the ordinary rest-processing Staff guard.

Shape variation is a declared PRIVATE FIELD test control: original Tuplet has no
setShape API. No public shape-change workflow is invented. Original initial
observer attempt incorrectly called that absent setter; raw attempt is preserved.
The corrected observer uses explicit field reflection and explicit null varargs.
Factor/base/Staff getter identity is observed from actual original references,
then tested through native retained Arcs. Fixture55,811 bytes losslessly interns
10 frames and original relation sets; whole inflation equals raw original data.
Native outputs do not supply fixture data. Compile failures/private-interface
warnings were corrected; the initial common test observer hit the ordinary null
Staff guard and was corrected to read its already-held actual groups.

Current raw-base kernel admits retained ordinary quarter/half note/rest shapes,
flags/beams and supported dot counts; whole/special/mirrored base duration and
complete raw sans-dot API remain explicit following dependencies, not silently
approximated. Fresh allocation callbacks, inspect/force/merge and capture-old/
create-new/remove-old second pass remain next. Ordinary fresh Tuplet MusicXML and
full/public nonUI completion flags remain false. See VALIDATION.json for checks.

Primary source: [TupletInter](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sig/inter/TupletInter.java),
[DurationFactor](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sheet/DurationFactor.java),
[Inters.byAbscissa](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sig/inter/Inters.java).
