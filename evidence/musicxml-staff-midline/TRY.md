# Compared small unit

GRID persistence now keeps its actual simplified StaffLine Curve objects in
immutable Staff geometry owners before publishing a report view. The same Staff
and Curve references move through OwnedPage, OwnedItems and the Music owner.
Existing serialized grid/report output remains identical. Middle-line reads use
the retained real owner, not mutable grid report reconstruction or missing-data
defaults. Missing actual Staff refuses without changing actual rhythm owners.

Pinned original comparisons cover thirteen loaded source samples and forty-five
samples from five declared real Staff controls: 1/2/4/5/6 lines, lower middle index
for even counts, interpolation/global-slope extrapolation, and ties-even integer
yAt. Original declared StaffLine inputs/retained points record the +0.5px
construction boundary. The native real measured source remains unchanged.

Initial control construction missed +0.5px and failed. Generic reflective yAt
selection also proved ambiguous between double and int, causing the next fixture
to carry doubles. Current observation explicitly selects yAt(int); previous raw
outputs and failure logs remain retained. Production interpolation/rounding was
not changed to conceal either observation issue.

Actual-owner tests keep original Staff/Curve references through the native
pipeline, then alter report staff/line fields and prove the actual middle line
unchanged. Missing Staff refuses with actual Chords/Slots/Voice clocks intact;
restoring the same owner restores access. Selected rhythm tests also compare
Staff/Curve references on success, later member failure and same-owner retry.
No report ownership is reconstructed or callback-time clocks erased. Polygon
lookup, mutable Staff/controller ownership, complete clear/detect and implicit
tuplets remain pending, with all full/nonUI flags false.
