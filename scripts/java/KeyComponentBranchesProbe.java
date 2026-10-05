// SPDX-License-Identifier: AGPL-3.0-or-later
// Original raster -> GRID -> original clef column -> original key browse/projection.
// Original component extraction/classification/assignment; stops before hard-slice fallback and clef/key consistency.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;

public class KeyComponentBranchesProbe extends GridFinalProbe {
    private final Object book;
    public KeyComponentBranchesProbe(Book b, SortedSet<Integer> ids) { super(b, ids); book = b; }
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
    static String canonicalGlyph(Object g) throws Exception {
        if (g == null) return "null";
        return glyph(g).replaceFirst("\\\"id\\\":-?[0-9]+,", "");
    }
    static String evaluation(Object eval) throws Exception {
        if (eval == null) return "null";
        return "{\"shape\":\"" + get(eval, "shape") + "\",\"grade\":" + get(eval, "grade") + "}";
    }
    static Object makeTable(Object buffer) throws Exception {
        Class<?> orientation = Class.forName("org.audiveris.omr.run.Orientation");
        Object factory = Class.forName("org.audiveris.omr.run.RunTableFactory")
            .getConstructor(orientation).newInstance(orientation.getField("VERTICAL").get(null));
        for (Method method : factory.getClass().getMethods()) {
            if (method.getName().equals("createTable") && method.getParameterCount() == 1
                    && method.getParameterTypes()[0].isInstance(buffer)) return method.invoke(factory, buffer);
        }
        throw new NoSuchMethodException("RunTableFactory.createTable(buffer)");
    }
    static List<?> retainedParts(Object extractor, Object range, Object roi) throws Exception {
        // Read-side replay of the original extraction preamble, using original methods only.
        // Candidate extraction/assignment below runs the original end-to-end methods separately.
        Object buffer = call(roi, "getAreaPixels", get(extractor, "staffFreeSource"), range);
        Object table = makeTable(buffer);
        Object origin = new java.awt.Point((Integer) call(range, "getStart"), (Integer) get(roi, "y"));
        List<?> parts = (List<?>) Class.forName("org.audiveris.omr.glyph.GlyphFactory")
            .getMethod("buildGlyphs", Class.forName("org.audiveris.omr.run.RunTable"), java.awt.Point.class)
            .invoke(null, table, origin);
        call(extractor, "purgeParts", parts, call(range, "getStop"));
        return parts;
    }
    static List<Object> candidatesBeforePurge(Object extractor, Object roi, List<?> peaks, List<?> retained, Object shape) throws Exception {
        Class<?> simpleGraph = Class.forName("org.jgrapht.graph.SimpleGraph");
        Class<?> clusterClass = Class.forName("org.audiveris.omr.glyph.GlyphCluster");
        call(get(extractor, "system"), "registerGlyphs", retained, null);
        Object graph = Class.forName("org.audiveris.omr.glyph.Glyphs")
            .getMethod("buildLinks", Collection.class, double.class)
            .invoke(null, retained, get(get(extractor, "params"), "maxPartGap"));
        Object inspector = Class.forName("org.jgrapht.alg.connectivity.ConnectivityInspector")
            .getConstructor(Class.forName("org.jgrapht.Graph")).newInstance(graph);
        Constructor<?> adapterConstructor = Class.forName("org.audiveris.omr.sheet.key.KeyExtractor$MultipleAdapter")
            .getDeclaredConstructor(Class.forName("org.audiveris.omr.sheet.key.KeyExtractor"), Class.forName("org.audiveris.omr.sheet.key.KeyRoi"),
                List.class, simpleGraph, Set.class, double.class);
        adapterConstructor.setAccessible(true);
        double minGrade = (Double) Class.forName("org.audiveris.omr.glyph.Grades").getField("keyAlterMinGrade1").get(null);
        List<Object> result = new ArrayList<>();
        for (Object set : (List<?>) call(inspector, "connectedSets")) {
            Object subGraph = clusterClass.getMethod("getSubGraph", Set.class, simpleGraph, boolean.class).invoke(null, set, graph, false);
            Object adapter = adapterConstructor.newInstance(extractor, roi, peaks, subGraph, Collections.singleton(shape), minGrade);
            Object cluster = clusterClass.getConstructor(Class.forName("org.audiveris.omr.glyph.GlyphCluster$GlyphAdapter"),
                Class.forName("org.audiveris.omr.glyph.GlyphGroup")).newInstance(adapter, null);
            call(cluster, "decompose");
            result.addAll((List<?>) get(adapter, "candidates"));
        }
        return result;
    }
    static List<String> serializeCandidates(List<?> candidates, List<String> parts, Object roi) throws Exception {
        List<?> slices = (List<?>) roi;
        List<String> result = new ArrayList<>();
        for (Object candidate : candidates) {
            List<Integer> membership = new ArrayList<>();
            for (Object part : (Set<?>) get(candidate, "parts")) {
                int index = parts.indexOf(canonicalGlyph(part));
                if (index < 0) throw new IllegalStateException("Candidate part not in retained part geometry");
                membership.add(index);
            }
            Collections.sort(membership);
            Object candidateGlyph = get(candidate, "glyph");
            java.awt.Point centroid = (java.awt.Point) call(candidateGlyph, "getCentroid");
            Object slice = call(roi, "sliceOf", centroid.x);
            result.add("{\"glyph\":" + canonicalGlyph(candidateGlyph) + ",\"part_indices\":" + membership
                + ",\"centroid\":[" + centroid.x + "," + centroid.y + "],\"slice_index\":" + slices.indexOf(slice)
                + ",\"eval\":" + evaluation(get(candidate, "eval")) + "}");
        }
        return result;
    }
    static String components(Object builder, Object shapeBuilder, Object shape) throws Exception {
        Object extractor = get(builder, "extractor"), range = get(shapeBuilder, "range"), roi = get(shapeBuilder, "roi");
        List<?> slices = (List<?>) roi;
        if (slices.isEmpty()) return "{\"attempted\":false,\"retained_parts\":[],\"candidates_after_overlap_purge\":[],\"slice_assignments\":[]}";
        List<?> retained = retainedParts(extractor, range, roi);
        List<String> parts = new ArrayList<>();
        for (Object part : retained) parts.add(canonicalGlyph(part));
        List<?> peaks = (List<?>) get(shapeBuilder, "peaks");
        List<Object> rawCandidates = candidatesBeforePurge(extractor, roi, peaks, retained, shape);
        List<String> beforePurge = serializeCandidates(rawCandidates, parts, roi);
        call(extractor, "purgeCandidates", rawCandidates);
        List<String> replayAfterPurge = serializeCandidates(rawCandidates, parts, roi);
        List<?> candidates = (List<?>) call(extractor, "retrieveCandidates", range, roi, peaks, Collections.singleton(shape));
        List<String> candidateJson = serializeCandidates(candidates, parts, roi);
        if (!replayAfterPurge.equals(candidateJson)) throw new IllegalStateException("Original-method replay disagrees with complete original retrieveCandidates");
        // Original connected-component assignment; it calls original retrieveCandidates again.
        call(extractor, "retrieveComponents", range, roi, peaks, shape);
        List<String> assigned = new ArrayList<>();
        int empty = 0;
        for (Object slice : slices) {
            Object alter = call(slice, "getAlter");
            if (alter == null) empty++;
            String alterJson = alter == null ? "null" : "{\"shape\":\"" + call(alter, "getShape")
                + "\",\"grade\":" + call(alter, "getGrade") + ",\"bounds\":" + rect(call(alter, "getBounds"))
                + ",\"pitch\":" + call(alter, "getPitch") + ",\"measured_pitch\":" + call(alter, "getMeasuredPitch")
                + ",\"glyph\":" + canonicalGlyph(call(alter, "getGlyph")) + "}";
            assigned.add("{\"slice_index\":" + slices.indexOf(slice) + ",\"rect\":" + rect(call(slice, "getRect"))
                + ",\"eval\":" + evaluation(call(slice, "getEval")) + ",\"glyph\":" + canonicalGlyph(call(slice, "getGlyph"))
                + ",\"alter\":" + alterJson + "}");
        }
        Object params = get(extractor, "params");
        List<String> parameters = new ArrayList<>();
        for (String name : List.of("maxPartCount", "maxEvalRank", "minPartWeight", "maxPartGap", "minGlyphWidth", "maxGlyphWidth", "minGlyphHeight", "maxGlyphHeight", "minGlyphWeight", "maxGlyphWeight"))
            parameters.add("\"" + name + "\":" + get(params, name));
        Class<?> grades = Class.forName("org.audiveris.omr.glyph.Grades");
        parameters.add("\"keyAlterMinGrade1\":" + grades.getField("keyAlterMinGrade1").get(null));
        parameters.add("\"intrinsicRatio\":" + grades.getField("intrinsicRatio").get(null));
        return "{\"attempted\":true,\"part_snapshot_method\":\"original extraction preamble replay\",\"params\":{" + String.join(",", parameters)
            + "},\"purge_replay_matches_direct\":true,\"purge_removed_count\":" + (beforePurge.size() - candidateJson.size()) + ",\"candidates_before_overlap_purge\":[" + String.join(",", beforePurge) + "],\"retained_parts\":[" + String.join(",", parts) + "],\"candidates_after_overlap_purge\":[" + String.join(",", candidateJson)
            + "],\"slice_assignments\":[" + String.join(",", assigned) + "],\"empty_slice_count_before_fallback\":" + empty + "}";
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
                        String componentJson = components(builder, shapeBuilder, entry.getKey());
                        shapes.add("{\"shape\":\"" + entry.getKey() + "\",\"inferred_signature\":" + inferred
                            + ",\"refined_signature\":" + refined + ",\"starts\":" + starts + ",\"slices\":[" + String.join(",", slices)
                            + "],\"components\":" + componentJson + ",\"final_state\":" + snapshot(shapeBuilder, "allocateSlices") + ",\"checkpoints\":[" + String.join(",", checkpoints) + "]}");
                    }
                    peakInfo += ",\"shapes\":[" + String.join(",", shapes) + "]";
                    staffs.add("{\"staff_id\":" + call(staff, "getId") + ",\"system_id\":" + call(system, "getId")
                        + ",\"measure_start\":" + measureStart + ",\"clef_stop\":" + clefStop + ",\"header_stop\":" + headerStop
                        + peakInfo + ",\"global_width\":" + globalWidth + ",\"browse_start\":" + browseStart + ",\"browse_stop\":" + browseStop
                        + ",\"raw_browse_rect\":" + rect(rawRect) + ",\"clamped_browse_rect\":" + rect(safeRect)
                        + ",\"projection\":{\"x_min\":" + xMin + ",\"x_max\":" + xMax + ",\"values\":" + values + "}}");
                }
            }
            Class<?> shapeClass = Class.forName("org.audiveris.omr.glyph.Shape");
            Class<?> pitchedClass = Class.forName("org.audiveris.omr.sig.inter.AbstractPitchedInter");
            double flatOffset = (Double) pitchedClass.getMethod("getAreaPitchOffset", shapeClass).invoke(null, shapeClass.getField("FLAT").get(null));
            double sharpOffset = (Double) pitchedClass.getMethod("getAreaPitchOffset", shapeClass).invoke(null, shapeClass.getField("SHARP").get(null));
            Field constantsField = Class.forName("org.audiveris.omr.sig.inter.AlterInter").getDeclaredField("constants");
            constantsField.setAccessible(true);
            double flatMassOffset = (Double) call(get(constantsField.get(null), "flatMassPitchOffset"), "getValue");
            Files.writeString(Path.of(System.getProperty("native.probe.output")), "{\"stage\":\"connected_components_before_hard_slice_fallback\",\"hard_slice_fallback_performed\":false,\"key_clef_consistency_performed\":false,\"flat_mass_pitch_offset\":" + flatMassOffset + ",\"area_pitch_offsets\":{\"FLAT\":" + flatOffset + ",\"SHARP\":" + sharpOffset + "},\"staffs\":[" + String.join(",", staffs) + "]}\n");
        } catch (Exception e) { throw new RuntimeException(e); }
    }
}
