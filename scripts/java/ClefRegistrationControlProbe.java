// SPDX-License-Identifier: AGPL-3.0-or-later
// Controlled-stage evidence: artificial candidate grades/shapes on a genuine
// original-image glyph. Calls original createValid and private registerClefs.
// This does not claim natural image-derived ambiguity or classifier coverage.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;

public class ClefRegistrationControlProbe extends ClefRegistrationProbe {
    private final Object book;
    public ClefRegistrationControlProbe(Book b, SortedSet<Integer> ids) { super(b, ids); book = b; }
    public void process() {
        try {
            Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
            call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(), "doit", sheet);
            headerPreparation(sheet);
            Object staff = ((List<?>) call(call(sheet, "getStaffManager"), "getStaves")).get(0);
            Object sig = call(call(staff, "getSystem"), "getSig");
            Class<?> staffClass = Class.forName("org.audiveris.omr.sheet.Staff");
            Object builder = Class.forName("org.audiveris.omr.sheet.clef.ClefBuilder").getConstructor(staffClass).newInstance(staff);
            call(builder, "setBrowseStart", call(staff, "getHeaderStart"));
            call(builder, "findClefs");
            Object genuineGlyph = null;
            for (Object inter : new ArrayList<>((Collection<?>) call(sig, "vertexSet"))) {
                if (inter.getClass().getSimpleName().equals("ClefInter") && call(inter, "getStaff") == staff) {
                    if (call(inter, "getShape").toString().equals("G_CLEF")) genuineGlyph = call(inter, "getGlyph");
                    call(inter, "remove");
                }
            }
            if (genuineGlyph == null) throw new IllegalStateException("Requires a naturally recognized G_CLEF glyph");
            Class<?> clefClass = Class.forName("org.audiveris.omr.sig.inter.ClefInter");
            Class<?> glyphClass = Class.forName("org.audiveris.omr.glyph.Glyph");
            Class<?> shapeClass = Class.forName("org.audiveris.omr.glyph.Shape");
            List<Object> supplied = new ArrayList<>();
            String[] shapes = {"G_CLEF_8VB", "G_CLEF", "G_CLEF_8VA"};
            Double[] grades = {0.5, 0.8, 0.8};
            List<String> input = new ArrayList<>();
            for (int i = 0; i < shapes.length; i++) {
                Object inter = clefClass.getMethod("createValid", glyphClass, shapeClass, Double.class, staffClass)
                    .invoke(null, genuineGlyph, shapeClass.getField(shapes[i]).get(null), grades[i], staff);
                supplied.add(inter);
                input.add("{\"shape\":\"" + shapes[i] + "\",\"grade\":" + grades[i] + "}");
            }
            call(builder, "registerClefs", supplied);
            List<Object> registered = new ArrayList<>();
            for (Object inter : (Collection<?>) call(sig, "vertexSet")) if (supplied.contains(inter)) registered.add(inter);
            List<String> inters = new ArrayList<>(), exclusions = new ArrayList<>();
            for (Object inter : registered) inters.add("{\"shape\":\"" + call(inter, "getShape") + "\",\"kind\":\"" + call(inter, "getKind")
                + "\",\"grade\":" + call(inter, "getGrade") + ",\"pitch\":" + call(inter, "getPitch")
                + ",\"bounds\":" + rect(call(inter, "getBounds")) + ",\"glyph\":" + canonicalGlyph(call(inter, "getGlyph")) + "}");
            for (Object edge : (Collection<?>) call(sig, "edgeSet")) {
                if (!edge.getClass().getSimpleName().equals("Exclusion")) continue;
                int source = registered.indexOf(call(sig, "getEdgeSource", edge));
                int target = registered.indexOf(call(sig, "getEdgeTarget", edge));
                if (source >= 0 && target >= 0) exclusions.add("{\"source\":" + source + ",\"target\":" + target + ",\"cause\":\"" + get(edge, "cause") + "\"}");
            }
            Files.writeString(Path.of(System.getProperty("native.probe.output")), "{\"controlled_input\":true,\"scope\":\"Original registerClefs on supplied candidates, not natural ambiguity\",\"input_candidates\":["
                + String.join(",", input) + "],\"input_glyph\":" + canonicalGlyph(genuineGlyph) + ",\"staffs\":[{\"staff_id\":" + call(staff, "getId")
                + ",\"clef_stop\":" + call(staff, "getClefStop") + ",\"inters\":[" + String.join(",", inters)
                + "],\"exclusions\":[" + String.join(",", exclusions) + "]}]}\n");
        } catch (Exception e) { throw new RuntimeException(e); }
    }
}
