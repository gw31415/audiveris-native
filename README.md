# Audiveris native recovery checkpoint

This private repository preserves base archives, exact-source recovery bundles and validation receipts. The Rust implementation is reconstructed from the pinned chain; it is not expanded into the repository root.

- Latest source: `a85dd02d81954790ceb86e186b5a26a45484d100`; tree `9c27570d50c8a134b36f46cd935b5beea9e761aa`, 4761 tracked files, fresh restore verified.
- Current stage: original ledger candidate retrieval and beam-overlap/maximum-length purges. Supplied sections and already-selected good full beam areas; ledger acceptance and GRID integration pending.
- Independent original ledger controls: 93 (54 positive, 39 absent); four focused integration tests plus one original line-fit library test passed. Formatting and all-target Clippy passed. Full suites unverified at this new source.
- Prior full baseline `7bc4213cea7ed27ee8355e319447dda3b9b7074f`: debug/release each passed 321 tests across 100 suites, zero failed/ignored, once each.
- [Latest source/validation manifest](checkpoints/sol-ledger-candidates-20261003/MANIFEST.json), including original bytecode, validation logs and fresh recovery proof.
- [Preserved curved-bounds source](checkpoints/sol-curved-clipped-bounds-20261003/MANIFEST.json).
- [Prior full validation](checkpoints/sol-rest-horizontal-full-20261003/MANIFEST.json).

Positive real-score rest conversion is not demonstrated. Native ledger staff checks/index lookup/reduction/registration and beam selection/GRID dispatch remain unfinished, as do general crossing/non-monotone clip bounds, curved ledger pitches and LEDGERS/HEADS/stems/rhythm/MusicXML. Full OMR remains unfinished.
