# Original ordinary-rest base durations

The pinned original Shape.getNoteDuration API was invoked directly for12 shapes
without image recognition. observed.json and PROVENANCE.json record the answers,
JAR/source/helper hashes and exact invocation. Nine ordinary short-rest/head
cases are compared by native tests. The three long/breve/whole cases document
Shape answers only; whole-measure duration remains its existing owner path.

Native exact duration now retains each ordinary rest fraction through augmentation.
The declared dotted-rest boundary test follows AbstractChord.getDurationSansTuplet
(3/2 or7/4) and checks that the old sixteenth tick adapter still rejects finer
fractions. That test is a source-contract control, not an additional original
RestChord runtime observation. RestChord Voice/Slot behavior is checked separately
on the unchanged Der Winter page. No source output supplies recognition input.

The first local stdout capture included logger output and was not valid JSON;
it remains separately in /tmp/real-rest-duration-original-v1. This fixture is
fromv2, which writes a dedicated JSON file and isolates runtime XDG/home paths
under its temporary output directory. No existing settings or original book were
changed. The bundled observer is reflection-only and uses the existing JAR.
