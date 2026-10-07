// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class RestAugmentationStateProbe extends RookieMappingProbe {
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        List<Object> measures = new ArrayList<>();
        for (Object system : (List<?>) call(sheet, "getSystems")) {
            for (Object stack : (List<?>) call(system, "getStacks")) {
                Object measure = call(stack, "getFirstMeasure");
                Object finalCheck = invoke(make("sheet.rhythm.MeasureRhythm", measure), "finalCheck");
                List<Object> chords = new ArrayList<>();
                for (Object chord : (Collection<?>) call(measure, "getStandardChords")) {
                    List<Object> notes = new ArrayList<>();
                    for (Object note : (Collection<?>) call(chord, "getNotes"))
                        notes.add(obj("bounds", rect(call(note, "getBounds")),
                                "step", String.valueOf(call(note, "getStep")),
                                "octave", call(note, "getOctave"),
                                "shape", String.valueOf(call(note, "getShape")),
                                "actual_chord_owner", call(note, "getChord") == chord));
                    chords.add(obj("notes", notes,
                            "highest_note_bounds", rect(call(call(chord, "getHighestNote"), "getBounds")),
                            "highest_note_absolute_pitch", call(call(chord, "getHighestNote"), "getAbsolutePitch"),
                            "heads_bounds", rect(call(chord, chord.getClass().getSimpleName().equals("HeadChordInter") ? "getHeadsBounds" : "getBounds")),
                            "bounds", rect(call(chord, "getBounds")),
                            "voice", call(call(chord, "getVoice"), "getId"),
                            "time_offset", fraction(call(chord, "getTimeOffset")),
                            "duration", fraction(call(chord, "getDuration"))));
                }
                List<Object> voices = new ArrayList<>();
                for (Object voice : (Collection<?>) call(measure, "getVoices"))
                    voices.add(obj("id", call(voice, "getId"),
                            "chord_bounds", ((Collection<?>) call(voice, "getChords")).stream().map(ch -> {
                                try { return rect(call(ch, "getBounds")); } catch (Exception ex) { throw new RuntimeException(ex); }
                            }).toList(), "termination", fraction(call(voice, "getTermination")),
                            "excess", fraction(field(voice, "excess"))));
                measures.add(obj("chords", chords, "voices", voices,
                        "final_check_ok", finalCheck,
                        "stack", StackSlotsProbe.state(measure),
                        "expected", fraction(call(stack, "getExpectedDuration")),
                        "actual", fraction(call(stack, "getActualDuration")),
                        "stack_excess", fraction(call(stack, "getExcess"))));
            }
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "saved recognition state after unmodified original full image/export; original finalCheck recomputes nonpersisted termination after reload; no declared mapping/clock changes",
                        "measures", measures));
        System.exit(0);
    }
}
