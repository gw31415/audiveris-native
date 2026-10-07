// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class StemChunkGlyphProbe extends RookieMappingProbe {
 static Object physical(Object g)throws Exception {Object table=call(g,"getRunTable");List<Object> runs=new ArrayList<>();for(int pos=0;pos<(int)call(table,"getSize");pos++){Iterator<?> it=(Iterator<?>)call(table,"iterator",pos);while(it.hasNext()){Object r=it.next();runs.add(List.of(pos,call(r,"getStart"),call(r,"getLength")));}}return obj("bounds",rect(call(g,"getBounds")),"orientation",call(table,"getOrientation").toString(),"runs",runs);}
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);
  Object vertical=c("run.Orientation").getField("VERTICAL").get(null);
  Object section=c("lag.DynamicSection").getConstructor(c("run.Orientation")).newInstance(vertical);
  call(section,"setId",1);call(section,"setFirstPos",528);
  int[][] runs={{246,2},{245,4},{245,4},{245,4}};
  for(int[] r:runs)call(section,"append",c("run.Run").getConstructor(int.class,int.class).newInstance(r[0],r[1]));
  Object factory=c("glyph.dynamic.StickFactory").getConstructor(c("run.Orientation"),c("sheet.SystemInfo"),c("glyph.dynamic.FilamentIndex"),java.util.function.Predicate.class,int.class,int.class,double.class).newInstance(vertical,sys,call(sheet,"getFilamentIndex"),null,5,0,.4);
  List<?> sticks=(List<?>)call(factory,"retrieveSticks",new ArrayList<>(List.of(section)),new ArrayList<>());
  List<Object> observed=new ArrayList<>();
  for(Object stick:sticks){Object glyph=call(stick,"toGlyph",(Object)null);String failure=null;
   try{call(stick,"getStartPoint",vertical);}catch(Exception e){Throwable t=e;while(t.getCause()!=null)t=t.getCause();failure=t.getClass().getName()+": "+t.getMessage();}
   observed.add(obj("glyph",physical(glyph),"forced_endpoint_exception",failure));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","declared section from native failing 4x4 input ink; original StickFactory.retrieveSticks/toGlyph control; not replay of image lookup selection", "vertical_sections",List.of(obj("id",1,"vertical",true,"first_pos",528,"runs",runs)),"candidates",observed));System.exit(0);
 }
}
