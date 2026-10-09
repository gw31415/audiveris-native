# Retained PAGE pickup state

`persisted-state.json` records the original Der Winter PAGE book's nine actual
Stack associations and exported measure identities. It is test-only data, never
an input to native recognition or owner construction. Physical System-local
Stack indices are distinct from PAGE score IDs; the latter are 0 for the pickup
and 1..8 thereafter. The third Stack keeps actual5/8 and excess1/8.

The original source/book/XML hashes are recorded. `extract_persisted.py` reads
those saved files without invoking Java or changing them and reproduces the
fixture. Runtime Voice termination is absent from the book serialization;
therefore this fixture alone supplies no callback or live-object identity proof.
`SOURCE-PROVENANCE.json` records the pinned source retrieval that accompanied the
original extraction. This covers the ordinary single-page nonempty/pickup
MeasureFixer subset; repeats, empty merging, cautionary and multiple rests remain
explicit frontiers. Full PAGE and recognition quality remain unaccepted.
