# sol-head-candidate-filter-20261003

Source `bbf8db60ed019ccbffae1fc54250490e86c8a5fd`; tree `c47ba4bf15cd168719b91515198147d89f737f54`; 4833 files.

Original per-scanner stable decreasing-grade, fixed representative, inclusive max dx aggregation; all 118 scanners yield 454 representatives from the certified 2224 proposals. Original seed conflict kernel (positive rectangles, IoU >= 0.1, grade >= head grade * 0.9, sorted head competitor slice) passes 21 controls. Prior catalog, seed, prefix and shared SIG reports are preserved exactly. Live seed conflicts await glyph-updated seed bounds; no native HeadInter/SIG registration yet.

352 debug workspace tests pass. HEADS and full OMR remain unfinished.

Restore: `python checkpoints/sol-head-candidate-filter-20261003/RESTORE.py --repository ARCHIVAL_CLONE --destination NEW_DIRECTORY`. Destination must not exist. No new credentials, downloads or toolchain setup.

See MANIFEST.json for complete hashes, validation and remaining work.

Separate remote clone/restoration verified source commit/tree and all 4833 source hashes, plus all 419 archival files and archival tree. Saved remote commit `15e48a1ef79d61a7b29ec988cbbfd8ab4f02e2b8`. Recovered CLI oracle comparison passes; tests were not rerun on the restored source.
