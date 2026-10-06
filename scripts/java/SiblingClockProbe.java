// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class SiblingClockProbe extends MappingRetryProbe {
    static Object inspectSibling(Object measure, boolean equalKnown) throws Exception {
        call(measure, "resetRhythm");
        List<Object> chords = new ArrayList<>((Collection<?>) call(measure, "getStandardChords"));
        Object sig = call(call(call(measure, "getStack"), "getSystem"), "getSig");
        Object same = make("sig.relation.SameTimeRelation");
        call(sig, "addEdge", chords.get(2), chords.get(0), same);
        Object rhythm = make("sheet.rhythm.MeasureRhythm", measure);
        Object retriever = make("sheet.rhythm.SlotsRetriever", measure, false);
        put(rhythm, "narrowSlotsRetriever", retriever);
        call(retriever, "buildSlots");
        Object declared = make("sheet.rhythm.Slot$MeasureSlot", 3, measure, chords);
        Object current = make("sheet.rhythm.Slot$CompoundSlot", 3, measure, List.of(declared));
        ((List) field(rhythm, "slots")).add(current);
        Object v1 = make("sheet.rhythm.Voice", chords.get(0), measure);
        call(measure, "addVoice", v1);
        Object v2 = make("sheet.rhythm.Voice", chords.get(1), measure);
        call(measure, "addVoice", v2);
        call(chords.get(3), "setVoice", v1);
        if (equalKnown) call(chords.get(0), "setTimeOffset", make("math.Rational", 1, 2));
        call(chords.get(3), "setTimeOffset", make("math.Rational", 3, 4));
        var ctor = c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(
                c("sheet.rhythm.MeasureRhythm"), c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper = ctor.newInstance(rhythm, current);
        invoke(mapper, "purgeRookiesSet");
        List<?> siblings = (List<?>) invoke(mapper, "getOrderedSiblings", chords.get(2));
        List<Object> times = new ArrayList<>();
        for (Object ch : chords) times.add(fraction(call(ch, "getTimeOffset")));
        List<Object> relations = new ArrayList<>();
        for (int i : List.of(0,1,3)) relations.add(obj("a",2,"b",i,"rel",String.valueOf(call(retriever,"getRel",chords.get(2),chords.get(i)))));
        Object input = obj("relations",relations,"chord_bounds", chords.stream().map(ch->{try{return rect(call(ch,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),
                "slot_chord_ids", ids((Collection<?>) call(current,"getChords"),chords),
                "rookies", ids((Collection<?>) field(mapper,"rookies"),chords),
                "equal_pair", List.of(2,0), "times", times, "initial_voice_members", List.of(List.of(0,3),List.of(1)));
        invoke(mapper, "createNewVoices");
        List<Object> voices = new ArrayList<>();
        for (Object v : (List<?>) call(measure,"getVoices")) voices.add(obj("id",call(v,"getId"),"chord_ids",ids((Collection<?>)call(v,"getChords"),chords)));
        Object output = obj("ordered_siblings", ids((Collection<?>)siblings,chords), "voices",voices,
                "rookie_voice",call(call(chords.get(2),"getVoice"),"getId"),"rookie_time",fraction(call(chords.get(2),"getTimeOffset")));
        call(sig,"removeEdge",same);
        return obj("name",equalKnown?"farther-equal-before-known-close":"unknown-equals-skip-to-known-close", "input",input,"output",output);
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet = call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
        Object sys = ((List<?>)call(sheet,"getSystems")).get(0);
        Object measure = call(((List<?>)call(sys,"getStacks")).get(1),"getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope","two declared original createNewVoices/getOrderedSiblings controls on saved actual chord/SIG objects; not automatic image polyphony",
                        "controls",List.of(inspectSibling(measure,true),inspectSibling(measure,false))));
        System.exit(0);
    }
}
