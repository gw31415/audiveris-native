// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.lang.reflect.Constructor;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;

public class RealNullPitchImageProbe extends RealNullHeaderImageProbe {
    private final Object book;
    public RealNullPitchImageProbe(Book book, SortedSet<Integer> ids) {
        super(book, ids);
        this.book = book;
    }
    static String nullable(Object value) { return value == null ? "null" : value.toString(); }
    static String heads(Object sheet) throws Exception {
        List<String> rows = new ArrayList<>();
        for (Object system : (List<?>) call(sheet, "getSystems")) {
            for (Object staff : (List<?>) call(system, "getStaves")) {
                if (field(call(staff, "getHeader"), "clef") != null) continue;
                for (Object head : (Collection<?>) call(call(system, "getSig"), "inters", c("sig.inter.HeadInter"))) {
                    if (call(head, "getStaff") != staff) continue;
                    Object chord = call(head, "getChord"), measure = call(chord, "getMeasure");
                    Object clef = call(measure, "getClefBefore", call(head, "getCenter"), staff);
                    Object key = call(measure, "getKeyBefore", staff);
                    rows.add("{\"system\":" + call(system, "getId") + ",\"staff\":" + call(staff, "getId")
                        + ",\"bounds\":" + rect(call(head, "getBounds")) + ",\"pitch\":" + call(head, "getPitch")
                        + ",\"clef_before\":" + inter(clef) + ",\"key_before\":" + inter(key)
                        + ",\"step\":\"" + call(head, "getStep") + "\",\"octave\":" + call(head, "getOctave")
                        + ",\"absolute_pitch\":" + call(head, "getAbsolutePitch")
                        + ",\"alter_with_declared_null_fifths\":" + call(head, "getAlteration", (Object) null)
                        + ",\"alter_with_declared_plus1_fifths\":" + call(head, "getAlteration", Integer.valueOf(1)) + "}");
                }
            }
        }
        return "[" + String.join(",", rows) + "]";
    }
    static String model(Object value) throws Exception {
        List<String> parts = new ArrayList<>();
        for (Object part : (List<?>) call(value, "getPart")) {
            List<String> measures = new ArrayList<>();
            for (Object measure : (List<?>) call(part, "getMeasure")) {
                List<String> clefs = new ArrayList<>(), keys = new ArrayList<>(), notes = new ArrayList<>();
                for (Object item : (List<?>) call(measure, "getNoteOrBackupOrForward")) {
                    if (item.getClass().getSimpleName().equals("Attributes")) {
                        for (Object clef : (List<?>) call(item, "getClef"))
                            clefs.add("{\"number\":" + nullable(call(clef, "getNumber"))
                                + ",\"sign\":\"" + call(clef, "getSign") + "\",\"line\":" + call(clef, "getLine") + "}");
                        for (Object key : (List<?>) call(item, "getKey"))
                            keys.add("{\"number\":" + nullable(call(key, "getNumber"))
                                + ",\"fifths\":" + call(key, "getFifths") + "}");
                    } else if (item.getClass().getSimpleName().equals("Note")) {
                        Object pitch = call(item, "getPitch");
                        notes.add("{\"staff\":" + nullable(call(item, "getStaff"))
                            + ",\"rest\":" + (call(item, "getRest") != null)
                            + ",\"step\":" + (pitch == null ? "null" : "\"" + call(pitch, "getStep") + "\"")
                            + ",\"octave\":" + (pitch == null ? "null" : call(pitch, "getOctave"))
                            + ",\"alter\":" + (pitch == null ? "null" : nullable(call(pitch, "getAlter")))
                            + ",\"duration\":" + nullable(call(item, "getDuration")) + "}");
                    }
                }
                measures.add("{\"number\":\"" + call(measure, "getNumber")
                    + "\",\"clefs\":[" + String.join(",", clefs) + "],\"keys\":[" + String.join(",", keys)
                    + "],\"notes\":[" + String.join(",", notes) + "]}");
            }
            parts.add("{\"id\":\"" + call(call(part, "getId"), "getId")
                + "\",\"measures\":[" + String.join(",", measures) + "]}");
        }
        return "[" + String.join(",", parts) + "]";
    }
    public void process() {
        try {
            Object stub = ((List<?>) call(book, "getStubs")).get(0);
            call(stub, "reachStep", c("step.OmrStep").getField("LINKS").get(null), false);
            Object sheet = call(stub, "getSheet");
            String links = heads(sheet);
            call(stub, "reachStep", c("step.OmrStep").getField("PAGE").get(null), false);
            String page = heads(sheet);
            Object score = ((List<?>) call(book, "getScores")).get(0);
            Constructor<?> constructor = c("score.PartwiseBuilder").getDeclaredConstructor(c("score.Score"));
            constructor.setAccessible(true);
            Object builder = constructor.newInstance(score);
            call(builder, "processScore");
            String model = model(field(builder, "scorePartwise"));
            Object current = field(builder, "current");
            List<String> finalKeys = new ArrayList<>();
            for (var entry : ((Map<?, ?>) field(current, "keys")).entrySet())
                finalKeys.add("{\"staff_index\":" + entry.getKey()
                    + ",\"fifths\":" + call(entry.getValue(), "getFifths") + "}");
            Collections.sort(finalKeys);
            Files.writeString(Path.of(System.getProperty("native.probe.output")),
                "{\"scope\":\"Fresh unchanged original PNG through LINKS and PAGE, then actual private PartwiseBuilder constructor/processScore on its real Score; null Staff clef getters and exported model/current key cache. Declared getAlteration(null/+1) calls are explicitly separate controls. No native inputs or fake Clef\",\"LINKS\":"
                    + links + ",\"PAGE\":" + page + ",\"exported_parts\":" + model
                    + ",\"final_logical_part_key_cache\":[" + String.join(",", finalKeys) + "]}\n");
        } catch (Exception error) {
            throw new RuntimeException(error);
        }
    }
}
