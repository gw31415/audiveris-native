// SPDX-License-Identifier: AGPL-3.0-or-later
// Original deleteWeakInters on declared stems, using an existing original
// createStem grade; this is not an automatic raster correction comparison.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class WeakStemPurgeProbe extends BeamOriginDependencyProbe {
    public static void main(String[] args) throws Exception {
        var mapper = new ObjectMapper();
        var creation = mapper.readTree(Path.of(args[1]).toFile()).get("cases").get(48);
        double weakGrade = creation.get("first").get("grade").asDouble();
        var bounds = creation.get("registered").get("bounds");
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases = new ArrayList<>();
        for (String name : List.of("weak-original-created-stem", "exact-threshold", "frozen-weak")) {
            Object book = c("sheet.Book").getMethod("loadBook", Path.class)
                    .invoke(null, Path.of(args[0]));
            Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
            Object system = ((List<?>) call(sheet, "getSystems")).get(0);
            Object sig = call(system, "getSig");
            int width = bounds.get(2).asInt(), height = bounds.get(3).asInt();
            Object image = Class.forName("ij.process.ByteProcessor")
                    .getConstructor(int.class, int.class, byte[].class)
                    .newInstance(width, height, new byte[width * height]);
            Object factory = c("run.RunTableFactory").getConstructor(c("run.Orientation"))
                    .newInstance(c("run.Orientation").getField("VERTICAL").get(null));
            Object glyph = c("glyph.Glyph").getConstructor(int.class, int.class, c("run.RunTable"))
                    .newInstance(bounds.get(0).asInt(), bounds.get(1).asInt(), call(factory, "createTable", image));
            double grade = name.equals("exact-threshold") ? .5 : weakGrade;
            Object stem = c("sig.inter.StemInter").getConstructor(c("glyph.Glyph"), Double.class)
                    .newInstance(glyph, grade);
            call(sig, "addVertex", stem);
            call(stem, "setContextualGrade", grade);
            if (name.equals("frozen-weak")) call(stem, "freeze");
            Collection<?> removed = (Collection<?>) call(sig, "deleteWeakInters");
            cases.add(obj("name", name, "grade", grade, "frozen", call(stem, "isFrozen"),
                    "selected_for_removal", removed.contains(stem),
                    "same_object_removed", call(stem, "isRemoved"),
                    "still_in_SIG", call(sig, "containsVertex", stem),
                    "physical_glyph_reference_retained", call(stem, "getGlyph") == glyph));
        }
        mapper.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[2]).toFile(),
                obj("scope", "three declared original live-SIG conditions; weak grade from original createStem fixture, not native output or automatic raster correction",
                        "original_creation_case", 48, "cases", cases));
        System.exit(0);
    }
}
