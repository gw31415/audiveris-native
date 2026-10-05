// SPDX-License-Identifier: AGPL-3.0-or-later
// Original retry profiles on actual image scheduler-created physical stems.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.lang.reflect.*;import java.awt.geom.*;
public class StemRetryProfileProbe extends StemImageHeadExpansionProbe {
 private final Object book;
 public StemRetryProfileProbe(Book b,SortedSet<Integer>ids){super(b,ids);book=b;}
 static String checked(Object checker,Object glyph,int profile)throws Exception {
  Object suite=call(checker,"getSuite",profile);Constructor<?>ct=Class.forName("org.audiveris.omr.sheet.stem.StemChecker$StickContext").getDeclaredConstructor(Class.forName("org.audiveris.omr.glyph.NearLine"));ct.setAccessible(true);Object ctx=ct.newInstance(glyph),result=call(suite,"getImpacts",ctx);if(!call(result,"getGrade").equals(call(call(checker,"checkStem",glyph,profile),"getGrade")))throw new IllegalStateException("grade replay");
  List<String> checks=new ArrayList<>();int i=0;for(Object c:(List<?>)call(suite,"getChecks")){checks.add("{\"name\":\""+call(c,"getName")+"\",\"raw_value\":"+call(c,"getValue",ctx)+",\"low\":"+call(c,"getLow")+",\"high\":"+call(c,"getHigh")+",\"covariant\":"+call(c,"isCovariant")+",\"weight\":"+call(result,"getWeight",i)+",\"impact\":"+call(result,"getImpact",i)+"}");i++;}
  return "{\"context_black\":"+get(ctx,"black")+",\"context_white\":"+get(ctx,"white")+",\"context_gap\":"+get(ctx,"gap")+",\"grade\":"+call(result,"getGrade")+",\"min_threshold\":"+call(checker,"getMinThreshold",profile)+",\"checks\":["+String.join(",",checks)+"]}";
 }
 public void process(){try {
 Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),scale=call(sheet,"getScale"),source=call(call(sheet,"getPicture"),"getSource",Class.forName("org.audiveris.omr.sheet.Picture$SourceKey").getField("NO_STAFF").get(null));
 Object factory=Class.forName("org.audiveris.omr.run.RunTableFactory").getConstructor(Class.forName("org.audiveris.omr.run.Orientation")).newInstance(Class.forName("org.audiveris.omr.run.Orientation").getField("HORIZONTAL").get(null)),table=call(factory,"createTable",source),image=Class.forName("org.audiveris.omr.glyph.Glyph").getConstructor(int.class,int.class,Class.forName("org.audiveris.omr.run.RunTable")).newInstance(0,0,table);
 Object checker=Class.forName("org.audiveris.omr.sheet.stem.StemChecker").getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet")).newInstance(sheet);List<String>cases=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems")){Object retr=inspect(sys,sheet);call(retr,"linkStems");for(Object stem:(Collection<?>)call(call(sys,"getSig"),"vertexSet"))if(stem.getClass().getSimpleName().equals("StemInter")){Object glyph=call(stem,"getGlyph");for(int p=0;p<=3;p++)cases.add("{\"glyph\":"+physical(glyph)+",\"parameters\":{\"interline\":"+call(scale,"getInterline")+",\"max_stem\":"+call(scale,"getMaxStem")+",\"global_slope\":"+call(call(sheet,"getSkew"),"getSlope")+",\"profile\":"+p+"},\"checked\":"+checked(checker,glyph,p)+"}");}}
 Object saved=get(checker,"pixelFilter");
 try{for(String mode:new String[]{"gap","cling"})for(int count:(mode.equals("gap")?new int[]{0,3,6,7,21,22,23}:new int[]{0,1,2,3})) {
  byte[] solid=new byte[128];Object gp=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(2,64,solid);
  Object vf=Class.forName("org.audiveris.omr.run.RunTableFactory").getConstructor(Class.forName("org.audiveris.omr.run.Orientation")).newInstance(Class.forName("org.audiveris.omr.run.Orientation").getField("VERTICAL").get(null));
  Object glyph=Class.forName("org.audiveris.omr.glyph.Glyph").getConstructor(int.class,int.class,Class.forName("org.audiveris.omr.run.RunTable")).newInstance(40,40,call(vf,"createTable",gp));
  byte[] pix=new byte[128*160];Arrays.fill(pix,(byte)255);for(int y=40;y<104;y++)for(int x=40;x<42;x++)pix[y*128+x]=0;
  if(mode.equals("gap"))for(int y=60;y<60+count;y++)for(int x=40;x<42;x++)pix[y*128+x]=(byte)255;
  if(mode.equals("cling"))for(int y=40+count;y<104;y++)for(int x=36;x<46;x++)pix[y*128+x]=0;
  Object raster=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(128,160,pix);
  set(checker,"pixelFilter",raster);Object img=Class.forName("org.audiveris.omr.glyph.Glyph").getConstructor(int.class,int.class,Class.forName("org.audiveris.omr.run.RunTable")).newInstance(0,0,call(factory,"createTable",raster));
  for(int p=0;p<=3;p++)cases.add("{\"controlled_mode\":\""+mode+"\",\"controlled_count\":"+count+",\"no_staff_image\":"+physical(img)+",\"glyph\":"+physical(glyph)+",\"parameters\":{\"interline\":"+call(scale,"getInterline")+",\"max_stem\":"+call(scale,"getMaxStem")+",\"global_slope\":"+call(call(sheet,"getSkew"),"getSlope")+",\"profile\":"+p+"},\"checked\":"+checked(checker,glyph,p)+"}");
 }}finally{set(checker,"pixelFilter",saved);}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"Original image-created glyphs with StemChecker profiles0..3; not native full scheduler\",\"no_staff_image\":"+physical(image)+",\"cases\":["+String.join(",",cases)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
