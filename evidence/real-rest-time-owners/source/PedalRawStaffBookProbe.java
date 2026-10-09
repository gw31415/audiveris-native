import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import java.awt.Rectangle;
public class PedalRawStaffBookProbe {
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
 static boolean is(Object x,String k){return x.getClass().getSimpleName().equals(k);}
 static Object row(Object sig,Object n,boolean effective)throws Exception{
  Map<String,Object> r=map("id",id(n),"object",label(n),"kind",n.getClass().getSimpleName(),"shape",field(n,"shape"),"bounds",rect(field(n,"bounds")),"raw_staff",id(field(n,"staff")),"raw_staff_object",label(field(n,"staff")),"raw_grade",field(n,"grade"),"raw_context",field(n,"ctxGrade"),"manual",field(n,"manual"),"frozen",field(n,"frozen"),"removed",field(n,"removed"),"registered",((Set<?>)call(sig,"vertexSet")).contains(n));
  if(effective)r.put("effective_context",call(n,"getContextualGrade"));
  if(is(n,"HeadChordInter")||is(n,"RestChordInter")){for(String f:List.of("voice","slot","timeOffset","duration")){try{Object v=field(n,f);r.put("raw_"+f,v==null?null:v.toString());r.put(f+"_object",label(v));}catch(NoSuchFieldException e){r.put("raw_"+f,"FIELD_NOT_PRESENT");}}}
  if(is(n,"ClefInter")){List<Object> owners=new ArrayList<>();Object sys=call(sig,"getSystem");for(Object p:(List<?>)call(sys,"getParts")){int mi=0;for(Object m:(List<?>)call(p,"getMeasures")){mi++;if(((Collection<?>)call(m,"getClefs")).contains(n))owners.add(map("part",id(p),"measure",mi,"measure_object",label(m)));}}r.put("actual_clef_memberships",owners);}
  return r;
 }
 static Object frame(Object sig,String title,boolean all,boolean effective)throws Exception{List<Object> rs=new ArrayList<>();for(Object n:(Set<?>)call(sig,"vertexSet"))if(all||is(n,"PedalInter"))rs.add(row(sig,n,effective));return map("stage",title,"system",id(call(sig,"getSystem")),"inters",rs);}
 static void listen(Object sig)throws Exception{Class<?> api=Class.forName("org.jgrapht.event.GraphListener");Object proxy=Proxy.newProxyInstance(api.getClassLoader(),new Class<?>[]{api},(o,m,args)->{if(m.getDeclaringClass()==Object.class)return switch(m.getName()){case "equals" -> o==args[0];case "hashCode" -> System.identityHashCode(o);default -> "raw-field-observer";};if(args==null||args.length!=1)return null;try{Object e=args[0];if(m.getName().equals("vertexRemoved")){Object n=call(e,"getVertex");events.add(map("stage",stage,"event",m.getName(),"inter",row(sig,n,false)));}else if(m.getName().equals("edgeAdded")||m.getName().equals("edgeRemoved")){Object rel=call(e,"getEdge"),s=call(e,"getEdgeSource"),t=call(e,"getEdgeTarget");if(is(s,"PedalInter")||is(t,"PedalInter")||is(s,"OctaveShiftInter")||is(t,"OctaveShiftInter"))events.add(map("stage",stage,"event",m.getName(),"relation_kind",rel.getClass().getSimpleName(),"relation_object",label(rel),"manual",call(rel,"isManual"),"source",row(sig,s,false),"target",row(sig,t,false)));}}catch(Exception ex){errors.add(ex.toString());}return null;});call(sig,"addGraphListener",proxy);}
 public static void main(String[] args)throws Exception{
  Object book=type("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object stub=((List<?>)call(book,"getStubs")).get(0),sheet=call(stub,"getSheet");
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   if(((Number)id(sys)).intValue()!=2)continue;
   Object sig=call(sys,"getSig");listen(sig);frames.add(frame(sig,"book_loaded_SYMBOlS_no_getStaff",false,false));
   stage="initial_MeasureFiller";call(make("sheet.rhythm.MeasureFiller",sys),"process");frames.add(frame(sig,stage,false,false));
   Object linker=make("sheet.symbol.SymbolsLinker",sys);
   for(String method:List.of("linkDynamics","linkTexts","linkPedals","linkWedges","linkFermatas","linkGraces","linkAugmentationDots","linkTuplets","linkOctaveShifts","linkNumbers")){
    stage=method;call(linker,method);frames.add(frame(sig,stage,false,false));
   }
   stage="key_candidates";for(Method m:type("sig.inter.KeyInter").getDeclaredMethods())if(m.getName().equals("lookupCandidates")&&m.getParameterCount()==1){m.setAccessible(true);m.invoke(null,sys);break;}
   frames.add(frame(sig,"before_reducer",true,false));
   Object reducer=make("sig.SigReducer",sys,true);
   Collection<?> headers=(Collection<?>)call(reducer,"getHeadersInters");Object predicate=field(type("sig.SigReducer"),"overlapPredicate");
   List<Object> filtered=new ArrayList<>();
   for(Object n:(Collection<?>)call(sig,"inters",predicate))if(!headers.contains(n)&&!(Boolean)call(n,"isRemoved")&&!(Boolean)call(n,"isImplicit"))filtered.add(n);
   @SuppressWarnings("unchecked") Comparator<Object> cmp=(Comparator<Object>)field(type("sig.inter.Inters"),"byAbscissa");filtered.sort(cmp);
   stage="detectOverlaps";call(reducer,"detectOverlaps",filtered);frames.add(frame(sig,stage,false,false));
   stage="checkFrozens";call(reducer,"analyzeFrozenInters");frames.add(frame(sig,stage,false,false));
   stage="first_context";call(sig,"contextualize");frames.add(frame(sig,stage,true,true));
   stage="analyzeChords_prolog";call(reducer,"analyzeChords");frames.add(frame(sig,stage,false,false));
   stage="second_context";call(sig,"contextualize");frames.add(frame(sig,stage,true,true));
   List<Object> queue=new ArrayList<>();for(Object n:(Set<?>)call(sig,"vertexSet"))if(!(Boolean)call(n,"isFrozen")&&!String.valueOf(call(n,"getShape")).equals("LEDGER")){Object cg=call(n,"getContextualGrade");if(cg==null)throw new IllegalStateException("source null effective context for "+id(n));if(((Number)cg).doubleValue()<.5)queue.add(row(sig,n,true));}
   frames.add(map("system",id(sys),"stage","selected_first_weak_queue","inters",queue));
   stage="first_weak_purge";call(sig,"deleteWeakInters");frames.add(frame(sig,stage,true,true));
   stage="declared_lazy_getStaff_control";List<Object> controls=new ArrayList<>();for(Object n:(Set<?>)call(sig,"vertexSet"))if(is(n,"PedalInter")){Object before=row(sig,n,false),staff=call(n,"getStaff");controls.add(map("before",before,"getter_staff",id(staff),"after",row(sig,n,false)));}frames.add(map("system",id(sys),"stage",stage,"controls",controls));
  }
  Files.writeString(Path.of(args[1]),json(map("scope","Bounded existing SYMBOLS book, system2 only; no recognition/image execution or source book save. Original linker component calls in process order, then original overlap/frozen/context/prolog/context/deleteWeakInters in first reducer order; stops before later consistency/epoch loops. Raw fields avoid Staff/cache materialization until separately declared final getStaff control. Persisted intrinsic grades rounded; not fresh image numeric parity.","input_book",args[0],"native_outputs_used",false,"frames",frames,"events",events,"errors",errors))+"\n");
 }
}
