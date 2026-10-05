package oracle;
import java.nio.file.*;
import java.util.*;
import org.audiveris.omr.sheet.Book;
import org.audiveris.omr.step.RunClass;
public class CoverageProbe extends RunClass {
 public CoverageProbe(Book book,SortedSet<Integer> ids){super(book,ids);}
 static Object get(Object o,String n)throws Exception{return o.getClass().getMethod(n).invoke(o);}
 static String strings(Collection<?> c){StringJoiner j=new StringJoiner(",","[","]");for(Object s:c)j.add("\""+s+"\"");return j.toString();}
 public void process(){try{
  Path out=Path.of(System.getProperty("oracle.output"));Files.createDirectories(out);
  StringJoiner sheets=new StringJoiner(",","[","]");
  for(Object stub:(List<?>)get(book,"getStubs")){
   Object sheet=get(stub,"getSheet"),scale=get(sheet,"getScale"),family=get(stub,"getMusicFamily"),black=get(scale,"getBlackHeadScale"),fs=get(scale,"getMusicFontScale");
   StringJoiner staves=new StringJoiner(",","[","]");
   for(Object system:(List<?>)get(sheet,"getSystems"))for(Object staff:(List<?>)get(system,"getStaves"))staves.add("{\"id\":"+get(staff,"getId")+",\"specific_interline\":"+get(staff,"getSpecificInterline")+",\"point_size\":"+get(staff,"getHeadPointSize")+",\"tablature\":"+get(staff,"isTablature")+",\"drum\":"+get(staff,"isDrum")+"}");
   Class<?> ss=Class.forName("org.audiveris.omr.glyph.ShapeSet"),sc=Class.forName("org.audiveris.omr.sheet.Sheet");
   StringJoiner sets=new StringJoiner(",","{","}");for(String n:new String[]{"All","Stem","Hollow"})sets.add("\""+n+"\":"+strings((Collection<?>)ss.getMethod("getTemplateNotes"+n,sc).invoke(null,sheet)));
   sheets.add("{\"family\":\""+family+"\",\"global_interline\":"+get(scale,"getInterline")+",\"mean_head_width\":"+get(black,"getWidthMean")+",\"music_point_size\":"+get(fs,"getPointSize")+",\"staves\":"+staves+",\"shapes\":"+sets+"}");
  }
  Files.writeString(out.resolve("coverage.json"),"{\"sheets\":"+sheets+",\"book_class_source\":\""+Book.class.getProtectionDomain().getCodeSource().getLocation()+"\"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
