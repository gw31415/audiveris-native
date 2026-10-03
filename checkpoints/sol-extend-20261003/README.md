# Bounded native parallel beam checkpoint

Latest: original parallel beam scan, pre-group geometric constraints, system containment, side-obstruction gate, selected point replacement and one-side merge/stem/spot/parallel integration. Whole initial snapshot loop and CLI remain unfinished.

104 independent original controls, 28 accepted extensions, two rejected-point no-fallback controls and two stem/spot/parallel scheduling controls. 38 focused tests, fmt and workspace all-targets Clippy passed. Earlier full 259 debug/259 release validation at 549d409 is historical, not a new full rerun.

Restore original verified base plus the cumulative archive in MANIFEST.json. Verify both archives, every changed-file hash and the final source Git tree. All previous archives and parent commits are preserved. Native code is Rust; Java remains only the oracle.
