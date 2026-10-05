// SPDX-License-Identifier: AGPL-3.0-or-later
// Original raster -> GRID -> original clef column -> original key browse/projection.
// Does not run key peak retrieval, key classification, or time recognition.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;

public class KeyPreparationProbe extends GridFinalProbe {
    private final Object book;
    public KeyPreparationProbe(Book b, SortedSet<Integer> ids) { super(b, ids); book = b; }
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
                    staffs.add("{\"staff_id\":" + call(staff, "getId") + ",\"system_id\":" + call(system, "getId")
                        + ",\"measure_start\":" + measureStart + ",\"clef_stop\":" + clefStop + ",\"header_stop\":" + headerStop
                        + ",\"global_width\":" + globalWidth + ",\"browse_start\":" + browseStart + ",\"browse_stop\":" + browseStop
                        + ",\"raw_browse_rect\":" + rect(rawRect) + ",\"clamped_browse_rect\":" + rect(safeRect)
                        + ",\"projection\":{\"x_min\":" + xMin + ",\"x_max\":" + xMax + ",\"values\":" + values + "}}");
                }
            }
            Files.writeString(Path.of(System.getProperty("native.probe.output")), "{\"staffs\":[" + String.join(",", staffs) + "]}\n");
        } catch (Exception e) { throw new RuntimeException(e); }
    }
}
