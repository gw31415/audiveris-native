// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.*;
public class BeamGroupsRenderer {
 public static void main(String[] args)throws Exception {
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);
  int top=220;LimitedScoreRenderer.staff(p,g,top,true);
  for(int x:new int[]{270,330,390,450}){
   LimitedScoreRenderer.event(p,g,top,x,0,"NOTEHEAD_BLACK");g.fillRect(x+18,top-14,2,37);
  }
  for(int x:new int[]{288,408})g.fillRect(x,top-14,62,6);
  for(int x:new int[]{540,630})LimitedScoreRenderer.event(p,g,top,x,2,"NOTEHEAD_BLACK");
  int[] xs={750,870,990,1110},pitches={0,-2,-4,-2};
  for(int i=0;i<xs.length;i++)LimitedScoreRenderer.event(p,g,top,xs[i],pitches[i],"NOTEHEAD_BLACK");
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",Path.of(args[0]).toFile());
 }
}
