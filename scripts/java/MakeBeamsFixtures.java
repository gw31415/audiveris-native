// SPDX-License-Identifier: AGPL-3.0-or-later
// Independently specified stroke/Bravura artwork; no recognition outputs supplied.
import java.awt.*;import java.awt.image.BufferedImage;import java.awt.font.TextLayout;import java.awt.geom.Rectangle2D;import java.nio.file.*;import java.util.*;import javax.imageio.ImageIO;
public class MakeBeamsFixtures {
 static String sha(Path p)throws Exception{return HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(p)));}
 public static void main(String[]args)throws Exception{
  Path out=Path.of(args[0]);Files.createDirectories(out);Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),fc=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),sc=Class.forName("org.audiveris.omr.glyph.Shape");mf.getMethod("populateAllSymbols").invoke(null);Object font=mf.getMethod("getBaseFont",fc,int.class).invoke(null,fc.getField("Bravura").get(null),16);TextLayout clef=(TextLayout)mf.getMethod("layoutShapeByCode",sc).invoke(font,sc.getField("G_CLEF").get(null)),head=(TextLayout)mf.getMethod("layoutShapeByCode",sc).invoke(font,sc.getField("NOTEHEAD_BLACK").get(null));java.util.List<String> records=new ArrayList<>();
  for(String name:new String[]{"single","double","triple","sloped","split","hook","short","thin","steep","heads-only","empty","noise"}){
   BufferedImage image=new BufferedImage(960,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=image.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,960,420);g.setColor(Color.BLACK);for(int y=120;y<=184;y+=16)g.fillRect(100,y,761,2);g.fillRect(858,120,2,66);for(int x:new int[]{550,650,750})g.fillRect(x,151,9,8);g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);clef.draw(g,116,168);java.util.List<String> strokes=new ArrayList<>(),heads=new ArrayList<>();
   if(name.equals("noise")){for(int i=0;i<30;i++){int x=280+(i%10)*11,y=81+(i/10)*11;g.fillRect(x,y,2,2);strokes.add("["+x+","+y+",2,2]");}}
   else if(!name.equals("empty")){
    int width=name.equals("short")?10:112,thick=name.equals("thin")?2:8,layers=name.equals("triple")?3:name.equals("double")?2:1;
    if(!name.equals("heads-only"))for(int layer=0;layer<layers;layer++)for(int dx=0;dx<width;dx++){if(name.equals("split")&&dx>=47&&dx<57)continue;int yy=86+layer*16+(name.equals("sloped")?dx/6:name.equals("steep")?dx:0);g.fillRect(280+dx,yy,1,thick);strokes.add("["+(280+dx)+","+yy+",1,"+thick+"]");}
    if(name.equals("hook")){g.fillRect(280,102,22,8);strokes.add("[280,102,22,8]");}
    for(int x:new int[]{280,390}){if(!name.equals("heads-only")){g.fillRect(x,89,2,77);strokes.add("["+x+",89,2,77]");}Rectangle2D r=head.getBounds();head.draw(g,(float)(x-9-r.getCenterX()),(float)(164-r.getCenterY()));heads.add("["+(x-9)+",164]");}
   }g.dispose();Path p=out.resolve("beams-"+name+".png");ImageIO.write(image,"PNG",p.toFile());records.add("{\"name\":\""+name+"\",\"output_filename\":\""+p.getFileName()+"\",\"output_sha256\":\""+sha(p)+"\",\"black_stroke_rectangles\":["+String.join(",",strokes)+"],\"notehead_centers\":["+String.join(",",heads)+"]}");
  }Files.writeString(out.resolve("render-manifest.json"),"{\"synthetic_independent_raster\":true,\"native_outputs_used\":false,\"interline\":16,\"width\":960,\"height\":420,\"staff_bounds\":[100,120,761,66],\"right_bar\":[858,120,2,66],\"scale_marks\":[[550,151,9,8],[650,151,9,8],[750,151,9,8]],\"clef_origin\":[116,168],\"fixtures\":["+String.join(",",records)+"]}\n");System.exit(0);
 }
}
