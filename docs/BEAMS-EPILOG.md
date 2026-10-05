# Native BEAMS epilog

finish_beams_epilog implements original BeamsStep.doEpilog -> SystemInfo.removeGroupedGlyphs(BEAM_SPOT): remove only free-glyph owner references whose canonical glyph has exactly the BEAM_SPOT group. Preserve registry objects, groups, survivor order, shared graph vertices, edges and IDs. Validate every reference before mutation. CLI invokes this after all supported native system rest boundaries complete; unsupported multi-staff/ledger geometry prevents epilog and BEAMS completion. GRID always enters the common graph even when candidate extraction is incomplete. The supported single-staff default geometry now reaches BEAMS.EPILOG; this is not full OMR, MusicXML or demonstrated positive real-score rest conversion. Eighteen focused tests, fmt and all-targets Clippy pass. Real image replay preserves graph and IDs exactly; scanned multi-staff input remains explicit unsupported. Full debug and release were unverified at the epilog implementation checkpoint. Subsequent fixed-pin runs at 019ed5da each found the obsolete CLI assertion below and stopped; later suites were not reached. Subsequent validation is recorded in the external contract checkpoint receipts. Historical 289-test release applies only to source 21f0833abd67dd4fa1dd0c60a74a686779329818.

## Current CLI contract

The top-level last_completed_substage reports the furthest boundary actually
completed, while last_completed_stage remains STEM_SEEDS for the full-step
milestones. Initial preparation/prolog/candidate snapshots retain their partial
beams_complete=false values. later_beams.beams_complete and epilog_complete
become true only when all native system rest boundaries and sheet epilog finish.
The supported single/default-five-line-staff synthetic inputs can reach
BEAMS.EPILOG; unsupported multi-staff/ledger cases remain incomplete and expose
their actual earlier boundary/errors. musicxml_complete remains false.

The CLI contract test runs beams-double.png with an empty environment and
PATH=/nonexistent, verifies epilog and preceding completion flags, shared graph
identity/endpoint invariants, unchanged mask digest and refusal to overwrite
existing files. Its old INITIAL_CANDIDATES assertion described an earlier
checkpoint. Updating that single obsolete expectation preserves the original
partial phase assertions, image checks and output refusal; it does not infer
full OMR or positive real-score rest recognition.
