// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Rectangle;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class MixedRestStartingProbe {
 static final IdentityHashMap<Object,Integer> tokens=new IdentityHashMap<>();
 static int token(Object o){if(o==null)return 0;return tokens.computeIfAbsent(o,k->tokens.size()+1);}
 static Class<?> c(String n)throws Exception{return Class.forName("org.audiveris.omr."+n);}
 static Object call(Object o,String name,Object...args)throws Exception{
  for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())for(Method m:k.getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){
   try{m.setAccessible(true);return m.invoke(o,args);}catch(IllegalArgumentException ignored){}
  }throw new NoSuchMethodException(o.getClass()+"."+name);
 }
 static Object make(String n,Object...args)throws Exception{for(Constructor<?> ctor:c(n).getDeclaredConstructors())if(ctor.getParameterCount()==args.length)try{ctor.setAccessible(true);return ctor.newInstance(args);}catch(IllegalArgumentException ignored){}throw new NoSuchMethodException(n);}
 static Object field(Object o,String n)throws Exception{for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try{Field f=k.getDeclaredField(n);f.setAccessible(true);return f.get(o);}catch(NoSuchFieldException ignored){}throw new NoSuchFieldException(n);}
 static Map<String,Object> map(Object...a){Map<String,Object> o=new LinkedHashMap<>();for(int i=0;i<a.length;i+=2)o.put((String)a[i],a[i+1]);return o;}
 static Object text(Object o){return o==null?null:o.toString();}
 static Object rect(Object o){if(o==null)return null;Rectangle b=(Rectangle)o;return new int[]{b.x,b.y,b.width,b.height};}
 static List<Integer> ids(Collection<?> values)throws Exception{List<Integer> out=new ArrayList<>();for(Object v:values)out.add(((Number)call(v,"getId")).intValue());return out;}
 static List<Object> edges(Object sig)throws Exception{List<Object> out=new ArrayList<>();for(Object e:(Collection<?>)call(sig,"edgeSet"))out.add(map("object",token(e),"kind",e.getClass().getSimpleName(),"source",call(call(sig,"getEdgeSource",e),"getId"),"target",call(call(sig,"getEdgeTarget",e),"getId"),"manual",call(e,"isManual")));return out;}
 static Object chord(Object ch,Object measure)throws Exception{
  Object voice=call(ch,"getVoice"),staff=call(ch,"getStaff"),slot=call(ch,"getSlot");List<Object> notes=new ArrayList<>();
  for(Object n:(Collection<?>)call(ch,"getMembers"))notes.add(map("id",call(n,"getId"),"object",token(n),"kind",n.getClass().getSimpleName(),"shape",text(call(n,"getShape")),"bounds",rect(call(n,"getBounds")),"manual",call(n,"isManual"),"staff",call(call(n,"getStaff"),"getId")));
  return map("id",call(ch,"getId"),"object",token(ch),"kind",ch.getClass().getSimpleName(),"bounds",rect(call(ch,"getBounds")),"staff",call(staff,"getId"),"staff_object",token(staff),"actual_measure_same",call(ch,"getMeasure")==measure,"actual_part_same",call(ch,"getPart")==call(measure,"getPart"),"manual",call(ch,"isManual"),"voice",voice==null?null:call(voice,"getId"),"voice_object",voice==null?null:token(voice),"slot",slot==null?null:call(slot,"getId"),"slot_object",slot==null?null:token(slot),"time_offset",text(call(ch,"getTimeOffset")),"duration",text(call(ch,"getDuration")),"notes",notes);
 }
 static Object state(Object measure,List<Object> chords)throws Exception{
  List<Object> cs=new ArrayList<>(),vs=new ArrayList<>();for(Object ch:chords)cs.add(chord(ch,measure));
  for(Object v:(Collection<?>)call(measure,"getVoices")){
   Object rest=call(v,"getWholeChord");vs.add(map("id",call(v,"getId"),"object",token(v),"kind",text(call(v,"getKind")),"starting_staff",call(call(v,"getStartingStaff"),"getId"),"is_measure_rest",call(v,"isMeasureRest"),"measure_rest_chord",rest==null?null:call(rest,"getId"),"chord_ids",ids((Collection<?>)call(v,"getChords")),"chord_objects",((Collection<?>)call(v,"getChords")).stream().map(MixedRestStartingProbe::token).toList(),"termination",text(call(v,"getTermination")),"excess",text(field(v,"excess"))));
  }
  return map("chords",cs,"voices",vs,"expected",text(call(call(measure,"getStack"),"getExpectedDuration")),"abnormal",call(measure,"isAbnormal"));
 }
 public static void main(String[] args)throws Exception{
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object stub=((List<?>)call(book,"getStubs")).get(0),sheet=call(stub,"getSheet");
  Object switches=call(stub,"getProcessingSwitches");boolean implicit=(Boolean)call(switches,"getValue",c("sheet.ProcessingSwitch").getField("implicitTuplets").get(null));
  List<Object> rows=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   int index=0;for(Object stack:(List<?>)call(sys,"getStacks")){index++;
    for(Object measure:(List<?>)call(stack,"getMeasures")){
     Object rhythm=make("sheet.rhythm.MeasureRhythm",measure);
     Collection<?> implicits=(Collection<?>)call(rhythm,"getImplicitTuplets");
     call(rhythm,"removeTuplets",implicits);call(rhythm,"clearInterleavedRests");call(rhythm,"detectInterleavedRests");call(measure,"resetRhythm");
     Object retriever=make("sheet.rhythm.SlotsRetriever",measure,false);
     List<?> narrow=(List<?>)call(retriever,"buildSlots"),wide=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,true),"buildSlots");
     List<?> built=(List<?>)call(rhythm,"buildCompoundSlots",narrow,wide);((List)field(rhythm,"slots")).addAll(built);
     List<Object> selected=new ArrayList<>((Collection<?>)call(measure,"getMeasureRestChords"));if(!built.isEmpty())selected.addAll((Collection<?>)call(built.get(0),"getChords"));
     selected.sort(Comparator.comparingInt(ch->{try{return ((Rectangle)call(ch,"getBounds")).y+((Rectangle)call(ch,"getBounds")).height/2;}catch(Exception e){throw new RuntimeException(e);}}));
     boolean hasRest=selected.stream().anyMatch(ch->ch.getClass().getSimpleName().equals("RestChordInter"));
     if(selected.size()<2 || !hasRest)continue;
     List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));Object sig=call(sys,"getSig");
     Object before=state(measure,chords);List<Object> beforeEdges=edges(sig);List<Integer> beforeVertices=ids((Collection<?>)call(sig,"vertexSet"));
     call(rhythm,"processStartingChords");Object after=state(measure,chords);List<Object> afterEdges=edges(sig);
     rows.add(map("system",call(sys,"getId"),"part",call(call(measure,"getPart"),"getId"),"physical_measure_index",index,"printed_stack_id",call(stack,"getIdValue"),"measure_object",token(measure),"measure_rest_chord_ids",ids((Collection<?>)call(measure,"getMeasureRestChords")),"starting_chord_ids",ids(selected),"before",before,"after",after,"vertex_identity_unchanged",beforeVertices.equals(ids((Collection<?>)call(sig,"vertexSet"))),"relation_identity_manual_endpoints_unchanged",beforeEdges.equals(afterEdges),"relations",afterEdges));
    }
   }
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),map("scope","Saved original actual owners: source removeImplicit/clear/detect/reset and geometric slots, then actual processStartingChords; no fresh image, native inputs, or full rhythm/export claim","implicit_tuplets",implicit,"mixed_starts",rows));System.exit(0);
 }
}
