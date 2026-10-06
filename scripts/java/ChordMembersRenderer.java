// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.Path;

public class ChordMembersRenderer {
    public static void main(String[] args) throws Exception {
        HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"), "populateAllSymbols");
        BufferedImage page = new BufferedImage(1280, 420, BufferedImage.TYPE_BYTE_GRAY);
        Graphics2D g = page.createGraphics();
        g.setColor(Color.WHITE);
        g.fillRect(0, 0, page.getWidth(), page.getHeight());
        g.setColor(Color.BLACK);
        LimitedScoreRenderer.staff(page, g, 220, true);
        BufferedImage head = HeadActualPageRenderer.symbol("Bravura", "NOTEHEAD_BLACK", 51);
        for (int i = 0; i < 4; i++) {
            int x = 270 + 120 * i;
            HeadActualPageRenderer.stamp(page, head, x, 220);
            HeadActualPageRenderer.stamp(page, head, x, 242);
            g.fillRect(x + head.getWidth() - 2, 184, 2, 59);
        }
        int[] pitches = {-4, -2, 0, -2};
        for (int i = 4; i < 8; i++)
            LimitedScoreRenderer.event(page, g, 220, 270 + 120 * i, pitches[i % 4], "NOTEHEAD_BLACK");
        g.dispose();
        javax.imageio.ImageIO.write(page, "PNG", Path.of(args[0]).toFile());
    }
}
