# Original Java candidate-math observation

Pinned original source/JAR: `9e1e55cd2746037d059345881c53e6a6754bffbd`; provenance and exact argv/hashes are in `provenance.json`. `observed.json` was written by `/tmp/NativeSlurCandidateMathProbe.java`, not by the Python exploration. Input `sequences.tsv` contains all153 unchanged actual native candidate point sequences plus4controls. Existing original OMR was opened for actual Sheet Scale (11); no recognition step was invoked.

Original `Circle` constructors, `SlursBuilder.computeModel/computeImpacts`, and `SlurInfo.getCurve` executed. A detached SlursBuilder was allocated without its constructor, then initialized with its original private Parameters(actualSheetScale); this avoids Curves construction, which would build a Skeleton. Only these params-dependent math methods were called. This is explicitly not an ordinary recognition/lifecycle observation.

`comparison.json` records equality of every input point sequence and all153 final math decisions:14accept,139reject, zero disagreements. Accepted candidates retain the original shared-global left/right side-model object (actual66-point threshold). Original minGrade is0.08000000000000002.

Compared with NumPy exploration, largest accepted absolute differences: center7.63e-6px; radius3.14e-4px; Bezier3.27e-4px; residual4.80e-6px; grade6.11e-7. The stated comparison bounds are1e-3px geometry,1e-5px residual,1e-6grade and1e-5radians/impacts. These are measured approximate numeric agreement, not bit equality.

**Intermediate difference:** for10candidate sequences original JAMA returns CircleModel while NumPy/exploration rejects earlier; all10 ultimately reject. One original selected radius is NaN. Final decision equality does not establish NULL/intermediate numeric parity; do not use Python early finite rejection as a source-faithful implementation. Original raw nullable/NaN states are retained in observed.json (nonfinite doubles encoded as strings).

For102finite fitted observations, actual constructor distance and public distance exhibit the pinned sqrt(sum)/N versus sqrt(sum/N) difference. Horizontal, diagonal and duplicate controls produce original singular-matrix exceptions in fitted construction and NULL model; the shallow single-pixel bump passes geometry/grade, demonstrating why fit alone cannot recognize a slur.

Still required: correct BINARY run support/true-gap validation, actual Head/Chord polygon linkage, competing alternatives, tie/orphan behavior, export notation and full musical-quality comparison. No image pipeline rerun, repository changes, Rust compile or Git operation was used for this observation.
