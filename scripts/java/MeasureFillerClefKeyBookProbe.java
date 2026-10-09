import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import java.awt.Rectangle;
public class MeasureFillerClefKeyBookProbe {
 static final Map<Object,Integer> names=new IdentityHashMap<>();
 static final List<Object> frames=new ArrayList<>(), events=new ArrayList<>(), errors=new ArrayList<>();
 static String stage="load";
 static Class<?> type(String s)throws Exception{return Class.forName("org.audiveris.omr."+s);}
 static Object field(Object o,String name)throws Exception{Class<?> base=o instanceof Class<?>?(Class<?>)o:o.getClass();for(Class<?> c=base;c!=null;c=c.getSuperclass())try{Field f=c.getDeclaredField(name);f.setAccessible(true);return f.get(o instanceof Class<?>?null:o);}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static Object call(Object o,String name,Object...args)throws Exception{for(Class<?> c=o.getClass();c!=null;c=c.getSuperclass())for(Method m:c.getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){m.setAccessible(true);try{return m.invoke(o,args);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(name);}
 static Object make(String s,Object...args)throws Exception{for(Constructor<?> c:type(s).getDeclaredConstructors()){c.setAccessible(true);try{return c.newInstance(args);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(s);}
 static Integer label(Object o){return o==null?null:names.computeIfAbsent(o,x->names.size()+1);}
 static Object id(Object o)throws Exception{return o==null?null:call(o,"getId");}
 static Map<String,Object> map(Object...xs){Map<String,Object> out=new LinkedHashMap<>();for(int i=0;i<xs.length;i+=2)out.put((String)xs[i],xs[i+1]);return out;}
 static Object rect(Object o){if(o==null)return null;Rectangle r=(Rectangle)o;return List.of(r.x,r.y,r.width,r.height);}
 static String json(Object v){if(v==null)return "null";if(v instanceof Number||v instanceof Boolean)return v.toString();if(v instanceof Map<?,?>m){List<String> r=new ArrayList<>();for(var e:m.entrySet())r.add(json(e.getKey())+":"+json(e.getValue()));return "{"+String.join(",",r)+"}";}if(v instanceof Collection<?>xs){List<String> r=new ArrayList<>();for(Object x:xs)r.add(json(x));return "["+String.join(",",r)+"]";}return "\""+v.toString().replace("\\","\\\\").replace("\"","\\\"").replace("\n","\\n").replace("\r","\\r")+"\"";}

 static List<Object> ids(Collection<?> values)throws Exception{List<Object> out=new ArrayList<>();for(Object x:values)out.add(map("id",id(x),"object",label(x)));return out;}
 static Object point(Object value){if(value==null)return null;java.awt.Point p=(java.awt.Point)value;return List.of(p.x,p.y);}
 static Object inter(Object sig,Object x)throws Exception{
  Object s=field(x,"staff"),p=field(x,"part");
  return map("id",id(x),"object",label(x),"kind",x.getClass().getSimpleName(),"shape",field(x,"shape"),"raw_bounds",rect(field(x,"bounds")),"raw_staff",id(s),"raw_staff_object",label(s),"raw_part",id(p),"raw_part_object",label(p),"removed",field(x,"removed"),"registered",((Set<?>)call(sig,"vertexSet")).contains(x));
 }
 static Object measureRef(Object sys,Object m)throws Exception{
  if(m==null)return null;Object p=call(m,"getPart"),st=call(m,"getStack");
  return map("measure_object",label(m),"part",id(p),"part_object",label(p),"local_measure_index",((List<?>)call(p,"getMeasures")).indexOf(m)+1,"stack_object",label(st),"stack_index",((List<?>)call(sys,"getStacks")).indexOf(st)+1);
 }
 static List<Object> memberships(Object sys)throws Exception{
  List<Object> out=new ArrayList<>();Object sig=call(sys,"getSig");
  for(Object p:(List<?>)call(sys,"getParts"))for(Object m:(List<?>)call(p,"getMeasures")){
   Collection<?> cs=(Collection<?>)field(m,"clefs"),ks=(Collection<?>)field(m,"keys");
   List<Object> cr=new ArrayList<>(),kr=new ArrayList<>();for(Object x:cs)cr.add(inter(sig,x));for(Object x:ks)kr.add(inter(sig,x));
   out.add(map("measure",measureRef(sys,m),"clefs_collection_object",label(cs),"clefs_collection_class",cs.getClass().getName(),"clefs",cr,"keys_collection_object",label(ks),"keys_collection_class",ks.getClass().getName(),"keys",kr));
  }return out;
 }
 static Object graph(Object sig)throws Exception{
  List<Object> es=new ArrayList<>();for(Object e:(Set<?>)call(sig,"edgeSet"))es.add(map("object",label(e),"kind",e.getClass().getSimpleName(),"source",id(call(sig,"getEdgeSource",e)),"target",id(call(sig,"getEdgeTarget",e)),"manual",call(e,"isManual")));
  return map("vertices",ids((Set<?>)call(sig,"vertexSet")),"edges",es);
 }
 static Object frame(Object sys,String stage)throws Exception{
  Object sig=call(sys,"getSig");List<Object> headers=new ArrayList<>();
  for(Object s:(List<?>)call(sys,"getStaves")){
   Object h=call(s,"getHeader");
   if(h!=null)headers.add(map("staff",id(s),"staff_object",label(s),"header_object",label(h),"header_start",field(h,"start"),"header_stop",field(h,"stop"),"clef",id(field(h,"clef")),"clef_object",label(field(h,"clef")),"key",id(field(h,"key")),"key_object",label(field(h,"key"))));
  }
  List<Object> raw=new ArrayList<>();for(Object x:(Set<?>)call(sig,"vertexSet"))if(x.getClass().getSimpleName().equals("ClefInter")||x.getClass().getSimpleName().equals("KeyInter"))raw.add(inter(sig,x));
  return map("stage",stage,"system",id(sys),"system_object",label(sys),"memberships",memberships(sys),"headers",headers,"raw_clef_key_vertices",raw,"graph",graph(sig));
 }
 static Object filters(Object sys)throws Exception{
  Object sig=call(sys,"getSig"),classes=field(type("sheet.rhythm.MeasureFiller"),"FILLING_CLASSES");
  Collection<?> filling=(Collection<?>)call(sig,"inters",classes);
  List<Object> initial=new ArrayList<>();for(Object x:filling)initial.add(inter(sig,x));
  List<Object> stacks=new ArrayList<>();
  for(Object stack:(List<?>)call(sys,"getStacks")){
   Collection<?> kept=(Collection<?>)call(stack,"filter",filling);List<Object> rows=new ArrayList<>();
   for(Object x:kept){
    Object before=inter(sig,x),c=call(x,"getCenter"),s=call(x,"getStaff"),p=call(x,"getPart"),m=p==null?null:call(stack,"getMeasureAt",p);
    Object areaStaff=s==null?null:call(call(s,"getPart"),"getStaffJustAbove",c);
    Object filterMeasure=s==null?null:call(call(s,"getPart"),"getMeasureAt",c);
    rows.add(map("before_declared_getPart_dispatch",before,"integer_center",point(c),"actual_staff",id(s),"staff_object",label(s),"staff_part",s==null?null:id(call(s,"getPart")),"staff_part_object",s==null?null:label(call(s,"getPart")),"area_selected_staff",id(areaStaff),"area_selected_staff_object",label(areaStaff),"filter_measure",measureRef(sys,filterMeasure),"dispatch_part",id(p),"dispatch_part_object",label(p),"dispatch_measure",measureRef(sys,m),"after_declared_getPart_dispatch",inter(sig,x)));
   }
   stacks.add(map("stack_index",((List<?>)call(sys,"getStacks")).indexOf(stack)+1,"stack_object",label(stack),"left",field(stack,"left"),"right",field(stack,"right"),"kept",rows));
  }
  List<Object> cn=new ArrayList<>();for(Object c:(Object[])classes)cn.add(((Class<?>)c).getSimpleName());
  return map("stage","declared_original_SIG_filling_collection_and_Stack_filter_then_getPart_observation","system",id(sys),"filling_classes",cn,"filling_collection",initial,"stacks",stacks);
 }
 public static void main(String[] args)throws Exception{
  try{
   Object book=type("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
   Object stub=((List<?>)call(book,"getStubs")).get(0),sheet=call(stub,"getSheet");
   List<?> systems=(List<?>)call(sheet,"getSystems");
   for(Object sys:systems)frames.add(frame(sys,"immediately_after_getSheet_afterReload_no_explicit_Filler"));
   for(Object sys:systems){
    frames.add(filters(sys));
    frames.add(frame(sys,"after_observational_filter_before_process"));
    Object filler=make("sheet.rhythm.MeasureFiller",sys);
    call(filler,"process");frames.add(frame(sys,"after_original_MeasureFiller_process_1"));
    call(filler,"process");frames.add(frame(sys,"after_original_MeasureFiller_process_2"));
   }
  }catch(Throwable ex){errors.add(ex.toString());throw ex;}
  finally{
   Files.writeString(Path.of(args[1]),json(map("scope","Existing original SYMBOLS saved-book only, both systems. Loading getSheet runs original afterReload including Measure.afterReload admission, so immediate loaded membership is not fresh pre-Filler membership. Explicit filter/getPart observation precedes two original MeasureFiller.process calls; only original object references, no image/reachStep/reducer/book save/native outputs.","input_book",args[0],"frames",frames,"errors",errors))+"\n");
  }
 }
}
