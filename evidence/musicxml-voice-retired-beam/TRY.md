# Actual retired BeamGroup getter and overlapping HIGH membership

```sh
cargo test --offline --locked -j2 --lib retired_beam_tests
```

The independent original control compares26 boundaries. Empty/purged old7
keeps six actual slots and signed termination/excess. BeamGroup's cached first
Chord still returns this object, and actual immediate predecessor chord5
imposes7 after tie1. Fresh Voice construction propagates across the four real
previous-Measure group Chords; fresh7 becomes1 and beam1 overrides tie2.
Old Voice re-addition with an overlapping member list is valid in original
HIGH sorting. Its live body becomes3; native actual lookup prefers this live
body over its retained copy. Re-purge updates the same retained entry. Later
Next imposes2, preferred1 wins. No arbitrary previous-Voice last Chord9 is used.

Two original/native discrepancies were reproduced: membership-only BeamGroup
getter rejected the retired owner during fresh construction, then numeric
ownership validation in HIGH rejected the legitimate overlapping old list.
The group getter now reads the existing actual Voice reference and still
validates its scalar/Measure metadata. HIGH retains System/Part/Measure/staff
checks while accepting reassigned Chords as sorting geometry. Renaming and
refinement continue to read actual reference identity, not a report owner plan.

Real original/native source/target bodies, cached IDs, group member order and
actual Measure membership, manual/relation identity, expected/actual/slots/
termination/excess are compared. Native late-preferred failure after staged
purge/tie/beam processing preserves Graph/Voice identities and live/retained
bodies, typed Chords, layout, manual status and expected/slot clocks; repair to
null preferred and retry matches original. Null predecessor and genuine reset
both fail atomically in native. Original refine after reset throws
NullPointerException: native atomic guard policy, not Java exception rollback
parity. Clock/reference clearing occurs only at genuine Measure reset.

Reproduce original in a fresh output directory with the existing checked
Linux oracle and a PAGE book made from the committed parallel-beam PNG:

```sh
python3 scripts/run_voice_lifecycle_control.py \
  --oracle-home /path/to/pinned-linux-oracle \
  --book /path/to/parallel-beam.omr \
  --probe VoiceRetiredBeamProbe --output /tmp/new-retired-beam-control
```

Public/global guards remain false/guarded. Original task/environment and
unobtained ce785c9 are unchanged; no historical649 rerun, prior fixture
replacement, archive duplication, Mac/GHA or new authentication/permissions.
