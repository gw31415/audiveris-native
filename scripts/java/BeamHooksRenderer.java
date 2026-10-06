// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only staff-relative inputs from the original font. No native outputs.
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
public class BeamHooksRenderer {
 public static void main(String[] args)throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  int levels=2;
  if(levels!=1&&levels!=2)throw new IllegalArgumentException("one/two levels only");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);
  int top=220;LimitedScoreRenderer.staff(p,g,top,true);
  int firstBeam=top-14-(levels-1)*11;
  for(int x:new int[]{270,330,390,450}){
   LimitedScoreRenderer.event(p,g,top,x,0,"NOTEHEAD_BLACK");
   g.fillRect(x+18,firstBeam,2,top+22-firstBeam+1);
  }
  g.fillRect(288,firstBeam,182,6);
  for(int x:new int[]{288,336,408,456})g.fillRect(x,firstBeam+11,14,6);
  int[] quarters=levels==1?new int[]{540,630}:new int[]{510,570,630};
  for(int i=0;i<quarters.length;i++)LimitedScoreRenderer.event(p,g,top,quarters[i],i%2==0?2:4,"NOTEHEAD_BLACK");
  int[] xs={750,870,990,1110},pitches={0,-2,-4,-2};
  for(int i=0;i<xs.length;i++)LimitedScoreRenderer.event(p,g,top,xs[i],pitches[i],"NOTEHEAD_BLACK");
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("beam-four-hooks.png").toFile());
 }
}
