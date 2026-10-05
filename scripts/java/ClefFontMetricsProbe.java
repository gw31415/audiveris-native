// SPDX-License-Identifier: AGPL-3.0-or-later
// Generic shape/size measurements from pinned original Bravura and official code.
// No image fixture or native result is an input to this observer.
import java.util.*;
import java.nio.file.*;
import java.awt.*;
import java.awt.font.TextLayout;
import java.awt.geom.*;

public class ClefFontMetricsProbe {
    public static void main(String[] args) throws Exception {
        Class<?> mf = Class.forName("org.audiveris.omr.ui.symbol.MusicFont");
        Class<?> familyClass = Class.forName("org.audiveris.omr.ui.symbol.MusicFamily");
        Class<?> shapeClass = Class.forName("org.audiveris.omr.glyph.Shape");
        Class<?> pitched = Class.forName("org.audiveris.omr.sig.inter.AbstractPitchedInter");
        mf.getMethod("populateAllSymbols").invoke(null);
        Object family = familyClass.getField("Bravura").get(null);
        java.util.List<String> entries = new ArrayList<>();
        for (String name : new String[]{"F_CLEF", "G_CLEF", "G_CLEF_8VA", "G_CLEF_8VB", "C_CLEF", "PERCUSSION_CLEF"}) {
            Object shape = shapeClass.getField(name).get(null);
            double pitchOffset = (Double) pitched.getMethod("getAreaPitchOffset", shapeClass).invoke(null, shape);
            for (int interline = 1; interline <= 128; interline++) {
                Object fs = shapeClass.getMethod("getFontSymbolByInterline", familyClass, int.class).invoke(shape, family, interline);
                TextLayout layout = (TextLayout) GridProbe.call(fs, "getLayout");
                Rectangle2D bounds = layout.getBounds();
                // Match registerClefs exactly: point-size value is supplied as this second interline.
                int pointSize = (Integer) mf.getMethod("getPointSize", int.class).invoke(null, interline);
                Object centroidFs = shapeClass.getMethod("getFontSymbolByInterline", familyClass, int.class).invoke(shape, family, pointSize);
                Object symbol = GridProbe.get(centroidFs, "symbol");
                GridProbe.call(symbol, "getCentroid", new Rectangle(0, 0, 1000, 1000));
                Point2D centroid = (Point2D) GridProbe.get(symbol, "centroidOffset");
                if (!Double.isFinite(bounds.getWidth()) || !Double.isFinite(bounds.getHeight()) || bounds.getWidth() <= 0 || bounds.getHeight() <= 0)
                    throw new IllegalStateException("Nonpositive layout: " + name + " interline=" + interline + " bounds=" + bounds);
                entries.add("{\"shape\":\"" + name + "\",\"interline\":" + interline
                    + ",\"layout_x\":" + bounds.getX() + ",\"layout_y\":" + bounds.getY()
                    + ",\"layout_width\":" + bounds.getWidth() + ",\"layout_height\":" + bounds.getHeight()
                    + ",\"centroid_offset_x\":" + centroid.getX() + ",\"centroid_offset_y\":" + centroid.getY()
                    + ",\"centroid_font_interline\":" + pointSize + ",\"area_pitch_offset\":" + pitchOffset + "}");
            }
        }
        Files.writeString(Path.of(args[0]), "{\"family\":\"Bravura\",\"upstream_commit\":\"9e1e55cd2746037d059345881c53e6a6754bffbd\",\"jar_sha256\":\"9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5\",\"font_resource\":\"res/Bravura.otf\",\"font_sha256\":\"dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2\",\"entries\":[" + String.join(",", entries) + "]}\n");
        System.exit(0);
    }
}
