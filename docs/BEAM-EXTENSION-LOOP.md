# Initial-snapshot beam extension loop

Native `beam_extension_loop::extend_beams` composes the existing merge, stem, spot and parallel stages in the uninterrupted original `BeamsBuilder.extendBeams` order at pinned upstream 9e1e55cd. It copies initial active registration order once, skips snapshot entries removed by earlier replacements, tries LEFT then RIGHT, and stops the side loop after the first accepted replacement. Appended beams remain visible to lookups but never become new iterations. Rejected selected points retain the established no-fallback behavior.

This boundary ends before late hooks, groups and multiple rests; CLI whole-page wiring and full OMR remain unfinished. Lookup work and system-area point queries have separate preflight bounds.

The unmodified pinned original JAR executed uninterrupted extendBeams on 13 independent fixture images. Native comparisons cover final candidates, impacts, pixel glyphs, exclusions and raw registration/tombstone order. These image cases had no accepted extensions; 37 additional uninterrupted original-loop controls cover mutation, replacement and initial-snapshot behavior with empty seed/spot ownership. Existing focused stage controls retain stem and spot acceptance coverage. Exact counts and failed-attempt logs are preserved in evidence/beam-extension-loop-checkpoint.

The first missing-fixture failure and an overconservative area-query-budget failure are retained. Both were corrected before successful validation. The preceding 267 debug/267 release full-workspace baseline is historical for implementation 8b1a9518, not a full-suite rerun of this new source.
