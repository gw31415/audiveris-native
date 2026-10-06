# Original next-unit observations (Rust not implemented at this receipt)

Original5.11.0 source `9e1e55cd2746037d059345881c53e6a6754bffbd`.
`scripts/java/FoundationSpecialChordProbe.java` loads the existing saved
System/Staff, clears SIG and declares physical stems/heads/beams. It calls
unmodified analyzeChords/contextualize with original default
allowSmallHeadOnStandardBeam=false. Eight declared conditions plus24 scalar
Shape duration/size/enum-order observations are saved. No new page fixture.

Original success does not imply native success. Pending Rust dependencies:
actual duration buckets (different shapes with the same duration support each
other), small/standard head exclusion before duration support, beam size
exclusion and head/beam size compatibility, typed SmallBeam context and owning
contacts. Full production consistency, small-beam builders and special-head
MusicXML/whole-page validation remain separately guarded. These are final
implementation targets, not planned omissions or caller-selected API additions.
