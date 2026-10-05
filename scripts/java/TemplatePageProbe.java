// SPDX-License-Identifier: AGPL-3.0-or-later
// Development oracle: original raster -> official VerticalFilter -> official
// ChamferDistance.Short -> official Bravura templates. No native intermediates.
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
import javax.imageio.ImageIO;
public class TemplatePageProbe {
 @SuppressWarnings({"unchecked","rawtypes"})
 public static void main(String[] args) throws Exception {
  var image=ImageIO.read(Path.of(args[0]).toFile());
  int w=image.getWidth(),h=image.getHeight();byte[] pixels=new byte[w*h];
  for(int y=0;y<h;y++)for(int x=0;x<w;x++)pixels[y*w+x]=(byte)image.getRaster().getSample(x,y,0);
  Class<?> bp=Class.forName("ij.process.ByteProcessor");
  Object b=bp.getConstructor(int.class,int.class,byte[].class).newInstance(w,h,pixels);
  Class<?> vf=Class.forName("org.audiveris.omr.image.VerticalFilter");
  Object filter=vf.getConstructor(bp,double.class,double.class).newInstance(b,0.7,0.9);
  Object binary=vf.getMethod("filteredImage").invoke(filter);
  byte[] bin=((byte[])bp.getMethod("getPixels").invoke(binary)).clone();
  var binaryHash=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bin));
  boolean mask=Boolean.parseBoolean(args[3]);
  // Diagnostic mask only. Does not claim DistancesBuilder semantic selection.
  for(int i=0;i<bin.length;i++)if(mask&&(i%997==0||i/w==h/2))bin[i]=(byte)255;
  Object source=bp.getConstructor(int.class,int.class,byte[].class).newInstance(w,h,bin);
  Class<?> cd=Class.forName("org.audiveris.omr.image.ChamferDistance$Short");
  Object distances=cd.getMethod("computeToFore",bp).invoke(cd.getConstructor().newInstance(),source);
  Class<?> dt=Class.forName("org.audiveris.omr.image.DistanceTable");
  var set=dt.getMethod("setValue",int.class,int.class);
  for(int i=0;i<bin.length;i++)if(mask&&(i%997==0||i/w==h/2))set.invoke(distances,i,-1);
  var get=dt.getMethod("getValue",int.class);var digest=MessageDigest.getInstance("SHA-256");
  for(int i=0;i<bin.length;i++){int v=(Integer)get.invoke(distances,i);digest.update((byte)(v>>>8));digest.update((byte)v);}
  var distanceHash=HexFormat.of().formatHex(digest.digest());
  Class<?> fc=Class.forName("org.audiveris.omr.image.TemplateFactory"),mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),sc=Class.forName("org.audiveris.omr.glyph.Shape"),ac=Class.forName("org.audiveris.omr.image.Anchored$Anchor"),tc=Class.forName("org.audiveris.omr.image.Template");
  Class.forName("org.audiveris.omr.ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object factory=fc.getMethod("getInstance").invoke(null),family=Enum.valueOf((Class)mf,"Bravura");
  Object catalog=fc.getMethod("getCatalog",mf,int.class).invoke(factory,family,51);
  var results=new StringJoiner(",","[","]");
  for(String line:Files.readAllLines(Path.of(args[1]))){String[]a=line.split("\\t");
   Object t=catalog.getClass().getMethod("getTemplate",sc).invoke(catalog,Enum.valueOf((Class)sc,a[0]));
   int x=Integer.parseInt(a[1]),y=Integer.parseInt(a[2]);Object anchor=a[3].equals("null")?null:Enum.valueOf((Class)ac,a[3]);
   double score=(Double)tc.getMethod("evaluate",int.class,int.class,ac,dt).invoke(t,x,y,anchor,distances);
   double hole=(Double)tc.getMethod("evaluateHole",int.class,int.class,ac,dt).invoke(t,x,y,anchor,distances);
   results.add("{\"score\":"+score+",\"hole_white_ratio\":"+hole+"}");
  }
  Files.writeString(Path.of(args[2]),"{\"width\":"+w+",\"height\":"+h+",\"binary_sha256\":\""+binaryHash+"\",\"distance_sha256_be_i16\":\""+distanceHash+"\",\"results\":"+results+"}");
  System.exit(0);
 }
}
