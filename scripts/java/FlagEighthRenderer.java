// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only input from original Bravura font and staff-relative anchors.
import java.awt.*;import java.awt.font.TextLayout;import java.awt.image.BufferedImage;import java.nio.file.*;
public class FlagEighthRenderer {
 public static void main(String[] args)throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);LimitedScoreRenderer.staff(p,g,220,true);
  int[] xs={270,390,450,540,630,750,870,990,1110};int[] pp={0,0,2,4,2,0,-2,-4,-2};
  for(int i=0;i<xs.length;i++)LimitedScoreRenderer.event(p,g,220,xs[i],pp[i],"NOTEHEAD_BLACK");
  Object family=HeadActualPageRenderer.cls("ui.symbol.MusicFamily").getField("Bravura").get(null),font=HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"getMusicFont",family,51),shape=HeadActualPageRenderer.cls("glyph.Shape").getField("FLAG_1").get(null);TextLayout flag=(TextLayout)HeadActualPageRenderer.call(font,"layoutShapeByCode",shape);
  g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);
  for(int x:new int[]{270,390})flag.draw(g,x+18,206); // Original glyph origin is stem tip.
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("flag-eighth-quarter.png").toFile());
 }
}
