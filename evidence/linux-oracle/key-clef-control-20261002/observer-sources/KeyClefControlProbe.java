// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only, constructed numerical controls against the UNMODIFIED JAR.
// No raster detections, classifier parity, graph registration, or fillMissingAlters claim.
import java.awt.Rectangle;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import sun.misc.Unsafe;

public class KeyClefControlProbe {
    static final Class<?> BUILDER = cls("org.audiveris.omr.sheet.key.KeyBuilder");
    static final Class<?> SHAPE_BUILDER = cls("org.audiveris.omr.sheet.key.KeyBuilder$ShapeBuilder");
    static final Class<?> PARAMS = cls("org.audiveris.omr.sheet.key.KeyBuilder$Parameters");
    static final Class<?> STAFF = cls("org.audiveris.omr.sheet.Staff");
    static final Class<?> SHAPE = cls("org.audiveris.omr.glyph.Shape");
    static final Class<?> KIND = cls("org.audiveris.omr.sig.inter.ClefInter$ClefKind");
    static final Class<?> CLEF = cls("org.audiveris.omr.sig.inter.ClefInter");
    static final Class<?> GLYPH = cls("org.audiveris.omr.glyph.Glyph");
    static final Class<?> ALTER = cls("org.audiveris.omr.sig.inter.KeyAlterInter");
    static final Class<?> ROI = cls("org.audiveris.omr.sheet.key.KeyRoi");
    static final Class<?> SLICE = cls("org.audiveris.omr.sheet.key.KeySlice");
    static final Class<?> KEY = cls("org.audiveris.omr.sig.inter.KeyInter");
    static final Class<?> GRADE = cls("org.audiveris.omr.sig.GradeUtil");
    static Unsafe unsafe;
    static double max1, max4, clefRatio, alterRatio;
    static final List<String> KINDS = List.of("TREBLE", "BASS", "BARITONE", "TENOR", "ALTO", "MEZZO_SOPRANO", "SOPRANO");
    static Class<?> cls(String name) { try { return Class.forName(name); } catch (Exception e) { throw new RuntimeException(e); } }
    static Field field(Class<?> cls, String name) throws Exception {
        for (Class<?> c = cls; c != null; c = c.getSuperclass()) {
            try { Field f = c.getDeclaredField(name); f.setAccessible(true); return f; }
            catch (NoSuchFieldException ignored) {}
        }
        throw new NoSuchFieldException(name);
    }
    static Object get(Object o, String name) throws Exception { return field(o.getClass(), name).get(o); }
    static void set(Object o, String name, Object value) throws Exception { field(o.getClass(), name).set(o, value); }
    static Object alloc(Class<?> c) throws Exception { return unsafe.allocateInstance(c); }
    static Object enumValue(Class<?> c, String name) throws Exception { return c.getField(name).get(null); }
    static Object invoke(Object o, String name, Class<?>[] types, Object... args) throws Exception {
        Method m = o.getClass().getDeclaredMethod(name, types); m.setAccessible(true); return m.invoke(o, args);
    }
    static int[] pitches(String kind, String shape) throws Exception {
        return (int[]) KEY.getMethod("getPitches", KIND, SHAPE).invoke(null, enumValue(KIND, kind), enumValue(SHAPE, shape));
    }
    static double contribution(double grade, double ratio) throws Exception {
        return (Double) GRADE.getMethod("contributionOf", double.class, double.class).invoke(null, grade, ratio);
    }
    static double contextual(double grade, double support) throws Exception {
        return (Double) GRADE.getMethod("contextual", double.class, double.class).invoke(null, grade, support);
    }
    static Map<String,Object> map(Object... values) {
        Map<String,Object> result = new LinkedHashMap<>();
        for (int i = 0; i < values.length; i += 2) result.put((String) values[i], values[i + 1]);
        return result;
    }
    static Map<String,Object> alter(double grade, double pitch) { return map("intrinsic_grade", grade, "measured_pitch", pitch); }
    static Map<String,Object> clef(String kind, double grade) { return map("kind", kind, "intrinsic_grade", grade); }
    static List<Map<String,Object>> exact(String kind, String shape, int n, double grade) throws Exception {
        List<Map<String,Object>> result = new ArrayList<>(); int[] p = pitches(kind, shape);
        for (int i = 0; i < n; i++) result.add(alter(grade, p[i]));
        return result;
    }
    static class ObservedDecision extends RuntimeException {}
    static class RecordingClefs extends ArrayList<Object> {
        Object retained;
        @Override public boolean retainAll(Collection<?> selected) {
            if (selected.size() != 1) throw new AssertionError("Expected one original-method winner");
            retained = selected.iterator().next();
            // Stop before fillMissingAlters, applyPitchImpact, createKeyInter, addEdge.
            throw new ObservedDecision();
        }
    }
    @SuppressWarnings("unchecked")
    static Map<String,Object> observe(String name, String shapeName, List<Map<String,Object>> slices,
                                       List<Map<String,Object>> clefInputs) throws Exception {
        Object shape = enumValue(SHAPE, shapeName), staff = alloc(STAFF), builder = alloc(BUILDER), params = alloc(PARAMS);
        set(staff, "id", 1); set(builder, "staff", staff); set(builder, "params", params);
        set(params, "maxDeltaPitch_1", max1); set(params, "maxDeltaPitch_4", max4);
        Object shapeBuilder = alloc(SHAPE_BUILDER);
        set(shapeBuilder, "this$0", builder); set(shapeBuilder, "keyShape", shape);
        Object roi = ROI.getConstructor(STAFF, SHAPE, int.class, int.class, int.class).newInstance(staff, shape, 0, 10, 1);
        set(shapeBuilder, "roi", roi);
        for (int i = 0; i < slices.size(); i++) {
            Object slice = SLICE.getConstructor(Rectangle.class, ROI).newInstance(new Rectangle(i * 10, 0, 8, 10), roi);
            ((List<Object>) roi).add(slice);
            Map<String,Object> a = slices.get(i);
            if (a != null) {
                Object instance = ALTER.getConstructor(GLYPH, SHAPE, Double.class, STAFF, Double.class, Double.class)
                    .newInstance(null, shape, a.get("intrinsic_grade"), null, a.get("measured_pitch"), a.get("measured_pitch"));
                // Original remove() is a no-op on these detached controls. No real graph is modified.
                set(instance, "removed", true); set(slice, "alter", instance);
            }
        }
        RecordingClefs clefs = new RecordingClefs();
        List<Map<String,Object>> candidates = new ArrayList<>();
        Integer bestCompatible = null; double bestCompatibleCtx = 0;
        for (int i = 0; i < clefInputs.size(); i++) {
            Map<String,Object> c = clefInputs.get(i); String kindName = (String) c.get("kind"); Object kind = enumValue(KIND, kindName);
            Constructor<?> ctor = CLEF.getDeclaredConstructor(GLYPH, SHAPE, Double.class, STAFF, KIND); ctor.setAccessible(true);
            Object instance = ctor.newInstance(null, get(kind, "shape"), c.get("intrinsic_grade"), null, kind);
            // ClefInter.remove(bool) needs real staff/graph otherwise. It returns immediately here.
            // This field is never consulted by either scoring method or the two selection loops.
            set(instance, "removed", true); clefs.add(instance);
            double[] grades = new double[slices.size()]; int count = 0; Double keyGrade = null, clefCtx = null;
            boolean excluded = kindName.equals("PERCUSSION");
            if (!excluded) {
                count = (Integer) invoke(shapeBuilder, "computePitchedGrades", new Class<?>[]{KIND, double[].class}, kind, grades);
                if (count > 0) {
                    keyGrade = (Double) invoke(shapeBuilder, "computeKeyGrade", new Class<?>[]{int.class, double[].class}, count, grades);
                    clefCtx = contextual((Double) c.get("intrinsic_grade"), contribution(keyGrade, clefRatio));
                    // Source-replayed local selection only; final bestClef is independently captured below.
                    if (clefCtx > bestCompatibleCtx) { bestCompatibleCtx = clefCtx; bestCompatible = i; }
                }
            }
            candidates.add(map("kind", kindName, "percussion_excluded", excluded, "alter_count", count,
                "pitched_grades", grades, "key_grade", keyGrade, "contextual_grade", clefCtx));
        }
        set(builder, "clefs", clefs);
        Boolean returned = null; boolean intercepted = false;
        try { returned = (Boolean) invoke(shapeBuilder, "checkWithClefs", new Class<?>[]{}); }
        catch (InvocationTargetException ex) {
            if (ex.getCause() instanceof ObservedDecision) intercepted = true;
            else throw ex;
        }
        Integer bestClef = clefs.retained == null ? null : clefs.indexOf(clefs.retained);
        if (intercepted != (bestClef != null)) throw new AssertionError("Bad capture");
        if (returned != null && returned) throw new AssertionError("Unexpected post-selection execution");
        boolean accepted = bestCompatible != null && bestCompatible.equals(bestClef);
        return map("name", name, "shape", shapeName, "slices", slices, "clefs", clefInputs,
            "expected", map("candidates", candidates, "best_compatible_index", bestCompatible,
                "best_compatible_contextual_grade", bestCompatible == null ? null : bestCompatibleCtx,
                "best_clef_index", bestClef, "key_accepted", accepted),
            "observation", map("check_with_clefs_return", returned, "stopped_at_original_retain_all", intercepted,
                "original_selected_clef_index", bestClef));
    }
    static String json(Object value) {
        if (value == null) return "null";
        if (value instanceof String s) return "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"") + "\"";
        if (value instanceof Number || value instanceof Boolean) return value.toString();
        if (value instanceof Map<?,?> m) {
            List<String> parts = new ArrayList<>(); for (var e : m.entrySet()) parts.add(json(e.getKey()) + ":" + json(e.getValue()));
            return "{" + String.join(",", parts) + "}";
        }
        if (value instanceof Iterable<?> list) { List<String> parts = new ArrayList<>(); for (Object v : list) parts.add(json(v)); return "[" + String.join(",", parts) + "]"; }
        if (value.getClass().isArray()) { List<String> parts = new ArrayList<>(); for (int i = 0; i < Array.getLength(value); i++) parts.add(json(Array.get(value, i))); return "[" + String.join(",", parts) + "]"; }
        throw new IllegalArgumentException(value.getClass().getName());
    }
    public static void main(String[] args) throws Exception {
        Field f = Unsafe.class.getDeclaredField("theUnsafe"); f.setAccessible(true); unsafe = (Unsafe) f.get(null);
        Object constants = field(BUILDER, "constants").get(null);
        max1 = (Double) get(constants, "maxDeltaPitch_1").getClass().getMethod("getValue").invoke(get(constants, "maxDeltaPitch_1"));
        max4 = (Double) get(constants, "maxDeltaPitch_4").getClass().getMethod("getValue").invoke(get(constants, "maxDeltaPitch_4"));
        Object clefRel = cls("org.audiveris.omr.sig.relation.ClefKeyRelation").getConstructor().newInstance();
        Object alterRel = cls("org.audiveris.omr.sig.relation.KeyAltersRelation").getConstructor().newInstance();
        clefRatio = (Double) clefRel.getClass().getMethod("getSourceRatio").invoke(clefRel);
        alterRatio = (Double) alterRel.getClass().getMethod("getSourceRatio").invoke(alterRel);
        List<Map<String,Object>> cases = new ArrayList<>(); List<Map<String,Object>> arrays = new ArrayList<>();
        for (String shape : List.of("SHARP", "FLAT")) {
            for (String kind : KINDS) {
                arrays.add(map("shape", shape, "kind", kind, "pitches", pitches(kind, shape)));
                for (int n = 1; n <= 7; n++) {
                    cases.add(observe("exact-" + shape + "-" + kind + "-" + n, shape, exact(kind, shape, n, 0.65), List.of(clef(kind, 0.4))));
                }
                List<Map<String,Object>> allClefs = new ArrayList<>();
                for (String candidate : KINDS) allClefs.add(clef(candidate, 0.2 + (KINDS.indexOf(candidate) * 0.07)));
                allClefs.add(clef("PERCUSSION", 0.8));
                cases.add(observe("all-clefs-" + shape + "-" + kind, shape, exact(kind, shape, 7, 0.55), allClefs));
            }
            String zeroPitchKind = shape.equals("SHARP") ? "BARITONE" : "TREBLE";
            for (int n : new int[]{1, 2, 3, 4, 5, 7}) {
                double threshold = n >= 4 ? max4 : max1 + ((max4 - max1) * (n - 1)) / 3;
                for (int sign : new int[]{-1, 1}) {
                    double boundary = sign * threshold;
                    for (double p : new double[]{Math.nextDown(boundary), boundary, Math.nextUp(boundary)}) {
                        List<Map<String,Object>> slices = exact(zeroPitchKind, shape, n, 0.7);
                        slices.set(0, alter(0.7, p));
                        cases.add(observe("boundary-" + shape + "-n" + n + "-pitch-" + Double.toHexString(p), shape, slices, List.of(clef(zeroPitchKind, 0.4))));
                    }
                }
            }
            cases.add(observe("empty-" + shape, shape, List.of(), List.of(clef("TREBLE", 0.6))));
            cases.add(observe("all-missing-" + shape, shape, Arrays.asList(null, null, null, null), List.of(clef("TREBLE", 0.6))));
            List<Map<String,Object>> sparse = exact("TREBLE", shape, 4, 0.6); sparse.set(1, null); sparse.set(3, null);
            cases.add(observe("missing-actual-count-two-" + shape, shape, sparse, List.of(clef("TREBLE", 0.4))));
            sparse = exact("TREBLE", shape, 3, 0.6); sparse.set(0, null); sparse.set(1, null);
            sparse.set(2, alter(0.6, pitches("TREBLE", shape)[2] + 1.25));
            cases.add(observe("missing-count-one-threshold-n3-" + shape, shape, sparse, List.of(clef("TREBLE", 0.4))));
            List<Map<String,Object>> partial = exact("TREBLE", shape, 4, 0.6); partial.set(2, alter(0.6, 100));
            cases.add(observe("early-reject-preserves-partial-" + shape, shape, partial, List.of(clef("TREBLE", 0.4))));
        }
        cases.add(observe("percussion-only", "SHARP", exact("TREBLE", "SHARP", 1, 0.8), List.of(clef("PERCUSSION", 0.9))));
        cases.add(observe("no-compatible", "SHARP", List.of(alter(0.8, 20)), List.of(clef("TREBLE", 0.6), clef("BASS", 0.7))));
        cases.add(observe("no-clefs", "SHARP", exact("TREBLE", "SHARP", 1, 0.8), List.of()));
        cases.add(observe("boosted-compatible-beats-stronger-incompatible", "SHARP", exact("TREBLE", "SHARP", 1, 0.8), List.of(clef("TREBLE", 0.4), clef("BASS", 0.65))));
        cases.add(observe("stronger-incompatible-wins", "SHARP", exact("TREBLE", "SHARP", 1, 0.2), List.of(clef("TREBLE", 0.2), clef("BASS", 0.8))));
        cases.add(observe("percussion-wins-by-intrinsic", "SHARP", exact("TREBLE", "SHARP", 1, 0.2), List.of(clef("TREBLE", 0.2), clef("PERCUSSION", 0.8))));
        cases.add(observe("compatible-tie-first", "SHARP", exact("TREBLE", "SHARP", 1, 0.8), List.of(clef("TREBLE", 0.4), clef("TREBLE", 0.4))));
        double tied = contextual(0.4, contribution(0.8, clefRatio));
        cases.add(observe("final-tie-compatible-first", "SHARP", exact("TREBLE", "SHARP", 1, 0.8), List.of(clef("TREBLE", 0.4), clef("BASS", tied))));
        cases.add(observe("final-tie-incompatible-first", "SHARP", exact("TREBLE", "SHARP", 1, 0.8), List.of(clef("BASS", tied), clef("TREBLE", 0.4))));
        cases.add(observe("zero-clef-grade-no-positive-context", "SHARP", exact("TREBLE", "SHARP", 1, 0.8), List.of(clef("TREBLE", 0.0))));
        cases.add(observe("zero-alter-grade-still-compatible", "SHARP", exact("TREBLE", "SHARP", 1, 0.0), List.of(clef("TREBLE", 0.4))));
        cases.add(observe("exact-threshold-zero-pitch-grade-still-compatible", "SHARP", List.of(alter(0.8, -3.5)), List.of(clef("TREBLE", 0.4))));
        Random random = new Random(5111002L);
        for (int i = 0; i < 80; i++) {
            String shape = i % 2 == 0 ? "SHARP" : "FLAT", kind = KINDS.get(i % 7); int n = 1 + random.nextInt(7); int[] ps = pitches(kind, shape);
            List<Map<String,Object>> slices = new ArrayList<>(), clefs = new ArrayList<>();
            for (int j = 0; j < n; j++) slices.add(random.nextInt(5) == 0 ? null : alter(random.nextDouble() * 0.8, ps[j] + (random.nextDouble() - 0.5) * 4.4));
            for (String ck : KINDS) clefs.add(clef(ck, random.nextDouble() * 0.8));
            Collections.shuffle(clefs, random); clefs.add(random.nextInt(clefs.size() + 1), clef("PERCUSSION", random.nextDouble() * 0.8));
            cases.add(observe("seeded-" + i, shape, slices, clefs));
        }
        Map<String,Object> result = map("controlled_input", true, "not_image_parity", true, "random_seed", 5111002,
            "methods_observed", List.of("KeyInter.getPitches", "KeyBuilder.ShapeBuilder.computePitchedGrades", "KeyBuilder.ShapeBuilder.computeKeyGrade", "GradeUtil.contributionOf", "GradeUtil.contextual", "KeyBuilder.ShapeBuilder.checkWithClefs through retainAll or false return"),
            "source_replayed_fields", List.of("best_compatible_index", "best_compatible_contextual_grade", "key_accepted for intercepted decisions"),
            "construction", "Original JAR objects; Unsafe only allocates incomplete KeyBuilder/ShapeBuilder/Parameters/Staff. Original KeyRoi, KeySlice, KeyAlterInter, ClefInter constructors are called. Detached inter removed flags avoid graph mutations. Recording clef list stops at original retainAll before fillMissingAlters/applyPitchImpact/createKeyInter.",
            "params", map("max_delta_pitch_1", max1, "max_delta_pitch_4", max4, "key_alters_source_ratio", alterRatio, "clef_key_source_ratio", clefRatio),
            "pitch_arrays", arrays, "cases", cases);
        Files.writeString(Path.of(args[0]), json(result) + "\n");
        System.out.println("Observed " + cases.size() + " constructed key-clef cases");
        System.exit(0);
    }
}
