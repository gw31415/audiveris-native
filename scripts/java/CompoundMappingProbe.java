// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class CompoundMappingProbe extends MappingRetryProbe {
    static Object inspectCompound(Object measure, String name) throws Exception {
        call(measure, "resetRhythm");
        List<Object> chords = new ArrayList<>((Collection<?>) call(measure, "getStandardChords"));
        if (chords.size() != 4) throw new IllegalStateException("saved four-chord measure required");
        Object rhythm = make("sheet.rhythm.MeasureRhythm", measure);
        Object retriever = make("sheet.rhythm.SlotsRetriever", measure, false);
        put(rhythm, "narrowSlotsRetriever", retriever);
        List<?> narrow = (List<?>) call(retriever, "buildSlots");
        List<?> wide = (List<?>) call(make("sheet.rhythm.SlotsRetriever", measure, true), "buildSlots");
        List<?> previous = (List<?>) invoke(rhythm, "buildCompoundSlots", narrow, wide);
        boolean separate = name.equals("separate-member-times") || name.equals("overlapping-continuation");
        List<Object> members = separate
                ? List.of(make("sheet.rhythm.Slot$MeasureSlot", 3, measure, List.of(chords.get(2))),
                          make("sheet.rhythm.Slot$MeasureSlot", 4, measure, List.of(chords.get(3))))
                : List.of(make("sheet.rhythm.Slot$MeasureSlot", 3, measure, chords.subList(2, 4)));
        Object current = make("sheet.rhythm.Slot$CompoundSlot", 3, measure, members);
        ((List) field(rhythm, "slots")).addAll(List.of(previous.get(0), previous.get(1), current));
        for (int i = 0; i < 2; i++) {
            Object voice = make("sheet.rhythm.Voice", chords.get(i), measure);
            call(measure, "addVoice", voice);
        }
        call(chords.get(0), "setTimeOffset", make("math.Rational", 0, 1));
        call(chords.get(1), "setTimeOffset", make("math.Rational", name.equals("agreeing-times") ? 0 : 1,
                name.equals("overlapping-continuation") ? 8 : 4));
        var ctor = c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(
                c("sheet.rhythm.MeasureRhythm"), c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper = ctor.newInstance(rhythm, current);
        invoke(mapper, "purgeRookiesSet");
        Object input = mapperInput(mapper, rhythm, measure, chords, current);
        Object ok = call(mapper, "mapChords");
        List<Object> states = new ArrayList<>(), voices = new ArrayList<>();
        for (int i = 0; i < chords.size(); i++) {
            Object voice = call(chords.get(i), "getVoice");
            states.add(obj("id", i, "voice", voice == null ? null : call(voice, "getId"),
                    "time_offset", fraction(call(chords.get(i), "getTimeOffset"))));
        }
        for (Object voice : (Collection<?>) call(measure, "getVoices")) {
            voices.add(obj("id", call(voice, "getId"), "chord_ids", ids((Collection<?>) call(voice, "getChords"), chords)));
        }
        return obj("name", name, "input", input, "output", mapperOutput(mapper, current, chords),
                "ok", ok, "abnormal", call(measure, "isAbnormal"), "chord_states", states, "voices", voices);
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        List<Object> controls = new ArrayList<>();
        for (String name : List.of("agreeing-times", "conflicting-times", "separate-member-times"))
            controls.add(inspectCompound(measure, name));
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "actual saved four-chord measure: full SlotMapper.mapChords with two rookie chords and two prior Voices; declared compound members, not automatic image polyphony",
                        "controls", controls));
        System.exit(0);
    }
}
