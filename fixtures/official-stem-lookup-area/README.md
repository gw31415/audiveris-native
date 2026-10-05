# Head lookup-area controls

Two independent original Audiveris 5.11 compilations/runs invoke private
`HeadLinker.SLinker.CLinker.buildLuArea` on 600 legitimate first-system Laura
HEAD builders after `StemsRetriever.process`. Each builder has five controls:
its part edge, a declared sloped limit, and below/equal/above reference-y ULP
limits. Original results are byte-identical. Native vertices/adjacent edges
and theoretical lines match exactly, normalizing ring start and winding only.
The 600 empty-area cases retain the original theoretical-line arithmetic.

Rust generates the input path from reference points, directions, pixel widths,
slope margin and declared part/line limits. It preserves the distinct original
LineUtil arithmetic for limit y and theoretical targets. Invalid/skewed and
self-crossing inputs are explicitly refused without changing inputs.

These are controlled geometry comparisons on the existing Laura image, not
independent scanned/special-staff page validation. Reference-point discovery,
target-beam selection, beam-side geometry, chunk registration, item expansion,
cache/head/SIG composition and epilog remain incomplete. HEADS/STEMS flags
remain false. The existing explicit-input collector CLI is unchanged.
