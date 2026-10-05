# Supplied clipped system ends

Native beam dispatch and multiple-rest mapping now consume supplied horizontal system-area ends for straight boundary paths. The area is the original north/south staff boundary band intersected with the supplied rectangle, retaining horizontal endpoint extensions, Java boundary containment, original closest/at-or-above selection and full-lag section rectangle intersections. This adds no neighbor-discovery heuristic.

The preserved original StaffManager computes clipped staff areas directly. The control probe intersects the original SystemManager area with java.awt.Area using supplied ends [250,1400], then calls original SystemInfo selection. Its real scanned Laura curves come from independently observed original GRID output. This tests the intersection and selection semantics, not native horizontal-neighbor discovery. Clipped curved boundary paths return an explicit unsupported error; invalid ends are rejected. Full-width curved areas retain their previous support.

Current native GRID still emits full-width ends. The native real-input CLI therefore verifies the full-width pipeline, while supplied-geometry controls exercise clipping. Horizontal staff-chain discovery and curved clipping remain unfinished. Positive real-score multiple-rest conversion is not demonstrated; full OMR remains unfinished. Full regression at this source is not yet run.
