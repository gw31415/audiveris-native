// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class UnknownClockProbe extends MappingRetryProbe {
    static Object inspectUnknown(Object measure, boolean known) throws Exception {
        call(measure, "resetRhythm");
        List<Object> chords = new ArrayList<>((Collection<?>) call(measure, "getStandardChords"));
        Object rhythm = make("sheet.rhythm.MeasureRhythm", measure);
        Object retriever = make("sheet.rhythm.SlotsRetriever", measure, false);
        put(rhythm, "narrowSlotsRetriever", retriever);
        List<?> narrow = (List<?>) call(retriever, "buildSlots");
        List<?> wide = (List<?>) call(make("sheet.rhythm.SlotsRetriever", measure, true), "buildSlots");
        List<?> slots = (List<?>) invoke(rhythm, "buildCompoundSlots", narrow, wide);
        ((List) field(rhythm, "slots")).addAll(slots);
        Object current = slots.get(2), rookie = chords.get(2);
        Object voice = make("sheet.rhythm.Voice", chords.get(0), measure);
        call(measure, "addVoice", voice);
        if (known) call(((List<?>) call(current, "getMembers")).get(0), "setTimeOffset", make("math.Rational", 1, 4));
        var ctor = c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(
                c("sheet.rhythm.MeasureRhythm"), c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper = ctor.newInstance(rhythm, current);
        invoke(mapper, "purgeRookiesSet");
        Object input = mapperInput(mapper, rhythm, measure, chords, current);
        invoke(mapper, "mapRookies");
        Map<String,Object> output = (Map<String,Object>) mapperOutput(mapper, current, chords);
        output.put("measure_abnormal", call(measure, "isAbnormal"));
        Object finalVoice = call(rookie, "getVoice");
        List<Object> voices = new ArrayList<>();
        for (Object v : (List<?>) call(measure, "getVoices")) voices.add(obj(
                "id", call(v, "getId"), "kind", String.valueOf(call(v, "getKind")),
                "starting_staff", call(call(v, "getStartingStaff"), "getId"),
                "chord_ids", ids((Collection<?>) call(v, "getChords"), chords)));
        return obj("name", known ? "known-narrow-unknown-active" : "unknown-narrow-unknown-active",
                "input", input, "output", output,
                "original_owner_state", obj("voices", voices,
                        "rookie_voice", finalVoice == null ? null : call(finalVoice, "getId"),
                        "rookie_time", fraction(call(rookie, "getTimeOffset")),
                        "abnormal", call(measure, "isAbnormal")),
                "original_rookie_state", obj("voice", finalVoice == null ? null : call(finalVoice, "getId"),
                        "time_offset", fraction(call(rookie, "getTimeOffset"))));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "two declared original mapRookies unknown-active-clock controls; abnormal/unknown state is not successful MusicXML",
                        "controls", List.of(inspectUnknown(measure, true), inspectUnknown(measure, false))));
        System.exit(0);
    }
}
