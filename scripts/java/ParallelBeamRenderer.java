// SPDX-License-Identifier: AGPL-3.0-or-later
// Development fixture using the existing original Bravura/staff renderer.
import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.Path;
public class ParallelBeamRenderer {
 public static void main(String[] args)throws Exception {
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,p.getWidth(),p.getHeight());g.setColor(Color.BLACK);
  LimitedScoreRenderer.staff(p,g,220,true);
  BufferedImage head=HeadActualPageRenderer.symbol("Bravura","NOTEHEAD_BLACK",51);
  int levels=args.length>1?Integer.parseInt(args[1]):1;
  if(levels!=1&&levels!=2)throw new IllegalArgumentException("one/two beam levels required");
  int firstBeam=184-(levels-1)*11;
  for(int x:new int[]{270,330,390,450}) {
   LimitedScoreRenderer.event(p,g,220,x,-4,"NOTEHEAD_BLACK");
   g.fillRect(x+18,firstBeam,2,221-firstBeam);
  }
  for(int i=0;i<levels;i++)g.fillRect(288,firstBeam+i*11,182,6);
  for(int x:levels==1?new int[]{540,630}:new int[]{510,570,630})LimitedScoreRenderer.event(p,g,220,x,-4,"NOTEHEAD_BLACK");
  int[] pitches={-4,-2,0,-2};
  for(int i=4;i<8;i++)LimitedScoreRenderer.event(p,g,220,270+120*i,pitches[i%4],"NOTEHEAD_BLACK");
  for(int i=0;i<8;i++) {
   int x=i<4&&levels==2?new int[]{270,510,570,630}[i]:270+120*i;
   HeadActualPageRenderer.stamp(p,head,x,264);
   g.fillRect(x+4,264,2,37);
  }
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",Path.of(args[0]).toFile());
 }
}
