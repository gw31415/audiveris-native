// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.lang.reflect.*;
import java.awt.*;
import java.awt.geom.*;

// Keep reference labels local to each fresh JVM; IDs from independent runs are not identities.
public class RealTupletProbeSupport extends RealLedgerMembershipImageProbe {
 static final Map<Object,Integer> labels=new IdentityHashMap<>();
 static final java.util.List<Object> candidates=new ArrayList<>();
 public RealTupletProbeSupport(Book book,SortedSet<Integer> ids){super(book,ids);}
 static Object field(Object o,String name)throws Exception{for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try{Field f=k.getDeclaredField(name);f.setAccessible(true);return f.get(o);}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static Integer label(Object o){return o==null?null:labels.computeIfAbsent(o,k->labels.size()+1);}
 static Object id(Object o)throws Exception{return o==null?null:call(o,"getId");}
 static Map<String,Object> map(Object...a){Map<String,Object> m=new LinkedHashMap<>();for(int i=0;i<a.length;i+=2)m.put((String)a[i],a[i+1]);return m;}
 static Object bounds(Object o)throws Exception{Rectangle b=(Rectangle)call(o,"getBounds");return b==null?null:java.util.List.of(b.x,b.y,b.width,b.height);}
 static Object point(Object p){if(p==null)return null;Point2D q=(Point2D)p;return java.util.List.of(q.getX(),q.getY());}
 static Object owner(Object sys,Object measure)throws Exception{if(measure==null)return null;for(Object p:(java.util.List<?>)call(sys,"getParts")){int i=0;for(Object m:(java.util.List<?>)call(p,"getMeasures")){i++;if(m==measure)return map("system",id(sys),"part",id(p),"measure_index",i,"part_object",label(p),"measure_object",label(m),"stack_object",label(call(m,"getStack")));}}throw new Exception("measure not owned by actual system");}
 static boolean containsIdentity(Collection<?> xs,Object o){return xs!=null&&xs.stream().anyMatch(x->x==o);}
 static java.util.List<Object> identities(Collection<?> values)throws Exception{java.util.List<Object> out=new ArrayList<>();for(Object v:values)out.add(map("id",id(v),"object",label(v)));return out;}
 static String json(Object v){if(v==null)return "null";if(v instanceof Number||v instanceof Boolean)return v.toString();if(v instanceof Map<?,?>m){java.util.List<String>s=new ArrayList<>();for(var e:m.entrySet())s.add(json(e.getKey())+":"+json(e.getValue()));return "{"+String.join(",",s)+"}";}if(v instanceof Collection<?>xs){java.util.List<String>s=new ArrayList<>();for(Object x:xs)s.add(json(x));return "["+String.join(",",s)+"]";}return "\""+v.toString().replace("\\","\\\\").replace("\"","\\\"").replace("\n","\\n").replace("\r","\\r")+"\"";}
 static Object clock(Object o,String f)throws Exception{Object v=field(o,f);return v==null?null:v.toString();}
}
