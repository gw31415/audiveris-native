// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class MappingRetryProbe extends RookieMappingProbe {
    static List<Object> pairs(Object set, List<Object> chords) throws Exception {
        List<Object> result = new ArrayList<>();
        for (Object pair : (Collection<?>) set) {
            result.add(obj("rookie", chords.indexOf(field(pair, "rookie")),
                    "active", chords.indexOf(field(pair, "active"))));
        }
        return result;
    }
    static Object mappingState(Object mapping, List<Object> chords) throws Exception {
        return mapping == null ? null : obj("cost", field(mapping, "cost"),
                "pairs", pairs(field(mapping, "pairs"), chords));
    }
    static Object inspectControl(Object measure, String name) throws Exception {
        call(measure, "resetRhythm");
        Object sys = call(call(measure, "getStack"), "getSystem");
        Object sig = call(sys, "getSig");
        List<Object> chords = new ArrayList<>((Collection<?>) call(measure, "getStandardChords"));
        Object rhythm = make("sheet.rhythm.MeasureRhythm", measure);
        Object retriever = make("sheet.rhythm.SlotsRetriever", measure, false);
        put(rhythm, "narrowSlotsRetriever", retriever);
        List<?> narrow = (List<?>) call(retriever, "buildSlots");
        List<?> wide = (List<?>) call(make("sheet.rhythm.SlotsRetriever", measure, true), "buildSlots");
        List<?> slots = (List<?>) invoke(rhythm, "buildCompoundSlots", narrow, wide);
        ((List) field(rhythm, "slots")).addAll(slots);
        Object current = slots.get(2);
        Object member = ((List<?>) call(current, "getMembers")).get(0);
        for (int i = 0; i < 2; i++) {
            Object voice = make("sheet.rhythm.Voice", chords.get(i), measure);
            call(measure, "addVoice", voice);
        }
        boolean late = name.contains("late");
        Object known = make("math.Rational", name.equals("exhausted") ? 3 : late ? 7 : 1,
                name.equals("exhausted") ? 4 : late ? 8 : 4);
        call(chords.get(0), "setTimeOffset", make("math.Rational", 0, 1));
        call(chords.get(1), "setTimeOffset", make("math.Rational", late ? 5 : 1, late ? 8 : 4));
        call(chords.get(2), "setTimeOffset", known);
        call(member, "setTimeOffset", known);
        Object added = null;
        if (name.startsWith("explicit-next")) {
            added = make("sig.relation.NextInVoiceRelation");
            call(sig, "addEdge", chords.get(1), chords.get(2), added);
        }
        var ctor = c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(
                c("sheet.rhythm.MeasureRhythm"), c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper = ctor.newInstance(rhythm, current);
        invoke(mapper, "purgeRookiesSet");
        List<?> rookies = (List<?>) field(mapper, "rookies");
        List<?> actives = (List<?>) invoke(mapper, "retrieveActives");
        List<Object> costs = new ArrayList<>(), activeEnds = new ArrayList<>(), durations = new ArrayList<>();
        for (Object a : actives) activeEnds.add(obj("id", chords.indexOf(a), "end", fraction(call(a, "getEndTime"))));
        for (Object r : rookies) {
            durations.add(obj("id", chords.indexOf(r), "duration", fraction(call(r, "getDuration"))));
            for (Object a : actives) costs.add(obj("rookie", chords.indexOf(r), "active", chords.indexOf(a),
                    "distance", call(field(rhythm, "voiceDistance"), "getDistance", a, r, (Object) null)));
        }
        Object input = obj("rookies", ids(rookies, chords), "actives", ids(actives, chords), "costs", costs,
                "black", pairs(field(mapper, "blackList"), chords), "next", pairs(field(mapper, "nextList"), chords),
                "narrow", List.of(obj("id", call(member, "getId"), "chord_ids", ids((Collection<?>) call(member, "getChords"), chords),
                        "time_offset", fraction(call(member, "getTimeOffset")))),
                "active_ends", activeEnds, "rookie_durations", durations,
                "expected", fraction(call(call(measure, "getStack"), "getExpectedDuration")));
        invoke(mapper, "mapRookies");
        Object output = obj("mapping", mappingState(field(mapper, "mapping"), chords),
                "black", pairs(field(mapper, "blackList"), chords),
                "remaining_rookies", ids((Collection<?>) field(mapper, "rookies"), chords),
                "narrow", List.of(obj("id", call(member, "getId"), "time_offset", fraction(call(member, "getTimeOffset")))));
        Object voice = call(chords.get(2), "getVoice");
        Object finalChord = obj("voice", voice == null ? null : call(voice, "getId"),
                "time_offset", fraction(call(chords.get(2), "getTimeOffset")));
        if (added != null) call(sig, "removeEdge", added);
        return obj("name", name, "input", input, "output", output, "original_rookie_state", finalChord);
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        List<Object> controls = new ArrayList<>();
        for (String name : List.of("switch-active", "exhausted", "late-ending", "explicit-next-late", "explicit-next-conflict")) {
            controls.add(inspectControl(measure, name));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "five declared original mapRookies known-time controls on actual saved chord/Voice objects; not automatic image polyphony",
                        "controls", controls));
        System.exit(0);
    }
}
