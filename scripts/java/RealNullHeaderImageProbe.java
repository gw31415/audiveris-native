// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.lang.reflect.Field;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;

public class RealNullHeaderImageProbe extends RealLedgerMembershipImageProbe {
    private final Object book;
    public RealNullHeaderImageProbe(Book book, SortedSet<Integer> ids) {
        super(book, ids);
        this.book = book;
    }
    static Object field(Object owner, String name) throws Exception {
        for (Class<?> type = owner.getClass(); type != null; type = type.getSuperclass()) {
            try {
                Field field = type.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(owner);
            } catch (NoSuchFieldException missing) {}
        }
        throw new NoSuchFieldException(name);
    }
    static String inter(Object value) throws Exception {
        if (value == null) return "null";
        return "{\"id\":" + call(value, "getId") + ",\"shape\":\""
            + call(value, "getShape") + "\",\"manual\":" + call(value, "isManual") + "}";
    }
    public void process() {
        try {
            Object stub = ((List<?>) call(book, "getStubs")).get(0);
            call(stub, "reachStep", c("step.OmrStep").getField("HEADERS").get(null), false);
            Object sheet = call(stub, "getSheet");
            Map<Object, Object[]> retained = new IdentityHashMap<>();
            for (Object system : (List<?>) call(sheet, "getSystems")) {
                for (Object staff : (List<?>) call(system, "getStaves")) {
                    retained.put(staff, new Object[] {call(staff, "getPart"), call(staff, "getHeader")});
                }
            }
            call(stub, "reachStep", c("step.OmrStep").getField("CHORDS").get(null), false);
            List<String> systems = new ArrayList<>();
            for (Object system : (List<?>) call(sheet, "getSystems")) {
                Object sig = call(system, "getSig");
                List<String> staves = new ArrayList<>();
                for (Object staff : (List<?>) call(system, "getStaves")) {
                    Object part = call(staff, "getPart"), header = call(staff, "getHeader");
                    Object[] before = retained.get(staff);
                    boolean retainedOwners = before != null && before[0] == part && before[1] == header;
                    if (!retainedOwners) throw new IllegalStateException("Staff/Part/Header identity changed");
                    List<String> chords = new ArrayList<>();
                    for (Object chord : (Collection<?>) call(sig, "inters", c("sig.inter.HeadChordInter"))) {
                        List<String> heads = new ArrayList<>();
                        Object measure = call(chord, "getMeasure");
                        for (Object head : (List<?>) call(chord, "getMembers")) {
                            if (call(head, "getStaff") != staff) continue;
                            boolean actual = call(chord, "getPart") == part && measure != null
                                && call(measure, "getPart") == part
                                && ((Collection<?>) call(measure, "getHeadChords")).contains(chord);
                            if (!actual) throw new IllegalStateException("Head/Chord/Part/Measure owner changed");
                            heads.add("{\"bounds\":" + rect(call(head, "getBounds"))
                                + ",\"shape\":\"" + call(head, "getShape")
                                + "\",\"pitch\":" + call(head, "getIntegerPitch")
                                + ",\"manual\":" + call(head, "isManual")
                                + ",\"clef_before\":" + inter(call(measure, "getClefBefore", call(head, "getCenter"), staff))
                                + ",\"same_actual_owners\":true}");
                        }
                        if (!heads.isEmpty()) {
                            chords.add("{\"bounds\":" + rect(call(chord, "getBounds"))
                                + ",\"measure_index\":" + (((List<?>) call(part, "getMeasures")).indexOf(measure) + 1)
                                + ",\"stack_index\":" + (((List<?>) call(system, "getStacks")).indexOf(call(measure, "getStack")) + 1)
                                + ",\"manual\":" + call(chord, "isManual")
                                + ",\"members\":[" + String.join(",", heads) + "]}");
                        }
                    }
                    staves.add("{\"staff\":" + call(staff, "getId") + ",\"part\":" + call(part, "getId")
                        + ",\"start\":" + call(staff, "getHeaderStart") + ",\"stop\":" + call(staff, "getHeaderStop")
                        + ",\"clef\":" + inter(field(header, "clef")) + ",\"key\":" + inter(field(header, "key"))
                        + ",\"time\":" + inter(field(header, "time"))
                        + ",\"header_inter_count\":" + ((Collection<?>) call(staff, "getHeaderInters")).size()
                        + ",\"same_HEADERS_Staff_Part_Header_objects\":true,\"chords\":[" + String.join(",", chords) + "]}");
                }
                systems.add("{\"system\":" + call(system, "getId") + ",\"staffs\":[" + String.join(",", staves) + "]}");
            }
            Files.writeString(Path.of(System.getProperty("native.probe.output")),
                "{\"scope\":\"Fresh unchanged original image through ordinary HEADERS then CHORDS; actual retained Staff/Part/Header and Head/Chord/Measure owners, no native input or inferred clef\",\"systems\":["
                + String.join(",", systems) + "]}\n");
        } catch (Exception error) {
            throw new RuntimeException(error);
        }
    }
}
