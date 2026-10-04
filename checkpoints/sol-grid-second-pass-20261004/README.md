# sol-grid-second-pass-20261004

Source `c5368cebbd4d4b9334dd4d9afbbd9e035b68ee67`; tree `dcf3b64359392dd3b8129af8f7dc03a48cf43e09`; 6254 files.

Selected original five-line two-interline GRID initial Staff construction from real runtime sections/filaments: pass1 length consistency, discarded-only ID-sorted pass2, comb rebuild per interline, main-scale curve fitting and combined layout. Actual independent synthetic cue raster matches 4 initial staffs, including 7/11 interlines and small flag, physical pixels and curves. Original input replay/empty/no-second-pass controls and 12 length boundaries reproduce from two original observers. Later GRID/HEADS/STEMS parity remains unverified; existing page CLI still stops explicitly for cue.

415 debug workspace tests pass. HEADS and full OMR remain unfinished.

Restore: `python checkpoints/sol-grid-second-pass-20261004/RESTORE.py --repository ARCHIVAL_CLONE --destination NEW_DIRECTORY`. Destination must not exist. No new credentials, downloads or toolchain setup.

See MANIFEST.json for complete hashes, validation and remaining work.

Separate remote clone/restoration verified source commit/tree and all 6254 source hashes, plus all 825 archival files and archival tree. Saved remote commit `410163a1ecb65a1363b7cad8f3af8580dd41a319`. Recovered CLI oracle comparison passes; tests were not rerun on the restored source.
