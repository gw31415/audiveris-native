// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only input from original Bravura renderer; no native results.
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
public class HeaderCommonRenderer {
 public static void main(String[] args)throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);
  for(int j=0;j<5;j++)g.fillRect(80,220+j*11,1120,2);
  for(int x:new int[]{80,710,1198})g.fillRect(x,220,2,46);
  HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","G_CLEF",51),100,242);
  HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","SHARP",51),145,220);
  HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","COMMON_TIME",51),165,242);
  int[] pitches={0,2,4,2,0,-2,-4,-2};
  int[] positions={270,390,510,630,750,870,990,1110};
  for(int i=0;i<pitches.length;i++)LimitedScoreRenderer.event(p,g,220,positions[i],pitches[i],"NOTEHEAD_BLACK");
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("g-major-header-common.png").toFile());
 }
}
