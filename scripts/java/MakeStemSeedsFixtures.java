// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent Bravura/stroke rasters for original STEM_SEEDS boundary.
import java.awt.*;import java.awt.image.BufferedImage;import java.awt.font.TextLayout;import java.awt.geom.Rectangle2D;import java.nio.file.*;import java.util.*;import javax.imageio.ImageIO;
public class MakeStemSeedsFixtures {
 static String sha(Path p)throws Exception{return HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(p)));}
 public static void main(String[]args)throws Exception{
  Path out=Path.of(args[0]);Files.createDirectories(out);Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),fc=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),sc=Class.forName("org.audiveris.omr.glyph.Shape");mf.getMethod("populateAllSymbols").invoke(null);Object font=mf.getMethod("getBaseFont",fc,int.class).invoke(null,fc.getField("Bravura").get(null),16);TextLayout clef=(TextLayout)mf.getMethod("layoutShapeByCode",sc).invoke(font,sc.getField("G_CLEF").get(null)),head=(TextLayout)mf.getMethod("layoutShapeByCode",sc).invoke(font,sc.getField("NOTEHEAD_BLACK").get(null));java.util.List<String> records=new ArrayList<>();
  for(String name:new String[]{"clean","heads","short","broken","sloped","empty","noise","mixed-width"}){
   BufferedImage image=new BufferedImage(960,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=image.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,960,420);g.setColor(Color.BLACK);for(int y=120;y<=184;y+=16)g.fillRect(100,y,761,2);g.fillRect(858,120,2,66);for(int x:new int[]{450,650,750})g.fillRect(x,150,12,3);g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);clef.draw(g,116,168);java.util.List<String> strokes=new ArrayList<>();
   if(name.equals("noise")){for(int i=0;i<18;i++){int x=280+(i%6)*9,y=105+(i/6)*20;g.fillRect(x,y,2,2);strokes.add("["+x+","+y+",2,2]");}}
   else if(!name.equals("empty")){int i=0;for(int x:new int[]{300,400,570}){int w=name.equals("mixed-width")?++i:2,y=name.equals("short")?140:100,h=name.equals("short")?12:65;
    if(name.equals("sloped")){for(int yy=0;yy<h;yy++){int xx=x+yy/5;g.fillRect(xx,y+yy,w,1);strokes.add("["+xx+","+(y+yy)+","+w+",1]");}}
    else{g.fillRect(x,y,w,h);strokes.add("["+x+","+y+","+w+","+h+"]");}
    if(name.equals("broken")){g.setColor(Color.WHITE);g.fillRect(x,132,w,3);g.setColor(Color.BLACK);}
    if(name.equals("heads")){Rectangle2D r=head.getBounds();head.draw(g,(float)(x-9-r.getCenterX()),(float)(164-r.getCenterY()));}
   }}g.dispose();Path p=out.resolve("stem-seeds-"+name+".png");ImageIO.write(image,"PNG",p.toFile());records.add("{\"name\":\""+name+"\",\"output_filename\":\""+p.getFileName()+"\",\"output_sha256\":\""+sha(p)+"\",\"black_stroke_rectangles\":["+String.join(",",strokes)+"],\"broken_white_rectangles\":"+(name.equals("broken")?"[[300,132,2,3],[400,132,2,3],[570,132,2,3]]":"[]")+",\"notehead_centers\":"+(name.equals("heads")?"[[291,164],[391,164],[561,164]]":"[]")+"}");
  }Files.writeString(out.resolve("render-manifest.json"),"{\"synthetic_independent_raster\":true,\"native_outputs_used\":false,\"interline\":16,\"staff_bounds\":[100,120,761,66],\"right_bar\":[858,120,2,66],\"clef_origin\":[116,168],\"fixtures\":["+String.join(",",records)+"]}\n");System.exit(0);
 }
}
