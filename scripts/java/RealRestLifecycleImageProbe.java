// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import org.audiveris.omr.sheet.SystemInfo;
import org.audiveris.omr.sheet.Staff;
import org.audiveris.omr.glyph.Glyph;
import org.audiveris.omr.classifier.Evaluation;
import org.audiveris.omr.sig.inter.Inter;
import java.nio.file.*;import java.util.*;import java.lang.reflect.*;import java.awt.*;import java.awt.geom.*;
public class RealRestLifecycleImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 static final Map<Object,Integer> labels=new IdentityHashMap<>();
 static final java.util.List<Object> candidates=new ArrayList<>();
 public RealRestLifecycleImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
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
 public static class ObservedFactory extends org.audiveris.omr.sheet.symbol.InterFactory {
  private final Object actualSystem;
  public ObservedFactory(SystemInfo system){super(system);actualSystem=system;}
  @Override public Inter create(Evaluation eval,Glyph glyph,Staff staff){try{
   String shape=field(eval,"shape").toString();if(!shape.endsWith("_REST"))return super.create(eval,glyph,staff);
   Object centroid=call(glyph,"getCentroid"),selected=call(actualSystem,"getClosestStaff",centroid);Object part=selected==null?null:call(selected,"getPart");Object measure=part==null?null:call(part,"getMeasureAt",centroid);
   Map<String,Object> row=map("order",candidates.size()+1,"system",id(actualSystem),"shape",shape,"evaluation_grade",field(eval,"grade"),"glyph_id",id(glyph),"glyph_object",label(glyph),"bounds",bounds(glyph),"centroid",point(centroid),"center2D",point(call(glyph,"getCenter2D")),"passed_staff",id(staff),"passed_staff_object",label(staff),"centroid_staff",id(selected),"centroid_staff_object",label(selected),"centroid_measure",owner(actualSystem,measure));
   Inter result=super.create(eval,glyph,staff);row.put("returns_null",result==null);row.put("rest_id",id(result));row.put("rest_object",label(result));if(result!=null){Object heldStaff=call(result,"getStaff");row.put("grade",call(result,"getGrade"));row.put("pitch",field(result,"pitch"));row.put("actual_staff",id(heldStaff));row.put("actual_staff_object",label(heldStaff));row.put("staff_note_registered",containsIdentity((Collection<?>)field(heldStaff,"notes"),result));row.put("rest_chord_before_build",id(call(result,"getChord")));}
   candidates.add(row);return result;
  }catch(Exception e){throw new RuntimeException(e);}}
 }
 static Object frame(Object sys,String stage)throws Exception{
  Object sig=call(sys,"getSig");java.util.List<Object> rests=new ArrayList<>(),chords=new ArrayList<>(),measures=new ArrayList<>(),relations=new ArrayList<>();
  for(Object rest:(Collection<?>)call(sig,"inters",c("sig.inter.RestInter"))){Object staff=call(rest,"getStaff"),chord=call(rest,"getChord"),glyph=call(rest,"getGlyph");rests.add(map("id",id(rest),"object",label(rest),"shape",call(rest,"getShape"),"bounds",bounds(rest),"grade",call(rest,"getGrade"),"manual",call(rest,"isManual"),"pitch",field(rest,"pitch"),"staff",id(staff),"staff_object",label(staff),"part",id(call(staff,"getPart")),"staff_note_registered",containsIdentity((Collection<?>)field(staff,"notes"),rest),"glyph_id",id(glyph),"glyph_object",label(glyph),"centroid",point(call(glyph,"getCentroid")),"chord_id",id(chord),"chord_object",label(chord)));}
  for(Object chord:(Collection<?>)call(sig,"inters",c("sig.inter.RestChordInter"))){Object measure=call(chord,"getMeasure"),staff=call(chord,"getStaff");chords.add(map("id",id(chord),"object",label(chord),"bounds",bounds(chord),"grade",call(chord,"getGrade"),"manual",call(chord,"isManual"),"staff",id(staff),"staff_object",label(staff),"part",id(call(chord,"getPart")),"measure",owner(sys,measure),"measure_registered",measure!=null&&containsIdentity((Collection<?>)call(measure,"getRestChords"),chord),"members",identities((Collection<?>)call(chord,"getMembers")),"voice",clock(chord,"voice"),"slot",clock(chord,"slot"),"time_offset",clock(chord,"timeOffset")));}
  for(Object part:(java.util.List<?>)call(sys,"getParts"))for(Object measure:(java.util.List<?>)call(part,"getMeasures"))measures.add(map("owner",owner(sys,measure),"rest_chords",identities((Collection<?>)call(measure,"getRestChords"))));
  for(Object relation:(Set<?>)call(sig,"edgeSet"))if(relation.getClass().getSimpleName().equals("Containment")){Object source=call(sig,"getEdgeSource",relation);if(c("sig.inter.RestChordInter").isInstance(source))relations.add(map("object",label(relation),"source",id(source),"source_object",label(source),"target",id(call(sig,"getEdgeTarget",relation)),"target_object",label(call(sig,"getEdgeTarget",relation)),"manual",call(relation,"isManual")));}
  return map("system",id(sys),"stage",stage,"rests",rests,"rest_chords",chords,"measures",measures,"containment",relations);
 }
 public void process(){try{
  Object stub=((java.util.List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CURVES").get(null),false);Object sheet=call(stub,"getSheet");Object step=make("sheet.symbol.SymbolsStep"),context=call(step,"doProlog",sheet),optionals=field(context,"optionalsMap");java.util.List<Object> frames=new ArrayList<>();
  for(Object sys:(java.util.List<?>)call(sheet,"getSystems")){
   frames.add(frame(sys,"before_buildSymbols_after_original_doProlog"));Object factory=new ObservedFactory((SystemInfo)sys);call(make("sheet.symbol.SymbolsBuilder",sys,factory),"buildSymbols",optionals);frames.add(frame(sys,"after_buildSymbols_before_rest_chords"));call(make("sheet.note.ChordsBuilder",sys),"buildRestChords");frames.add(frame(sys,"after_buildRestChords_before_lateChecks"));call(factory,"lateChecks");frames.add(frame(sys,"after_lateChecks"));
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),json(map("scope","Fresh unchanged image -> ordinary CURVES (including CHORDS); original SymbolsStep.doProlog; sequential systems call original SymbolsBuilder.buildSymbols, ChordsBuilder.buildRestChords and InterFactory.lateChecks in source order. InterFactory subclass delegates every call to original super.create and only observes actual Rest results. No original epilog/LINKS/RHYTHMS in these frames; no native outputs.","instrumentation","Reflection getters plus raw pitch/clock fields. IdentityHashMap labels denote actual Java references; compile-only signatures absent at runtime. Per-system serial schedule replaces AbstractSystemStep scheduler, without changing within-system source order.","native_outputs_used",false,"factory_candidates",candidates,"frames",frames))+"\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
