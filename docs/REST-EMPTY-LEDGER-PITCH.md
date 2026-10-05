# Native rest pitch with an empty ledger map

This scoped implementation continues the validated e13359df source checkpoint.
The original Staff.pitchPositionOf computes in-staff pitch from first/last-line
ordinates and the line-count span. Outside the staff it walks signed consecutive
ledgerLineMap indices, interpolates between ledger curves, and falls back from
the last known line using specificInterline. For the native pre-LEDGERS BEAMS
boundary that map is empty, so the exact fallback is -4 + 2*(y-first)/interline
above a default five-line staff, or 4 + 2*(y-last)/interline below it. The measured
interline matters: extrapolating the in-staff first/last span is not equivalent.

native_rest_mapping::Mapping::pitch_at implements the in-staff calculation and
this empty-ledger fallback. position still uses the actual integer beam center
for staff-at-or-above and the actual median endpoints for pitch. External pitch
no longer marks every beam/rest in that supported system unsupported; admission
can reject the non-rest candidate normally and continue. Native GRID now emits
explicit empty ledger_lines. Legacy missing fields denote the same pre-LEDGERS
contract; nonempty or malformed ledger representations reject explicitly.
No native ledger extraction, populated ledger interpolation or multi-staff
association is claimed by this stage.

Original multi-staff semantics traced in retained Staff/SystemInfo and
StaffManager bytecode are area-based: getClosestStaff first enumerates staff
areas containing the point. With multiple containing areas it uses truncated
integer Staff.distanceTo, keeping the first staff on ties. getStaffAtOrAbove
returns that staff if its first line is at/above the integer beam center, or
the preceding staff in system order otherwise. Native GRID exposes system
areas/section membership but does not yet provide these original staff areas,
horizontal-neighbor/vertical-neighbor construction or page-clipped membership.
A nearest-line substitute would not preserve this behavior, so those paths
remain unsupported. Populated ledgerLineMap construction and GeoPath.yAtXExt
evaluation are also still missing from native staff data.

EmptyLedgerPitchControlProbe uses the pinned original unmodified Staff class
with supplied sloped LineInfo curves, an empty original ledgerLineMap and actual
specificInterline=16. Its 21 independent outputs cover line boundaries, staff
middle, just-outside points and farther external points at three abscissas.
Native outputs are never supplied to that oracle. The original fixture uses
a staff span different from four specific interlines to distinguish the two
formulas. The existing uninterrupted original conversion controls still cover
six positive and five negative synthetic conversions. Actual beams-single CLI
and scanned Laura checks are separately recorded: no positive real-score rest
conversion is demonstrated, and Laura's multi-staff rest path remains explicit
unsupported. Full debug/release at this changed source are UNVERIFIED; the
297-test-per-profile result belongs only to e13359df and its immutable recovery
checkpoint deec43364e2b6b348cca4b38f798f534c8bd1925.
