// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent synthetic two-staff Bravura inputs; no recognition answers used.
import java.awt.*;import java.awt.image.BufferedImage;import java.awt.font.TextLayout;import java.awt.geom.*;import java.nio.file.*;import java.util.*;import javax.imageio.ImageIO;
public class MakeKeyColumnFixtures {
 static String sha(Path p)throws Exception{return HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(p)));}
 public static void main(String[] args)throws Exception{
  Path out=Path.of(args[0]);Files.createDirectories(out);Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),shape=Class.forName("org.audiveris.omr.glyph.Shape");mf.getMethod("populateAllSymbols").invoke(null);Object font=mf.getMethod("getBaseFont",family,int.class).invoke(null,family.getField("Bravura").get(null),16);
  Map<String,TextLayout> layouts=new HashMap<>();for(String s:new String[]{"G_CLEF","FLAT","SHARP","BRACE"})layouts.put(s,(TextLayout)mf.getMethod("layoutShapeByCode",shape).invoke(font,shape.getField(s).get(null)));
  String[] names={"uniform-flats","last-flat-missing","all-bottom-missing","wrong-sharps","middle-flat-missing","bottom-shifted","bottom-no-clef","bottom-noise","uniform-empty","last-sharp-missing","bottom-bridge","bottom-first-only","bottom-trailing-block","bottom-ink-blocks","one-flat-trailing"};
  java.util.List<String> records=new ArrayList<>();
  for(String name:names){
   BufferedImage im=new BufferedImage(960,470,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=im.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,960,470);g.setColor(Color.BLACK);
   for(int dy:new int[]{0,130}){for(int y=120+dy;y<=184+dy;y+=16)g.fillRect(100,y,761,2);for(int x:new int[]{450,650,750})g.fillRect(x,150+dy,12,3);}
   for(int x:new int[]{100,500,858})g.fillRect(x,120,2,196);
   g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);
   TextLayout brace=layouts.get("BRACE");Rectangle2D bounds=brace.getBounds();AffineTransform old=g.getTransform();g.translate(76,119);g.scale(20/bounds.getWidth(),198/bounds.getHeight());brace.draw(g,(float)-bounds.getX(),(float)-bounds.getY());g.setTransform(old);
   java.util.List<String> staffRecipes=new ArrayList<>();
   for(int n=0;n<2;n++){
    int dy=n*130;boolean sharp=name.equals("last-sharp-missing")||(n==1&&name.equals("wrong-sharps"));boolean clef=!(n==1&&name.equals("bottom-no-clef"));if(clef)layouts.get("G_CLEF").draw(g,116,168+dy);
    int count=name.equals("one-flat-trailing")?1:sharp?3:4;if(name.equals("uniform-empty")||(n==1&&(name.equals("all-bottom-missing")||name.equals("bottom-noise")||name.equals("bottom-ink-blocks"))))count=0;if(n==1&&name.equals("last-flat-missing"))count=3;if(n==1&&name.equals("last-sharp-missing"))count=2;if(n==1&&name.equals("bottom-first-only"))count=1;
    int shift=(n==1&&name.equals("bottom-shifted"))?12:0;int[] pitches=sharp?new int[]{-4,-1,-5}:new int[]{0,-3,1,-2};java.util.List<String> indices=new ArrayList<>();
    for(int i=0;i<count;i++){if(n==1&&name.equals("middle-flat-missing")&&i==1)continue;indices.add(""+i);layouts.get(sharp?"SHARP":"FLAT").draw(g,174+shift+i*(sharp?24:22),152+dy+pitches[i]*8);}
    if(n==1&&name.equals("bottom-noise"))for(int i=0;i<12;i++)g.fillRect(176+(i%4)*12,258+(i/4)*18,2,2);
    if(n==1&&name.equals("bottom-bridge"))g.fillRect(175,253,23,2);
    if(n==1&&name.equals("bottom-trailing-block"))g.fillRect(257,255,20,16);
    if(n==1&&name.equals("one-flat-trailing"))g.fillRect(191,276,20,15);
    if(n==1&&name.equals("bottom-ink-blocks"))for(int i=0;i<4;i++)g.fillRect(176+22*i,267+pitches[i]*8,12,30);
    staffRecipes.add("{\"staff_index\":"+n+",\"clef\":"+clef+",\"shape\":\""+(sharp?"SHARP":"FLAT")+"\",\"drawn_indices\":["+String.join(",",indices)+"],\"x_shift\":"+shift+",\"y_offset\":"+dy+"}");
   }
   g.dispose();Path target=out.resolve("key-column-"+name+".png");ImageIO.write(im,"PNG",target.toFile());records.add("{\"name\":\""+name+"\",\"output_filename\":\""+target.getFileName()+"\",\"output_sha256\":\""+sha(target)+"\",\"staff_recipes\":["+String.join(",",staffRecipes)+"]}");
  }
  Files.writeString(out.resolve("render-manifest.json"),"{\"synthetic_independent_raster\":true,\"native_outputs_used\":false,\"interline\":16,\"brace_target_bounds\":[76,119,20,198],\"bottom_noise\":\"twelve 2x2 blocks at x176+(i%4)*12,y258+(i/4)*18\",\"bottom_bridge\":[175,253,23,2],\"bottom_trailing_block\":[257,255,20,16],\"one_flat_trailing\":[191,276,20,15],\"bottom_ink_blocks\":\"four 12x30 blocks x176+22*i,y267+flatPitch[i]*8\",\"fixtures\":["+String.join(",",records)+"]}\n");System.exit(0);
 }
}
