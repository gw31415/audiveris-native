// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class NewVoiceProbe extends MappingRetryProbe {
    static Object inspectNewVoice(Object measure, boolean assigned) throws Exception {
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
        call(chords.get(0), "setTimeOffset", make("math.Rational", 3, 4));
        call(rookie, "setTimeOffset", make("math.Rational", 1, 4));
        call(((List<?>) call(current, "getMembers")).get(0), "setTimeOffset", make("math.Rational", 1, 4));
        if (assigned) call(rookie, "setVoice", voice);
        var ctor = c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(
                c("sheet.rhythm.MeasureRhythm"), c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper = ctor.newInstance(rhythm, current);
        invoke(mapper, "purgeRookiesSet");
        Object input = mapperInput(mapper, rhythm, measure, chords, current);
        invoke(mapper, "mapRookies");
        invoke(mapper, "createNewVoices");
        List<Object> voices = new ArrayList<>();
        for (Object v : (List<?>) call(measure, "getVoices")) voices.add(obj(
                "id", call(v, "getId"), "kind", String.valueOf(call(v, "getKind")),
                "starting_staff", call(call(v, "getStartingStaff"), "getId"),
                "chord_ids", ids((Collection<?>) call(v, "getChords"), chords)));
        return obj("name", assigned ? "already-voiced" : "known-time-unmapped",
                "input", input, "chord_bounds", chords.stream().map(ch -> {try {return rect(call(ch, "getBounds"));} catch(Exception e) {throw new RuntimeException(e);}}).toList(),
                "output", obj("voices", voices, "rookie_voice", call(call(rookie, "getVoice"), "getId"),
                        "rookie_time", fraction(call(rookie, "getTimeOffset")),
                        "abnormal", call(measure, "isAbnormal")));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "two declared original actual-object known-time createNewVoices controls; not automatic image polyphony",
                        "controls", List.of(inspectNewVoice(measure, false), inspectNewVoice(measure, true))));
        System.exit(0);
    }
}
