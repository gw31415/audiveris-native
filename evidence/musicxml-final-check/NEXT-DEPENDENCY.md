# Actual Voice event binding

MeasureRhythm.finalCheck now requires Voice/time on every standard chord and,
when expected duration is known, checks all retained Voices. Unknown expectation
skips both duration checks and their abnormal gate as in Java. This does not
remove the stricter current image admission for complete ordinary measures.

Next bind each generated event to actual mapped Graph Voice/onset/duration,
without serial clock overwrite or first-Voice reassignment. The owned
BEGIN/backup/forward writer already follows all retained Voices. Compare that
shared event-binding path on existing actual original multi-Voice controls,
then a same-image original full recognition/export before opening image guards.
No new image/complete flag; general Rational, tuplets, Merged/shared stacks,
lyrics/slur/tie and remaining nonUI dependencies are still partial.
