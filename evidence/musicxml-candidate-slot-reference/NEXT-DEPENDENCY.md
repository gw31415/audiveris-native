# Next previous-slot merge dependency

Slot constructor bindings and cached xOffset are now retained independently from
current membership/ordered views. Live active filtering and Voice extinction use
the same getters. Explicit constructor/list-mutation controls compare original
removed references and last-constructor rebinding; full merge/image polyphony is
not claimed. Current cache geometry remains zero-skew one-staff only.

Next mergeWithPreviousSlot: EQUAL and known previous chord time, append rookie to
previous object, setAndPushTime, remove current ordered slot, renumber later IDs.
Original does not rebind rookie to previous Slot or recompute constructor offsets.
The ordered mutable candidate objects and ID propagation still need authoritative
ownership; current compound views are immutable during image processing. Preserve
cached reference keys when adding mutable order, rather than reconstructing them
from changed membership. Shared narrow clocks remain separate.

Then general compound mapping and retained multiple-Voice PartwiseBuilder traversal,
exact Rational/implicit tuplets, Merged/shared stacks, lyrics and complete cleaner.
No planned original nonUI omissions or true global flags; original all-rest XSD
rejection remains negative, and no new image admission is claimed.
