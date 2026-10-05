# Native bounded curve containment

This independent library translates the numerical ray-crossing, Y-monotone
subdivision and quadratic/cubic inversion operations used by OpenJDK's Area
curves. It accepts a closed path of line/quadratic/cubic Bezier segments and
performs nonzero-winding point containment. It does not implement general Area
union/intersection or the rest of Java2D.

The Audiveris caller assembles the native single-column system boundary from
stored staff Bezier curves, with the original flat page extensions and reversed
lower path. This preserves exact integer points on rising/falling boundaries;
a simple `y >= north && y < south` test does not.

Original-source files are retained under upstream/. They were read from the
already-local pinned development JDK source archive, with no network retrieval.
Source/license hashes are in provenance.json. The library and translations
retain GPL version 2 only with the Classpath exception, including that exception
for this modified native library. Linked Audiveris modules retain their separate
AGPL-3.0-or-later terms. See LICENSE and NOTICE.

The parent integration independently compares 49,248 queries against original
NaturalSpline/GeoPath/Area operations from the pinned Audiveris runtime,
including line, half-pixel, quadratic extrema, cubic knots and non-monotone-x
cases. Full native image pipelines are checked separately.

Native bounds reject oversized/nonfinite paths and unconverged roots. Curves are
subdivided once; quadratic/cubic root operations preserve the original floating
operation order. Each cubic inversion has at most three 256-step refinements and
one 256-step fallback. Query-work estimates allow the caller to limit a batch.
The implementation does not invoke Java or a subprocess at runtime.

## fdlibm-compatible hypot

`hypot(x, y)` translates the already-local OpenJDK `FdLibm.Hypot` primitive,
including high-word splitting, scaling, nonfinite inputs, and operation order.
It is used for native BeamStructure fitting because platform `hypot` differences
can flip a strict split-height comparison even when the delta is one ULP.
5,321 direct original-runtime vectors cover all exponent ranges, subnormal and
overflow boundaries, signed inputs, infinities/NaNs, and beam-style coefficients.
Finite/infinite vector results require exact IEEE-754 bits; NaN results require
NaN classification. The source and license are preserved under `upstream/`.

## Line-only normalized subtraction

`LinearArea::polygon`, `subtract`, `contains` and `bounds` provide the bounded
line-only `AreaOp`/`Order1` operations needed for BeamItem core/belt masks. They
preserve Y-band edge ordering, equal-edge selection, curve-link/chain assembly,
link absorption and subcurve arithmetic. This is necessary because simply
combining outer/core containment loses original integer boundary pixels.
It does not add curved CAG or general Java2D rendering. Inputs are bounded to
128 edges; sweep iterations/links are each capped at 1,024, before the original
large-operation intermediate-consumption branch. Oversized or
nonfinite input and inconsistent topology fail closed.

All eight cached source classes (AreaOp, Order1, Edge, CurveLink, ChainEnd,
Area, Rectangle2D, RectangularShape) have
identical `javap -c -p` output between the cached JDK21 and official runtime25.
The root BeamItem controls compare original normalized core/belt membership,
including 307 independent horizontal, sloped, subpixel, noisy, sheet-edge,
threshold and degenerate cases. No test fixture or original process is accessed
by the native implementation.

The cached bounding rectangle also preserves original contour-order move-to
and top/bottom endpoint accumulation. Each update rounds origin + extent again;
using direct extrema can change integer sampling even for BeamItem
parallelograms. Integer bounds and the containment precheck share this rectangle.
The independent item review adds 576 actual parallelogram cases and 424,201
integer point checks, including ULP-neighbor coordinates and core-ratio controls.
The full generic polygon/Java2D API is not claimed as a validated feature.
