# sol-head-staff-catalog-20261003

Source `e7b58c71070f414aa64594a2641e8555d5acac82`; tree `ff07cd6efb37f4edc11893105842f185aa18ddd4`; 5090 files.

Original staff-line Scanner ordering, one-line three-pitch/no-ledger contract, default DrumSet pitch-dependent motifs and ShapeSet processing-switch filters in original enum order. Native scalar planner covers 384 controlled original staff/switch/main-versus-specific-scale cases, 75 outside/missing drum pitches, and two original virtual tablature buildHeads skips with zero SIG vertex change. Sheet main interline is now explicitly passed into seed and ledger Scanner y offsets; physical pitch spacing remains staff-specific. Grid tablature status is retained, and page orchestration explicitly rejects unpropagated non-51 staff catalogs before preparing pixels. Existing certified Bravura51 ordinary five-line/ledger page path, all previous eight Laura fields and shared SIG remain identical; heads_complete=false. This is a staff/catalog contract, not a full special-staff page lookup trace.

374 debug workspace tests pass. HEADS and full OMR remain unfinished.

Restore: `python checkpoints/sol-head-staff-catalog-20261003/RESTORE.py --repository ARCHIVAL_CLONE --destination NEW_DIRECTORY`. Destination must not exist. No new credentials, downloads or toolchain setup.

See MANIFEST.json for complete hashes, validation and remaining work.

Separate remote clone/restoration verified source commit/tree and all 5090 source hashes, plus all 519 archival files and archival tree. Saved remote commit `88989355317587d98f92cad6d45024ffbbbb6065`. Recovered CLI oracle comparison passes; tests were not rerun on the restored source.
