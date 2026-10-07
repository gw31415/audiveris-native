// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.Path;
public class HalfRestAugmentationRenderer {
 static int[] ink(BufferedImage g) {
  int lo=g.getWidth(),hi=-1;
  for(int y=0;y<g.getHeight();y++)for(int x=0;x<g.getWidth();x++) {
   int c=g.getRGB(x,y);if((c>>>24)>=140&&((c>>16)&255)<140){lo=Math.min(lo,x);hi=Math.max(hi,x);}
  }return new int[]{lo,hi};
 }
 public static void main(String[] args)throws Exception {
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);LimitedScoreRenderer.staff(p,g,220,true);
  LimitedScoreRenderer.event(p,g,220,270,0,"NOTEHEAD_BLACK");
  BufferedImage rest=HeadActualPageRenderer.symbol("Bravura","HALF_REST",51),dot=HeadActualPageRenderer.symbol("Bravura","AUGMENTATION_DOT",51);
  HeadActualPageRenderer.stamp(p,rest,510,239);
  int[] rb=ink(rest),db=ink(dot);int dotX=510+rb[1]+6-(db[0]+(db[1]-db[0]+1)/2);
  HeadActualPageRenderer.stamp(p,dot,dotX,247);
  LimitedScoreRenderer.event(p,g,220,750,-2,"NOTEHEAD_VOID");LimitedScoreRenderer.event(p,g,220,990,0,"NOTEHEAD_VOID");g.dispose();javax.imageio.ImageIO.write(p,"PNG",Path.of(args[0]).toFile());
 }
}
