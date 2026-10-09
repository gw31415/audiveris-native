# Independent real-page structure baseline

Development-only comparison of existing independent MuseScore 3 source scores against existing original Audiveris XML. No Java or native image recognition rerun, no score/image modification, no recognition side input.

Run: `python3 scripts/compare_score_structure_quality.py TRUTH.mscx RECOGNIZED.musicxml NEW-OUTPUT.json` (output must not exist).

13 focused Python contract tests pass. They observe label invariance; simultaneous voices versus a real chord; voice splits with identical notes; pitch errors versus conditional voice membership; pickup whole-measure rests; wrong/unclosed/nonreciprocal slurs; extra ties against tie-free truth; explicit unsupported timing/ties/cross-staff; grace ordering; and the prior two real event baselines.

| Category | AnLaura matched / truth / recognized | Der Winter matched / truth / recognized |
|---|---:|---:|
| Note/rest events | 73 / 101 / 92 | 102 / 124 / 124 |
| Complete actual chord/rest groups | 50 / 70 / 65 | 86 / 106 / 106 |
| Multiple-note chords | 13 / 20 / 18 | 14 / 16 / 16 |
| Full within-measure voice successor edges | 36 / 58 / 49 | 62 / 79 / 79 |
| Common-chord conditional voice successor edges | 38 / 39 / 38 | 62 / 62 / 62 |
| Meters | 12 / 12 / 12 | 27 / 27 / 27 |
| Occupied/forward measure extents | 8 / 12 / 12 | 26 / 27 / 27 |
| Explicit irregular/implicit markers | 12 / 12 / 12 | 27 / 27 / 27 |
| Rest events | 0 (not exercised) | 9 / 9 / 9 |
| Slur endpoints | 0 (not exercised) | 12 / 12 / 12 |
| Slur pairs | 0 (not exercised) | 6 / 6 / 6 |

Full chord and voice metrics include errors in pitch and time because endpoints retain their musical identity. Conditional voice metrics project both voices onto complete chord anchors appearing exactly once in both scores; they cover only 50 / 70 truth AnLaura chords and 86 / 106 truth Der Winter chords, and cannot hide omitted notes in the full categories. Arbitrary labels are excluded. Across-measure voice continuity is not scored.

Tie-bearing truth, cross-staff chords, composite meter and other unsupported timing/location encodings fail explicitly. Both real references are tie-free: empty tie categories are not evidence of supported tie recognition. XML sound/notation tie endpoints are nevertheless counted to expose spurious ties. Slur visual placement, dynamics/hairpins, articulation, beam grouping, lyrics and engraving are outside this first metric unit. AnLaura's 4 HairPin endpoints are inventoried as unscored.

All outputs retain `quality_accepted: false`. Baselines describe original recognition against independent truth; they do not assert native parity or quality acceptance. Full missing/extra anchors and relation issues are in each JSON. `RESULT.json` records commands, exit status and all input/script hashes. Initial root-level JSON observations are explicitly superseded by these validated files.
