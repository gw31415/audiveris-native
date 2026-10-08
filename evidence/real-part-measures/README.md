# Actual PartBarline, per-Staff Measure and shared Stack layout

The unchanged full An Laura page is recognized independently by original normal
reachStep(TEXTS,false), then original MeasuresBuilder in memory before SIG
serialization. The observer records original actual PartBarline member objects,
per-Staff Measure abscissae, effective versus contained left bars and shared Stack
membership. Four retained Parts with Staff counts1/2/1/2 own eight Measures across
four system Stacks. The lower and upper piano Staff remain one actual Part.

Native allocates StaffBarlines in original column-then-Staff order, PartBarlines
in original Part order, and Measures on those same retained Part/Staff/Curve Arcs.
PartBarlines hold the canonical StaffBarline objects with actual physical keys.
The same right PartBarline becomes the next Measure's effective left boundary;
a back-to-back bar is also held as its contained left bar. Stack containers hold
the same Measure Arc objects that Part containers hold, and Measures reference
the common Stack Arc. Geometry/report projections never supply ownership.
The single-Staff continuation now consumes these actual layout bounds; the piano
Chord dispatch remains an explicit next boundary.

Registration and layout allocation stage together. A late noncanonical Part Staff
leaves physical bodies and manual relation identities unchanged. Substituting
another Staff with the same ID is rejected. Existing Voice/Slot/expectedDuration
clocks are not rebuilt or cleared by this pre-rhythm layout stage. Selected owner
rollback and exact rational XML regressions remain required.

Complete columns are supported; original minority delete/majority expand and
short courtesy-Measure Head purging remain explicit boundaries if an input needs
them. Back-to-back layout references are supported but recognition/export repeat
semantics remain guarded. The real image still produces no native MusicXML and
quality acceptance is false. Constructor/owner observations are development
comparisons only, never inputs to native image recognition.
