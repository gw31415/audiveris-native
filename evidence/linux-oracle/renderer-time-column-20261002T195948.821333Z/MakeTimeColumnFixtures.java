// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent time-signature artwork from pinned Bravura and declared geometry.
import java.awt.*;import java.awt.image.BufferedImage;import java.awt.font.TextLayout;import java.awt.geom.*;import java.nio.file.*;import java.util.*;import javax.imageio.ImageIO;
public class MakeTimeColumnFixtures {
 static String sha(Path p)throws Exception{return HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(p)));}
 static void centered(Graphics2D g,TextLayout t,float x,float y){Rectangle2D b=t.getBounds();t.draw(g,(float)(x-b.getCenterX()),(float)(y-b.getCenterY()));}
 public static void main(String[]args)throws Exception{
  Path out=Path.of(args[0]);Files.createDirectories(out);Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),shape=Class.forName("org.audiveris.omr.glyph.Shape");mf.getMethod("populateAllSymbols").invoke(null);Object font=mf.getMethod("getBaseFont",family,int.class).invoke(null,family.getField("Bravura").get(null),16);
  Map<String,TextLayout> layouts=new HashMap<>();for(String s:new String[]{"G_CLEF","BRACE","COMMON_TIME","CUT_TIME","TIME_TWO","TIME_THREE","TIME_FOUR","TIME_FIVE","TIME_SIX","TIME_SEVEN","TIME_EIGHT","TIME_TWELVE"})layouts.put(s,(TextLayout)mf.getMethod("layoutShapeByCode",shape).invoke(font,shape.getField(s).get(null)));
  Map<Integer,String> digits=Map.of(2,"TIME_TWO",3,"TIME_THREE",4,"TIME_FOUR",5,"TIME_FIVE",6,"TIME_SIX",7,"TIME_SEVEN",8,"TIME_EIGHT",12,"TIME_TWELVE");
  String[] names={"common","cut","pair-2-2","pair-2-4","pair-3-4","pair-4-4","pair-5-4","pair-6-8","pair-7-8","pair-12-8","absent","noise","numerator-only","unsupported-3-3","misaligned-3-4","far-3-4","two-staff-same","two-staff-mismatch","two-staff-missing","two-staff-common-pair"};
  java.util.List<String> records=new ArrayList<>();
  for(String name:names){boolean two=name.startsWith("two-staff");BufferedImage im=new BufferedImage(960,two?470:420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=im.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,im.getWidth(),im.getHeight());g.setColor(Color.BLACK);for(int n=0;n<(two?2:1);n++){int dy=130*n;for(int y=120+dy;y<=184+dy;y+=16)g.fillRect(100,y,761,2);for(int x:new int[]{450,650,750})g.fillRect(x,150+dy,12,3);}for(int x:new int[]{500,858})g.fillRect(x,120,2,two?196:66);if(two)g.fillRect(100,120,2,196);
   g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);if(two){TextLayout t=layouts.get("BRACE");Rectangle2D b=t.getBounds();AffineTransform old=g.getTransform();g.translate(76,119);g.scale(20/b.getWidth(),198/b.getHeight());t.draw(g,(float)-b.getX(),(float)-b.getY());g.setTransform(old);}
   java.util.List<String> recipes=new ArrayList<>();for(int n=0;n<(two?2:1);n++){int dy=130*n;layouts.get("G_CLEF").draw(g,116,168+dy);String whole=null;int num=3,den=4;boolean drawNum=true,drawDen=true;int x=name.equals("far-3-4")?215:184,denDx=name.equals("misaligned-3-4")?22:0;
    if(name.equals("common")||(name.equals("two-staff-common-pair")&&n==0))whole="COMMON_TIME";if(name.equals("cut"))whole="CUT_TIME";
    if(name.startsWith("pair-")){String[] bits=name.split("-");num=Integer.parseInt(bits[1]);den=Integer.parseInt(bits[2]);}if(name.equals("unsupported-3-3"))den=3;if(name.equals("two-staff-mismatch")&&n==1)num=4;if(name.equals("two-staff-common-pair")){num=4;den=4;}
    if(name.equals("numerator-only"))drawDen=false;if(name.equals("absent")||name.equals("noise")||(name.equals("two-staff-missing")&&n==1)){drawNum=false;drawDen=false;}
    if(whole!=null)centered(g,layouts.get(whole),x,152+dy);else{if(drawNum)centered(g,layouts.get(digits.get(num)),x,136+dy);if(drawDen)centered(g,layouts.get(digits.get(den)),x+denDx,168+dy);}
    if(name.equals("noise"))for(int i=0;i<12;i++)g.fillRect(172+(i%4)*8,128+(i/4)*18,2,2);
    recipes.add("{\"staff_index\":"+n+",\"whole_shape\":"+(whole==null?"null":"\""+whole+"\"")+",\"numerator\":"+(whole==null&&drawNum?num:"null")+",\"denominator\":"+(whole==null&&drawDen?den:"null")+",\"x_center\":"+x+",\"denominator_x_shift\":"+denDx+",\"y_offset\":"+dy+"}");
   }g.dispose();Path target=out.resolve("time-column-"+name+".png");ImageIO.write(im,"PNG",target.toFile());records.add("{\"name\":\""+name+"\",\"output_filename\":\""+target.getFileName()+"\",\"output_sha256\":\""+sha(target)+"\",\"staff_recipes\":["+String.join(",",recipes)+"]}");
  }
  Files.writeString(out.resolve("render-manifest.json"),"{\"synthetic_independent_raster\":true,\"native_outputs_used\":false,\"interline\":16,\"number_centers_y\":[136,168],\"whole_center_y\":152,\"centered_by\":\"original TextLayout visual bounds\",\"noise\":\"twelve 2x2 blocks x172+(i%4)*8,y128+(i/4)*18\",\"fixtures\":["+String.join(",",records)+"]}\n");System.exit(0);
 }
}
