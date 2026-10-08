# Same real-image original/native baseline

Original full unmodified Audiveris5.11 export on the saved CC0 An Laura complete
page succeeded. Native on the same input fails at multiple-Part beam surrounding
selection and writes no XML. BASELINE.json records pixel/source/JAR/argv/output
hashes and counts. laura-original.musicxml is the development-only source output,
never a native input or substitute for recognition. Independent score-truth quality
comparison is pending; original counts alone are not a success metric.

The same-named erased-staff development fixture has different pixels and its first
no_regular_lines diagnostic is excluded. The untouched page normalization matches
the historical complete render; historical renderer exit -6 remains qualified.
No new Mac rendering, alternate environment, authentication or archive transfer.
See docs/IMAGE-MUSICXML-ACCEPTANCE.md for the revised actual acceptance and remaining
recognition connections. This is a real failure baseline, not quality parity.

Independent MuseScore3 event comparison now finds73 of101 exact staff/measure/
onset/MIDI/duration/grace events in the original92-event export (precision0.79348,
recall0.72277,F1=0.75648). ORIGINAL-EVENT-QUALITY.json lists every missing/extra
event. This is a preliminary event metric, with voice continuity/chord grouping/
notation relations still unscored; quality_accepted stays false. Two declared
notation controls verify chord/grace/rest/backup/staff timing and error detection.
