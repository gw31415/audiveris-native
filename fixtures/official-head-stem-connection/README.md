# Original selected STEMS boundary

The source-backed `HeadStemConnectionProbe` runs the unmodified, pinned Audiveris
5.11 Java distribution on Linux. Its compile-only ABI shims are excluded from the
runtime classpath. No native output supplies an expected value. A second run of
the same source independently reproduces every byte of `controls.json`.

Controls cover 2,112 head reference points (12 StemHeads, three original fonts,
four exact sizes, clipped bounds and sheet-level learned offsets), 288 gap grades,
5,760 physical/null-glyph relation checks, 40 physical part unions and geometry
fits, 12 eight-state SIG lifecycles, and 144 selected-part-to-head SIG batches.
Parallel relations, edge removal, abnormal flags, prior SIG vertices/relations
and staff-note order are observed on the original live sheet.

The Rust boundary receives explicitly selected parts, an already assessed stem
grade, the certified font bank/staff sizes, and sheet-level learned means. It
normalizes every union to VERTICAL, including a single horizontal input part.
Original `GlyphFactory.buildGlyph` creates a new object even for a single part;
native canonical-content registry ID reuse is a separate identity contract.
Acquisition and SIG registration commit together. Malformed, stale, duplicate
or unsupported requests leave both indexes unchanged. Geometric rejection
retains an abnormal physical stem without creating a relation. Stable native
relation IDs preserve the surviving parallel relation when another is removed.
Native IDs are not compared to JVM object identities.

This implements physical assembly, constructor geometry, default consistency=1
connection checks and nonmanual callbacks for explicitly selected stems. It does
not run automatic StemBuilder seed/section selection or intrinsic grading,
populate other consistency values, handle manual/chord/context links, or run
STEMS cleaner/epilog. Arbitrary relation resurrection/reordering is not certified
by these lifecycle observations. Single-pixel/nonvertical stems, unavailable
exact assets, unsupported profiles and manual links fail explicitly. An original
horizontal fitted median produces an empty ribbon; its geometry is preserved
but supported vertical-stem registration rejects it.

The default CLI does not invoke this development boundary. The previous 357
dependency controls, 13 same-Laura controls, three independent actual-font
synthetic whole pages and default Laura reports remain separate and preserved.
Special staff full-page native parity remains incomplete. HEADS/STEMS completion
flags remain false.

The registered head-request controls in this fixture use a single physical
part as contact. Multiple-part head requests require the explicit nullable
corner-stump API certified in official-head-stem-stump; implicit union contact
is rejected. Generic multi-part union/geometry controls remain preserved.
