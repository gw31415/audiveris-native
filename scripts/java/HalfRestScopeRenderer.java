// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent input renderer, no native recognition output.
import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.*;
public class HalfRestScopeRenderer {
 public static void main(String[] args)throws Exception{
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  for(int size:new int[]{239,240,241}){
   BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);LimitedScoreRenderer.staff(p,g,220,true);
   HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","HALF_REST",51),270,size);
   LimitedScoreRenderer.event(p,g,220,510,0,"NOTEHEAD_BLACK");LimitedScoreRenderer.event(p,g,220,630,-2,"NOTEHEAD_BLACK");
   LimitedScoreRenderer.event(p,g,220,750,-2,"NOTEHEAD_VOID");LimitedScoreRenderer.event(p,g,220,990,0,"NOTEHEAD_VOID");g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("half-rest-y"+size+".png").toFile());
  }
 }
}
