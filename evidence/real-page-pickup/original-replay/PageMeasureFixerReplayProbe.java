// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import java.security.MessageDigest;
import com.fasterxml.jackson.databind.ObjectMapper;

public class PageMeasureFixerReplayProbe extends MixedRestStartingProbe {
 static final ObjectMapper JSON = new ObjectMapper();
 static void put(Object o,String name,Object value)throws Exception {
  for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try {Field f=k.getDeclaredField(name);f.setAccessible(true);f.set(o,value);return;}catch(NoSuchFieldException ignored){}
  throw new NoSuchFieldException(name);
 }
 static String hash(Object value)throws Exception {return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(JSON.writeValueAsBytes(value)));}
 static Object identity(Object stack)throws Exception {
  List<Object> ms=new ArrayList<>();
  for(Object m:(Collection<?>)call(stack,"getMeasures")) {
   List<Object> cs=new ArrayList<>(),vs=new ArrayList<>();
   for(Object ch:(Collection<?>)call(m,"getStandardChords")) {
    List<Integer> members=new ArrayList<>();for(Object n:(Collection<?>)call(ch,"getMembers"))members.add(token(n));
    cs.add(map("id",call(ch,"getId"),"object",token(ch),"measure",token(call(ch,"getMeasure")),"part",token(call(ch,"getPart")),"staff",token(call(ch,"getStaff")),"members",members,"slot",token(call(ch,"getSlot")),"voice",token(call(ch,"getVoice")),"manual",call(ch,"isManual")));
   }
   for(Object v:(Collection<?>)call(m,"getVoices"))vs.add(map("object",token(v),"id",call(v,"getId"),"measure",token(call(v,"getMeasure")),"chord_objects",((Collection<?>)call(v,"getChords")).stream().map(PageMeasureFixerReplayProbe::token).toList()));
   ms.add(map("measure",token(m),"part",token(call(m,"getPart")),"stack",token(call(m,"getStack")),"chords",cs,"voices",vs));
  }
  Object sig=call(call(stack,"getSystem"),"getSig");List<Object> vertices=new ArrayList<>();
  for(Object v:(Collection<?>)call(sig,"vertexSet"))vertices.add(map("id",call(v,"getId"),"object",token(v),"manual",call(v,"isManual")));
  return map("stack",token(stack),"measures",ms,"slots",((Collection<?>)call(stack,"getSlots")).stream().map(PageMeasureFixerReplayProbe::token).toList(),"vertices",vertices,"relations",edges(sig));
 }
 static Object state(Object measure,List<Object> chords)throws Exception {
  List<Object> cs=new ArrayList<>(),vs=new ArrayList<>();for(Object ch:chords)cs.add(chord(ch,measure));
  for(Object v:(Collection<?>)call(measure,"getVoices")) {
   Object rest=call(v,"getWholeChord"),staff=call(v,"getStartingStaff");
   vs.add(map("id",call(v,"getId"),"object",token(v),"kind",text(call(v,"getKind")),"starting_staff",staff==null?null:call(staff,"getId"),"is_measure_rest",call(v,"isMeasureRest"),"measure_rest_chord",rest==null?null:call(rest,"getId"),"chord_ids",ids((Collection<?>)call(v,"getChords")),"chord_objects",((Collection<?>)call(v,"getChords")).stream().map(PageMeasureFixerReplayProbe::token).toList(),"termination",text(call(v,"getTermination")),"excess",text(field(v,"excess"))));
  }
  return map("chords",cs,"voices",vs,"expected",text(call(call(measure,"getStack"),"getExpectedDuration")),"abnormal",call(measure,"isAbnormal"));
 }
 static Object stateStack(Object stack)throws Exception {
  List<Object> peers=new ArrayList<>(),slots=new ArrayList<>();
  for(Object m:(Collection<?>)call(stack,"getMeasures")) {
   Object peer=call(m,"getStack");List<Object> chords=new ArrayList<>((Collection<?>)call(m,"getStandardChords"));
   Map<String,Object> row=new LinkedHashMap<>((Map<String,Object>)state(m,chords));
   row.put("part",call(call(m,"getPart"),"getId"));row.put("measure_object",token(m));row.put("actual_stack_same",peer==stack);row.put("stack_object",token(peer));
   row.put("shared_expected",text(call(peer,"getExpectedDuration")));row.put("shared_actual",text(call(peer,"getActualDuration")));row.put("shared_excess",text(call(peer,"getExcess")));
   peers.add(row);
  }
  for(Object s:(Collection<?>)call(stack,"getSlots"))slots.add(map("object",token(s),"id",call(s,"getId"),"time_offset",text(call(s,"getTimeOffset"))));
  return map("system",call(call(stack,"getSystem"),"getId"),"stack_object",token(stack),"id",call(stack,"getIdValue"),"special",text(field(stack,"special")),"implicit",call(stack,"isImplicit"),"expected",text(call(stack,"getExpectedDuration")),"actual",text(call(stack,"getActualDuration")),"excess",text(call(stack,"getExcess")),"measures",peers,"slots",slots);
 }
 static Object frame(String name,Object stack,Object baseline,Object voice)throws Exception {
  Object currentIdentity=identity(stack);
  if(!baseline.equals(currentIdentity))throw new IllegalStateException("owner/relation identity changed at "+name);
  return map("stage",name,"checked_voice_object",voice==null?null:token(voice),"checked_voice_id",voice==null?null:call(voice,"getId"),"identity_reference_sha256",hash(currentIdentity),"actual_owners_slots_relations_and_manual_status_retained",true,"state",stateStack(stack));
 }
 static Object replay(Object stack,boolean pickup)throws Exception {
  Object baseline=identity(stack);List<Object> frames=new ArrayList<>();frames.add(frame("LOADED_POST_PAGE_BEFORE_REPLAY",stack,baseline,null));
  call(stack,"resetSpecial");frames.add(frame("AFTER_SOURCE_RESET_SPECIAL",stack,baseline,null));
  for(Object m:(Collection<?>)call(stack,"getMeasures"))for(Object v:(Collection<?>)call(m,"getVoices")) {
   frames.add(frame("BEFORE_VOICE_CHECK_DURATION",stack,baseline,v));call(v,"checkDuration");frames.add(frame("AFTER_VOICE_CHECK_DURATION",stack,baseline,v));
  }
  Object fixer=make("score.MeasureFixer");put(fixer,"stack",stack);Object termination=call(fixer,"getStackTermination");put(fixer,"stackTermination",termination);
  boolean detected=(Boolean)call(fixer,"isPickup",stack);
  if(pickup) {
   if(!detected)throw new IllegalStateException("first actual Stack was not pickup");
   call(stack,"setPickup");call(fixer,"setId",0);
  }
  frames.add(frame(pickup?"AFTER_SELECTED_ORIGINAL_PICKUP_BRANCH":"AFTER_DURATION_REPLAY",stack,baseline,null));
  return map("scope","Saved post-PAGE owner replay; original resetSpecial and Voice.checkDuration methods, then selected MeasureFixer getters/pickup branch only", "common_termination",text(termination),"isPickup",detected,"selected_pickup_branch_executed",pickup,"frames",frames);
 }
 static Object controls(Object stack)throws Exception {
  Object positive=null;
  for(Object v:(Collection<?>)call(stack,"getVoices"))if(field(v,"excess")!=null){positive=v;break;}
  if(positive==null)throw new IllegalStateException("no actual positive-excess Voice");
  Object voice=positive,measure=call(voice,"getMeasure"),last=call(voice,"getLastChord");
  Object oldOffset=call(last,"getTimeOffset"),oldTerm=field(voice,"termination"),oldVoiceExcess=field(voice,"excess"),oldStackExcess=call(stack,"getExcess"),oldAbnormal=call(measure,"isAbnormal");
  Object baseline=identity(stack),before=stateStack(stack);List<Object> cases=new ArrayList<>();
  for(String name:List.of("null-last-clock-keeps-old-termination-and-excess","underflow-keeps-old-positive-excess","positive-check-updates-one-shared-excess")) {
   put(voice,"termination",oldTerm);put(voice,"excess",oldVoiceExcess);call(stack,"setExcess",oldStackExcess);call(measure,"setAbnormal",oldAbnormal);call(last,"setTimeOffset",oldOffset);
   if(name.startsWith("null"))call(last,"setTimeOffset",(Object)null);
   else if(name.startsWith("underflow"))call(last,"setTimeOffset",make("math.Rational",0,1));
   else call(stack,"setExcess",make("math.Rational",1,64));
   Object b=frame("DECLARED_CONTROL_BEFORE_CHECK",stack,baseline,voice);call(voice,"checkDuration");Object a=frame("DECLARED_CONTROL_AFTER_CHECK",stack,baseline,voice);
   cases.add(map("name",name,"declared_in_memory_control",true,"voice_object",token(voice),"voice_id",call(voice,"getId"),"last_chord_id",call(last,"getId"),"last_chord_object",token(last),"before",b,"after",a));
  }
  put(voice,"termination",oldTerm);put(voice,"excess",oldVoiceExcess);call(stack,"setExcess",oldStackExcess);call(measure,"setAbnormal",oldAbnormal);call(last,"setTimeOffset",oldOffset);
  if(!hash(before).equals(hash(stateStack(stack))))throw new IllegalStateException("declared fields not restored exactly");
  if(!baseline.equals(identity(stack)))throw new IllegalStateException("declared owner refs not preserved");
  return map("scope","Declared in-memory branches on held actual positive-excess Voice; not natural image states", "cases",cases,"held_values_and_refs_restored",true);
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  Object sys=((List<?>)call(sheet,"getSystems")).get(0);List<?> stacks=(List<?>)call(sys,"getStacks");
  List<Object> rows=List.of(replay(stacks.get(0),true),replay(stacks.get(2),false));
  Object controls=controls(stacks.get(2));
  JSON.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),map("source_commit","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","Saved original post-PAGE Book replay; selected two Stack duration checks and first pickup branch, no whole-page process or recognition", "fresh_image_recognition",false,"whole_PAGE_executed",false,"whole_MeasureFixer_process_executed",false,"native_outputs_used",false,"replays",rows,"declared_controls",controls));System.exit(0);
 }
}
