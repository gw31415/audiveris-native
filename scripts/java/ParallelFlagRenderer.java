// SPDX-License-Identifier: AGPL-3.0-or-later
// Development fixture using the existing original Bravura/staff renderer.
import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.Path;
public class ParallelFlagRenderer {
 public static void main(String[] args)throws Exception {
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,p.getWidth(),p.getHeight());g.setColor(Color.BLACK);
  LimitedScoreRenderer.staff(p,g,220,true);
  BufferedImage head=HeadActualPageRenderer.symbol("Bravura","NOTEHEAD_BLACK",51);
  for(int x:new int[]{270,390,450,540,630})LimitedScoreRenderer.event(p,g,220,x,-4,"NOTEHEAD_BLACK");
  Object family=HeadActualPageRenderer.cls("ui.symbol.MusicFamily").getField("Bravura").get(null),font=HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"getMusicFont",family,51),shape=HeadActualPageRenderer.cls("glyph.Shape").getField("FLAG_1").get(null);
  java.awt.font.TextLayout flag=(java.awt.font.TextLayout)HeadActualPageRenderer.call(font,"layoutShapeByCode",shape);
  g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);
  for(int x:new int[]{270,390})flag.draw(g,x+18,184);
  g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_OFF);
  int[] pitches={-4,-2,0,-2};
  for(int i=0;i<4;i++)LimitedScoreRenderer.event(p,g,220,750+120*i,pitches[i],"NOTEHEAD_BLACK");
  for(int x:new int[]{270,450,540,630,750,870,990,1110}) {
   HeadActualPageRenderer.stamp(p,head,x,264);
   g.fillRect(x+4,264,2,37);
  }
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",Path.of(args[0]).toFile());
 }
}
