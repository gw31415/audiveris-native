// SPDX-License-Identifier: AGPL-3.0-or-later
// One independent normal staff input: four quarters then a hanging whole rest.
import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.*;
public class MeasureRestRenderer {
 public static void main(String[] args)throws Exception {
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();
  g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);LimitedScoreRenderer.staff(p,g,220,true);
  for(int i=0;i<4;i++)LimitedScoreRenderer.event(p,g,220,270+120*i,new int[]{0,2,4,2}[i],"NOTEHEAD_BLACK");
  HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","WHOLE_REST",51),950,235);
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",Path.of(args[0]).toFile());
 }
}
