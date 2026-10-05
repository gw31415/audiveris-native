// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only bounded input using existing original Bravura rendering.
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
public class LedgerQuarterRenderer {
 public static void main(String[] args)throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);
  LimitedScoreRenderer.staff(p,g,220,true);
  int[] pitches={6,4,2,0,0,-2,-4,-2};
  // C4 uses the first lower ledger, eleven pixels below the bottom staff line.
  g.fillRect(272,275,21,2);
  for(int i=0;i<pitches.length;i++)LimitedScoreRenderer.event(p,g,220,270+120*i,pitches[i],"NOTEHEAD_BLACK");
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("ledger-quarter.png").toFile());
 }
}
