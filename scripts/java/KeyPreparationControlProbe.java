// SPDX-License-Identifier: AGPL-3.0-or-later
// Controlled calls on original raster-derived staff/graph and staff-free pixels.
// Grade changes/rectangles are explicit test input, not natural image detections.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.awt.Rectangle;

public class KeyPreparationControlProbe extends GridFinalProbe {
    private final Object book;
    public KeyPreparationControlProbe(Book b, SortedSet<Integer> ids) { super(b, ids); book = b; }
    public void process() {
        try {
            Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
            call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(), "doit", sheet);
            Object staff = ((List<?>) call(call(sheet, "getStaffManager"), "getStaves")).get(0);
            Object sig = call(call(staff, "getSystem"), "getSig");
            Class<?> relation = Class.forName("org.audiveris.omr.sig.relation.BarConnectionRelation");
            List<String> controls = new ArrayList<>();
            for (Object bar : (List<?>) call(staff, "getBarlines")) {
                boolean connected = (Boolean) call(sig, "hasRelation", bar, new Class<?>[]{relation});
                int start = ((Rectangle) call(bar, "getBounds")).x;
                double originalGrade = (Double) call(bar, "getGrade");
                for (double grade : new double[]{0.5999999999999999, 0.6, 0.6000000000000001}) {
                    call(bar, "setGrade", grade);
                    for (int[] bounds : new int[][]{{start - 1, start + 8}, {start, start + 8}, {start - 1, start}, {start - 2, start - 1}}) {
                        controls.add("{\"bar_start\":" + start + ",\"grade\":" + grade + ",\"connected\":" + connected + ",\"x_min\":" + bounds[0]
                            + ",\"x_max\":" + bounds[1] + ",\"observed_stop\":" + call(staff, "getBrowseStop", bounds[0], bounds[1]) + "}");
                    }
                }
                call(bar, "setGrade", originalGrade);
            }
            Object extractor = Class.forName("org.audiveris.omr.sheet.key.KeyExtractor")
                .getConstructor(Class.forName("org.audiveris.omr.sheet.Staff")).newInstance(staff);
            int width = (Integer) call(sheet, "getWidth"), height = (Integer) call(sheet, "getHeight");
            List<String> projections = new ArrayList<>();
            Rectangle[] rectangles = {new Rectangle(-5, -5, 25, 25), new Rectangle(width - 10, height - 10, 25, 25),
                                     new Rectangle(295, 70, 20, 95)};
            int[] starts = {0, width - 12, 290};
            for (int i = 0; i < rectangles.length; i++) {
                Object projection = call(extractor, "getProjection", starts[i], rectangles[i]);
                int xMin = (Integer) call(projection, "getXMin"), xMax = (Integer) call(projection, "getXMax");
                List<Integer> values = new ArrayList<>();
                for (int x = xMin; x <= xMax; x++) values.add((Integer) call(projection, "getValue", x));
                projections.add("{\"measure_start\":" + starts[i] + ",\"raw_rect\":" + rect(rectangles[i])
                    + ",\"safe_rect\":" + rect(call(sheet, "clamp", rectangles[i])) + ",\"x_min\":" + xMin
                    + ",\"x_max\":" + xMax + ",\"values\":" + values + "}");
            }
            Files.writeString(Path.of(System.getProperty("native.probe.output")), "{\"controlled_input\":true,\"staff_id\":" + call(staff, "getId")
                + ",\"browse_stop_cases\":[" + String.join(",", controls) + "],\"projection_cases\":[" + String.join(",", projections) + "]}\n");
        } catch (Exception e) { throw new RuntimeException(e); }
    }
}
