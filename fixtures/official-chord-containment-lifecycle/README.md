# Actual original chord identity callbacks

Original5.11.0 source `9e1e55cd2746037d059345881c53e6a6754bffbd`.
One registered chord receives two members, a stem, then loses a member, the stem
and its last member. Six ordered observations compare actual members/bounds/
leading-note/locations/intrinsic mean grade/contextual grade. Native uses real
typed edges; no member metadata can substitute for them. Mean member best grade
uses context when present, otherwise intrinsic. setGrade does not reset context.

Removal here deletes relations, not heads/stems or physical glyphs. A last-member
relation removal leaves a live empty chord with null grade/bounds/locations, as
original; automatic extensive vertex removal/dying ensemble and full Part/stack/
lyrics ownership cleanup are separate pending units. Existing image MusicMeasures
now invokes the addition callbacks; native uncached music vertex cleanup invokes
incident identity removal callbacks. Actual image success and declared operation
comparisons are recorded separately. No public caller-selected OMR API is added.

The initial observer mishandled the null Rectangle after the last member. Its
serializer was corrected; that failure is not attributed to the original engine.
