// SPDX-License-Identifier: AGPL-3.0-or-later
// Development fixture using the existing original Bravura/staff renderer.
import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.Path;
public class ParallelHalfRenderer {
 public static void main(String[] args)throws Exception {
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,p.getWidth(),p.getHeight());g.setColor(Color.BLACK);
  LimitedScoreRenderer.staff(p,g,220,true);
  BufferedImage head=HeadActualPageRenderer.symbol("Bravura","NOTEHEAD_BLACK",51);
  for(int x:new int[]{270,510})LimitedScoreRenderer.event(p,g,220,x,-4,"NOTEHEAD_VOID");
  int[] pitches={-4,-2,0,-2};
  for(int i=4;i<8;i++)LimitedScoreRenderer.event(p,g,220,270+120*i,pitches[i%4],"NOTEHEAD_BLACK");
  for(int i=0;i<8;i++) {
   int x=270+120*i;
   HeadActualPageRenderer.stamp(p,head,x,264);
   g.fillRect(x+4,264,2,37);
  }
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",Path.of(args[0]).toFile());
 }
}
