# Late hooks after initial-snapshot extension

Native `beam_late_hooks::build_hooks` implements the pinned original buildHooks/browseHooks/checkHookGlyph boundary. Normal beams use a fixed snapshot, TOP before BOTTOM. Parameter selection uses parent height, with standard winning ties; small parameters skip hook discovery. The fixed unassigned sorted spot pool retains ownership and order throughout this stage.

Glyph width and horizontal mean thickness gates precede the proposed hook median. Its slope and height come from the parent; centroid projection uses glyph bounds x through x+width and the original +0.5 vertical offset. Overlap checks intersected active inter bounds, then center containment in each existing beam/hook area. Original hook impacts and grade acceptance precede append-only registration and originating-spot assignment. No parent is removed and newly accepted spots are not pruned from the current pool.

66 independent uninterrupted original-JAR controls match, including 26 accepted hooks, width/thickness limits, overlap, spot order, small parameters and distance impact. The complete native pipeline then matches late-hook additions on all 13 original images (3 hooks added). Five focused Rust tests across three targets, fmt and all-targets Clippy pass. Previous full 267 debug/release validation remains a prior-source baseline; full suite at this new tree is unverified.

This boundary stops before beam groups, multiple rests and epilog. CLI wiring and full OMR remain unfinished. Recovery uses the immutable preceding snapshot-loop source checkpoint plus a small exact text patch and Git commit preimage; no new binary archive is required.
