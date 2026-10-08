# Actual sloping Beam/Hook cleaner contours

Original PageCleaner fills the horizontal parallelogram, then draws its closed
miter contour at margin1.1 on the actual runtime. NearestPixelQuarter normalizes
each stroked vertex; filling retains the raw area. Native builds the offset
convex contour and samples the union of original fill and normalized stroke at
pixel centers. It keeps miter-limit, collapsed/nonconvex/disjoint-row and resource
bounds explicit. This is not an inflated bounding-box erasure of sloping ink.

Fresh unchanged image CHORDS observes18 Beam/Hook bodies:14 genuinely sloping
pixel contours and4 normalized rectangles. Source fixtures only observe results
and are not native inputs. Eight separate declared controls exercise positive/
negative slopes, normalized boundary values, a short hook-sized ribbon and two
previous guarded geometry cases. The actual owner target compares native median/
height and every source pixel row without mutating graph or physical ownership.

Existing rectangle and Hook observations stay regression contracts. Wider strokes
and collapsed or bevel-limited contours remain separate dependencies if an actual
input requires them. Full native image XML and independent quality acceptance are
still pending; these raster observations alone do not prove musical quality.
