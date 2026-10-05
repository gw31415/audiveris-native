// SPDX-License-Identifier: AGPL-3.0-or-later
// Original raster GRID + original private ClefBuilder methods; no algorithm replacement.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.lang.reflect.*;
public class ClefCandidateProbe extends GridFinalProbe {
 private final Object book;public ClefCandidateProbe(Book b,SortedSet<Integer>ids){super(b,ids);book=b;}
 static void put(Object o,String name,Object v)throws Exception{for(Class<?>c=o.getClass();c!=null;c=c.getSuperclass()){try{Field f=c.getDeclaredField(name);f.setAccessible(true);f.set(o,v);return;}catch(NoSuchFieldException e){}}throw new NoSuchFieldException(name);}
 static String parts(Object builder,boolean first)throws Exception{List<String>out=new ArrayList<>();for(Object g:(List<?>)call(builder,"getParts",first))out.add(glyph(g));return "["+String.join(",",out)+"]";}
 static String best(Object builder,boolean first)throws Exception{List<String>out=new ArrayList<>();for(var e:((Map<?,?>)call(builder,"getBestMap",first)).entrySet()){Object inter=e.getValue();out.add("{\"kind\":\""+e.getKey()+"\",\"shape\":\""+call(inter,"getShape")+"\",\"grade\":"+call(inter,"getGrade")+",\"glyph\":"+glyph(call(inter,"getGlyph"))+"}");}return "["+String.join(",",out)+"]";}
 public void process(){try{
 Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);headerPreparation(sheet);List<String>staffs=new ArrayList<>();
 for(Object staff:(List<?>)call(call(sheet,"getStaffManager"),"getStaves")){
  Object builder=Class.forName("org.audiveris.omr.sheet.clef.ClefBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.Staff")).newInstance(staff);call(builder,"setBrowseStart",call(staff,"getHeaderStart"));Object outer=call(builder,"getOuterRect"),inner=call(builder,"getInnerRect",outer);put(builder,"outerRect",outer);put(builder,"innerRect",inner);
  String first=parts(builder,true),second=parts(builder,false),firstBest=best(builder,true),secondBest=best(builder,false);
  staffs.add("{\"staff_id\":"+call(staff,"getId")+",\"outer\":"+rect(outer)+",\"inner\":"+rect(inner)+",\"first_parts\":"+first+",\"second_parts\":"+second+",\"first_best\":"+firstBest+",\"second_best\":"+secondBest+"}");
 }
 Class<?>shape=Class.forName("org.audiveris.omr.glyph.Shape"),pitched=Class.forName("org.audiveris.omr.sig.inter.AbstractPitchedInter");double offset=(Double)pitched.getMethod("getAreaPitchOffset",shape).invoke(null,shape.getField("F_CLEF").get(null));
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"f_clef_pitch_offset\":"+offset+",\"staffs\":["+String.join(",",staffs)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
