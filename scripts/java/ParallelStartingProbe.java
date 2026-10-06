// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class ParallelStartingProbe extends MappingRetryProbe {
    static Object inspectStarting(Object measure, boolean owned) throws Exception {
        call(measure, "resetRhythm");
        List<Object> chords = new ArrayList<>((Collection<?>) call(measure, "getStandardChords"));
        Object rhythm = make("sheet.rhythm.MeasureRhythm", measure);
        Object member = make("sheet.rhythm.Slot$MeasureSlot", 1, measure, chords.subList(0, 2));
        Object current = make("sheet.rhythm.Slot$CompoundSlot", 1, measure, List.of(member));
        ((List) field(rhythm, "slots")).add(current);
        if (owned) {
            Object voice = make("sheet.rhythm.Voice", chords.get(0), measure);
            call(measure, "addVoice", voice);
            call(chords.get(0), "setTimeOffset", make("math.Rational", 1, 4));
        }
        invoke(rhythm, "processStartingChords");
        List<Object> voices = new ArrayList<>(), states = new ArrayList<>();
        for (Object voice : (Collection<?>) call(measure, "getVoices"))
            voices.add(obj("id", call(voice, "getId"), "chord_ids", ids((Collection<?>) call(voice, "getChords"), chords)));
        for (int i = 0; i < chords.size(); i++) {
            Object voice = call(chords.get(i), "getVoice");
            states.add(obj("id", i, "voice", voice == null ? null : call(voice, "getId"),
                    "time_offset", fraction(call(chords.get(i), "getTimeOffset"))));
        }
        return obj("name", owned ? "already-owned-start" : "parallel-unowned-start", "input_starting", List.of(0, 1),
                "starting_order", ids((Collection<?>) call(current, "getChords"), chords),
                "voices", voices, "chord_states", states);
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "actual saved four quarter chords, declared first compound containing two starting chords; original processStartingChords; not new image polyphony or valid export of duplicate old ownership",
                        "controls", List.of(inspectStarting(measure, false), inspectStarting(measure, true))));
        System.exit(0);
    }
}
