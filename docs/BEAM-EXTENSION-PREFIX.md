# Bounded extension orchestration prefix

side_prefix performs the original default-reach side lookup, attempts a direct
merge only on LEFT, and returns NeedDownstream when stem/spot processing is next.
For a found side beam, maximum extension uses Math.rint(max(0, signed gap -
Math.rint(0.25 * interline))). RIGHT never calls extendToBeam. The parallel flag
only indicates when the original loop would attempt parallel after stem and spot
both reject; it does not implement their predicates or report recognition success.

initial_merge_prefix captures initial active registration/SIG order once, begins
with LEFT, skips snapshot beams removed by an earlier merge, and excludes appended
merged beams from new iterations. It stops at the first pending stem branch and
returns that exact frontier. An entirely merge-resolved snapshot may finish even
though appended merged beams remain active. It is not a full extension loop and
cannot skip downstream branches to reach RIGHT. Callers can inspect RIGHT with
side_prefix as a separately controlled boundary.

37 independently supplied controls invoke original HorizontalSide.values,
SIG.inters, getSideBeam and extendToBeam on a real Book/Sheet/Grid/SIG. The slice
observer follows retained prefix bytecode and stops before unimplemented methods;
it does not execute or prove equivalence of the uninterrupted whole extendBeams
method. Controls match 16 accepted merges, LEFT/RIGHT decisions, pending maximum
extension rounding, raw order/ties, removed IDs, glyph identity, fixed snapshot
iteration and end conditions. Oracle inputs never read native outputs.

22 focused debug tests, fmt and workspace/all-targets clippy pass. Full workspace
and release checks were not rerun. Source base remains 8ebbf976. See
 evidence/beam-extension-prefix-checkpoint/validation.json and provenance.json.
Next boundaries are original-controlled extendToPoint/isSideInSystem, stem/spot
and parallel branches, followed by an uninterrupted original loop observer.
No CLI whole-page extension orchestration, late hooks or beam groups were added.
