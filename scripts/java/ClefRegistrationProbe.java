// SPDX-License-Identifier: AGPL-3.0-or-later
// Observes the unmodified official GRID + header starts + ClefBuilder.findClefs.
// This deliberately stops before key/time recognition and final clef selection.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;

public class ClefRegistrationProbe extends GridFinalProbe {
    private final Object book;
    public ClefRegistrationProbe(Book b, SortedSet<Integer> ids) { super(b, ids); book = b; }
    static String canonicalGlyph(Object glyph) throws Exception {
        if (glyph == null) return "null";
        return glyph(glyph).replaceFirst("\\\"id\\\":-?[0-9]+,", "");
    }
    public void process() {
        try {
            Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
            call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(), "doit", sheet);
            headerPreparation(sheet);
            List<String> staffs = new ArrayList<>();
            for (Object staff : (List<?>) call(call(sheet, "getStaffManager"), "getStaves")) {
                Object builder = Class.forName("org.audiveris.omr.sheet.clef.ClefBuilder")
                    .getConstructor(Class.forName("org.audiveris.omr.sheet.Staff")).newInstance(staff);
                call(builder, "setBrowseStart", call(staff, "getHeaderStart"));
                call(builder, "findClefs");
                Object sig = call(call(staff, "getSystem"), "getSig");
                List<Object> clefs = new ArrayList<>();
                for (Object inter : (Collection<?>) call(sig, "vertexSet")) {
                    if (inter.getClass().getSimpleName().equals("ClefInter") && call(inter, "getStaff") == staff) clefs.add(inter);
                }
                // Stable descending intrinsic grade; insertion ties retain the official enum-map order.
                clefs.sort((a, b) -> {
                    try { return Double.compare((Double) call(b, "getGrade"), (Double) call(a, "getGrade")); }
                    catch (Exception e) { throw new RuntimeException(e); }
                });
                List<String> inters = new ArrayList<>(), exclusions = new ArrayList<>();
                for (Object inter : clefs) {
                    inters.add("{\"shape\":\"" + call(inter, "getShape") + "\",\"kind\":\"" + call(inter, "getKind")
                        + "\",\"grade\":" + call(inter, "getGrade") + ",\"pitch\":" + call(inter, "getPitch")
                        + ",\"bounds\":" + rect(call(inter, "getBounds")) + ",\"glyph\":" + canonicalGlyph(call(inter, "getGlyph")) + "}");
                }
                for (Object edge : (Collection<?>) call(sig, "edgeSet")) {
                    if (!edge.getClass().getSimpleName().equals("Exclusion")) continue;
                    int source = clefs.indexOf(call(sig, "getEdgeSource", edge));
                    int target = clefs.indexOf(call(sig, "getEdgeTarget", edge));
                    if (source >= 0 && target >= 0) exclusions.add("{\"source\":" + source + ",\"target\":" + target + ",\"cause\":\"" + get(edge, "cause") + "\"}");
                }
                staffs.add("{\"staff_id\":" + call(staff, "getId") + ",\"clef_stop\":" + call(staff, "getClefStop")
                    + ",\"inters\":[" + String.join(",", inters) + "],\"exclusions\":[" + String.join(",", exclusions) + "]}");
            }
            Files.writeString(Path.of(System.getProperty("native.probe.output")), "{\"staffs\":[" + String.join(",", staffs) + "]}\n");
        } catch (Exception e) { throw new RuntimeException(e); }
    }
}
