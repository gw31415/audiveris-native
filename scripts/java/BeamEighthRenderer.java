// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only input from original Bravura font and staff-relative anchors.
import java.awt.*;import java.awt.font.TextLayout;import java.awt.image.BufferedImage;import java.nio.file.*;
public class BeamEighthRenderer {
 public static void main(String[] args)throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);LimitedScoreRenderer.staff(p,g,220,true);
  int[] xs={270,390,450,540,630,750,870,990,1110};int[] pp={0,0,2,4,2,0,-2,-4,-2};
  for(int i=0;i<xs.length;i++)LimitedScoreRenderer.event(p,g,220,xs[i],pp[i],"NOTEHEAD_BLACK");
  g.fillRect(288,206,122,6); // Beam joins the two ordinary upward stems.
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("beam-eighth-quarter.png").toFile());
 }
}
