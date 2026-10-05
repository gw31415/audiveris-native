# Bounded extendToBeam checkpoint

The native method now samples the original middle area when the signed gap is
at least Math.rint(0.2 * interline), rejecting a black ratio below 0.6 or NaN.
AreaMask's denominator includes the whole area; foreground pixels are clipped
to the image. Short gaps and overlaps skip this gate. Alignment eligibility
remains in the existing side lookup; extendToBeam adds no slope predicate.

Accepted candidates use the existing mergeOf and register/append/remove sequence.
Rejected gates and merge evaluations preserve candidate state, exclusions and
registry. Native bounded-domain errors remain explicit. VIP diagnostic flags are
not modeled. The raster is supplied by the caller as the original pixelFilter;
this boundary is not wired into whole-page CLI orchestration.

Independent original controls execute the pinned unmodified Audiveris 5.11.0
JAR on a real Book/Sheet/Grid/SIG: 61 controls, 26 accepted merges. They cover
gap threshold neighbors, 23/40, 24/40 and 25/40 occupancy, swapped input order,
sloped and mismatched medians, rejection state, appended order, removals, and
shared glyph identity. Four gate-only cases cover empty and clipped areas.
Inputs are not generated from Rust outputs. Both observers register supplied
beams on a black seed raster before switching to the independent test raster.

20 focused debug tests, fmt check, and workspace/all-targets clippy pass.
Full debug/release workspace tests and release build were not repeated for this
change. The source base is 8ebbf976f568f91077d32221d52366d71559ab75;
see evidence/beam-extend-to-beam-checkpoint/validation.json and provenance.json.
Full extension orchestration, late hooks/groups, and downstream OMR are unfinished.
