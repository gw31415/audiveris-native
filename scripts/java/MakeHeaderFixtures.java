// SPDX-License-Identifier: AGPL-3.0-or-later
// Synthetic independent score rasters, rendered with pinned official Bravura.
import java.awt.*;
import java.awt.image.BufferedImage;
import java.awt.font.TextLayout;
import java.nio.file.*;
import javax.imageio.ImageIO;

public class MakeHeaderFixtures {
    public static void main(String[] args) throws Exception {
        Path folder = Path.of(args[0]);
        Files.createDirectories(folder);
        Class<?> mf = Class.forName("org.audiveris.omr.ui.symbol.MusicFont");
        Class<?> family = Class.forName("org.audiveris.omr.ui.symbol.MusicFamily");
        Class<?> shape = Class.forName("org.audiveris.omr.glyph.Shape");
        mf.getMethod("populateAllSymbols").invoke(null);
        Object font = mf.getMethod("getBaseFont", family, int.class).invoke(null, family.getField("Bravura").get(null), 16);
        TextLayout clef = (TextLayout) mf.getMethod("layoutShapeByCode", shape).invoke(font, shape.getField("G_CLEF").get(null));
        for (String name : new String[]{"one-sharp", "three-sharps", "one-flat", "four-flats", "empty", "noise"}) {
            BufferedImage image = new BufferedImage(960, 420, BufferedImage.TYPE_BYTE_GRAY);
            Graphics2D g = image.createGraphics();
            g.setColor(Color.WHITE); g.fillRect(0,0,960,420); g.setColor(Color.BLACK);
            for (int y=120; y<=184; y+=16) g.fillRect(100,y,761,2);
            for (int x:new int[]{500,858}) g.fillRect(x,120,2,66);
            for (int x:new int[]{450,650,750}) g.fillRect(x,150,12,3);
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);
            clef.draw(g,116,168);
            boolean sharp = name.contains("sharp");
            int count = name.equals("one-sharp") || name.equals("one-flat") ? 1 : name.equals("three-sharps") ? 3 : name.equals("four-flats") ? 4 : 0;
            TextLayout alter = (TextLayout) mf.getMethod("layoutShapeByCode", shape).invoke(font, shape.getField(sharp ? "SHARP" : "FLAT").get(null));
            int[] pitches = sharp ? new int[]{-4,-1,-5} : new int[]{0,-3,1,-2};
            for (int i=0; i<count; i++) alter.draw(g,174 + i * (sharp ? 24 : 22), 152 + pitches[i] * 8);
            if (name.equals("noise")) for (int i=0;i<12;i++) g.fillRect(176+(i%4)*12,128+(i/4)*18,2,2);
            g.dispose();
            ImageIO.write(image,"PNG",folder.resolve("rendered-header-"+name+".png").toFile());
        }
        System.exit(0);
    }
}
