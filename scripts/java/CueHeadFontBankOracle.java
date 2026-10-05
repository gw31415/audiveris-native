// SPDX-License-Identifier: AGPL-3.0-or-later
// Calls unmodified official Audiveris classes. This adapter is not a native implementation.
import java.nio.file.*;
import java.util.*;
import java.awt.*;
import java.awt.geom.*;
import java.awt.font.TextLayout;
import java.awt.image.BufferedImage;
public class CueHeadFontBankOracle {
 static Object get(Object o,String n)throws Exception{return o.getClass().getMethod(n).invoke(o);}
 static String rect(Rectangle2D r){return "["+r.getX()+","+r.getY()+","+r.getWidth()+","+r.getHeight()+"]";}
 static double score(Object template,int w,int h,byte[] pixels)throws Exception{
  Class<?> bp=Class.forName("ij.process.ByteProcessor"),dt=Class.forName("org.audiveris.omr.image.DistanceTable"),ac=Class.forName("org.audiveris.omr.image.Anchored$Anchor");
  Object image=bp.getConstructor(int.class,int.class).newInstance(w,h);
  for(int i=0;i<pixels.length;i++)bp.getMethod("set",int.class,int.class).invoke(image,i,pixels[i]&255);
  Class<?> c=Class.forName("org.audiveris.omr.image.ChamferDistance$Short");Object table=c.getMethod("computeToFore",bp).invoke(c.getConstructor().newInstance(),image);
  return (Double)template.getClass().getMethod("evaluate",int.class,int.class,ac,dt).invoke(template,0,0,null,table);
 }
 @SuppressWarnings({"unchecked","rawtypes"})
 public static void main(String[]args)throws Exception{
  Path out=Path.of(args[0]);Files.createDirectories(out);
  Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),sc=Class.forName("org.audiveris.omr.glyph.Shape"),fontc=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),symbolc=Class.forName("org.audiveris.omr.ui.symbol.TemplateSymbol"),bc=Class.forName("org.audiveris.omr.image.TemplateFactory$Builder");
  fontc.getMethod("populateAllSymbols").invoke(null);Object family=Enum.valueOf((Class)mf,args[1]);
  var ctor=bc.getDeclaredConstructor(sc,mf,int.class);ctor.setAccessible(true);var build=bc.getDeclaredMethod("buildTemplate");build.setAccessible(true);
  StringJoiner assets=new StringJoiner(",","[","]");
  for(int size:new int[]{28,44})for(String name:new String[]{"BREVE","WHOLE_NOTE","NOTEHEAD_VOID","NOTEHEAD_BLACK"}){
   Object shape=Enum.valueOf((Class)sc,name),font=fontc.getMethod("getMusicFont",mf,int.class).invoke(null,family,size);
   Object symbol=symbolc.getConstructor(sc,mf).newInstance(shape,family);
   BufferedImage image=(BufferedImage)symbolc.getMethod("buildImage",fontc).invoke(symbol,font);
   TextLayout layout=(TextLayout)fontc.getMethod("layoutShapeByCode",sc).invoke(font,shape);
   Rectangle fat=(Rectangle)symbolc.getMethod("getFatBounds",fontc).invoke(symbol,font);
   Object template=build.invoke(ctor.newInstance(shape,family,size));
   byte[] binary=new byte[image.getWidth()*image.getHeight()];int foreground=0;
   for(int y=0;y<image.getHeight();y++)for(int x=0;x<image.getWidth();x++){int color=image.getRGB(x,y);boolean black=(color>>>24)>=140&&((color>>16)&255)<140;binary[y*image.getWidth()+x]=(byte)(black?0:255);if(black)foreground++;}
   String file=name+"-"+size+".raw";Files.write(out.resolve(file),binary);
   StringJoiner points=new StringJoiner(",","[","]"),anchors=new StringJoiner(",","[","]");
   for(Object p:(java.util.List<?>)get(template,"getKeyPoints"))points.add("["+p.getClass().getField("x").get(p)+","+p.getClass().getField("y").get(p)+","+p.getClass().getField("d").get(p)+"]");
   for(var entry:((Map<?,?>)get(template,"getOffsets")).entrySet()){Point2D p=(Point2D)entry.getValue();anchors.add("{\"name\":\""+entry.getKey()+"\",\"x\":"+p.getX()+",\"y\":"+p.getY()+"}");}
   StringJoiner scores=new StringJoiner(",","[","]");scores.add("{\"input\":\"official\",\"score\":"+score(template,image.getWidth(),image.getHeight(),binary)+"}");
   if(args.length>2)for(boolean aa:new boolean[]{false,true}){byte[] nativePixels=Files.readAllBytes(Path.of(args[2]).resolve(name+"-"+size+"-aa"+aa+".raw"));if(nativePixels.length!=binary.length)throw new IllegalArgumentException("Native size mismatch");scores.add("{\"input\":\"aa"+aa+"\",\"score\":"+score(template,image.getWidth(),image.getHeight(),nativePixels)+"}");}
   assets.add("{\"shape\":\""+name+"\",\"point_size\":"+size+",\"width\":"+image.getWidth()+",\"height\":"+image.getHeight()+",\"foreground\":"+foreground+",\"file\":\""+file+"\",\"layout_bounds\":"+rect(layout.getBounds())+",\"fat_bounds\":"+rect(fat)+",\"slim_bounds\":"+rect((Rectangle)get(template,"getSlimBounds"))+",\"points\":"+points+",\"anchors\":"+anchors+",\"scores\":"+scores+"}");
  }
  StringJoiner sizing=new StringJoiner(",","[","]");Object font=fontc.getMethod("getMusicFont",mf,int.class).invoke(null,family,44);
  for(double width:new double[]{15}){Object scale=fontc.getMethod("buildMusicFontScale",double.class).invoke(font,width);sizing.add("{\"measured_head_width\":"+width+",\"point_size\":"+get(scale,"getPointSize")+"}");}
  StringJoiner metrics=new StringJoiner(",","[","]");
  for(int size:new int[]{50,51,54}){Object f=fontc.getMethod("getMusicFont",mf,int.class).invoke(null,family,size);TextLayout layout=(TextLayout)fontc.getMethod("layoutShapeByCode",sc).invoke(f,Enum.valueOf((Class)sc,"NOTEHEAD_BLACK"));metrics.add("{\"point_size\":"+size+",\"width\":"+layout.getBounds().getWidth()+"}");}
  Files.writeString(out.resolve("calibration.json"),metrics.toString()+"\n");
  Files.writeString(out.resolve("assets.json"),"{\"schema\":1,\"audiveris\":\"5.11.0\",\"commit\":\"9e1e55cd2746037d059345881c53e6a6754bffbd\",\"family\":\""+args[1]+"\",\"java\":\""+System.getProperty("java.version")+"\",\"os\":\""+System.getProperty("os.name")+" "+System.getProperty("os.arch")+"\",\"sizing\":"+sizing+",\"assets\":"+assets+"}\n");
  System.exit(0);
 }
}
