// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.awt.Rectangle;
import java.awt.geom.Line2D;
import com.fasterxml.jackson.databind.ObjectMapper;

public class FlagChordStateProbe extends RookieMappingProbe {
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object system = ((List<?>) call(sheet, "getSystems")).get(0);
        Object sig = call(system, "getSig");
        List<Object> nodes = new ArrayList<>(), edges = new ArrayList<>(), flags = new ArrayList<>(), stems = new ArrayList<>();
        for (Object n : (Collection<?>) call(sig, "vertexSet")) {
            String kind = n.getClass().getSimpleName();
            if (!Set.of("StemInter", "HeadInter", "BeamInter", "BeamHookInter", "FlagInter").contains(kind)) continue;
            Map<String, Object> data = obj("id", call(n, "getId"), "kind", kind,
                    "shape", String.valueOf(call(n, "getShape")), "bounds", rect(call(n, "getBounds")),
                    "grade", call(n, "getGrade"), "contextual_grade", call(n, "getBestGrade"));
            if (kind.equals("StemInter")) {
                stems.add(n);
                Line2D line = (Line2D) call(n, "getMedian");
                data.put("median", List.of(line.getX1(), line.getY1(), line.getX2(), line.getY2()));
                data.put("origin", obj("system", call(system, "getId"), "phase", "STEM", "local", call(n, "getId")));
            }
            if (kind.equals("FlagInter")) { flags.add(n); data.put("value", call(n, "getValue")); }
            nodes.add(data);
        }
        for (Object e : (Collection<?>) call(sig, "edgeSet")) {
            String kind = e.getClass().getSimpleName();
            if (!Set.of("HeadStemRelation", "BeamStemRelation", "FlagStemRelation").contains(kind)) continue;
            java.awt.geom.Point2D p = (java.awt.geom.Point2D) call(e, "getExtensionPoint");
            Map<String, Object> data = obj("kind", kind, "source", call(call(sig, "getEdgeSource", e), "getId"),
                    "target", call(call(sig, "getEdgeTarget", e), "getId"),
                    "extension_point", p == null ? null : List.of(p.getX(), p.getY()));
            if (kind.equals("HeadStemRelation")) data.put("head_side", String.valueOf(call(e, "getHeadSide")));
            edges.add(data);
        }
        Comparator<Object> byX = Comparator.comparingInt(n -> { try { return ((Rectangle) call(n, "getBounds")).x; } catch (Exception e) { throw new RuntimeException(e); } });
        stems.sort(byX); flags.sort(byX);
        Object flag = flags.get(0);
        java.lang.reflect.Method lookup = c("sig.inter.AbstractFlagInter").getDeclaredMethod("lookupLink", List.class, int.class);
        lookup.setAccessible(true);
        Object link = lookup.invoke(flag, stems, 1);
        Object stem = link.getClass().getField("partner").get(link), relation = link.getClass().getField("relation").get(link);
        java.awt.geom.Point2D p = (java.awt.geom.Point2D) call(relation, "getExtensionPoint");
        Object control = obj("shape", String.valueOf(call(flag, "getShape")), "value", call(flag, "getValue"),
                "bounds", rect(call(flag, "getBounds")), "stem_id", call(stem, "getId"),
                "stem_direction", call(stem, "computeDirection"), "grade", call(relation, "getGrade"),
                "dx", call(relation, "getDx"), "dy", call(relation, "getDy"), "extension_point", List.of(p.getX(), p.getY()));
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("scope", "read-only original saved multi-head flag owners; one actual flag lookup control, distinct from full image export",
                        "interline", call(call(sheet, "getScale"), "getInterline"),
                        "max_stem", call(call(sheet, "getScale"), "getMaxStem"), "system", call(system, "getId"),
                        "snapshot", obj("inters", nodes, "relations", edges), "control", control));
        System.exit(0);
    }
}
