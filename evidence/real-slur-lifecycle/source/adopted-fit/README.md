# Existing Java fit observation test proposal

This `/tmp` package projects **all 157 existing actual Java observations**: 134 singles and 19 pairs from actual native component point sequences, plus four explicit controls. Expected values come only from `/tmp/native-slur-math-original-v1/observed.json`; every point sequence and kind was checked against that run's existing `sequences.tsv`. Neither NumPy estimates nor previous Rust outputs supply expectations.

The compact fixture `source-observations.json` is 41,779 bytes. It contains 39 present selected models, 118 NULL models, 15 present Impacts, and 39 present Beziers. Fourteen native candidates and one deliberately shallow control pass mathematical acceptance. Case 104 preserves a present Circle with NaN radius/distance and a present Bezier containing NaNs; those states are not converted into NULL. Horizontal, diagonal, and duplicate controls retain NULL final models. These are mathematical observations, not recognized slurs.

`fit_source_observations.rs` is a `#[cfg(test)]` module proposal for the current `src/music_slurs/fit.rs`. It uses only the existing exposed `Fit` fields: `selected_after_impacts`, `impacts.grade`, `bezier`, and `math_accepted_before_binary_and_head_links`, plus the selected Circle's center/radius/distance/ccw/above/first-angle/last-angle. No production API changes or private constructor calls are required.

Suggested adoption, to be performed by the owner of the frozen checkout:

1. Copy the fixture to `fixtures/official-short-slur-fit/source-observations.json`.
2. Copy the test proposal beside `src/music_slurs/fit.rs`, then append `include!("fit_source_observations.rs");` to `fit.rs`. The included module is guarded by `#[cfg(test)]`.
3. Run only the targeted `actual_java_sequences_preserve_nullable_geometry_and_math_decisions` test in the next coordinated native validation.

The test requires numerical equality for finite center, radius, and post-impacts distance. First/last angles, Bezier coordinates, and grade use absolute tolerance `1e-12`; no relative tolerance conceals larger errors. Finite fixture scalars are roundtrip decimal strings parsed by Rust `str::parse::<f64>` so expected bits do not depend on serde_json floating-parser features. Original nonfinite strings and nullability remain separate contracts. The existing standalone comparison reported these equality/tolerance bounds; **the newly adopted kernel has not been executed by this unit**.

Original source pin: `9e1e55cd2746037d059345881c53e6a6754bffbd`. `SOURCE.json` records original observed/TSV/JAR/observer/runner/book hashes and the inspected adopted-fit hash. The original observer loaded an existing OMR for actual Scale 11, allocated a detached SlursBuilder without its constructor, initialized original Parameters, and invoked original Circle, computeModel/computeImpacts, and SlurInfo curve math. It did not run image recognition or ordinary lifecycle. The fixture retains that qualification.

The current public Fit omits rough/fitted constructor observations, selected-before-impacts state, individual impact components, and left/right side-model object identity. This test makes no claims about those omitted fields, original Skeleton extraction, BINARY run/gap checks, actual Head/Chord linkage, graph callbacks, or MusicXML quality. In particular, a rejected candidate with a changed *unselected* fitted circle cannot be diagnosed by this reduced API test.

`project_existing_observation.py` records the deterministic projection, input equality, and fixed provenance checks. It only reads existing observations; it does not execute Java, Rust, or image recognition. Its successful run is a fixture integrity check, not a kernel test pass. `HASHES.json` covers package deliverables. No repo edits, builds, Git, original/native executions, or Band tests were performed. The proposed Rust test file was formatted with the existing rustfmt.
