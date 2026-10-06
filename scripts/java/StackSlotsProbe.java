// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class StackSlotsProbe extends FoundationOverlapProbe {
 static Object state(Object measure)throws Exception {
  Object stack=call(measure,"getStack"); List<Object> slots=new ArrayList<>(),voices=new ArrayList<>();
  for(Object slot:(List<?>)call(stack,"getSlots")) {
   List<Object> incoming=new ArrayList<>();
   for(Object ch:(List<?>)call(slot,"getChords")) incoming.add(rect(call(ch,"getBounds")));
   slots.add(obj("id",call(slot,"getId"),"time_offset",String.valueOf(call(slot,"getTimeOffset")),"incoming_bounds",incoming));
  }
  for(Object voice:(List<?>)call(measure,"getVoices")) {
   List<Object> table=new ArrayList<>();
   for(Object slot:(List<?>)call(stack,"getSlots")) {
    Object info=call(voice,"getSlotInfo",slot);
    if(info!=null) table.add(obj("slot_id",call(slot,"getId"),"status",info.getClass().getField("status").get(info).toString(),"chord_bounds",rect(call(info.getClass().getField("chord").get(info),"getBounds"))));
   }
   voices.add(obj("id",call(voice,"getId"),"slots",table));
  }
  return obj("slots",slots,"voices",voices,"abnormal",call(measure,"isAbnormal"));
 }
 static void generate(Object measure)throws Exception {
  Object rhythm=make("sheet.rhythm.StackRhythm",call(measure,"getStack"));
  Method method=rhythm.getClass().getDeclaredMethod("generateStackSlots");method.setAccessible(true);method.invoke(rhythm);
 }
 static void clearTables(Object measure)throws Exception {
  for(Object voice:(List<?>)call(measure,"getVoices")){Field slots=voice.getClass().getDeclaredField("slots");slots.setAccessible(true);slots.set(voice,null);}
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
  Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get(),voice=call(call(group,"getFirstChord"),"getVoice"),measure=call(voice,"getMeasure");
  clearTables(measure);generate(measure);Object saved=state(measure);
  Object last=call(voice,"getLastChord");call(voice,"removeChord",last);call(last,"justAssignVoice",(Object)null);
  Object second=make("sheet.rhythm.Voice",last,measure);call(measure,"addVoice",second);
  call(last,"setTimeOffset",make("math.Rational",9,16));clearTables(measure);generate(measure);Object continued=state(measure);
  call(last,"setTimeOffset",make("math.Rational",3,4));clearTables(measure);generate(measure);Object boundary=state(measure);
  call(last,"setTimeOffset",(Object)null);clearTables(measure);generate(measure);Object missing=state(measure);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","saved original normal stack slots plus declared second-voice offset 9/16 (CONTINUE),3/4 (equal-end free),null (abnormal); not automatic polyphony matching", "saved",saved,"continued",continued,"boundary",boundary,"missing",missing));System.exit(0);
 }
}
