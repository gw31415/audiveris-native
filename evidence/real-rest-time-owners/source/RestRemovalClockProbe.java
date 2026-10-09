// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared in-memory method controls on reloaded original owners; never recognition input.
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
public class RestRemovalClockProbe extends PedalRawStaffBookProbe {
 static Object sig,rest,chord,staff,part,measure,stack,containment,voice,slot,slotInfo;
 static boolean active=false,inside=false;
 static final List<Object> observations=new ArrayList<>();
 static Object value(Object o,String f)throws Exception{return o==null?null:field(o,f);}
 static String rational(Object o){return o==null?null:o.toString();}
 static Object ref(Object o)throws Exception{return o==null?null:map("object",label(o),"class",o.getClass().getSimpleName());}
 static List<Integer> refs(Object o){List<Integer> r=new ArrayList<>();if(o instanceof Collection<?>xs)for(Object x:xs)r.add(label(x));return r;}
 static boolean has(Object xs,Object o){return xs instanceof Collection<?> c&&c.stream().anyMatch(x->x==o);}
 static boolean registered(Object o)throws Exception{return ((Collection<?>)call(sig,"vertexSet")).stream().anyMatch(x->x==o);}
 static Object inter(Object o)throws Exception{return map("id",id(o),"object",label(o),"kind",o.getClass().getSimpleName(),"shape",value(o,"shape"),"bounds",rect(value(o,"bounds")),"bounds_object",label(value(o,"bounds")),"raw_staff_object",label(value(o,"staff")),"raw_part_object",label(value(o,"part")),"manual",value(o,"manual"),"removed",value(o,"removed"),"registered",registered(o));}
 static Object snapshot()throws Exception{
  Map<String,Object> c=new LinkedHashMap<>((Map<String,Object>)inter(chord));
  c.putAll(map("measure_object",label(value(chord,"measure")),"voice_object",label(value(chord,"voice")),"slot_object",label(value(chord,"slot")),"time_offset",rational(value(chord,"timeOffset")),"time_offset_object",label(value(chord,"timeOffset"))));
  List<Object> members=new ArrayList<>();if(registered(chord))for(Object edge:(Collection<?>)call(sig,"outgoingEdgesOf",chord))if(is(edge,"Containment"))members.add(label(call(sig,"getEdgeTarget",edge)));
  c.put("registered_member_objects",members);
  List<Object> voiceSlots=new ArrayList<>();if(voice!=null&&value(voice,"slots") instanceof Map<?,?>m)for(var e:m.entrySet())voiceSlots.add(map("slot_id",e.getKey(),"slot_voice_object",label(e.getValue()),"chord_object",label(value(e.getValue(),"chord")),"status",value(e.getValue(),"status")));
  return map("rest",inter(rest),"chord",c,
   "staff",map("object",label(staff),"id",id(staff),"notes_object",label(value(staff,"notes")),"contains_rest",has(value(staff,"notes"),rest),"part_object",label(value(staff,"part"))),
   "part",map("object",label(part),"id",id(part),"contains_measure",has(value(part,"measures"),measure)),
   "measure",map("object",label(measure),"part_object",label(value(measure,"part")),"stack_object",label(value(measure,"stack")),"contains_chord",has(value(measure,"restChords"),chord),"voice_objects",refs(value(measure,"voices"))),
   "stack",map("object",label(stack),"contains_measure",has(value(stack,"measures"),measure),"slot_objects",refs(value(stack,"slots")),"expected_duration",rational(value(stack,"expectedDuration")),"expected_duration_object",label(value(stack,"expectedDuration")),"excess",rational(value(stack,"excess"))),
   "voice",voice==null?null:map("object",label(voice),"id",id(voice),"measure_object",label(value(voice,"measure")),"chord_objects",refs(value(voice,"chords")),"slot_table_object",label(value(voice,"slots")),"slot_entries",voiceSlots,"termination",rational(value(voice,"termination")),"termination_object",label(value(voice,"termination")),"excess",rational(value(voice,"excess")),"excess_object",label(value(voice,"excess"))),
   "slot",slot==null?null:map("object",label(slot),"id",id(slot),"stack_object",label(value(slot,"stack")),"incoming_objects",refs(value(slot,"incomings")),"time_offset",rational(value(slot,"timeOffset")),"time_offset_object",label(value(slot,"timeOffset"))),
   "containment",map("object",label(containment),"kind",containment.getClass().getSimpleName(),"manual",call(containment,"isManual"),"registered",call(sig,"containsEdge",containment)));
 }
 public static void capture(String hook,Object receiver,Object arg){
  if(!active||inside)return;boolean relevant=receiver==rest||receiver==chord||receiver==voice||receiver==staff||receiver==measure||receiver==containment;
  if(!relevant)return;inside=true;try{observations.add(map("hook",hook,"receiver",ref(receiver),"argument",ref(arg),"state",snapshot()));}catch(Throwable ex){errors.add(map("hook",hook,"error",ex.toString()));}finally{inside=false;}
 }
 static void graphListen()throws Exception{Class<?> api=Class.forName("org.jgrapht.event.GraphListener");Object proxy=Proxy.newProxyInstance(api.getClassLoader(),new Class<?>[]{api},(o,m,args)->{
  if(m.getDeclaringClass()==Object.class)return switch(m.getName()){case "equals"->o==args[0];case "hashCode"->System.identityHashCode(o);default->"rest-clock-observer";};
  if(args==null||args.length!=1||!active)return null;Object e=args[0];if(m.getName().equals("edgeRemoved")){Object edge=call(e,"getEdge");if(edge==containment)observations.add(map("hook","GraphListener.edgeRemoved","relation_object",label(edge),"source_object",label(call(e,"getEdgeSource")),"target_object",label(call(e,"getEdgeTarget")),"state",snapshot()));}
  else if(m.getName().equals("vertexRemoved")){Object n=call(e,"getVertex");if(n==rest||n==chord)observations.add(map("hook","GraphListener.vertexRemoved","vertex_object",label(n),"state",snapshot()));}return null;});call(sig,"addGraphListener",proxy);}
 static void require(boolean condition,String message){if(!condition)throw new AssertionError(message);}
 public static void main(String[] args)throws Exception{
  Object book=type("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object stub=((List<?>)call(book,"getStubs")).get(0),sheet=call(stub,"getSheet"),system=null;
  for(Object s:(List<?>)call(sheet,"getSystems"))if(((Number)id(s)).intValue()==2)system=s;
  require(system!=null,"source System2 missing");sig=call(system,"getSig");
  call(make("sheet.rhythm.MeasureFiller",system),"process");Object linker=make("sheet.symbol.SymbolsLinker",system);
  for(String method:List.of("linkDynamics","linkTexts","linkPedals","linkWedges","linkFermatas","linkGraces","linkAugmentationDots","linkTuplets","linkOctaveShifts","linkNumbers"))call(linker,method);
  for(Object n:(Collection<?>)call(sig,"vertexSet"))if(is(n,"RestInter")&&((Number)id(n)).intValue()==4436)rest=n;
  require(rest!=null,"observed Rest4436 missing");require(String.valueOf(value(rest,"shape")).equals("QUARTER_REST"),"actual source rest shape");require(rect(value(rest,"bounds")).equals(List.of(1122,879,10,32)),"actual source rest bounds");
  List<Object> containments=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"incomingEdgesOf",rest))if(is(r,"Containment"))containments.add(r);
  require(containments.size()==1,"actual singleton containment required");containment=containments.get(0);chord=call(sig,"getEdgeSource",containment);
  require(is(chord,"RestChordInter")&&((List<?>)call(chord,"getMembers")).equals(List.of(rest)),"actual singleton RestChord required");
  staff=value(rest,"staff");measure=value(chord,"measure");require(staff!=null&&measure!=null,"actual original owner required");part=value(measure,"part");stack=value(measure,"stack");
  require(has(value(staff,"notes"),rest)&&has(value(measure,"restChords"),chord),"actual original membership required");
  require(value(chord,"voice")==null&&value(chord,"slot")==null&&value(chord,"timeOffset")==null,"source initial clocks must be null");
  require(((Collection<?>)value(measure,"voices")).isEmpty(),"no unrelated source Voice may be reset");require(((List<?>)value(stack,"slots")).isEmpty(),"no source Slot may be reset");
  Object loaded=snapshot();
  call(rest,"setManual",true);call(chord,"setManual",true);call(containment,"setManual",true);
  Object offset=make("math.Rational",7,64),expected=make("math.Rational",Integer.parseInt(args[2]),4);
  voice=make("sheet.rhythm.Voice",chord,measure);call(measure,"addVoice",voice);
  slot=make("sheet.rhythm.Slot",1,stack,List.of(chord));((List<Object>)value(stack,"slots")).add(slot);call(slot,"setTimeOffset",offset);call(chord,"setTimeOffset",offset);
  Object begin=Enum.valueOf((Class)type("sheet.rhythm.SlotVoice$ChordStatus"),"BEGIN");slotInfo=make("sheet.rhythm.SlotVoice",chord,begin);call(voice,"putSlotInfo",slot,slotInfo);
  call(stack,"setExpectedDuration",expected);call(voice,"checkDuration");
  require(String.valueOf(value(voice,"termination")).equals(args[2].equals("3")?"-25/64":"7/64"),"source termination disagrees with declared quarter duration");
  Object prepared=snapshot();graphListen();active=true;call(rest,"remove",true);active=false;Object after=snapshot();
  Files.writeString(Path.of(args[1]),json(map("scope","Declared in-memory singleton QUARTER_REST extensive removal after original MeasureFiller and ordered SymbolsLinker methods on a reloaded SYMBOLS Book. Original actual owners and manual relation; fresh declared Voice/Slot at 7/64. Not fresh image processing, not ordinary LINKS clock evidence, no Java rollback claim.","book",args[0],"expected_numerator",args[2],"selected_system",2,"selected_rest_id",4436,"pre_declaration",loaded,"prepared",prepared,"events",observations,"after",after,"errors",errors,"book_save_invoked",false,"reachStep_invoked",false))+"\n");
  require(errors.isEmpty(),"observer errors "+errors);
 }
}
