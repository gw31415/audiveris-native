// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class MeasureFinalCheckProbe extends CompoundMappingProbe {
    static Object state(Object measure, Object rhythm, String name, Object mutation) throws Exception {
        Object ok = invoke(rhythm, "finalCheck"), stack = call(measure, "getStack");
        List<Object> voices = new ArrayList<>();
        for (Object voice : (Collection<?>) call(measure, "getVoices"))
            voices.add(obj("id", call(voice, "getId"), "termination", fraction(call(voice, "getTermination")),
                    "excess", fraction(field(voice, "excess"))));
        return obj("name", name, "mutation", mutation, "ok", ok, "voices", voices,
                "expected", fraction(call(stack, "getExpectedDuration")),
                "abnormal", call(measure, "isAbnormal"), "stack_excess", fraction(call(stack, "getExcess")));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        Object mapping = inspectCompound(measure, "separate-member-times");
        Object rhythm = make("sheet.rhythm.MeasureRhythm", measure), stack = call(measure, "getStack");
        List<?> chords = new ArrayList<>((Collection<?>) call(measure, "getStandardChords"));
        List<Object> observations = new ArrayList<>();
        observations.add(state(measure, rhythm, "all-voices-underflow", obj()));
        call(chords.get(3), "setTimeOffset", make("math.Rational", 1, 1));
        observations.add(state(measure, rhythm, "later-voice-excess", obj("chord", 3, "time", "1")));
        call(chords.get(3), "setTimeOffset", make("math.Rational", 1, 2));
        call(measure, "setAbnormal", false);
        observations.add(state(measure, rhythm, "underflow-retains-excess", obj("chord", 3, "time", "1/2", "abnormal", false)));
        call(chords.get(0), "setTimeOffset", (Object) null);
        observations.add(state(measure, rhythm, "missing-nonlast-clock", obj("chord", 0, "time", null)));
        call(chords.get(0), "setTimeOffset", make("math.Rational", 0, 1));
        call(stack, "setExpectedDuration", (Object) null);
        call(measure, "setAbnormal", true);
        observations.add(state(measure, rhythm, "unknown-expected-skips-duration", obj("chord", 0, "time", "0", "expected", null, "abnormal", true)));
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "actual saved mapped four-quarter/two-Voice measure, declared sequential clocks/expected-duration controls; private MeasureRhythm.finalCheck, not automatic polyphonic image recognition/export",
                        "mapping", mapping, "observations", observations));
        System.exit(0);
    }
}
