# Original special-head overlap predicates

Audiveris5.11.0 source `9e1e55cd2746037d059345881c53e6a6754bffbd`.
`scripts/java/FoundationSpecialHeadOverlapProbe.java` loads the existing saved
System/Staff, clears SIG and declares physical heads/stems, calling unmodified
`SigReducer.detectOverlaps` and `contextualize` twice.

Six conditions compare all four original `HeadsOvalSmall` shapes and two
standard-size nonordinary heads. Positive small-head/stem overlap produces an
exclusion for black/whole/breve small heads; existing HeadStem Support prevents
it for small void. Standard cross and breve retain the original contact
exemption. Directional pixel/core geometry, intrinsic/contextual grades,
canonical exclusion cause/endpoints and repeat behavior are observed.

No new image fixture, full special-head page recognition, duration export or
admission guard removal is claimed. Later special-head chord consistency and
MusicXML dependencies remain guarded. Existing ordinary overlap controls and
Linux image CLI regressions remain distinct comparisons.
