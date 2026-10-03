# Audiveris native recovery checkpoint

This private repository preserves the base archive, cumulative implementation recovery artifacts and validation receipts. The current Rust implementation is reconstructed from the pinned recovery chain; it is not expanded into the repository root.

- Verified source commit: `7bc4213cea7ed27ee8355e319447dda3b9b7074f`
- Exact source tree: `52ea03605e0a777a638c1fcb76e3ea7cb6259f64`; 4,752 tracked files, fresh restore verified.
- Full debug and release: 321 tests passed across 100 suites per profile, zero failed or ignored. Each profile ran once at this source.
- [Full validation manifest](checkpoints/sol-rest-horizontal-full-20261003/MANIFEST.json)
- [Complete debug/release logs and receipts](checkpoints/sol-rest-horizontal-full-20261003/EVIDENCE.json)
- [Pinned source recovery manifest](https://github.com/gw31415/audiveris-native/blob/9cbcc2db25d379272fe028c16daf39669e71f531/checkpoints/sol-rest-horizontal-20261003/MANIFEST.json)

Four horizontal raster inputs reach the supported BEAMS.EPILOG boundary with zero rest conversions. Positive real-score rest conversion remains unproven; full OMR and MusicXML remain unfinished. Remaining dependencies include clipped curved system bounds, curved ledger paths/native ledger discovery and later LEDGERS/HEADS/stems/rhythm/MusicXML stages.
