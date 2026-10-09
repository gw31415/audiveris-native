import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;

/** Declared in-memory HEADER-membership reset replay. No image/step/book save. */
public class MeasureFillerFirstAddReplayProbe extends MeasureFillerClefKeyBookProbe {
 static void set(Object o,String n,Object value)throws Exception{
  for(Class<?> c=o.getClass();c!=null;c=c.getSuperclass())try{Field f=c.getDeclaredField(n);f.setAccessible(true);f.set(o,value);return;}catch(NoSuchFieldException e){}
  throw new NoSuchFieldException(n);
 }
 static void require(boolean b,String s){if(!b)throw new AssertionError(s);}
 static Object scalar(Object x){return x==null?null:x.toString();}
 static Object rawRef(Object x){return map("object",label(x),"value",scalar(x));}
 static List<Object> objectRefs(Collection<?> xs){List<Object> out=new ArrayList<>();if(xs!=null)for(Object x:xs)out.add(label(x));return out;}
 static Object clocks(Object sys)throws Exception{
  Object sig=call(sys,"getSig");List<Object> stacks=new ArrayList<>(),ms=new ArrayList<>(),cs=new ArrayList<>();
  for(Object stack:(List<?>)call(sys,"getStacks")){
   Object slots=field(stack,"slots");
   stacks.add(map("stack",label(stack),"expected",rawRef(field(stack,"expectedDuration")),"actual",rawRef(field(stack,"actualDuration")),"excess",rawRef(field(stack,"excess")),"slots_collection",label(slots),"slots",objectRefs((Collection<?>)slots)));
  }
  for(Object p:(List<?>)call(sys,"getParts"))for(Object m:(List<?>)call(p,"getMeasures")){
   List<Object> voices=new ArrayList<>();Object voiceList=field(m,"voices"),stack=call(m,"getStack");
   for(Object voice:(Collection<?>)voiceList){Object voiceSlots=field(voice,"slots"),chords=field(voice,"chords");
    voices.add(map("object",label(voice),"id",field(voice,"id"),"measure",label(field(voice,"measure")),"starting_staff",label(field(voice,"startingStaff")),"slots_object",label(voiceSlots),"slots",scalar(voiceSlots),"chords_object",label(chords),"chords",objectRefs((Collection<?>)chords),"termination",rawRef(field(voice,"termination")),"excess",rawRef(field(voice,"excess"))));
   }
   ms.add(map("measure",measureRef(sys,m),"voice_collection",label(voiceList),"voices",voices,"head_chords_collection",label(field(m,"headChords")),"head_chords",objectRefs((Collection<?>)field(m,"headChords")),"rest_chords_collection",label(field(m,"restChords")),"rest_chords",objectRefs((Collection<?>)field(m,"restChords")),"shared_stack_expected",rawRef(call(stack,"getExpectedDuration")),"shared_stack_excess",rawRef(call(stack,"getExcess"))));
  }
  for(Object x:(Set<?>)call(sig,"vertexSet"))if(type("sig.inter.AbstractChordInter").isInstance(x)){
   Object beams=field(x,"beams");
   cs.add(map("id",id(x),"object",label(x),"kind",x.getClass().getSimpleName(),"actual_measure",label(field(x,"measure")),"voice",label(field(x,"voice")),"slot",label(field(x,"slot")),"time",rawRef(field(x,"timeOffset")),"beams_collection",label(beams),"beams",objectRefs((Collection<?>)beams),"bounds_object",label(field(x,"bounds")),"bounds",rect(field(x,"bounds")),"head_object",label(field(x,"headLocation")),"head",point(field(x,"headLocation")),"tail_object",label(field(x,"tailLocation")),"tail",point(field(x,"tailLocation")),"dots",field(x,"dotsNumber")));
  }
  return map("stacks",stacks,"measure_peers",ms,"chords",cs);
 }
 static Object rawHeaders(Object sys)throws Exception{
  Object sig=call(sys,"getSig");List<Object> values=new ArrayList<>();
  for(Object x:(Set<?>)call(sig,"vertexSet"))if(x.getClass().getSimpleName().equals("ClefInter")||x.getClass().getSimpleName().equals("KeyInter")){
   values.add(map("inter",inter(sig,x),"bounds_object",label(field(x,"bounds")),"grade",field(x,"grade"),"contextual_grade",field(x,"ctxGrade"),"manual",call(x,"isManual"),"frozen",call(x,"isFrozen")));
  }return values;
 }
 static class HeldMeasure {
  Object measure;Collection<Object> clefs,keys;List<Object> oldClefs,oldKeys;
  @SuppressWarnings("unchecked") HeldMeasure(Object m)throws Exception{measure=m;clefs=(Collection<Object>)field(m,"clefs");keys=(Collection<Object>)field(m,"keys");oldClefs=new ArrayList<>(clefs);oldKeys=new ArrayList<>(keys);}
  void verify()throws Exception{require(field(measure,"clefs")==clefs&&field(measure,"keys")==keys,"actual collection identity changed");}
  void restore()throws Exception{verify();clefs.clear();clefs.addAll(oldClefs);keys.clear();keys.addAll(oldKeys);}
 }
 static class HeldSystem {
  Object system;List<HeldMeasure> measures=new ArrayList<>();Map<Object,Object> parts=new IdentityHashMap<>();Set<Object> headers=Collections.newSetFromMap(new IdentityHashMap<>());
  Object chord,stack,oldTime,oldExpected;String graphBefore,clocksBefore,headersBefore,membershipBefore;
  HeldSystem(Object sys)throws Exception{
   system=sys;Object sig=call(sys,"getSig");
   for(Object p:(List<?>)call(sys,"getParts"))for(Object m:(List<?>)call(p,"getMeasures"))measures.add(new HeldMeasure(m));
   for(Object staff:(List<?>)call(sys,"getStaves")){Object h=call(staff,"getHeader");if(h!=null)for(String n:List.of("clef","key")){Object x=field(h,n);if(x!=null)headers.add(x);}}
   for(Object x:(Set<?>)call(sig,"vertexSet"))if(x.getClass().getSimpleName().equals("ClefInter")||x.getClass().getSimpleName().equals("KeyInter"))parts.put(x,field(x,"part"));
   for(Object x:(Set<?>)call(sig,"vertexSet"))if(type("sig.inter.HeadChordInter").isInstance(x)){chord=x;break;}
   require(chord!=null,"actual existing HeadChord required");stack=call(field(chord,"measure"),"getStack");oldTime=field(chord,"timeOffset");oldExpected=field(stack,"expectedDuration");
   graphBefore=json(graph(sig));clocksBefore=json(clocks(sys));headersBefore=json(rawHeaders(sys));membershipBefore=json(memberships(sys));
  }
  void verifyCollections()throws Exception{for(HeldMeasure m:measures)m.verify();}
  void restore()throws Exception{
   for(HeldMeasure m:measures)m.restore();for(var e:parts.entrySet())set(e.getKey(),"part",e.getValue());set(chord,"timeOffset",oldTime);set(stack,"expectedDuration",oldExpected);
   verifyCollections();require(json(graph(call(system,"getSig"))).equals(graphBefore),"SIG identity/manual changed");require(json(clocks(system)).equals(clocksBefore),"original clocks/cache owner restoration failed");require(json(rawHeaders(system)).equals(headersBefore),"raw Header cache restoration failed");require(json(memberships(system)).equals(membershipBefore),"original membership restoration failed");
  }
 }
 static Object step(Object sys,String name,Object x,Object stack,Object measure)throws Exception{
  return map("stage",name,"system",id(sys),"system_object",label(sys),"inter",x==null?null:inter(call(sys,"getSig"),x),"stack_object",label(stack),"measure",measureRef(sys,measure),"all_peer_memberships",memberships(sys));
 }
 public static void main(String[] args)throws Exception{
  List<HeldSystem> held=new ArrayList<>();List<Object> replay=new ArrayList<>();List<Object> failures=new ArrayList<>();boolean restored=false;
  try{
   Object book=type("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object stub=((List<?>)call(book,"getStubs")).get(0),sheet=call(stub,"getSheet");
   for(Object sys:(List<?>)call(sheet,"getSystems"))held.add(new HeldSystem(sys));
   for(HeldSystem h:held){
    Object sys=h.system,sig=call(sys,"getSig");List<Object> trace=new ArrayList<>();Object initial=frame(sys,"after_reload_before_declared_control");
    set(h.chord,"timeOffset",make("math.Rational",7,64));set(h.stack,"expectedDuration",make("math.Rational",1,2));
    Object declaredClocks=clocks(sys);String stableClocks=json(declaredClocks);
    // Keep the actual collection objects and the actual nonheader SYMBOLS Clef.
    for(HeldMeasure m:h.measures){m.clefs.removeIf(h.headers::contains);m.keys.removeIf(h.headers::contains);}
    h.verifyCollections();trace.add(step(sys,"after_declared_Header_only_membership_clear",null,null,null));
    Object classes=field(type("sheet.rhythm.MeasureFiller"),"FILLING_CLASSES");Collection<?> filling=(Collection<?>)call(sig,"inters",classes);
    List<Object> fillingRows=new ArrayList<>();for(Object x:filling)fillingRows.add(inter(sig,x));
    int calls=0;
    for(Object stack:(List<?>)call(sys,"getStacks")){
     Collection<?> kept=(Collection<?>)call(stack,"filter",filling);List<Object> keptRows=new ArrayList<>();for(Object x:kept)keptRows.add(inter(sig,x));
     trace.add(map("stage","original_Stack_filter_completed_before_additions","system",id(sys),"stack_index",((List<?>)call(sys,"getStacks")).indexOf(stack)+1,"stack_object",label(stack),"kept",keptRows,"all_peer_memberships",memberships(sys)));
     for(Object x:kept){
      trace.add(step(sys,"before_original_Inter_getPart",x,stack,null));Object part=call(x,"getPart");Object measure=part==null?null:call(stack,"getMeasureAt",part);
      trace.add(step(sys,"before_original_Measure_addInter",x,stack,measure));
      if(part!=null){call(measure,"addInter",x);calls++;}
      h.verifyCollections();require(json(clocks(sys)).equals(stableClocks),"held real Chord7/64/Stack expected/owners changed during add");require(json(graph(sig)).equals(h.graphBefore),"SIG identity/manual changed during add");
      trace.add(step(sys,"after_original_Measure_addInter",x,stack,measure));
     }
    }
    Object directResult=memberships(sys);String directSignature=json(directResult);
    call(make("sheet.rhythm.MeasureFiller",sys),"process");h.verifyCollections();require(json(memberships(sys)).equals(directSignature),"ordinary MeasureFiller differs from declared ordered source calls");require(json(clocks(sys)).equals(stableClocks),"ordinary repeat changed clocks");require(json(graph(sig)).equals(h.graphBefore),"ordinary repeat changed SIG");
    replay.add(map("system",id(sys),"initial",initial,"declared_clock_control",map("actual_chord",id(h.chord),"chord_object",label(h.chord),"actual_stack_object",label(h.stack),"time_offset","7/64","expected_duration","1/2","actual_voices_or_slots_created",false),"clocks_before_replay",declaredClocks,"filling_collection",fillingRows,"trace",trace,"original_addInter_calls",calls,"after_direct_calls",directResult,"after_ordinary_MeasureFiller_process",memberships(sys),"all_collection_objects_retained",true,"all_intermediate_SIG_manual_identity_and_clocks_unchanged",true,"ordinary_process_idempotent_after_direct_source_calls",true));
   }
  }catch(Throwable ex){failures.add(ex.toString());throw ex;}
  finally{
   try{for(HeldSystem h:held)h.restore();restored=true;}catch(Throwable e){failures.add("RESTORE: "+e);}
   Files.writeString(Path.of(args[1]),json(map("scope","Declared actual-owner replay after loading original saved SYMBOLS Book. Header-only collection contents are removed; actual collection objects and nonheader member remain. Same source SIG.inters and Stack.filter complete before sequential original getPart/getMeasureAt/addInter calls; original MeasureFiller.process then confirms idempotence. Not fresh recognition, not fresh first-fill callback instrumentation. One actual existing Chord clock7/64 and shared Stack expected1/2 are declared controls, restored with old memberships/raw Part caches and original clocks.","source_pin","9e1e55cd2746037d059345881c53e6a6754bffbd","book",args[0],"systems",replay,"original_state_restored",restored,"errors",failures))+"\n");
   if(!restored)throw new AssertionError("restore failed");
  }
 }
}
