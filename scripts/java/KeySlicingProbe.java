// SPDX-License-Identifier: AGPL-3.0-or-later
// Original raster -> GRID -> original clef column -> original key browse/projection.
// Calls original retrieveHiLoPeaks only; no key classification or time recognition.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;

public class KeySlicingProbe extends GridFinalProbe {
    private final Object book;
    public KeySlicingProbe(Book b, SortedSet<Integer> ids) { super(b, ids); book = b; }
    static String snapshot(Object shapeBuilder, String phase) throws Exception {
        Object range = get(shapeBuilder, "range");
        List<String> peaks = new ArrayList<>();
        for (Object peak : (List<?>) get(shapeBuilder, "peaks")) peaks.add("{\"min\":" + get(peak, "min")
            + ",\"main\":" + get(peak, "main") + ",\"max\":" + get(peak, "max")
            + ",\"height\":" + get(peak, "height") + ",\"area\":" + get(peak, "area") + "}");
        return "{\"phase\":\"" + phase + "\",\"range\":{\"browse_start\":" + get(range, "browseStart")
            + ",\"browse_stop\":" + get(range, "browseStop") + ",\"start\":" + get(range, "start")
            + ",\"stop\":" + get(range, "stop") + "},\"peaks\":[" + String.join(",", peaks) + "]}";
    }
    public void process() {
        try {
            Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
            call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(), "doit", sheet);
            Class<?> staffClass = Class.forName("org.audiveris.omr.sheet.Staff");
            Class<?> columnClass = Class.forName("org.audiveris.omr.sheet.key.KeyColumn");
            Constructor<?> ctor = Class.forName("org.audiveris.omr.sheet.key.KeyBuilder")
                .getDeclaredConstructor(columnClass, staffClass, int.class, int.class, int.class, boolean.class);
            ctor.setAccessible(true);
            List<String> staffs = new ArrayList<>();
            for (Object system : (List<?>) call(sheet, "getSystems")) {
                Object headerBuilder = Class.forName("org.audiveris.omr.sheet.header.HeaderBuilder")
                    .getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(system);
                call(headerBuilder, "computeHeaderStarts");
                Object clefColumn = get(headerBuilder, "clefColumn");
                int clefOffset = (Integer) call(clefColumn, "retrieveClefs");
                call(headerBuilder, "setSystemClefStop", clefOffset);
                int globalWidth = (Integer) get(headerBuilder, "maxHeaderWidth");
                Object keyColumn = get(headerBuilder, "keyColumn");
                for (Object staff : (List<?>) call(system, "getStaves")) {
                    int measureStart = (Integer) call(staff, "getHeaderStart");
                    int headerStop = (Integer) call(staff, "getHeaderStop");
                    Integer clefStop = (Integer) call(staff, "getClefStop");
                    int browseStart = (clefStop != null ? clefStop : headerStop) + 1;
                    Object builder = ctor.newInstance(keyColumn, staff, globalWidth, measureStart, browseStart, true);
                    int browseStop = (Integer) call(staff, "getBrowseStop", browseStart, measureStart + globalWidth);
                    Object rawRect = call(builder, "getBrowseRect", browseStart, browseStop);
                    Object safeRect = call(sheet, "clamp", rawRect);
                    Object projection = get(builder, "projection");
                    int xMin = (Integer) call(projection, "getXMin");
                    int xMax = (Integer) call(projection, "getXMax");
                    List<Integer> values = new ArrayList<>();
                    for (int x = xMin; x <= xMax; x++) values.add((Integer) call(projection, "getValue", x));
                    List<String> peaks = new ArrayList<>();
                    List<?> hilos = (List<?>) call(builder, "retrieveHiLoPeaks");
                    for (Object peak : hilos) peaks.add("{\"min\":" + get(peak, "min") + ",\"main\":" + get(peak, "main") + ",\"max\":" + get(peak, "max") + "}");
                    Object params = get(builder, "params");
                    String peakInfo = ",\"params\":{\"min_peak_cumul\":" + get(params, "minPeakCumul")
                        + ",\"min_value\":" + ((Integer) get(params, "maxSpaceCumul") + 1)
                        + ",\"min_peak_derivative\":" + get(params, "minPeakDerivative")
                        + ",\"min_gain_ratio\":" + get(params, "minGainRatio") + "},\"peaks\":[" + String.join(",", peaks) + "]";
                    List<String> shapes = new ArrayList<>();
                    for (var entry : ((Map<?,?>) get(builder, "shapeBuilders")).entrySet()) {
                        Object shapeBuilder = entry.getValue();
                        List<String> checkpoints = new ArrayList<>();
                        call(shapeBuilder, "browseArea", hilos);
                        checkpoints.add(snapshot(shapeBuilder, "browseArea"));
                        for (String phase : List.of("refineAreaStart", "mergePeaks", "purgeLightPeaks")) {
                            call(shapeBuilder, phase);
                            checkpoints.add(snapshot(shapeBuilder, phase));
                        }
                        int inferred = (Integer) call(shapeBuilder, "inferSignature");
                        checkpoints.add(snapshot(shapeBuilder, "inferSignature"));
                        int refined = (Integer) call(shapeBuilder, "refineSignature", inferred);
                        checkpoints.add(snapshot(shapeBuilder, "refineSignature"));
                        List<?> starts = (List<?>) call(shapeBuilder, "computeStarts", refined);
                        if (!starts.isEmpty()) call(shapeBuilder, "allocateSlices", starts);
                        List<String> slices = new ArrayList<>();
                        for (Object slice : (List<?>) get(shapeBuilder, "roi")) slices.add(rect(call(slice, "getRect")));
                        shapes.add("{\"shape\":\"" + entry.getKey() + "\",\"inferred_signature\":" + inferred
                            + ",\"refined_signature\":" + refined + ",\"starts\":" + starts + ",\"slices\":[" + String.join(",", slices)
                            + "],\"final_state\":" + snapshot(shapeBuilder, "allocateSlices") + ",\"checkpoints\":[" + String.join(",", checkpoints) + "]}");
                    }
                    peakInfo += ",\"shapes\":[" + String.join(",", shapes) + "]";
                    staffs.add("{\"staff_id\":" + call(staff, "getId") + ",\"system_id\":" + call(system, "getId")
                        + ",\"measure_start\":" + measureStart + ",\"clef_stop\":" + clefStop + ",\"header_stop\":" + headerStop
                        + peakInfo + ",\"global_width\":" + globalWidth + ",\"browse_start\":" + browseStart + ",\"browse_stop\":" + browseStop
                        + ",\"raw_browse_rect\":" + rect(rawRect) + ",\"clamped_browse_rect\":" + rect(safeRect)
                        + ",\"projection\":{\"x_min\":" + xMin + ",\"x_max\":" + xMax + ",\"values\":" + values + "}}");
                }
            }
            Files.writeString(Path.of(System.getProperty("native.probe.output")), "{\"staffs\":[" + String.join(",", staffs) + "]}\n");
        } catch (Exception e) { throw new RuntimeException(e); }
    }
}
