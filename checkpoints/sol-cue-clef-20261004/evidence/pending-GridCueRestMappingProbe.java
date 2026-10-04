// SPDX-License-Identifier: AGPL-3.0-or-later
// Actual original raster GRID: sheet/staff scales, neighbor selection and pitch.
// No supplied native geometry, expectations, fabricated objects or recognition changes.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.awt.geom.Point2D;
import java.nio.file.*;

public class GridCueRestMappingProbe extends GridFinalProbe {
 static String json(Object o) {
  if(o==null)return "null";
  if(o instanceof Map<?,?> m){List<String> keys=new ArrayList<>();for(Object k:m.keySet())keys.add(k.toString());Collections.sort(keys);List<String> values=new ArrayList<>();for(String k:keys)values.add("\""+k+"\":"+json(m.get(k)));return "{"+String.join(",",values)+"}";}
  if(o instanceof List<?> l){List<String> values=new ArrayList<>();for(Object x:l)values.add(json(x));return "["+String.join(",",values)+"]";}
  if(o instanceof Number || o instanceof Boolean)return o.toString();
  throw new IllegalArgumentException("unsupported observer value");
 }
 private final Object book;
 public GridCueRestMappingProbe(Book b, SortedSet<Integer> ids) {super(b,ids);book=b;}
 public void process() {
  try {
   Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
   call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
   Object scale=call(sheet,"getScale");int main=(Integer)call(scale,"getInterline");
   int width=(Integer)call(sheet,"getWidth"),height=(Integer)call(sheet,"getHeight");
   List<Object> records=new ArrayList<>();
   for(Object sys:(List<?>)call(sheet,"getSystems")) {
    List<?> staffs=(List<?>)call(sys,"getStaves");
    List<Object> staffStates=new ArrayList<>();TreeSet<Double> xs=new TreeSet<>(),ys=new TreeSet<>();
    Collections.addAll(xs,-1.,0.,.25,(double)width-1,(double)width);
    Collections.addAll(ys,-1.,0.,.25,(double)height-1,(double)height);
    for(Object staff:staffs){int il=(Integer)call(staff,"getSpecificInterline");
     staffStates.add(Map.of("id",call(staff,"getId"),"specific_interline",il));
     double left=((Number)get(staff,"left")).doubleValue(),right=((Number)get(staff,"right")).doubleValue();
     for(double x:new double[]{left-1,left,left+.25,(left+right)/2,right-.25,right,right+1})xs.add(x);
     Object first=call(staff,"getFirstLine"),last=call(staff,"getLastLine");
     double x=(left+right)/2,top=((Number)call(first,"yAt",x)).doubleValue(),bottom=((Number)call(last,"yAt",x)).doubleValue();
     for(double y:new double[]{top-main*.9-1,top-main*.9,top-main*.9+.25,top-1,top,top+.25,(top+bottom)/2,bottom-.25,bottom,bottom+1,bottom+main*.9-.25,bottom+main*.9,bottom+main*.9+1})ys.add(y);
    }
    List<Object> queries=new ArrayList<>();
    for(double x:xs)for(double y:ys){Point2D p=new Point2D.Double(x,y);
     Object closest=Class.forName("org.audiveris.omr.sheet.StaffManager").getMethod("getClosestStaff",Point2D.class,List.class).invoke(null,p,staffs);
     Object above=call(sys,"getStaffAtOrAbove",p);
     Map<String,Object> q=new LinkedHashMap<>();q.put("x",x);q.put("y",y);q.put("closest",closest==null?null:call(closest,"getId"));q.put("above",above==null?null:call(above,"getId"));
     List<Object> pitches=new ArrayList<>();for(Object staff:staffs)pitches.add(Map.of("staff_id",call(staff,"getId"),"pitch",call(staff,"pitchPositionOf",p)));
     q.put("pitches",pitches);queries.add(q);
    }
    records.add(Map.of("system_id",call(sys,"getId"),"staffs",staffStates,"queries",queries));
   }
   Files.writeString(Path.of(System.getProperty("native.probe.output")),json(Map.of("native_outputs_used",false,"main_interline",main,"width",width,"height",height,"systems",records)));
  }catch(Exception e){throw new RuntimeException(e);}
 }
}
