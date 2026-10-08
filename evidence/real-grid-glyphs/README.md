# Retained constructor-produced physical GRID glyphs

The recognition path now creates one typed physical Arc when GlyphRegistry
registers a native glyph, moves the same registry through brace registration,
and carries its physical objects through GRID/PAGE/STEMS. Duplicate pixel
registration returns the existing physical object. Report glyph arrays remain
views; no report rehydration supplies a downstream physical owner.

The report-only GRID facade retains its historical signature. Errors from typed
physical construction are checked before the owned GRID result is published;
an invalid physical registry cannot enter a downstream recognition owner.

Brace erasure resolves its actual graph key and GRID glyph ID against that held
registry, preserving original physical bounds rather than using the extended
Inter rectangle. The actual image target observes the same Arc references across
PAGE-to-STEMS, blanks the report's glyph list, compares both Brace glyphs with
original physical pixel runs and checks that graph/registry storage remain intact.
Existing real Chord/header/30 structural-raster observations remain in that target.

This is recognition-required physical identity, not project/serialization or editor
compatibility. Full real image XML and independent musical quality remain pending.
