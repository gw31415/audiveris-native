# Bounded native duration-control proposal

This directory is a proposal only. No checkout file was edited, no Rust compilation/native test/image recognition was run. Rustfmt parsed the proposed file. `proposed.patch` adds one block inside the existing cfg(test) target:

`music_voices::group_tests::original_saved_group_getters_and_declared_beam_rest_follow_actual_ensemble`

## Ready small control

After the target's existing positive-excess then null-offset checks, reuse cloned held native Graph/Voice/Measure owners. Change only the actual last Chord offset to zero. Its existing source-compared duration is 1/4 and expected duration is 1. Assert the termination becomes -3/4, old Voice and Stack excess remain 1/4, and abnormal remains true. Assert actual Voice and Measure Arcs, Slot Arcs/values, Chord ID ownership, and Graph/relation/manual state remain unchanged by check_duration. The original caller's earlier state is untouched because the declared control uses staging clones.

This adapts the observed original old-excess→underflow branch to the existing fast synthetic owner's known quarter-note clock. It is a declared contract control, not a second image-quality example or byte-for-byte replay of Der Winter's eighth-note input. The source observation is `/tmp/real-rest-page-fixer-replay-original-v3/summary.json`, control `underflow-keeps-old-positive-excess`; the original quarter-note duration and expected=1 are already independently observed in `fixtures/official-voice-end-time/controls.json` and `fixtures/official-voice-duration/controls.json`.

The existing target recognizes its small synthetic beam-two-systems fixture once; this addition does not add an image-recognition call. A filtered rerun of this existing target suffices for the new block. Existing real Der Winter/An Laura image targets need not be rerun for this test-only hunk.

## Per-Voice peer-excess observation remains separate

The existing fast owner's Measures belong to distinct one-Part Stacks. They are not two actual peers of one shared Stack; changing their geometry or copying one Measure's ownership would invalidate an ownership claim. A test which repeats the production fan-out loop manually would also not verify the production loop.

The smallest honest future observation is a cfg(test)-only callback hook immediately before each actual check_duration and immediately after the existing peer publication in fix_recognized_page_measures. It should record all staged peer Measure views, canonical Stack timing, the selected actual Voice Arc, and the ordered actual Voice terminations/excesses. The hook reads live owners and serializes observations; it must not source owners from reports. Capture both states in the next already-required real-owner test run, reusing its existing owned Der Winter graph; no additional image run solely for this evidence.

To reproduce the observed shared-growth branch, declare the held Stack and all peer Measure excess views as 1/64 while preserving the actual positive Voice's clock/end and old Voice excess. Calling the ordinary PAGE method should show the +1/8 Voice's check change every peer view to 1/8 immediately, with all earlier callback entries seeing their current accumulated value. Restore exact held timing and PAGE metadata/certificates afterward. This can be done in a future existing-image helper; avoid introducing a general editing API for the test.

Until that hook/control runs, source intermediate peer publication is observed, native publication order is source-reviewed, and the actual native comparison covers final shared states plus owner/rollback identity. Do not claim native per-Voice intermediate runtime parity.
