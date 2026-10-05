# Independent BEAMS prolog geometry review

`BeamAreaBoundaryProbe.java` runs the locally cached original Audiveris
`NaturalSpline`, `GeoPath`, `ReversePathIterator` and bundled Java `Area`.
The observer assembles single-column system areas according to the pinned
`StaffManager.getGlobalLine` and `SystemManager.computeSystemArea` bytecode.
It does not use native results or expected native coordinates.

The resulting 49,248 integer-centroid queries cover two systems with a four-pixel
vertical margin and eight boundary shapes: ascending, descending, horizontal,
half-pixel, quadratic maximum/minimum, oscillating cubic and nonmonotone-x cubic.
Each query records the original area membership of both systems. System x bounds
are inclusive from 10 through 91; control queries range from 10 through 90.

The initial independent review found 100 mismatches in the native y-range
approximation, including exact sloping boundary points and extrema. Java `Area`
uses horizontal-ray crossings: a blanket north-inclusive/south-exclusive y test
cannot reproduce those contacts. The retained controls form the regression
boundary for its replacement. The native Y-monotone crossing replacement now
passes all 49,248 retained queries, eliminating all 100 initial discrepancies.
This finding is distinct from the 13 ordinary raster fixtures, which already
matched at the native prolog boundary.

`provenance.json` records JAR, bundled runtime, compiler, source and output hashes,
plus the exact local compile/run commands. The local JAR is the pinned Audiveris
5.11.0 build at `9e1e55cd2746037d059345881c53e6a6754bffbd`.
The commands require existing local dependencies and perform no network access.

Run the retained controls and public malformed-input/scope checks with:

```sh
cargo test --locked --offline --test beam_prolog_review
```
