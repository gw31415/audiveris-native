# HEADS default range catalog checkpoint

Implementation `3f015272fde1c071351296fd80c25ad8c1bc4448`; tree `cae6a56c2fd3526bd56819fb11b54386ca3cf4e2`. Parent saved main `fa645b4d4170850b4d3e41b44d4ee81bd5a04e25`.

349 debug workspace tests pass. All 118 Laura scanners match original 5.11: 2,224 accepted and 407 weak stemless candidates rejected. Existing black/void prefix 2,221 and seed 418→358 remain exact, as does shared SIG. Original whole/breve renderer and floating point boundary controls pass.

Restore with `python checkpoints/sol-head-range-catalog-20261003/RESTORE.py --repository ARCHIVAL_CLONE --destination NEW_DIRECTORY`. The destination must not exist. No toolchain setup, download or credential changes are needed.

Range aggregation, seed conflicts, glyph retrieval, HeadInter/SIG registration and full OMR remain unfinished. See MANIFEST.json for hashes, limitations and validation evidence.

A separate remote clone of main `8bfaa3136da2be8e75e91083bd40efa3c6bc5568` was restored and verified: all 4,826 source file SHA256 values and implementation tree match; all 396 archival files and archival tree also match. The recovered CLI report passes the original-oracle verifier. Tests were not rerun on the recovered source. See evidence/REMOTE-READBACK.json.
