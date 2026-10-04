# sol-stem-chunk-registration-20261004

Source `97b7313ba1e3ed11471061b032c851652e9722d1`; tree `1ac68119e1fc7bea45d06747d5ac8c89a75e7abe`; 6462 files.

LookupChunks raw-registration/removeAll prefix: every raw chunk is registered before excluding existing canonical parts, preserving duplicate identity/order and leaving excluded glyphs in the registry. Two independent original runs certify 2448 controls/6025 raw entries from 612 actual Laura first-system builders; 1364 actual raw physical glyphs equal prior saved lookup results. Original indices are freshly initialized per builder on the actual sheet with strongly held canonical objects; JVM numeric IDs normalize to canonical identity tokens. Native late invalid/budget input refusals preserve registry state and groups. Whole shared-page registry/filter/cache/head/SIG/epilog integration is unfinished; current CLI reports and false completion flags are preserved.

445 debug workspace tests pass. HEADS and full OMR remain unfinished.

Restore: `python checkpoints/sol-stem-chunk-registration-20261004/RESTORE.py --repository ARCHIVAL_CLONE --destination NEW_DIRECTORY`. Destination must not exist. No new credentials, downloads or toolchain setup.

See MANIFEST.json for complete hashes, validation and remaining work.

Separate remote clone/restoration verified source commit/tree and all 6462 source hashes, plus all 1204 archival files and archival tree. Saved remote commit `1d7fe702457900e54496ef2a37cdc3fd3a21ede3`. Recovered CLI oracle comparison passes; tests were not rerun on the restored source.
