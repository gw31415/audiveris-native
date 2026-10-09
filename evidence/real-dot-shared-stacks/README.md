# Dot membership on actual shared Stack geometry

Original SystemInfo.getStackAt uses the first surrounding actual Staff and its
Measure abscissae, excluding left and including right. A Stack is shared by the
vocal and piano Parts; addInter selects the Measure by actual Part. Null Part
stays unassigned. Removing a Dot performs the same Stack/Part lookup before
relation callbacks. Re-population is idempotent and does not clear rhythm clocks.

The native graph now holds the constructor MeasureLayout/Part/Stack and Staff/
area Arcs. No ownership is reconstructed from GRID reports. Rebinding rejects
replacement geometry; malformed late binding preserves the live graph. The
single-Staff primitive controls still use their declared geometry-less binding.
This is a recognition ownership unit, not general editor/undo work.

A fresh unchanged An Laura original image through CHORDS provides52 boundary/
null/outside queries and24 declared Dot add/populate/remove controls on the real
six Staffs/eight Measures/four shared Stacks. The repeated registration control
calls original Stack.addInter; full PageRhythm.populateFRATs/filter is not yet
compared. These declared dots test lifecycle,
not recognition of actual dotted notes. Native actual candidate recognition,
curved Staff distance lookup, later symbol reduction and shared rhythm/output
remain separate dependencies. Quality acceptance stays false.

VALIDATION.json records focused tests and existing63+correction8/exact7/64 XML
bytes/XSD. The actual full-image CLI frontier is observed independently.
