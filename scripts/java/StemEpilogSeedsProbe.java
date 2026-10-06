// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only observation of unmodified original finalizeBeams on saved HEADS.
import java.nio.file.Path;
import java.awt.geom.Area;
import java.awt.geom.Rectangle2D;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class StemEpilogSeedsProbe extends BeamOriginDependencyProbe {
    static List<Object> glyphs(Collection<?> values) throws Exception {
        List<Object> out = new ArrayList<>();
        for (Object g : values) out.add(glyph(g));
        return out;
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class)
                .invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object group = c("glyph.GlyphGroup").getField("VERTICAL_SEED").get(null);
        List<Object> systems = new ArrayList<>();
        for (Object sys : (List<?>) call(sheet, "getSystems")) {
            List<?> grouped = (List<?>) call(sys, "getGroupedGlyphs", group);
            Object r = c("sheet.stem.StemsRetriever").getConstructor(c("sheet.SystemInfo"))
                    .newInstance(sys);
            call(r, "finalizeBeams");
            List<Object> areas = new ArrayList<>();
            for (Object a : (List<?>) get(r, "noStemAreas")) {
                Rectangle2D b = ((Area) a).getBounds2D();
                areas.add(new double[]{b.getX(), b.getY(), b.getWidth(), b.getHeight()});
            }
            systems.add(obj("system", call(sys, "getId"), "areas", areas,
                    "before", glyphs(grouped), "after", glyphs((List<?>) get(r, "systemSeeds")),
                    "grouped_after", glyphs((List<?>) call(sys, "getGroupedGlyphs", group)),
                    "max_bar_overlap", get(get(r, "params"), "maxBarOverlap")));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("native_outputs_used", false, "scope", "fresh original finalizeBeams local seed list on saved HEADS; not full raster STEMS", "systems", systems));
        System.exit(0);
    }
}
