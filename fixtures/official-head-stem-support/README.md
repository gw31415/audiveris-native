# Original physical stem-length support

The source-backed HeadStemSupportProbe runs the pinned, unmodified original
Audiveris 5.11 distribution. Its original HeadStemRelation.setConsistency(head,
physicalStem) inputs come from actual constructed original head/stem objects and
the live sheet scale; no native output supplies expected values. Two independently
compiled source-backed invocations must reproduce all exported control bytes.

1,536 controls span the 12 StemHeads, both stem sides and vertical endpoints,
eight actual fitted stem lengths and four gap configurations. The new physical
matrix uses Bravura/51pt, main interline 11, profile 0 and an ordinary staff.
Earlier three-font/four-size reference and profile 0/1/2 connection controls
remain preserved; this matrix does not certify whole STEMS on those pages.
1,152 relations are accepted (192 small-head cases), and 384 are geometrically
rejected. Original
support population uses the fitted median y-span divided by the sheet main
interline, then the 2.8 neutral length. Smallness comes from the head shape,
independently of whether a staff is small. Source support uses (4 * consistency)
* relation grade; target support, relation grade, eligibility and abnormal
callbacks remain unchanged. Prior SIG objects/relations and staff-note order
are observed as preserved in every original registration.

Rust register_selected_with_consistency adds this explicit optional boundary to
the previously certified selected-parts transaction. The original unset API stays
unchanged. Both registry and SIG remain atomic; shared SIG independently verifies
measured consistency against the stored target median and head shape. Signed
stem origins use the original absolute pixel traversal/fitting order;
the existing nonnegative beam geometry contract stays unchanged. Negative top
bounds are certified by 96 original physical controls. Existing
native vertex/relation IDs remain stable when this support data is updated. A
native consistency_interline field identifies the caller-supplied main scale
used for verification; it is not a JVM object identity or a completion claim.

Zero/reversed/nonfinite medians, unavailable exact assets, unsupported profiles,
manual links and malformed consistency/ratios fail explicitly. The standalone
original small zero-length +Infinity kernel remains preserved, but such geometry
cannot be serialized/registered as a supported physical vertical stem.

This does not implement automatic STEMS seed/section selection, intrinsic stem
grading, SIG contextual grade computation, chord/manual links, cleaner or epilog.
The default CLI still does not invoke these explicitly selected boundaries.
Actual special-staff full-page native parity, complete HEADS and complete STEMS
remain false. Previous Laura, 13 same-image controls, all24 selected HEADS kernels,
three independent original-font synthetic pages and 357 STEMS dependencies remain
separate and preserved; synthetic pages are not published score scans.

The registered head-request controls in this fixture use a single physical
part as contact. Multiple-part head requests require the explicit nullable
corner-stump API certified in official-head-stem-stump; implicit union contact
is rejected. Generic multi-part union/geometry controls remain preserved.
