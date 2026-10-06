# Original stem-bound HeadChord geometry

Original5.11.0 source `9e1e55cd2746037d059345881c53e6a6754bffbd`.
`FoundationStemChordProbe.java` declares actual Containment/ChordStem relations
on an existing saved System/Staff. Four conditions observe up/down/tied-distance
leading note, bottom-up member order, integer locations and head-union + tail-point
bounds (Rectangle.add(Point) does not add a pixel to max coordinates).

Native shares this original unit between actual MEASURES/CHORDS creation and
head-chord overlap ownership. First typed ChordStem wins; serialized bounds or
stem_id cannot substitute for live edges. Existing image/cache/CLI regressions
are separate from these declared geometry comparisons.

The empty-member original chord has stem bounds but location getters throw
NullPointerException without a leading note. That failure is retained in the
observer boundary and is not invoked again; native exposes no fabricated note
location. Empty input has no chord bounds. Complete ensemble grade/ownership
callbacks and special/shared-head page recognition remain pending, not omissions.
