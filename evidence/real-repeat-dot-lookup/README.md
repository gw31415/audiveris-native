# Repeat rejection from actual Staff pitch and physical Barline lookup

The coarse late Measure-edge guard cannot distinguish ordinary augmentation dots
from repeat signs. Original instantCheckRepeat checks actual closest Staff pitch
near ±1 before plain bar lookup, and occurs before staccato. Native now makes that
instant decision from retained System/Staff/virtual Ledger curves and canonical
live Barline bodies, leaving genuine positive construction explicitly pending.
No repeat vertex or fake negative is inserted. Null closest Staff stays null.

On the unchanged original An Laura image,97 actual eligible DOT candidates have
no bar link and generate zero repeat dots. The source also supplies168 declared
boxes around21 physical bars, for profile0/1 and all four sides; all plain bar
lookups are positive. They compare rounded lookup box versus center2D references,
median/width, weighted grade, minimum horizontal gap/first tie,5.0 source support
and zero target support. Declared boxes are lookup controls, not image recognition
or positive constructor/pair/weak-purge completion. Graph/clock/manual identity
and actual Part/Measure references remain unchanged during this plain lookup.

Focused tests and retained63+correction8/exact7/64 XML bytes/XSD are recorded in
VALIDATION.json. Same unchanged image CLI frontier remains separate from native
real-score XML and independent quality acceptance, both still unfinished.
