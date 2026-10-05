// SPDX-License-Identifier: AGPL-3.0-or-later
// Constructed graph controls; calls unmodified original Staff.getCompetingClefs.
import java.awt.Rectangle;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import sun.misc.Unsafe;

public class ClefCompetitionControlProbe {
    static Unsafe unsafe;
    static Class<?> cls(String name) throws Exception { return Class.forName(name); }
    static Field field(Class<?> cls, String name) throws Exception {
        for (Class<?> c = cls; c != null; c = c.getSuperclass()) {
            try { Field f = c.getDeclaredField(name); f.setAccessible(true); return f; }
            catch (NoSuchFieldException ignored) {}
        }
        throw new NoSuchFieldException(name);
    }
    static Object get(Object o, String n) throws Exception { return field(o.getClass(), n).get(o); }
    static void set(Object o, String n, Object v) throws Exception { field(o.getClass(), n).set(o, v); }
    static Map<String,Object> map(Object... v) {
        Map<String,Object> m = new LinkedHashMap<>(); for (int i = 0; i < v.length; i += 2) m.put((String)v[i], v[i+1]); return m;
    }
    static Map<String,Object> node(int id, int staff, int left) { return map("id", id, "staff_id", staff, "left", left, "is_clef", true); }
    static Map<String,Object> edge(int source, int target) { return map("source", source, "target", target); }
    static String json(Object v) {
        if (v == null) return "null";
        if (v instanceof String s) return "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"") + "\"";
        if (v instanceof Number || v instanceof Boolean) return v.toString();
        if (v instanceof Map<?,?> m) { List<String> a = new ArrayList<>(); for (var e : m.entrySet()) a.add(json(e.getKey()) + ":" + json(e.getValue())); return "{" + String.join(",", a) + "}"; }
        if (v instanceof Iterable<?> list) { List<String> a = new ArrayList<>(); for (Object i : list) a.add(json(i)); return "[" + String.join(",", a) + "]"; }
        throw new IllegalArgumentException(v.toString());
    }
    static Map<String,Object> observe(String name, List<Map<String,Object>> nodes, List<Map<String,Object>> edges, int targetStaff, int x) throws Exception {
        Class<?> staffClass = cls("org.audiveris.omr.sheet.Staff"), systemClass = cls("org.audiveris.omr.sheet.SystemInfo");
        Class<?> sigClass = cls("org.audiveris.omr.sig.SIGraph"), shapeClass = cls("org.audiveris.omr.glyph.Shape");
        Class<?> clefClass = cls("org.audiveris.omr.sig.inter.ClefInter"), interClass = cls("org.audiveris.omr.sig.inter.Inter");
        Class<?> alterClass = cls("org.audiveris.omr.sig.inter.KeyAlterInter"), glyphClass = cls("org.audiveris.omr.glyph.Glyph");
        Object system = unsafe.allocateInstance(systemClass);
        Constructor<?> sc = sigClass.getDeclaredConstructor(); sc.setAccessible(true); Object sig = sc.newInstance();
        set(sig, "system", system); set(system, "sig", sig);
        Object delegate = get(sig, "delegate");
        Map<Integer,Object> staffs = new LinkedHashMap<>(), inters = new LinkedHashMap<>();
        staffs.put(targetStaff, unsafe.allocateInstance(staffClass));
        for (Map<String,Object> n : nodes) {
            int id = (Integer)n.get("staff_id");
            if (!staffs.containsKey(id)) staffs.put(id, unsafe.allocateInstance(staffClass));
        }
        for (var s : staffs.entrySet()) { set(s.getValue(), "id", s.getKey()); set(s.getValue(), "system", system); }
        for (Map<String,Object> n : nodes) {
            Object inter;
            if ((Boolean)n.get("is_clef")) inter = clefClass.getConstructor(shapeClass, Double.class).newInstance(shapeClass.getField("G_CLEF").get(null), 0.5);
            else inter = alterClass.getConstructor(glyphClass, shapeClass, Double.class, staffClass, Double.class, Double.class)
                .newInstance(null, shapeClass.getField("SHARP").get(null), 0.5, null, 0.0, 0.0);
            set(inter, "id", n.get("id")); set(inter, "staff", staffs.get(n.get("staff_id")));
            set(inter, "bounds", new Rectangle((Integer)n.get("left"), 0, 10, 20)); set(inter, "sig", sig);
            // Populate actual original DirectedMultigraph directly: avoid unrelated sheet index,
            // graph listeners and ClefInter.added registration. Retrieval remains unmodified.
            delegate.getClass().getMethod("addVertex", Object.class).invoke(delegate, inter);
            inters.put((Integer)n.get("id"), inter);
        }
        List<Map<String,Object>> insertions = new ArrayList<>(), effectiveEdges = new ArrayList<>();
        Class<?> exclusion = cls("org.audiveris.omr.sig.relation.Exclusion");
        Constructor<?> ec = exclusion.getDeclaredConstructor(); ec.setAccessible(true);
        for (Map<String,Object> e : edges) {
            boolean accepted = false; String error = null;
            try { accepted = (Boolean) delegate.getClass().getMethod("addEdge", Object.class, Object.class, Object.class)
                .invoke(delegate, inters.get(e.get("source")), inters.get(e.get("target")), ec.newInstance()); }
            catch (InvocationTargetException ex) { error = ex.getCause().getClass().getName() + ": " + ex.getCause().getMessage(); }
            insertions.add(map("source", e.get("source"), "target", e.get("target"), "accepted", accepted, "error", error));
            if (accepted) effectiveEdges.add(e);
        }
        List<?> result = (List<?>) staffClass.getMethod("getCompetingClefs", int.class).invoke(staffs.get(targetStaff), x);
        List<Integer> ids = new ArrayList<>(); for (Object inter : result) ids.add((Integer) interClass.getMethod("getId").invoke(inter));
        List<Map<String,Object>> incident = new ArrayList<>();
        if (!result.isEmpty()) {
            Object first = result.get(0);
            for (Object e : (Set<?>) sigClass.getMethod("getExclusions", interClass).invoke(sig, first)) {
                Object source = sigClass.getMethod("getEdgeSource", Object.class).invoke(sig, e);
                Object target = sigClass.getMethod("getEdgeTarget", Object.class).invoke(sig, e);
                incident.add(map("source", interClass.getMethod("getId").invoke(source), "target", interClass.getMethod("getId").invoke(target)));
            }
        }
        return map("name", name, "nodes", nodes, "edges", edges, "effective_edges", effectiveEdges,
            "target_staff_id", targetStaff, "x", x, "expected_ids", ids,
            "edge_insertions", insertions, "observed_incident_exclusions", incident);
    }
    public static void main(String[] args) throws Exception {
        Field f = Unsafe.class.getDeclaredField("theUnsafe"); f.setAccessible(true); unsafe = (Unsafe)f.get(null);
        List<Map<String,Object>> cases = new ArrayList<>();
        cases.add(observe("empty-graph", List.of(), List.of(), 1, 100));
        cases.add(observe("foreign-staff-only", List.of(node(1,2,10)), List.of(), 1, 100));
        List<Map<String,Object>> equal = List.of(node(3,1,20),node(1,1,10),node(2,1,20));
        for (int x : new int[]{9,10,11,20,21,100}) cases.add(observe("stable-left-sort-x" + x, equal, List.of(), 1, x));
        cases.add(observe("stable-left-reversed-equal", List.of(node(2,1,20),node(1,1,10),node(3,1,20)), List.of(), 1, 21));
        List<Map<String,Object>> nodes = List.of(node(10,1,100),node(20,1,10),node(30,1,20),node(40,1,30),node(50,1,40),node(60,2,200));
        cases.add(observe("mixed-direction-interleaved-order", nodes, List.of(edge(10,20),edge(30,10),edge(10,40),edge(50,10)), 1, 101));
        cases.add(observe("duplicate-parallel-and-reverse", nodes, List.of(edge(10,20),edge(10,20),edge(20,10),edge(30,10),edge(10,40)), 1, 101));
        cases.add(observe("foreign-neighbor-filtered", nodes, List.of(edge(10,60),edge(60,10),edge(10,20),edge(30,10)), 1, 101));
        cases.add(observe("self-edge-construction", nodes, List.of(edge(10,10),edge(10,20),edge(30,10)), 1, 101));
        cases.add(observe("future-neighbor-retained", List.of(node(1,1,10),node(2,1,200)), List.of(edge(1,2)), 1, 20));
        cases.add(observe("x-exact-tied-left-partners-retained", List.of(node(1,1,10),node(2,1,20),node(3,1,20),node(4,1,30)), List.of(edge(1,2),edge(3,1),edge(1,4)), 1, 20));
        Map<String,Object> nonClef = node(7,1,500); nonClef.put("is_clef",false);
        cases.add(observe("non-clef-neighbor-filtered", List.of(node(1,1,10),node(2,1,20),nonClef), List.of(edge(7,2),edge(2,1)), 1, 100));
        Random random = new Random(5111012L);
        for (int i = 0; i < 60; i++) {
            List<Map<String,Object>> ns = new ArrayList<>(), es = new ArrayList<>(); int count = 1 + random.nextInt(12);
            for (int j = 1; j <= count; j++) ns.add(node(j, 1 + random.nextInt(3), random.nextInt(6) * 10));
            Collections.shuffle(ns, random);
            for (int j = 0; j < count * 2; j++) es.add(edge(1 + random.nextInt(count), 1 + random.nextInt(count)));
            cases.add(observe("seeded-" + i, ns, es, 1 + random.nextInt(3), random.nextInt(7) * 10));
        }
        Files.writeString(Path.of(args[0]), json(map("controlled_input",true,"not_image_parity",true,"random_seed",5111012,
            "methods_observed",List.of("Staff.getCompetingClefs", "SIGraph.getExclusions", "DirectedMultigraph.addEdge"),
            "construction","Original detached inter objects and original SIGraph DirectedMultigraph delegate; direct graph insertion avoids unrelated sheet-index and inter-added mutations. No selection algorithm is replayed.",
            "cases",cases)) + "\n");
        System.out.println("Observed " + cases.size() + " constructed clef competition cases"); System.exit(0);
    }
}
