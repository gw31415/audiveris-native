// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only Bravura input; staff-relative original glyph baselines.
import java.awt.*;
import java.awt.font.TextLayout;
import java.awt.image.BufferedImage;
import java.nio.file.*;
public class HeaderClefRenderer {
 public static void main(String[] args)throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  String clef=args.length>1?args[1]:"F_CLEF";
  if(!clef.equals("F_CLEF")&&!clef.equals("C_CLEF"))throw new IllegalArgumentException(clef);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);
  for(int j=0;j<5;j++)g.fillRect(80,220+j*11,1120,2);
  for(int x:new int[]{80,710,1198})g.fillRect(x,220,2,46);
  Object family=HeadActualPageRenderer.cls("ui.symbol.MusicFamily").getField("Bravura").get(null);
  Object font=HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"getMusicFont",family,51);
  Object shape=HeadActualPageRenderer.cls("glyph.Shape").getField(clef).get(null);
  TextLayout layout=(TextLayout)HeadActualPageRenderer.call(font,"layoutShapeByCode",shape);
  g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);
  layout.draw(g,100,clef.equals("F_CLEF")?231:242); // F fourth line; C middle line.
  int timeX=(int)Math.ceil(100+layout.getBounds().getMaxX())+11;
  for(int y:new int[]{231,253})HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","TIME_FOUR",51),timeX,y);
  int[] pitches={0,2,4,2,0,-2,-4,-2};
  for(int i=0;i<pitches.length;i++)LimitedScoreRenderer.event(p,g,220,270+120*i,pitches[i],"NOTEHEAD_BLACK");
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve((clef.equals("F_CLEF")?"bass":"alto")+"-header-quarter.png").toFile());
 }
}
