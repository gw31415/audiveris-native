// SPDX-License-Identifier: AGPL-3.0-or-later
// Original Glyph NearLine and StemChecker on independently constructed rasters.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.geom.*;import java.lang.reflect.*;
public class StemGlyphGradeProbe extends HeadStemConnectionProbe {
 public StemGlyphGradeProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String checkedGlyph(Object checker,Object glyph,int profile)throws Exception {
  Object suite=call(checker,"getSuite",profile);Constructor<?> ctor=Class.forName("org.audiveris.omr.sheet.stem.StemChecker$StickContext").getDeclaredConstructor(Class.forName("org.audiveris.omr.glyph.NearLine"));ctor.setAccessible(true);Object context=ctor.newInstance(glyph),result=call(suite,"getImpacts",context);Object actual=call(checker,"checkStem",glyph,profile);if(!call(result,"getGrade").equals(call(actual,"getGrade")))throw new IllegalStateException("grade replay");
  List<String> raw=new ArrayList<>();for(Object c:(List<?>)call(suite,"getChecks"))raw.add("{\"name\":"+q(call(c,"getName"))+",\"raw_value\":"+call(c,"getValue",context)+",\"low\":"+call(c,"getLow")+",\"high\":"+call(c,"getHigh")+",\"covariant\":"+call(c,"isCovariant")+"}");
  return "{\"context_black\":"+get(context,"black")+",\"context_white\":"+get(context,"white")+",\"context_gap\":"+get(context,"gap")+",\"impacts\":"+StemSeedsProbe.impacts(result)+",\"raw_checks\":["+String.join(",",raw)+"],\"min_threshold\":"+call(checker,"getMinThreshold",profile)+"}";
 }
 @Override void controls(Path out)throws Exception {
  Object scale=call(sheet,"getScale"),checker=make("sheet.stem.StemChecker",sheet),saved=get(checker,"pixelFilter"),vertical=en("run.Orientation","VERTICAL");List<String> cases=new ArrayList<>();
  try{for(String orientation:new String[]{"VERTICAL","HORIZONTAL"})for(int height:new int[]{2,3,7,15,20,31,40,64})for(int thick:new int[]{1,2,3})for(int pattern=0;pattern<4;pattern++)for(int mask=0;mask<3;mask++){
   int width=thick+(pattern==1||pattern==2?(height-1)/12:pattern==3?1:0);byte[] pixels=new byte[width*height];Arrays.fill(pixels,(byte)255);for(int y=0;y<height;y++){int x=pattern==1?y/12:pattern==2?(height-1)/12-y/12:pattern==3?y%2:0;for(int k=0;k<thick;k++)pixels[y*width+x+k]=0;}
   Object bp=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(width,height,pixels),table=call(make("run.RunTableFactory",en("run.Orientation",orientation)),"createTable",bp),glyph=make("glyph.Glyph",40,40,table);byte[] image=new byte[128*160];Arrays.fill(image,(byte)255);for(int y=0;y<height;y++)for(int x=0;x<width;x++)image[(40+y)*128+40+x]=pixels[y*width+x];
   if(mask==1)for(int y=0;y<height;y++)for(int x=36;x<40+width+4;x++)image[(40+y)*128+x]=0;
   if(mask==2)for(int y=height/3;y<2*height/3;y++)for(int x=0;x<width;x++)image[(40+y)*128+40+x]=(byte)255;
   Object raster=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(128,160,image),imageTable=call(make("run.RunTableFactory",en("run.Orientation","HORIZONTAL")),"createTable",raster),imageGlyph=make("glyph.Glyph",0,0,imageTable);setField(checker,"pixelFilter",raster);
   String candidate="{\"bounds\":"+rect(call(glyph,"getBounds"))+",\"start\":"+pt((Point2D)call(glyph,"getStartPoint",vertical))+",\"stop\":"+pt((Point2D)call(glyph,"getStopPoint",vertical))+",\"line\":"+line((Line2D)call(glyph,"getLine"))+",\"mean_distance\":"+call(glyph,"getMeanDistance")+"}";
   for(int profile:new int[]{0,1})cases.add("{\"pattern\":"+pattern+",\"mask\":"+mask+",\"glyph\":"+canonicalGlyph(glyph)+",\"image\":"+canonicalGlyph(imageGlyph)+",\"candidate\":"+candidate+",\"parameters\":{\"interline\":"+call(scale,"getInterline")+",\"max_stem\":"+call(scale,"getMaxStem")+",\"global_slope\":"+call(call(sheet,"getSkew"),"getSlope")+",\"profile\":"+profile+"},\"checked\":"+checkedGlyph(checker,glyph,profile)+"}");
  }}finally{setField(checker,"pixelFilter",saved);}
  String document="{\"native_outputs_used\":false,\"scope\":\"Original physical Glyph NearLine plus StemChecker profiles 0/1 on independent controlled pixel filters; not automatic part selection, StemBuilder cache or full-page STEMS.\",\"cases\":["+String.join(",",cases)+"]}\n"; Files.writeString(out.resolveSibling("stem-glyph-grade-controls.json"), document.replace("-Infinity","\"-Infinity\"").replace("Infinity","\"Infinity\"").replace("NaN","\"NaN\""));
 }
}
