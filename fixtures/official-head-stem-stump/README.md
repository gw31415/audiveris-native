# Original explicit corner-stump contact

Original HeadLinker.SLinker.CLinker.checkStemRelation passes its `stump` Glyph
to HeadStemRelation.checkRelation, independently of the assembled StemInter
glyph. The captured pinned 5.11 caller bytecode records this contract.

Two independently compiled source-backed HeadStemStumpProbe invocations on the
original Linux runtime reproduce every control byte. All inputs are constructed
original objects on the live Laura sheet; no native output supplies an expected
value. These are explicit object controls, not automatic STEMS terminals on
Laura or another score.

6,048 controls cover 12 StemHeads, both sides/endpoints, seven vertical gaps,
three horizontal offsets, nullable/physical corner stumps and profiles 0/1/2.
The matrix uses Bravura/51pt, main interline 11 and an ordinary staff. Each
selected stem consists of two sparse physical parts, while the contact stump
is explicitly independent. Native acquisition/union, constructor geometry,
contact gaps, measured consistency, support ratios, nonmanual shared SIG flags
and prior state are compared to the original. 4,272 relations are accepted and
1,776 rejected; 3,024 controls have no physical stump. Substituting whole union
bounds for the stump incorrectly accepts 1,104 originally rejected requests.

Rust register_selected_with_stumps validates a caller-selected optional physical
stump, then atomically acquires selected union glyphs and registers measured
head relations. Null contact uses the original template reference point.
Unspecified contact on multiple-part head requests fails explicitly in the old
APIs. Their previously certified single-part behavior is retained. Generic
multi-part glyph assembly remains available; stumps are validated, not silently
registered as new glyphs. Native vertex/relation identity stays stable on
repeat calls. Malformed, stale or duplicate requests roll back both indexes;
ordinary geometric rejection retains the abnormal registered physical stem.

This does not select/extract HeadLinker stumps or stem parts from raster/sections,
implement StemBuilder's single-part bypass, glyph-to-StemInter cache, intrinsic
grade rejection side effects or profile4 artificial branch, compute SIG
contextual grades, or run cleaner/finalizer/epilog. Completion flags remain
false. Previous 357 STEMS kernels, single-part relation/support controls, 13
same-Laura HEADS controls, all24 selected HEADS kernels and three independent
original-font synthetic whole pages remain separately preserved. Actual
special-staff native full-page parity and published score scans remain pending.
