// SPDX-License-Identifier: AGPL-3.0-or-later
// Original synthetic scores; clef glyphs rendered with pinned Bravura (SIL OFL).
import java.awt.*;import java.awt.image.*;import java.awt.font.*;import java.nio.file.*;import javax.imageio.ImageIO;
public class MakeClefFixtures {
 public static void main(String[]args)throws Exception{
  Class<?>mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),fam=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),shape=Class.forName("org.audiveris.omr.glyph.Shape");mf.getMethod("populateAllSymbols").invoke(null);Object font=mf.getMethod("getBaseFont",fam,int.class).invoke(null,fam.getField("Bravura").get(null),16);
  for(String name:new String[]{"G_CLEF","F_CLEF","C_CLEF","EMPTY","NOISE"}){
   BufferedImage image=new BufferedImage(960,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=image.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,960,420);g.setColor(Color.BLACK);
   for(int y=120;y<=184;y+=16)g.fillRect(100,y,761,2);for(int x:new int[]{400,700,858})g.fillRect(x,120,2,66);for(int x:new int[]{250,300,500})g.fillRect(x,150,12,3);
   if(name.equals("NOISE")){for(int i=0;i<12;i++)g.fillRect(114+(i%4)*12,128+(i/4)*18,2,2);}
   else if(!name.equals("EMPTY")){TextLayout layout=(TextLayout)mf.getMethod("layoutShapeByCode",shape).invoke(font,shape.getField(name).get(null));g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);layout.draw(g,116,name.equals("G_CLEF")?168:name.equals("F_CLEF")?136:152);}
   g.dispose();ImageIO.write(image,"PNG",Path.of(args[0],"rendered-clef-"+name.toLowerCase()+".png").toFile());
  }System.exit(0);
 }
}
