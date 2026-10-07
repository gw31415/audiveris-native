// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.Path;

public class InconsistentAugmentationRenderer {
    public static void main(String[] args) throws Exception {
        HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"), "populateAllSymbols");
        BufferedImage page = new BufferedImage(1280, 420, BufferedImage.TYPE_BYTE_GRAY);
        Graphics2D g = page.createGraphics();
        g.setColor(Color.WHITE);
        g.fillRect(0, 0, page.getWidth(), page.getHeight());
        g.setColor(Color.BLACK);
        LimitedScoreRenderer.staff(page, g, 220, true);
        BufferedImage head = HeadActualPageRenderer.symbol("Bravura", "NOTEHEAD_BLACK", 51);
        boolean beamed = args.length > 1 && args[1].equals("beam");
        boolean doubleFlag = args.length > 1 && args[1].equals("flag2");
        int tail = doubleFlag ? 173 : 184;
        for (int i = 0; i < 4; i++) {
            int x = 270 + (beamed || doubleFlag ? 60 : 120) * i;
            HeadActualPageRenderer.stamp(page, head, x, 220);
            HeadActualPageRenderer.stamp(page, head, x, 242);
            g.fillRect(x + head.getWidth() - 2, tail, 2, 243 - tail);
        }
        if (beamed) {
            g.fillRect(288, 184, 182, 6);
            for (int x : new int[]{540, 630})
                LimitedScoreRenderer.event(page, g, 220, x, -4, "NOTEHEAD_BLACK");
        }
        if (doubleFlag) {
            Object family = HeadActualPageRenderer.cls("ui.symbol.MusicFamily").getField("Bravura").get(null);
            Object font = HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"), "getMusicFont", family, 51);
            Object shape = HeadActualPageRenderer.cls("glyph.Shape").getField("FLAG_2").get(null);
            java.awt.font.TextLayout flag = (java.awt.font.TextLayout) HeadActualPageRenderer.call(font, "layoutShapeByCode", shape);
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            for (int x : new int[]{270, 330, 390, 450}) flag.draw(g, x + 18, tail);
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_OFF);
            for (int x : new int[]{510, 570, 630})
                LimitedScoreRenderer.event(page, g, 220, x, -4, "NOTEHEAD_BLACK");
        }
        int[] pitches = {-4, -2, 0, -2};
        for (int i = 4; i < 8; i++)
            LimitedScoreRenderer.event(page, g, 220, 270 + 120 * i, pitches[i % 4], "NOTEHEAD_BLACK");
        BufferedImage dot = HeadActualPageRenderer.symbol("Bravura", "AUGMENTATION_DOT", 51);
        HeadActualPageRenderer.stamp(page, dot, 302, 237);
        g.dispose();
        javax.imageio.ImageIO.write(page, "PNG", Path.of(args[0]).toFile());
    }
}
