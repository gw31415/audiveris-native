# Live original HEADERS boundary

These 71 fixtures call the pinned original GRID implementation and then original
`HeaderBuilder.processHeader` for every system. They are captured immediately,
before persistence or reload, with no graph/list/recognition hooks. They include
final clef selection, header bar purge, and header freezing.

Use these fixtures for live numeric grades and transient clef/key/time ranges.
The companion `../official-headers-expanded` is the official batch CLI result
after save/reload, which drops transient ranges and rounds grades to three decimal
places. Apart from those declared serialization effects, selected objects,
glyphs, bounds, flags, values, staff stops and SIG identities/order/relations
matched on all 71 inputs.

Exact input/output/source/JAR hashes and execution receipts are indexed by
`evidence/headers-expanded-oracle-validation.json`. Run with
`scripts/linux_oracle.py --probe headers-live --fixtures <input names or PNG paths>`.
These are development-only upstream oracle results; no native parity claim is
implied by this directory alone.
