// SPDX-License-Identifier: AGPL-3.0-or-later
// Live snapshot after original GRID and HeaderBuilder.processHeader, before persistence/reload.
// No native recognition output, injected candidates, or copied expected values.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;

public class HeadersLiveProbe extends GridFinalProbe {
    private final Object book;
    public HeadersLiveProbe(Book b, SortedSet<Integer> ids) { super(b, ids); book = b; }
    static String selected(Object inter) throws Exception {
        if (inter == null) return "null";
        Object glyph = call(inter, "getGlyph");
        String extra = inter.getClass().getSimpleName().equals("ClefInter")
            ? ",\"clef_kind\":\"" + call(inter, "getKind") + "\",\"pitch\":" + call(inter, "getPitch") : "";
        if (inter.getClass().getSimpleName().equals("KeyInter")) extra += ",\"fifths\":" + call(inter, "getFifths");
        if (Class.forName("org.audiveris.omr.sig.inter.AbstractTimeInter").isInstance(inter)) extra += ",\"numerator\":" + call(inter, "getNumerator") + ",\"denominator\":" + call(inter, "getDenominator");
        Object shape = call(inter, "getShape");
        String shapeJson = shape == null ? "null" : "\"" + shape + "\"";
        return "{\"id\":" + call(inter, "getId") + ",\"kind\":\"" + inter.getClass().getSimpleName()
            + "\",\"shape\":" + shapeJson + ",\"grade\":" + call(inter, "getGrade")
            + ",\"contextual_grade\":" + call(inter, "getContextualGrade") + ",\"bounds\":" + rect(call(inter, "getBounds"))
            + ",\"frozen\":" + call(inter, "isFrozen") + ",\"glyph\":" + (glyph == null ? "null" : glyph(glyph)) + extra + "}";
    }
    static String range(Object range) throws Exception {
        if (range == null) return "null";
        return "{\"valid\":" + get(range, "valid") + ",\"browse_start\":" + get(range, "browseStart")
            + ",\"browse_stop\":" + get(range, "browseStop") + ",\"start\":" + get(range, "start")
            + ",\"stop\":" + get(range, "stop") + "}";
    }
    public void process() {
        try {
            Object stub = ((List<?>) call(book, "getStubs")).get(0);
            Object sheet = call(stub, "getSheet");
            call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
            for(Object system:(List<?>)call(sheet,"getSystems")) {
                Object builder=Class.forName("org.audiveris.omr.sheet.header.HeaderBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(system);
                call(builder,"processHeader");
            }
            List<String> staffs = new ArrayList<>(), systems = new ArrayList<>();
            for (Object staff : (List<?>) call(call(sheet, "getStaffManager"), "getStaves")) {
                Object header = call(staff, "getHeader");
                staffs.add("{\"staff_id\":" + call(staff, "getId") + ",\"system_id\":" + call(call(staff, "getSystem"), "getId")
                    + ",\"start\":" + get(header, "start") + ",\"stop\":" + get(header, "stop")
                    + ",\"clef\":" + selected(get(header, "clef")) + ",\"key\":" + selected(get(header, "key"))
                    + ",\"time\":" + selected(get(header, "time")) + ",\"clef_range\":" + range(get(header, "clefRange"))
                    + ",\"key_range\":" + range(get(header, "keyRange")) + ",\"time_range\":" + range(get(header, "timeRange")) + "}");
            }
            for (Object system : (List<?>) call(sheet, "getSystems")) {
                Object sig = call(system, "getSig");
                List<String> inters = new ArrayList<>(), relations = new ArrayList<>();
                for (Object inter : (Collection<?>) call(sig, "vertexSet")) inters.add(selected(inter));
                for (Object relation : (Collection<?>) call(sig, "edgeSet")) relations.add("{\"kind\":\"" + relation.getClass().getSimpleName()
                    + "\",\"source\":" + call(call(sig, "getEdgeSource", relation), "getId")
                    + ",\"target\":" + call(call(sig, "getEdgeTarget", relation), "getId") + "}");
                systems.add("{\"id\":" + call(system, "getId") + ",\"inters\":[" + String.join(",", inters)
                    + "],\"relations\":[" + String.join(",", relations) + "]}");
            }
            Files.writeString(Path.of(System.getProperty("native.probe.output")), "{\"live_header_builder_complete\":true,\"persisted_or_reloaded\":false,\"staffs\":["
                + String.join(",", staffs) + "],\"systems\":[" + String.join(",", systems) + "]}\n");
        } catch (Exception e) { throw new RuntimeException(e); }
    }
}
