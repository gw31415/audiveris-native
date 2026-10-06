// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class InferredSlotTimeProbe extends MappingRetryProbe {
    static Object inspectInferred(Object measure, boolean agree) throws Exception {
        call(measure, "resetRhythm");
        List<Object> chords = new ArrayList<>((Collection<?>) call(measure, "getStandardChords"));
        Object rhythm = make("sheet.rhythm.MeasureRhythm", measure);
        Object retriever = make("sheet.rhythm.SlotsRetriever", measure, false);
        put(rhythm, "narrowSlotsRetriever", retriever);
        List<?> narrow = (List<?>) call(retriever, "buildSlots");
        List<?> wide = (List<?>) call(make("sheet.rhythm.SlotsRetriever", measure, true), "buildSlots");
        List<?> previous = (List<?>) invoke(rhythm, "buildCompoundSlots", narrow, wide);
        Object declared = make("sheet.rhythm.Slot$MeasureSlot", 3, measure, chords.subList(2, 4));
        Object current = make("sheet.rhythm.Slot$CompoundSlot", 3, measure, List.of(declared));
        ((List) field(rhythm, "slots")).addAll(List.of(previous.get(0), previous.get(1), current));
        for (int i = 0; i < 2; i++) {
            Object voice = make("sheet.rhythm.Voice", chords.get(i), measure);
            call(measure, "addVoice", voice);
        }
        call(chords.get(0), "setTimeOffset", make("math.Rational", 0, 1));
        call(chords.get(1), "setTimeOffset", make("math.Rational", agree ? 0 : 1, 4));
        var ctor = c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(
                c("sheet.rhythm.MeasureRhythm"), c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper = ctor.newInstance(rhythm, current);
        invoke(mapper, "purgeRookiesSet");
        Object input = mapperInput(mapper, rhythm, measure, chords, current);
        invoke(mapper, "mapRookies");
        List<Object> finalChords = new ArrayList<>();
        for (int i = 2; i < 4; i++) {
            Object voice = call(chords.get(i), "getVoice");
            finalChords.add(obj("id", i, "voice", voice == null ? null : call(voice, "getId"),
                    "time_offset", fraction(call(chords.get(i), "getTimeOffset"))));
        }
        return obj("name", agree ? "agreeing-inferred-times" : "conflicting-inferred-times",
                "input", input, "output", mapperOutput(mapper, current, chords),
                "original_rookie_states", finalChords);
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "two declared original actual-object controls: two rookies in one initially unknown narrow slot, two previous Voices; not image polyphony",
                        "controls", List.of(inspectInferred(measure, true), inspectInferred(measure, false))));
        System.exit(0);
    }
}
